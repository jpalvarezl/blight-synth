---
title: Composition Domain
summary: Focused context for tracker and future generative composition runtimes.
status: current
updated: 2026-08-08
issues: [113, 134, 138, 145, 181]
---

# Composition Domain

## Read first

1. [Current product specification](../spec/current-product.md)
2. [Target system boundaries](../architecture/system-boundaries.md)
3. [ADR 0003 — Event-source contract](../decisions/0003-event-source-contract.md) and its [routing page](../architecture/event-source-contract.md)
4. [ADR 0008 — Portable state envelope](../decisions/0008-portable-state-envelope.md) for persisted composition/runtime state
5. Issue [#145](https://github.com/jpalvarezl/blight-synth/issues/145)
6. The assigned issue

## Owns

- Versioned composition documents and runtime/interpreter state.
- Clock/input interpretation and generation of timestamped engine events.
- Tracker semantics today; experimental character-grid/generative semantics later.
- Seeded randomness, replay, live-edit snapshots, and composition-specific migrations where promised.
- Abstract outbound events for host-routed MIDI/OSC side effects.

## Must not own

- Oscillators, effects, audio buffers/devices, UDP sockets, filesystem UI, or optional plugin APIs.
- Direct mutation of audio-thread object graphs.

## Current code entry points

- `sequencer/src/models/`
- `sequencer/src/timing/`
- `audio_backend/src/player/mod.rs`
- `tracker_gui/src/tabs/` (reference/debug UI, not required future product)

## Open direction

The final interaction model is intentionally undecided. The existing tracker remains one event source. [ADR 0003](../decisions/0003-event-source-contract.md) records the target boundary: the engine consumes bounded timestamped events and the tracker `Player` becomes one adapter, so a second runtime needs no DSP-engine changes. Issue [#113](https://github.com/jpalvarezl/blight-synth/issues/113) requires small tracker, ORCA-like, and hybrid spikes before production UI selection.

## Tracker host threading

The egui thread owns authored `Song` and display state but not `BlightAudio`, CPAL, factories, or RT-queue retry. A dedicated NRT tracker audio-control worker receives semantic requests, performs preparation/hydration, and preserves command FIFO order under RT-ring saturation. High-rate continuous controls still move to #101 coalescing rather than growing this transitional request stream.

## Key invariant

A second composition runtime must not require changes to DSP, mixer routing, device code, or optional plugin wrappers.
