---
title: Optional Plugin Domain
summary: Focused context for deferred VST3, AU, and AUv3 work.
status: current
updated: 2026-07-14
issues: [114, 115, 116, 117, 118, 119, 140, 141, 142, 143]
---

# Optional Plugin Domain

## Priority

Plugins are optional and do not block M0–M3. Read this domain only for architecture boundary checks or when the optional milestones are explicitly activated.

## Read first

1. [ADR 0001](../decisions/0001-product-and-host-priorities.md)
2. [Target system boundaries](../architecture/system-boundaries.md)
3. Assigned optional milestone issue

## If pursued

- One plugin instance owns one complete composition/audio engine and project.
- Rust processes DAW-provided buffers/events; it never opens CPAL in plugin mode.
- Compiled Svelte assets are embedded; no Bun/Vite child process or fixed OSC ports.
- JUCE/APVTS exposes a stable fixed host surface and macros; dynamic node parameters stay in the engine manifest.
- Separate instrument-only products and multi-output are deferred.

## Current code entry points

No production plugin wrapper exists yet. Retired JUCE scaffolding remains available in Git history and must not be restored as a Bun/OSC-based plugin runtime.

AUv3 platform/UI/state feasibility is a separate optional decision because app-extension constraints differ from desktop AU/VST3.
