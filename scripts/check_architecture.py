#!/usr/bin/env python3
"""Enforce the current M0 Cargo dependency direction without third-party packages."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

# These are current baseline rules. Issue #130 will tighten them after file and
# platform resource loading leaves `dsp`.
FORBIDDEN = {
    "dsp": {
        "audio_backend",
        "sequencer",
        "cpal",
        "rosc",
        "tokio",
        "eframe",
        "egui_extras",
        "rfd",
    },
    "sequencer": {
        "audio_backend",
        "dsp",
        "cpal",
        "rosc",
        "tokio",
        "eframe",
        "egui_extras",
        "rfd",
    },
    "utils": {
        "audio_backend",
        "dsp",
        "sequencer",
        "cpal",
        "rosc",
        "tokio",
        "eframe",
    },
}
REQUIRED = {"audio_backend": {"dsp", "sequencer"}}


def metadata() -> dict:
    try:
        result = subprocess.run(
            ["cargo", "metadata", "--format-version", "1", "--no-deps"],
            cwd=ROOT,
            check=True,
            text=True,
            capture_output=True,
        )
    except FileNotFoundError:
        raise SystemExit("error: cargo is required")
    except subprocess.CalledProcessError as error:
        detail = error.stderr.strip() or error.stdout.strip()
        raise SystemExit(f"error: cargo metadata failed: {detail}")
    return json.loads(result.stdout)


def main() -> int:
    packages = {
        package["name"]: {dependency["name"] for dependency in package["dependencies"]}
        for package in metadata()["packages"]
    }
    errors = []

    for package, forbidden in FORBIDDEN.items():
        if package not in packages:
            errors.append(f"workspace package `{package}` is missing")
            continue
        for dependency in sorted(packages[package] & forbidden):
            errors.append(f"`{package}` must not depend on `{dependency}`")

    for package, required in REQUIRED.items():
        if package not in packages:
            errors.append(f"workspace package `{package}` is missing")
            continue
        for dependency in sorted(required - packages[package]):
            errors.append(f"`{package}` must depend on `{dependency}`")

    if errors:
        print("architecture dependency check failed:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    print("architecture dependency check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
