#!/usr/bin/env bash
set -euo pipefail

# Hardware-free validation for the standalone OSC work.
# This does not open an audio device or bind OSC ports.

cargo fmt --all -- --check
cargo test -p audio_backend
cargo check --workspace
cargo check -p audio_backend --bin dsp-core
cargo check -p audio_backend --example osc_control
cargo check -p audio_backend --example play_song_file
