---
title: Frontend Domain
summary: Focused context for the host-neutral Svelte/TypeScript composition UI.
status: current
updated: 2026-08-10
issues: [105, 106, 107, 108, 109, 110, 112, 253, 254]
---

# Frontend Domain

## Read first

1. [NOW](../NOW.md)
2. [Current product specification](../spec/current-product.md)
3. The assigned child issue listed under current-slice parent [#253](https://github.com/jpalvarezl/blight-synth/issues/253)

## Owns

- Shared Svelte components and interaction state.
- A typed, mockable `EngineClient` boundary.
- Composition editing, generic parameter controls, transport, metering, and diagnostics.
- Production static assets suitable for a desktop shell and optional embedded plugin editor.

## Must not own

- Direct CPAL, UDP, child-process, filesystem, or JUCE calls inside browser components.
- DSP or composition-runtime execution.
- Duplicated parameter ranges/IDs outside the canonical manifest.

## Current code entry points

- `tracker_gui/` is the current debug/reference UI.
- No production Svelte workspace exists yet; it is tracked by issue [#105](https://github.com/jpalvarezl/blight-synth/issues/105).
- Retired Svelte scaffolding remains available in Git history but is not a second implementation tree.

## Open direction

The current slice deliberately avoids selecting the final composition interface. Build only the transport/gain/meter shell and typed client boundary in [NOW](../NOW.md); tracker/ORCA/hybrid prototyping remains deferred.
