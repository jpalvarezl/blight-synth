# blight-synth

blight-synth is a modular synthesizer application built in Rust, featuring a dedicated audio backend and a graphical frontend interface. The project is organized into several Rust crates and a frontend GUI (built with Tauri), enabling real-time audio synthesis and user interaction.

## Project Structure

- `audio_backend/` — Core audio engine. Handles audio device management, synthesis, streaming, and processing. Written in Rust.
- `harmony/` — Music theory utilities (notes, scales, etc.) for use by the synth engine. Written in Rust.
- `frontend/` — Graphical User Interface (GUI) for operating the synth, built with Tauri. (Details are in the folder; not covered here.)
- `assets/` — Data files for notes and other resources.
- `scripts/` — Utility scripts (e.g., for generating note data).

## Setup

1. Clone the repository.
2. Build the Rust backend:
   ```sh
   cargo build --release
   ```
3. (Optional) See `frontend/README.md` for GUI setup instructions.

## audio_backend Architecture

The `audio_backend` crate is responsible for all audio processing and device management. Its architecture is modular and consists of the following main components:

```
+-------------------+
|   audio_backend   |
+-------------------+
        |
        v
+-------------------+
|   devices/        |  <-- Audio device & stream management (cpal)
+-------------------+
        |
        v
+-------------------+
|   synths/         |  <-- Synthesis engine (Oscillator, ADSR, Synthesizer, Voice Manager)
+-------------------+
        |
        v
+-------------------+
|   harmony crate   |  <-- Music theory (notes, scales)
+-------------------+
```

- **devices/**: Manages audio devices and streaming using the `cpal` library. Includes stream creation, buffer management, and audio callback logic.
- **synths/**: Implements synthesis algorithms (oscillators, ADSR envelopes, voice management, etc.).
- **harmony/**: Provides music theory utilities (note frequencies, scales, etc.) used by the synth engine.

## Main Dependencies

- [cpal](https://github.com/RustAudio/cpal): Cross-platform audio I/O in Rust.
- [tauri](https://tauri.app/): Framework for building desktop apps with web technologies (used in the frontend).

## Details

- Audio streaming is managed by the `make_stream` function in `audio_backend/devices/`, which sets up and runs the audio stream using `cpal`.
- The synthesis engine (in `synths/`) supports multiple waveforms and envelopes, and is designed for extensibility.
- The frontend GUI (see `frontend/`) allows users to control the synth in real time.

---
For more details, see the documentation in each subfolder.
