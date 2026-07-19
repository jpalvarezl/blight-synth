#!/usr/bin/env python3
"""Reject accidental direct logging/printing in known callback-reachable modules."""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCANNED = [
    ROOT / "engine" / "src",
    ROOT / "audio_backend" / "src" / "player",
    ROOT / "audio_backend" / "src" / "standalone" / "audio_processor",
    ROOT / "dsp" / "src" / "effects",
    ROOT / "dsp" / "src" / "instruments" / "mod.rs",
    ROOT / "dsp" / "src" / "instruments" / "synth_nodes",
    ROOT / "dsp" / "src" / "synth_infra" / "effects.rs",
    ROOT / "dsp" / "src" / "synth_infra" / "voice.rs",
]
DIRECT_LOG = re.compile(
    r"(?:\blog::)?\b(?:trace|debug|info|warn|error)!\s*\(|\b(?:e?println)!\s*\("
)


def rust_files(path: Path):
    if path.is_file():
        yield path
    elif path.is_dir():
        yield from path.rglob("*.rs")


def main() -> int:
    errors = []
    for root in SCANNED:
        for path in rust_files(root):
            for line_number, line in enumerate(path.read_text().splitlines(), start=1):
                if DIRECT_LOG.search(line):
                    errors.append(
                        f"{path.relative_to(ROOT)}:{line_number}: direct callback log/print: {line.strip()}"
                    )

    if errors:
        print(
            "RT logging check failed; use dsp::rt_{debug,info,warn,error}_log! for developer diagnostics:",
            file=sys.stderr,
        )
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    print("RT logging check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
