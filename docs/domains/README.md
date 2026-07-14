---
title: Domain Context Index
summary: Small context packets that route tasks to contracts and current code entry points.
status: current
updated: 2026-07-14
---

# Domain Context Index

Choose one primary domain for an issue. Read another only when the issue explicitly crosses the boundary.

- [Audio engine](audio-engine.md) — DSP, instruments, voices, effects, RT processing.
- [Composition](composition.md) — tracker and future generative/ORCA-like event sources.
- [Standalone host](standalone-host.md) — CPAL, OSC, process lifecycle, file/resource adapters.
- [Frontend](frontend.md) — Svelte/TypeScript, `EngineClient`, desktop shell.
- [Plugins](plugins.md) — optional VST3/AU/AUv3 and JUCE bridge.

Each page states what the domain owns, what it must not own, read-first contracts, current code entry points, and active issues.
