#!/usr/bin/env python3
"""Generate docs/work/burndown.md from GitHub issue metadata.

GitHub is canonical. The generated Markdown is an offline/Obsidian snapshot.
No third-party Python packages are required.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from collections import defaultdict
from pathlib import Path

REPO = "jpalvarezl/blight-synth"
ROOT = Path(__file__).resolve().parents[2]
OUTPUT = ROOT / "docs" / "work" / "burndown.md"
SIZE_POINTS = {"size:S": 1, "size:M": 3, "size:L": 5}
STATUS_LABELS = ("status:ready", "status:in-progress", "status:blocked")


def gh_json(endpoint: str) -> list[dict]:
    items: list[dict] = []
    separator = "&" if "?" in endpoint else "?"
    page = 1
    while True:
        paged_endpoint = f"{endpoint}{separator}page={page}"
        try:
            result = subprocess.run(
                ["gh", "api", paged_endpoint],
                check=True,
                text=True,
                capture_output=True,
            )
        except FileNotFoundError:
            raise SystemExit("error: GitHub CLI (`gh`) is required")
        except subprocess.CalledProcessError as error:
            detail = error.stderr.strip() or error.stdout.strip()
            raise SystemExit(f"error: GitHub API request failed: {detail}")
        value = json.loads(result.stdout)
        if not isinstance(value, list):
            raise SystemExit(f"error: expected a JSON list from {paged_endpoint}")
        items.extend(value)
        if len(value) < 100:
            return items
        page += 1


def issue_status(labels: set[str], state: str) -> str:
    if state == "closed":
        return "done"
    matches = [label for label in STATUS_LABELS if label in labels]
    if len(matches) > 1:
        return "invalid-status"
    if matches:
        return matches[0].removeprefix("status:")
    return "backlog"


def issue_size(labels: set[str]) -> tuple[str, int | None]:
    matches = sorted(label for label in labels if label.startswith("size:"))
    if len(matches) != 1:
        return ("unsized" if not matches else "invalid-size", None)
    label = matches[0]
    return label.removeprefix("size:"), SIZE_POINTS.get(label)


def escape(text: str) -> str:
    return text.replace("|", "\\|").replace("\n", " ")


def render(issues: list[dict]) -> str:
    roadmap = []
    for issue in issues:
        if "pull_request" in issue:
            continue
        labels = {label["name"] for label in issue.get("labels", [])}
        if "roadmap-task" not in labels:
            continue
        status = issue_status(labels, issue["state"])
        size, points = issue_size(labels)
        milestone = issue.get("milestone")
        roadmap.append(
            {
                "number": issue["number"],
                "title": issue["title"],
                "url": issue["html_url"],
                "state": issue["state"],
                "status": status,
                "size": size,
                "points": points,
                "milestone_number": milestone["number"] if milestone else 9999,
                "milestone": milestone["title"] if milestone else "No milestone",
                "assignees": [user["login"] for user in issue.get("assignees", [])],
                "updated_at": issue["updated_at"],
            }
        )

    roadmap.sort(key=lambda item: (item["milestone_number"], item["number"]))
    latest = max((item["updated_at"] for item in roadmap), default="unknown")
    groups: dict[tuple[int, str], list[dict]] = defaultdict(list)
    for item in roadmap:
        groups[(item["milestone_number"], item["milestone"])].append(item)

    lines = [
        "---",
        "title: Generated Roadmap Burndown",
        "summary: Offline Obsidian snapshot generated from GitHub roadmap issue metadata.",
        "status: generated",
        f"source-updated: {latest}",
        "generator: scripts/docs/sync_roadmap.py",
        "---",
        "",
        "# Generated Roadmap Burndown",
        "",
        "> [!warning] Generated file",
        "> GitHub Issues are canonical. Do not edit this page manually. Run `python3 scripts/docs/sync_roadmap.py`.",
        "",
        f"Data snapshot through `{latest}`.",
        "",
        "## Summary",
        "",
        "| Milestone | Open | Done | Ready | In progress | Blocked | Backlog | Sized points done/total | Unsized |",
        "|---|---:|---:|---:|---:|---:|---:|---:|---:|",
    ]

    for (_number, name), items in sorted(groups.items()):
        counts = defaultdict(int)
        done_points = 0
        total_points = 0
        unsized = 0
        for item in items:
            counts[item["status"]] += 1
            if item["points"] is None:
                unsized += 1
            else:
                total_points += item["points"]
                if item["status"] == "done":
                    done_points += item["points"]
        open_count = sum(1 for item in items if item["state"] == "open")
        done_count = counts["done"]
        lines.append(
            f"| {escape(name)} | {open_count} | {done_count} | {counts['ready']} | "
            f"{counts['in-progress']} | {counts['blocked']} | {counts['backlog']} | "
            f"{done_points}/{total_points} | {unsized} |"
        )

    open_groups = [
        (key, items) for key, items in sorted(groups.items()) if any(item["state"] == "open" for item in items)
    ]
    if open_groups:
        (number, name), items = open_groups[0]
        del number
        lines.extend(["", "## Current milestone", "", f"### {name}", ""])
        for item in items:
            if item["state"] != "open":
                continue
            owner = ", ".join(f"@{name}" for name in item["assignees"]) or "unassigned"
            lines.append(
                f"- [ ] [#{item['number']}]({item['url']}) {item['title']} "
                f"— `{item['status']}`, `{item['size']}`, {owner}"
            )

    lines.extend(["", "## All roadmap tasks", ""])
    for (_number, name), items in sorted(groups.items()):
        lines.extend([f"### {name}", ""])
        for item in items:
            checked = "x" if item["state"] == "closed" else " "
            owner = ", ".join(f"@{name}" for name in item["assignees"]) or "unassigned"
            lines.append(
                f"- [{checked}] [#{item['number']}]({item['url']}) {item['title']} "
                f"— `{item['status']}`, `{item['size']}`, {owner}"
            )
        lines.append("")

    lines.extend(
        [
            "## Status rules",
            "",
            "See [Work and Parallelization System](README.md). Open issues without a workflow label are backlog. "
            "Epics and unsized issues are excluded from point totals until split/estimated.",
            "",
        ]
    )
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true", help="fail if the committed snapshot is stale")
    parser.add_argument("--stdout", action="store_true", help="print instead of writing")
    args = parser.parse_args()

    issues = gh_json(f"repos/{REPO}/issues?state=all&labels=roadmap-task&per_page=100")
    content = render(issues)

    if args.stdout:
        print(content, end="")
        return 0
    if args.check:
        current = OUTPUT.read_text() if OUTPUT.exists() else ""
        if current != content:
            print(f"stale generated roadmap: run {Path(__file__).as_posix()}", file=sys.stderr)
            return 1
        return 0

    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT.write_text(content)
    print(f"wrote {OUTPUT.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
