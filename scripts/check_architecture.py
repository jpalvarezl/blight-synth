#!/usr/bin/env python3
"""Enforce the current M0 Cargo dependency direction without third-party packages."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

# These are the current M0 baseline rules. Host, file, and platform resource
# dependencies must not leak into portable processing/model crates.
FORBIDDEN = {
    "dsp": {
        "audio_backend",
        "engine",
        "sequencer",
        "cpal",
        "rosc",
        "tokio",
        "eframe",
        "egui_extras",
        "rfd",
        "hound",
        "os_dls",
    },
    "engine": {
        "audio_backend",
        "sequencer",
        "cpal",
        "rosc",
        "tokio",
        "eframe",
        "egui_extras",
        "rfd",
        "hound",
        "os_dls",
    },
    "sequencer": {
        "audio_backend",
        "dsp",
        "engine",
        "cpal",
        "rosc",
        "tokio",
        "eframe",
        "egui_extras",
        "rfd",
        "hound",
        "os_dls",
    },
    "utils": {
        "audio_backend",
        "dsp",
        "engine",
        "sequencer",
        "cpal",
        "rosc",
        "tokio",
        "eframe",
        "egui_extras",
        "rfd",
        "hound",
        "os_dls",
    },
}
REQUIRED = {
    "engine": {"dsp"},
    "audio_backend": {"dsp", "engine", "sequencer"},
}
ALLOWED = {
    # Keep the host-independent render runtime deliberately narrow. Any new
    # dependency requires an explicit architecture change rather than merely
    # avoiding the known-forbidden list above.
    "engine": {"dsp"},
}
STANDALONE_OPTIONAL_DEPENDENCIES = {"cpal", "env_logger", "ringbuf", "rosc", "tokio"}


def metadata() -> dict:
    try:
        result = subprocess.run(
            [
                "cargo",
                "metadata",
                "--format-version",
                "1",
                "--no-deps",
                "--locked",
            ],
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
    workspace = metadata()
    package_records = {package["name"]: package for package in workspace["packages"]}
    packages = {
        name: {dependency["name"] for dependency in package["dependencies"]}
        for name, package in package_records.items()
    }
    errors = []

    for package, forbidden in FORBIDDEN.items():
        if package not in packages:
            errors.append(f"workspace package `{package}` is missing")
            continue
        for dependency in sorted(packages[package] & forbidden):
            errors.append(f"`{package}` must not depend on `{dependency}`")

    for package, allowed in ALLOWED.items():
        if package not in packages:
            errors.append(f"workspace package `{package}` is missing")
            continue
        for dependency in sorted(packages[package] - allowed):
            errors.append(f"`{package}` has non-allowlisted dependency `{dependency}`")

    for package, required in REQUIRED.items():
        if package not in packages:
            errors.append(f"workspace package `{package}` is missing")
            continue
        for dependency in sorted(required - packages[package]):
            errors.append(f"`{package}` must depend on `{dependency}`")

    audio_backend = package_records.get("audio_backend")
    if audio_backend is not None:
        dependency_records = {
            dependency["name"]: dependency
            for dependency in audio_backend["dependencies"]
        }
        standalone_feature = set(audio_backend["features"].get("standalone", []))
        for dependency in sorted(STANDALONE_OPTIONAL_DEPENDENCIES):
            record = dependency_records.get(dependency)
            if record is None:
                errors.append(f"`audio_backend` standalone dependency `{dependency}` is missing")
                continue
            if not record["optional"]:
                errors.append(
                    f"`audio_backend` standalone dependency `{dependency}` must be optional"
                )
            if f"dep:{dependency}" not in standalone_feature:
                errors.append(
                    f"`audio_backend` standalone feature must enable `dep:{dependency}`"
                )

        tokio = dependency_records.get("tokio")
        if tokio is not None:
            tokio_features = set(tokio["features"])
            if "rt-multi-thread" in tokio_features:
                errors.append("Tokio must not enable the multi-thread runtime")
            if "rt" not in tokio_features:
                errors.append("Tokio current-thread runtime requires the `rt` feature")

        dsp_core = next(
            (
                target
                for target in audio_backend["targets"]
                if target["name"] == "dsp-core" and "bin" in target["kind"]
            ),
            None,
        )
        if dsp_core is None or "standalone" not in dsp_core["required-features"]:
            errors.append("`dsp-core` must require the `standalone` feature")

    if errors:
        print("architecture dependency check failed:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    print("architecture dependency check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
