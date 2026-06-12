---
tags: [binary, osc]
sources: ["[[concepts/m1-dsp-core-standalone]]"]
last-updated: 2025-05-11
source-file: audio_backend/src/bin/dsp-core.rs
source-sha: 73af077e381d3a4352087664f6a6c8f1527025d4
source-mtime: 1781291696092
last-synced: 2026-06-12
---
# dsp-core (binary)

Standalone DSP core entry point (`audio_backend` bin target `dsp-core`). Added for #102. Runs audio + OSC together.

## Flow (`#[tokio::main]`)

1. `env_logger::init()`.
2. `BlightAudio::new()` — keeps the audio stream alive for process lifetime.
3. Install the reserved master gain effect: `MixerCmd::AddMasterEffect` with `create_stereo_gain(MASTER_GAIN_EFFECT_ID, 1.0)` — the target of `/param/set gain`.
4. `OscServer::bind()`, grab `audio.meter_state()`.
5. Print `READY` + flush stdout — readiness contract for the future Bun host (smoke script greps `^READY$`).
6. `tokio::select!` over `osc_server.run_with_meter(&mut audio, &meter)` and `tokio::signal::ctrl_c()` for shutdown.

## Validation

- `scripts/check_audio_backend_osc.sh` — hardware-free (fmt check, tests, `cargo check --workspace --all-targets`).
- `scripts/smoke_osc_standalone.sh` — opens real audio device + OSC ports; starts dsp-core, waits for `READY`, runs the `osc_control` example, shuts down. See [[sources/osc-validation-scripts]].
