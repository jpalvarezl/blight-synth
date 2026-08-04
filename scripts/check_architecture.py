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
    # These exact allowlists make dependency growth in portable crates an
    # explicit architecture change rather than a way to route around the
    # known-forbidden list above.
    "dsp": {"arrayvec", "log", "utils"},
    # `loom` is test-only and deterministically models ADR 0005's atomic handoff.
    "engine": {"dsp", "loom", "param_manifest"},
    "param_manifest": {"serde", "serde_json"},
    "sequencer": {"anyhow", "bincode", "clap", "serde", "serde_json", "serde_with"},
    "utils": {"serde", "serde_json"},
    "os_dls": {"riff"},
}
OPTIONAL_DEPENDENCY_FEATURES = {
    "cpal": "device-host",
    "ringbuf": "device-host",
    "env_logger": "standalone-process",
    "rosc": "standalone-process",
    "tokio": "standalone-process",
}
DEVICE_HOST_EXAMPLES = {
    "cycle_waveforms",
    "envelope",
    "master_gain",
    "sample_playback_from_file",
    "sample_playback_from_gl_instruments",
    "simple_setup",
    "simple_song",
    "voice_effects",
}
STANDALONE_PROCESS_EXAMPLES = {
    "meter_listen",
    "osc_control",
    "play_song_file",
    "polyphonic_song",
}
HOST_FREE_EXAMPLES = {"render_song", "update_offline_references"}


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
        for dependency, feature in sorted(OPTIONAL_DEPENDENCY_FEATURES.items()):
            record = dependency_records.get(dependency)
            if record is None:
                errors.append(f"`audio_backend` {feature} dependency `{dependency}` is missing")
                continue
            if not record["optional"]:
                errors.append(
                    f"`audio_backend` {feature} dependency `{dependency}` must be optional"
                )
            enabled = set(audio_backend["features"].get(feature, []))
            if f"dep:{dependency}" not in enabled:
                errors.append(
                    f"`audio_backend` {feature} must enable `dep:{dependency}`"
                )

        standalone_alias = set(audio_backend["features"].get("standalone", []))
        if "standalone-process" not in standalone_alias:
            errors.append("`audio_backend` standalone alias must enable `standalone-process`")

        tokio = dependency_records.get("tokio")
        if tokio is not None:
            tokio_features = set(tokio["features"])
            if "rt-multi-thread" in tokio_features:
                errors.append("Tokio must not enable the multi-thread runtime")
            if "rt" not in tokio_features:
                errors.append("Tokio current-thread runtime requires the `rt` feature")

        example_targets = {
            target["name"]: target
            for target in audio_backend["targets"]
            if "example" in target["kind"]
        }
        for example in sorted(DEVICE_HOST_EXAMPLES):
            target = example_targets.get(example)
            if target is None or "device-host" not in target.get("required-features", []):
                errors.append(f"device-host example `{example}` must require `device-host`")
        for example in sorted(STANDALONE_PROCESS_EXAMPLES):
            target = example_targets.get(example)
            if target is None or "standalone-process" not in target.get("required-features", []):
                errors.append(
                    f"standalone-process example `{example}` must require `standalone-process`"
                )
        for example in sorted(HOST_FREE_EXAMPLES):
            target = example_targets.get(example)
            if target is None:
                errors.append(f"host-free example `{example}` is missing")
            elif "standalone" in target.get("required-features", []):
                errors.append(f"host-free example `{example}` must not require `standalone`")

        dsp_core = next(
            (
                target
                for target in audio_backend["targets"]
                if target["name"] == "dsp-core" and "bin" in target["kind"]
            ),
            None,
        )
        if dsp_core is None or "standalone-process" not in dsp_core.get(
            "required-features", []
        ):
            errors.append("`dsp-core` must require the `standalone-process` feature")

    if errors:
        print("architecture dependency check failed:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    print("architecture dependency check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
