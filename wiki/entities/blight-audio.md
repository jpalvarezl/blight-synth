---
tags: [module, audio, nrt]
sources: []
last-updated: 2025-05-11
source-file: audio_backend/src/audio_frontend/blight_audio.rs
source-sha: a02f14187b11b4bd5b3d54d1cdaaf8260171c840
source-mtime: 1781291635201
last-synced: 2026-06-12
---
# BlightAudio

Public-facing audio backend API; lives in the NRT (non-realtime) world. Defined in `audio_frontend/mod.rs`, impl in `audio_frontend/blight_audio.rs`.

## Fields

`command_tx: HeapProd<Command>`, `instrument_factory`, `voice_factory`, `resource_manager`, `effect_factory`, `meter: Arc<MeterState>`, `_stream: cpal::Stream`.

## Construction

`new()` / `with_song(song)`:
- cpal default host/device; SPSC ring buffer (`SharedRb`, capacity 1024) split into `command_tx`/`command_rx`.
- builds `AudioProcessor` (seeded with `Arc<MeterState>`), moves it into the cpal output callback, `stream.play()`.

## Accessors

- `send_command(Command)` — push to the audio thread.
- factory getters (`get_effect_factory`, `get_instrument_factory`, …).
- `meter_state() -> Arc<MeterState>` — cheap `Arc` clone; read levels via `MeterState::take_levels` (see [[entities/meter-state]]).
