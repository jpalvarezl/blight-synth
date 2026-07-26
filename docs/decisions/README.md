---
title: Architecture Decision Index
summary: Durable decisions with status, rationale, and supersession history.
status: current
updated: 2026-07-26
---

# Architecture Decision Records

ADRs record decisions that affect multiple domains or constrain future work. They are not task logs.

## Process

1. Copy the [ADR template](../templates/adr.md).
2. Use the next four-digit number and a short kebab-case name.
3. Mark it `proposed` while discussion is open.
4. Link the deciding GitHub issue.
5. Once accepted, preserve the text. Reverse it with a new ADR whose `supersedes` field points to the old one.

## Decisions

| ADR | Status | Decision |
|---|---|---|
| [0001](0001-product-and-host-priorities.md) | Accepted | Standalone experimental composition is primary; composition UX remains open; plugins are optional complete-engine hosts. |
| [0002](0002-device-host-osc-split.md) | Proposed | Split a reusable in-process `device_host` from the OSC standalone process adapter; OSC is a transport over the shared `BlightAudio` control boundary, not an owner of device-host semantics. |
| [0003](0003-event-source-contract.md) | Proposed | Host/producer-side clock adapters fill bounded, already-offset event blocks for the engine; current pull, NRT lookahead, ordering, and fail-closed recovery are explicit. |
| [0004](0004-parameter-manifest.md) | Proposed | One serializable parameter manifest is the single source of truth for parameter metadata across Rust DSP/engine, project state, OSC, JUCE/APVTS, and Svelte; a bounded string-free runtime lookup keyed by stable ID serves the audio thread. |
