---
title: Standalone Host Domain
summary: Focused context for CPAL, OSC, process lifecycle, and project/resource adapters.
status: current
updated: 2026-07-19
issues: [104, 120, 122, 123, 139, 156, 161]
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

- `audio_backend/src/standalone/audio_frontend/blight_audio.rs`
- `audio_backend/src/standalone/audio_processor/mod.rs`
- `audio_backend/src/standalone/osc.rs`
- `audio_backend/src/bin/dsp-core.rs`
- `audio_backend/src/song_hydration.rs`
- `audio_backend/src/resources.rs`
- `audio_backend/src/standalone/meter.rs`

## Threading/runtime decision

Tokio is temporarily allowed only behind `audio_backend`'s `standalone` feature. It now uses the current-thread runtime, so the intentional steady-state model is one main/control thread plus the CPAL audio callback thread (excluding platform-owned threads). M2 issue #161 removes Tokio once protocol/lifecycle behavior is stable; OSC remains encoded with `rosc` independently of that runtime choice.

## Feature boundary

The default `standalone` feature owns optional CPAL, ring-buffer, OSC, logging, and Tokio dependencies plus the `dsp-core` binary and device/network examples. `audio_backend --no-default-features` retains tracker composition, shared hydration, resources, and deterministic offline rendering without compiling the standalone device/network modules.

## Current status

The standalone host has a working OSC/song/gain/meter vertical slice and delegates sound rendering to `engine`. WAV and macOS DLS decoding plus `ResourceManager` remain non-RT file adapters shared with offline rendering; portable DSP retains only immutable sample data and processing. M2 evolves the protocol/lifecycle without duplicating engine or frontend logic.
