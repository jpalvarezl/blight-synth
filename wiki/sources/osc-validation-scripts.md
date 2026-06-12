---
tags: [scripts, osc, ci]
sources: [scripts/check_audio_backend_osc.sh, scripts/smoke_osc_standalone.sh, audio_backend/examples/osc_control.rs]
last-updated: 2025-05-11
---

# OSC Validation Scripts

No `.github/workflows` CI exists; these scripts are the quality gate for OSC/DSP-core work.

## scripts/check_audio_backend_osc.sh (hardware-free)

Does NOT open an audio device or bind OSC ports. Runs:
```
cargo fmt --all -- --check
cargo test -p audio_backend
cargo check --workspace --all-targets
```

## scripts/smoke_osc_standalone.sh (manual, needs hardware)

Opens the default audio output and binds dsp-core `:9000` / osc_control `:9001`. Starts `dsp-core`, polls the log for `^READY$` (timeout `READY_TIMEOUT_SECONDS`, default 20), runs `cargo run -p audio_backend --example osc_control`, then kills dsp-core on exit.

## audio_backend/examples/osc_control.rs

Sends `/song/load calibration.json`, `/param/set gain -6.0`, `/transport/play`, then reads `/meter/level` for ~2s (prints every 10th, plus a total count), then `/transport/stop`. Demonstrates the [[entities/meter-state|meter]] stream end-to-end.
