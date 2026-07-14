#!/usr/bin/env python3
"""Validate docs frontmatter and local Markdown links without dependencies."""

from __future__ import annotations

import re
import sys
from pathlib import Path
from urllib.parse import unquote

ROOT = Path(__file__).resolve().parents[2]
DOCS = ROOT / "docs"
REQUIRED_KEYS = ("title", "summary", "status")
LINK_RE = re.compile(r"(?<!!)\[[^\]]+\]\(([^)]+)\)")
FENCE_RE = re.compile(r"^\s*```")


def frontmatter(text: str, path: Path) -> list[str]:
    errors = []
    lines = text.splitlines()
    if not lines or lines[0].strip() != "---":
        return [f"{path}: missing YAML frontmatter"]
    try:
        end = lines[1:].index("---") + 1
    except ValueError:
        return [f"{path}: unterminated YAML frontmatter"]
    keys = {
        line.split(":", 1)[0].strip()
        for line in lines[1:end]
        if ":" in line and not line.startswith((" ", "\t"))
    }
    for key in REQUIRED_KEYS:
        if key not in keys:
            errors.append(f"{path}: missing frontmatter key `{key}`")
    return errors


def without_fenced_code(text: str) -> str:
    output = []
    in_fence = False
    for line in text.splitlines():
        if FENCE_RE.match(line):
            in_fence = not in_fence
            continue
        if not in_fence:
            output.append(line)
    return "\n".join(output)


def local_links(text: str, path: Path) -> list[str]:
    errors = []
    for raw in LINK_RE.findall(without_fenced_code(text)):
        target = raw.strip().split(maxsplit=1)[0].strip("<>")
        if not target or target.startswith(("#", "http://", "https://", "mailto:")):
            continue
        target = unquote(target.split("#", 1)[0].split("?", 1)[0])
        resolved = (path.parent / target).resolve()
        try:
            resolved.relative_to(ROOT.resolve())
        except ValueError:
            errors.append(f"{path}: local link escapes repository: {raw}")
            continue
        if resolved.is_dir():
            resolved = resolved / "README.md"
        if not resolved.exists():
            errors.append(f"{path}: broken local link `{raw}`")
    return errors


def main() -> int:
    errors = []
    pages = sorted(DOCS.rglob("*.md"))
    if not pages:
        print("error: no documentation pages found", file=sys.stderr)
        return 1
    for page in pages:
        text = page.read_text()
        display = page.relative_to(ROOT)
        errors.extend(frontmatter(text, display))
        errors.extend(local_links(text, page))
    if errors:
        print("documentation check failed:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1
    print(f"documentation check passed: {len(pages)} pages")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
