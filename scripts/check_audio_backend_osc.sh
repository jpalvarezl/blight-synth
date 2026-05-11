#!/usr/bin/env bash
set -euo pipefail

# Hardware-free validation for standalone OSC work.
# Does not open an audio device or bind OSC ports.

cargo fmt --all -- --check
cargo test -p audio_backend
cargo check --workspace --all-targets
