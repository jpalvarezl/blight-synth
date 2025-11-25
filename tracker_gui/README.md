# Tracker GUI

A lightweight tracker-style editor built with egui that drives the shared `audio_backend` in real time. The workflow, visuals, and hex-centric editing flow take heavy inspiration from the Dirtywave M8 tracker, so anyone familiar with the M8 can quickly map concepts and plan follow-up features.

## Run It

```bash
cargo run -p tracker_gui
```

## Architecture

```mermaid
flowchart LR
    UI[egui Widgets] -->|mutate Song + intents| Manager[Instrument Manager]
    Manager --> Sync[InstrumentSync]
    Manager --> BackendHelpers[backend::hydrate/send]
    Sync -->|queue| AudioMgr[AudioManager]
    BackendHelpers --> AudioMgr
    AudioMgr -->|dispatch()| Queue[BlightAudio command queue]
    Queue --> DSP[Audio thread / effects]
```

- **UI widgets** edit the `Song` model and emit intents (waveform changes, effect toggles, envelope tweaks).
- **Instrument Manager** turns those intents into backend requests, delegating hydration to `instrument_manager::backend` and rehydration scheduling to `InstrumentSync`.
- **AudioManager** owns the single `BlightAudio` instance. Every command flows through `dispatch`, ensuring all updates hit the backend queue instead of touching DSP state directly.

## Event Flow

1. A user gesture updates egui controls in `ui_components/`.
2. The editor mutates `Song` data and calls helpers such as `show_amp_envelope_editor` or `show_effect_panels`.
3. Those helpers notify `InstrumentManagerWindow`, which either sends immediate parameter messages (e.g., envelopes) or schedules a full rehydrate via `InstrumentSync`.
4. `InstrumentSync::apply_pending` asks `AudioManager` to `hydrate_instrument` for any IDs that changed.
5. `AudioManager::dispatch` wraps every backend call and pushes it onto the `BlightAudio` command queue running on the audio thread.

## Key Modules

- `app.rs`: boots egui and wires together the main tabs.
- `instrument_manager/`: instrument editing UI, backend hydration helpers, and sync queue.
- `ui_components/`: reusable widgets (effect panels, envelope editors, hex editors).
- `audio.rs`: the authoritative gateway to `audio_backend`, handling playback, hydration, and queued commands.

## Feature Snapshot

- Arrangement/Chain/Phrase editors with hexadecimal workflows.
- Live playback with loop + transport controls.
- Instrument-level effect editing (reverb/delay) and shared envelope widgets.
- JSON/Bincode import/export via the sequencer crate.
