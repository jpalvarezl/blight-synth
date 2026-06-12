---
tags: [overview, architecture, rust, audio]
sources: [README.md, Cargo.toml]
last-updated: 2025-01-15
---

# Overview

blight-synth is a modular real-time synthesizer built as a Rust Cargo workspace, with a graphical frontend. The audio engine splits work across a non-real-time (NRT) control thread (`BlightAudio`) and a real-time (RT) audio thread (`AudioProcessor`) communicating over a lock-free SPSC command queue; the RT side runs a command processor, instrument manager, and player/sequencer that read song data (arrangement → chains → phrases → events), generate per-voice mono buffers through synth nodes (oscillators, drum nodes, sample player) and ADSR/pitch envelopes, apply per-voice and master effect chains, mix to stereo, and stream out via CPAL. Resource management (samples, wavetables, voice/instrument/effect factories) lives on the NRT side.

## Workspace members (Cargo.toml)

- `dsp` — DSP primitives: effects, factories (`dsp/src/{commands.rs,effects/,factories/}`).
- `audio_backend` — core audio engine: device mgmt, synthesis, streaming (`audio_backend/src/{audio_frontend/,audio_processor/,player/,osc.rs,commands.rs,song_hydration.rs,lib.rs}`).
- `utils` — music theory utilities (notes, scales).
- `sequencer` — sequencing/timing engine for pattern-based composition.
- `tracker_gui` — GUI (workspace `default-members`); README describes a Tauri `frontend/`, but the active default is `tracker_gui` (likely egui/eframe — see deps). Flagged below.
- `os_dls` — purpose TBD (DLS / OS sound resources?).

## Notable dependencies

- `cpal` — audio I/O. `hound` — WAV. `rosc` — OSC. `ringbuf`/`crossbeam` — lock-free queues. `eframe`/`egui_extras`/`rfd` — GUI. `bincode`/`serde`/`serde_json`/`serde_with` — (de)serialization. `clap` — CLI. `tokio` — async runtime.

## Assets & data

- `assets/` — `notes.csv`, `notes.json`, `generate_notes_json.py`.
- Root-level `calibration.json`, `drum_crap.json` — purpose TBD.
- `audio_backend/examples/` — many runnable examples (oscillators, envelopes, polyphonic songs, sample playback, voice effects).

> [!contradiction]
> README lists `frontend/` (Tauri) and crates `sequencer`, `utils` but no `dsp`/`tracker_gui`/`os_dls`; Cargo.toml workspace members are `dsp, audio_backend, utils, sequencer, tracker_gui, os_dls` with `default-members = ["tracker_gui"]` and GUI deps `eframe`/`egui_extras`. The active GUI appears to be `tracker_gui` (egui), not the README's Tauri `frontend/`. Reconcile when those crates are ingested.
