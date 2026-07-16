---
title: Standalone Host Domain
summary: Focused context for CPAL, OSC, process lifecycle, and project/resource adapters.
status: current
updated: 2026-07-15
issues: [104, 120, 122, 123, 139]
---

# Standalone Host Domain

## Read first

1. [Target system boundaries](../architecture/system-boundaries.md)
2. [OSC protocol snapshot](../osc-spec.md) when changing transport
3. Assigned M2 issue

## Owns

- CPAL device discovery, format/channel conversion, and callback adaptation.
- OSC/external control protocol and non-RT I/O.
- Standalone process lifecycle and diagnostics.
- Project/resource file loading and immutable engine handoff.
- Meter transport from shared engine telemetry.

## Must not own

- A second player/synthesizer implementation.
- DSP math or composition document semantics.
- UI component state.
- Optional plugin runtime behavior.

## Current code entry points

- `audio_backend/src/audio_frontend/blight_audio.rs`
- `audio_backend/src/audio_processor/mod.rs`
- `audio_backend/src/osc.rs`
- `audio_backend/src/bin/dsp-core.rs`
- `audio_backend/src/song_hydration.rs`
- `audio_backend/src/resources.rs`
- `audio_backend/src/meter.rs`

## Current status

The current host has a working initial OSC/song/gain/meter vertical slice. WAV and macOS DLS decoding plus `ResourceManager` live here as non-RT adapters; portable DSP retains only immutable sample data and processing. M2 migrates the host onto the host-independent engine and versioned protocol; do not duplicate it in frontend code.
