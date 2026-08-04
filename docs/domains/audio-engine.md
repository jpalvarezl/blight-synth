---
title: Audio Engine Domain
summary: Focused context for DSP, instruments, effects, rendering, and RT contracts.
status: current
updated: 2026-08-03
issues: [101, 132, 133, 134, 135, 136, 137, 174, 186, 187, 188, 201, 203, 204, 212]
---

# Audio Engine Domain

## Read first

1. [Target system boundaries](../architecture/system-boundaries.md)
2. [Real-time audio contract](../architecture/realtime-contract.md) for callback-reachable changes
3. Owning GitHub issue
4. The narrow current code entry point below

Read parameter/state/composition pages only when the issue changes those contracts.

## Owns

- Host-independent audio rendering over caller-provided buffers.
- Instruments, voices, effects, routing, polyphony, parameter application.
- Timestamped event consumption and sample-offset rendering.
- RT-safe telemetry handoff.

## Must not own

- CPAL devices, UDP/OSC sockets, filesystem loading, UI state, Bun/JUCE lifecycle.
- Tracker `Song`/grid document semantics.
- WAV/DLS decoding, filesystem resource management, or platform asset discovery.
- External MIDI or network I/O.

## Current code entry points

- `engine/src/lib.rs`
- `engine/src/events.rs` (canonical current-block event schema/order/application contract)
- `dsp/src/lib.rs`
- `dsp/src/synth_infra/`
- `dsp/src/instruments/`
- `dsp/src/effects/`
- `audio_backend/src/standalone/audio_processor/mod.rs` (standalone callback adapter)
- `audio_backend/src/offline.rs`
- `audio_backend/src/player/tracker_engine_adapter.rs` (tracker-only adapter)

Do not read every effect/instrument implementation unless the issue targets it. Generic instrument/mixer rendering belongs to `engine`; tracker track caching and document interpretation remain in `audio_backend` until the composition adapter is extracted. End-to-end behavior is characterized by the [offline render contract](../architecture/offline-render-contract.md).

## Command ownership

- `engine::InstrumentCmd` targets one instrument and owns instrument creation, note/synth control, instrument/voice effect installation, and instrument effect parameters.
- `engine::MixerCmd` targets only the master mixer/effect pipeline and never carries an instrument ID.
- `audio_backend::SequencerCmd` owns song loading/playback; `TransportCmd` owns adapter transport.
- `audio_backend::Command` remains the compatibility queue envelope and re-exports engine command types.

These are transitional control-plane commands. The canonical engine-facing current-block API is `engine::TimestampedEvent` plus `Engine::process_with_events`, implemented by #201. `engine::BoundedEventAdmission` provides #203's NRT-prepared fixed-capacity multi-producer admission, canonical merge, fail-closed status, and reserved recovery slot. #204 routes first-party tracker rows and queued live note/release commands through that shared bounded lane. Continuous parameters follow [ADR 0005](../decisions/0005-coalesced-parameter-publication.md). #213 implements the host-independent generation-bound normalized atomic store, fixed RT drain, confirmation/failure status, and manifest class validation through the `engine` coalesced-parameter API; target binding, RT mapping, and engine-owned smoothing remain #214, with host lifecycle and OSC integration remaining #215/#216. Other structural and continuous commands remain transitional until their owning migration lands.

## Current hazards already tracked

Fixed 4096-frame buffers, tracker-coupled rendering, dynamic deallocation/collections in RT commands, unbounded queue draining, incomplete polyphonic note-off/stealing, and no-op effect graph commands. See the linked M1 issues rather than creating local workarounds.

## Verify

Use hardware-free tests first: `cargo test --workspace --all-targets`. Audio-device examples are manual validation, not baseline tests.
