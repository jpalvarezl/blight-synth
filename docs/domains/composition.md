---
title: Composition Domain
summary: Focused context for tracker and future generative composition runtimes.
status: current
updated: 2026-07-14
issues: [113, 134, 138, 145]
---

# Composition Domain

## Read first

1. [Current product specification](../spec/current-product.md)
2. [Target system boundaries](../architecture/system-boundaries.md)
3. Issue [#145](https://github.com/jpalvarezl/blight-synth/issues/145)
4. The assigned issue

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

The final interaction model is intentionally undecided. The existing tracker remains one event source. Issue [#113](https://github.com/jpalvarezl/blight-synth/issues/113) requires small tracker, ORCA-like, and hybrid spikes before production UI selection.

## Key invariant

A second composition runtime must not require changes to DSP, mixer routing, device code, or optional plugin wrappers.
