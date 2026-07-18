#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
output_dir="${1:-$repo_root/target/offline-renders}"
mkdir -p "$output_dir"

for song in calibration ending_theme_no_effect; do
  cargo run \
    --manifest-path "$repo_root/Cargo.toml" \
    -p audio_backend \
    --example render_song \
    -- \
    "$repo_root/$song.json" \
    "$output_dir/$song.wav"
done

printf 'Rendered supported reference songs to %s\n' "$output_dir"
if command -v afplay >/dev/null 2>&1; then
  printf 'Listen with: afplay "%s/calibration.wav"\n' "$output_dir"
fi
