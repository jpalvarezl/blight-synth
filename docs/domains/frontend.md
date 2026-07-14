---
title: Frontend Domain
summary: Focused context for the host-neutral Svelte/TypeScript composition UI.
status: current
updated: 2026-07-14
issues: [105, 106, 107, 108, 109, 110, 111, 112, 113]
---

# Frontend Domain

## Read first

1. [Current product specification](../spec/current-product.md)
2. Issue [#113](https://github.com/jpalvarezl/blight-synth/issues/113) for composition UX
3. Assigned M3 issue

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

Do not port tracker screens by default. Prototype a valuable tracker interaction, an ORCA-like character grid, and a hybrid before selecting the production composition interface.
