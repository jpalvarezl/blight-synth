#!/usr/bin/env bash
set -euo pipefail

# Opens the default audio device and plays calibration.json via the existing
# audio_backend command queue. Pass duration seconds as first arg.

DURATION_SECONDS="${1:-10}"
export RUST_LOG="${RUST_LOG:-info}"

echo "RUST_LOG=${RUST_LOG}"
cargo run -p audio_backend --example play_song_file -- calibration.json "${DURATION_SECONDS}"
