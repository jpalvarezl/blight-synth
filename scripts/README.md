---
title: Repository Validation Scripts
summary: Local commands corresponding to the hardware-free CI baseline and manual audio checks.
status: current
updated: 2026-07-19
issues: [131]
---

# Repository Validation Scripts

## CI-equivalent checks

Run from the repository root:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
cargo clippy -p audio_backend --no-default-features --all-targets -- -D warnings
cargo test -p audio_backend --no-default-features --all-targets
python3 scripts/check_architecture.py
python3 scripts/check_rt_logging.py
python3 scripts/docs/check_docs.py
python3 scripts/docs/sync_roadmap.py --stdout > /dev/null
```

These commands must not open an audio device. The architecture checker enforces the current crate boundary, while the RT logging checker rejects accidental direct logger/printing calls in known callback-reachable modules. Developer callback diagnostics use `dsp::rt_debug_log!`, which is compiled out when `debug_assertions` are disabled.

The committed roadmap is an offline snapshot of live GitHub metadata. Maintainers regenerate it after roadmap changes with `python3 scripts/docs/sync_roadmap.py`; CI exercises the generator but does not require an unrelated live issue update to be committed in every code PR.

## Offline render regression workflow

- `scripts/render_reference_songs.sh` — render the supported current-schema reference songs to playable WAVs under `target/offline-renders/`.
- `cargo test -p audio_backend --test offline_golden` — verify deterministic end-to-end song references.
- `cargo run -p audio_backend --example update_offline_references -- --update-reference` — intentionally rewrite reviewed hashes/metrics and generate listening WAVs. Never run this as an automatic CI repair.

See [`docs/architecture/offline-render-contract.md`](../docs/architecture/offline-render-contract.md) before accepting reference changes.

## Focused and manual checks

- `scripts/check_audio_backend_osc.sh` — hardware-free OSC/audio-backend checks.
- `scripts/smoke_meter_streaming.sh` — headless meter transport smoke test.
- `scripts/smoke_osc_standalone.sh` — manual standalone OSC/audio smoke test.
- `scripts/play_calibration.sh` — manual audio-device playback.

TypeScript checks will be added to CI when the production `gui/` workspace lands in M3.
