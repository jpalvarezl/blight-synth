#!/usr/bin/env python3
"""Reconcile GitHub work metadata with active packets and generated docs.

GitHub remains canonical. By default this command is read-only and fails on
contradictions. `--fix-docs` updates committed generated/routing files.
`--fix-github` is required for label/assignee mutations.
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import sync_roadmap  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]
ACTIVE_DIR = ROOT / "docs" / "work" / "active"
ACTIVE_INDEX = ACTIVE_DIR / "README.md"
BURNDOWN = ROOT / "docs" / "work" / "burndown.md"
STATUS_LABELS = {"status:ready", "status:in-progress", "status:blocked"}
ISSUE_RE = re.compile(r"^issue:\s*(\d+)\s*$", re.MULTILINE)
DEPENDS_RE = re.compile(
    r"^Depends on:[^\S\n]*([^\n]*)$", re.MULTILINE | re.IGNORECASE
)
NUMBER_RE = re.compile(r"#(\d+)")
ACTIVE_LINK_RE = re.compile(r"^- \[Issue #(\d+) .*\]\((issue-[^)]+\.md)\)$", re.MULTILINE)


@dataclass(frozen=True)
class Packet:
    path: Path
    issue: int


def labels(issue: dict) -> set[str]:
    return {item["name"] for item in issue.get("labels", [])}


def workflow_status(issue: dict) -> str | None:
    matches = STATUS_LABELS & labels(issue)
    if len(matches) == 1:
        return next(iter(matches))
    return None


def packets() -> list[Packet]:
    result = []
    for path in sorted(ACTIVE_DIR.glob("issue-*.md")):
        match = ISSUE_RE.search(path.read_text())
        if not match:
            raise SystemExit(f"error: {path.relative_to(ROOT)} has no numeric `issue:` frontmatter")
        result.append(Packet(path, int(match.group(1))))
    return result


def dependencies(issue: dict) -> list[int]:
    match = DEPENDS_RE.search(issue.get("body") or "")
    return [int(value) for value in NUMBER_RE.findall(match.group(1))] if match else []


def render_active_index(current: str, active: list[Packet], issues: dict[int, dict]) -> str:
    heading = "## Active"
    prefix = current.split(heading, 1)[0].rstrip()
    lines = [prefix, "", heading, ""]
    if not active:
        lines.append("_No active packets. Create one when an issue moves to `status:in-progress`._")
    else:
        for packet in active:
            issue = issues.get(packet.issue)
            title = issue["title"] if issue else packet.path.stem
            title = title.replace("\\", "\\\\").replace("[", "\\[").replace("]", "\\]")
            lines.append(f"- [Issue #{packet.issue} — {title}]({packet.path.name})")
    return "\n".join(lines) + "\n"


def validate(
    issue_list: list[dict], active: list[Packet], active_index: str, burndown: str
) -> tuple[list[str], list[str]]:
    errors: list[str] = []
    warnings: list[str] = []
    issue_map = {item["number"]: item for item in issue_list if "pull_request" not in item}
    packet_by_issue: dict[int, list[Packet]] = {}
    for packet in active:
        packet_by_issue.setdefault(packet.issue, []).append(packet)
        issue = issue_map.get(packet.issue)
        if issue is None:
            errors.append(f"packet {packet.path.name} references missing issue #{packet.issue}")
            continue
        if issue["state"] != "open":
            errors.append(f"packet {packet.path.name} references closed issue #{packet.issue}")
        if workflow_status(issue) != "status:in-progress":
            errors.append(
                f"packet #{packet.issue} exists but issue status is {workflow_status(issue) or 'backlog'}"
            )
    for number, values in packet_by_issue.items():
        if len(values) > 1:
            errors.append(f"issue #{number} has multiple active packets")

    for issue in issue_map.values():
        issue_labels = labels(issue)
        statuses = STATUS_LABELS & issue_labels
        if len(statuses) > 1:
            errors.append(f"issue #{issue['number']} has multiple workflow labels: {sorted(statuses)}")
        if issue["state"] == "closed":
            if statuses:
                warnings.append(f"closed issue #{issue['number']} still has workflow label(s) {sorted(statuses)}")
            continue
        status = workflow_status(issue)
        is_epic = "size:epic" in issue_labels
        if status == "status:in-progress":
            if not issue.get("assignees"):
                errors.append(f"in-progress issue #{issue['number']} has no assignee")
            if not is_epic and issue["number"] not in packet_by_issue:
                errors.append(f"in-progress leaf issue #{issue['number']} has no active packet")
        deps = dependencies(issue)
        unresolved = [number for number in deps if issue_map.get(number, {}).get("state") != "closed"]
        if status == "status:blocked" and not deps:
            warnings.append(
                f"blocked issue #{issue['number']} has no machine-readable `Depends on:` line"
            )
        if status == "status:blocked" and deps and not unresolved:
            errors.append(f"blocked issue #{issue['number']} has no unresolved dependencies")
        if status == "status:ready" and unresolved:
            errors.append(f"ready issue #{issue['number']} has unresolved dependencies {unresolved}")

    expected_index = render_active_index(active_index, active, issue_map)
    if active_index != expected_index:
        current_members = ACTIVE_LINK_RE.findall(active_index)
        expected_members = ACTIVE_LINK_RE.findall(expected_index)
        if current_members != expected_members:
            errors.append("active packet index membership is stale: run reconcile_work.py --fix-docs")
        else:
            warnings.append("active packet index titles differ from live GitHub")
    expected_burndown = sync_roadmap.render(issue_list)
    if burndown != expected_burndown:
        warnings.append(
            "generated burndown snapshot differs from live GitHub; "
            "run reconcile_work.py --fix-docs at a task boundary"
        )
    return errors, warnings


def run_gh(*args: str) -> None:
    subprocess.run(["gh", *args], check=True)


def github_commands(issue_list: list[dict]) -> list[list[str]]:
    issue_map = {item["number"]: item for item in issue_list if "pull_request" not in item}
    commands: list[list[str]] = []
    for issue in issue_map.values():
        statuses = STATUS_LABELS & labels(issue)
        if issue["state"] == "closed":
            if statuses:
                command = ["issue", "edit", str(issue["number"]), "--repo", sync_roadmap.REPO]
                for status in sorted(statuses):
                    command.extend(["--remove-label", status])
                commands.append(command)
            continue
        if workflow_status(issue) != "status:blocked":
            continue
        deps = dependencies(issue)
        if deps and all(issue_map.get(number, {}).get("state") == "closed" for number in deps):
            commands.append([
                "issue", "edit", str(issue["number"]), "--repo", sync_roadmap.REPO,
                "--remove-label", "status:blocked", "--add-label", "status:ready",
            ])
    return commands


def fix_github(issue_list: list[dict]) -> None:
    for command in github_commands(issue_list):
        run_gh(*command)


def fetch_issues() -> list[dict]:
    # Reconciliation needs non-roadmap active packets too; sync_roadmap.render
    # applies its own roadmap-task filter when producing the burndown.
    return sync_roadmap.gh_json(
        f"repos/{sync_roadmap.REPO}/issues?state=all&per_page=100"
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true", help="validate without modifying files")
    parser.add_argument("--fix-docs", action="store_true", help="remove closed packets and regenerate docs")
    parser.add_argument("--fix-github", action="store_true", help="explicitly clean safe GitHub status transitions")
    args = parser.parse_args()
    if not (args.check or args.fix_docs or args.fix_github):
        args.check = True

    issue_list = fetch_issues()
    if args.fix_github:
        fix_github(issue_list)
        issue_list = fetch_issues()
    issue_map = {item["number"]: item for item in issue_list if "pull_request" not in item}

    active = packets()
    if args.fix_docs:
        for packet in active:
            issue = issue_map.get(packet.issue)
            if issue is not None and issue["state"] == "closed":
                packet.path.unlink()
                print(f"removed {packet.path.relative_to(ROOT)}")
        active = packets()
        current_index = ACTIVE_INDEX.read_text()
        ACTIVE_INDEX.write_text(render_active_index(current_index, active, issue_map))
        BURNDOWN.write_text(sync_roadmap.render(issue_list))
        print("reconciled active packet index and generated burndown")

    active_text = ACTIVE_INDEX.read_text()
    burndown_text = BURNDOWN.read_text()
    errors, warnings = validate(issue_list, packets(), active_text, burndown_text)
    for warning in warnings:
        print(f"warning: {warning}", file=sys.stderr)
    if errors:
        for error in errors:
            print(f"error: {error}", file=sys.stderr)
        return 1
    print("work-state reconciliation passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
