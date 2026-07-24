---
title: Standalone Host Domain
summary: Focused context for CPAL, OSC, process lifecycle, and project/resource adapters.
status: current
updated: 2026-07-24
issues: [104, 120, 122, 123, 139, 156, 161, 182, 190]
---

# Standalone Host Domain

## Read first

1. [Target system boundaries](../architecture/system-boundaries.md)
2. [Device host boundary contract (draft)](../architecture/device-host-boundary.md) and [ADR 0002 — device host vs OSC split](../decisions/0002-device-host-osc-split.md)
3. [OSC protocol snapshot](../osc-spec.md) when changing transport
4. Assigned M2 issue

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

Shared device host (feature `device-host`):

- `audio_backend/src/device_host/audio_frontend/blight_audio.rs`
- `audio_backend/src/device_host/audio_processor/mod.rs`
- `audio_backend/src/device_host/meter.rs`

OSC standalone-process adapter (feature `standalone-process`, depends on `device-host`):

- `audio_backend/src/standalone_process/osc.rs`
- `audio_backend/src/standalone_process/control_worker.rs`
- `audio_backend/src/bin/dsp-core.rs`

Shared, host-free:

- `audio_backend/src/song_hydration.rs`
- `audio_backend/src/resources.rs`

## Threading/runtime decision

Tokio is temporarily allowed only behind `audio_backend`'s `standalone-process` feature. The current-thread runtime owns UDP, metering cadence, and shutdown polling but never blocks on RT queue capacity. A dedicated NRT control worker owns the non-`Send` CPAL stream, factories/resources, song preparation, and ordered retry into the RT ring; the CPAL callback remains the only RT owner. M2 issue #161 removes Tokio and may collapse the I/O/control topology once protocol/lifecycle behavior is stable; OSC remains encoded with `rosc` independently of that runtime choice.

## Feature boundary

The device host and OSC adapter are split across two layered features (ADR 0002). `device-host` owns the optional CPAL and ring-buffer dependencies and compiles the shared in-process host (`BlightAudio`, the RT callback adapter, metering, factories); Rust clients such as the tracker depend only on it. `standalone-process` depends on `device-host` and additionally owns the OSC, logging, and Tokio dependencies plus the `dsp-core` binary and device/network examples. `standalone` is retained as a compatibility alias of `standalone-process`, so `default = ["standalone"]` is unchanged. `audio_backend --no-default-features` compiles neither layer and retains tracker composition, shared hydration, resources, and deterministic offline rendering without the standalone device/network modules.

## Current status

The standalone host has a working OSC/song/gain/meter vertical slice and delegates sound rendering to `engine`. WAV and macOS DLS decoding plus `ResourceManager` remain non-RT file adapters shared with offline rendering; portable DSP retains only immutable sample data and processing. M2 evolves the protocol/lifecycle without duplicating engine or frontend logic.
