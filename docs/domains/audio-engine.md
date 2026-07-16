---
title: Audio Engine Domain
summary: Focused context for DSP, instruments, effects, rendering, and RT contracts.
status: current
updated: 2026-07-15
issues: [132, 133, 134, 135, 136, 137]
---

# Audio Engine Domain

## Read first

1. [Target system boundaries](../architecture/system-boundaries.md)
2. Owning GitHub issue
3. The narrow current code entry point below

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

- `dsp/src/lib.rs`
- `dsp/src/synth_infra/`
- `dsp/src/instruments/`
- `dsp/src/effects/`
- `audio_backend/src/audio_processor/mod.rs`
- `audio_backend/src/player/tracker_synthesizer.rs`

Do not read every effect/instrument implementation unless the issue targets it.

## Current hazards already tracked

Fixed 4096-frame buffers, tracker-coupled rendering, dynamic deallocation/collections in RT commands, unbounded queue draining, incomplete polyphonic note-off/stealing, and no-op effect graph commands. See the linked M1 issues rather than creating local workarounds.

## Verify

Use hardware-free tests first: `cargo test --workspace --all-targets`. Audio-device examples are manual validation, not baseline tests.
