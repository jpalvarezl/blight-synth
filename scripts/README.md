---
title: Repository Validation Scripts
summary: Local commands corresponding to the hardware-free CI baseline and manual audio checks.
status: current
updated: 2026-07-15
issues: [131]
---

# Repository Validation Scripts

## CI-equivalent checks

Run from the repository root:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
python3 scripts/check_architecture.py
python3 scripts/docs/check_docs.py
python3 scripts/docs/sync_roadmap.py --stdout > /dev/null
```

These commands must not open an audio device. The architecture checker enforces the current M0 dependency baseline, including the rule that portable DSP/model crates cannot depend on host, file-decoder, or platform-resource layers. #157 will reconcile the final M0 graph after engine/host extraction.

The committed roadmap is an offline snapshot of live GitHub metadata. Maintainers regenerate it after roadmap changes with `python3 scripts/docs/sync_roadmap.py`; CI exercises the generator but does not require an unrelated live issue update to be committed in every code PR.

## Focused and manual checks

- `scripts/check_audio_backend_osc.sh` — hardware-free OSC/audio-backend checks.
- `scripts/smoke_meter_streaming.sh` — headless meter transport smoke test.
- `scripts/smoke_osc_standalone.sh` — manual standalone OSC/audio smoke test.
- `scripts/play_calibration.sh` — manual audio-device playback.

TypeScript checks will be added to CI when the production `gui/` workspace lands in M3.
