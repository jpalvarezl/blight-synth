---
title: Architecture Decision Index
summary: Durable decisions with status, rationale, and supersession history.
status: current
updated: 2026-08-08
---

# Architecture Decision Records

ADRs record decisions that affect multiple domains or constrain future work. They are not task logs.

## Process

1. Copy the [ADR template](../templates/adr.md).
2. Use the next four-digit number and a short kebab-case name.
3. Mark it `proposed` while discussion is open.
4. Link the deciding GitHub issue.
5. Once accepted, preserve the text. Add a compatible decision with a new ADR whose `amends` field points to the earlier ADR; reverse it with a new ADR whose `supersedes` field points to the old one.

## Decisions

| ADR | Status | Decision |
|---|---|---|
| [0001](0001-product-and-host-priorities.md) | Accepted | Standalone experimental composition is primary; composition UX remains open; plugins are optional complete-engine hosts. |
| [0002](0002-device-host-osc-split.md) | Proposed | Split a reusable in-process `device_host` from the OSC standalone process adapter; OSC is a transport over the shared `BlightAudio` control boundary, not an owner of device-host semantics. |
| [0003](0003-event-source-contract.md) | Proposed | Host/producer-side clock adapters fill bounded, already-offset event blocks for the engine; current pull, NRT lookahead, ordering, and fail-closed recovery are explicit. |
| [0004](0004-parameter-manifest.md) | Accepted | One serializable parameter manifest is the single source of truth for parameter metadata across Rust DSP/engine, project state, OSC, JUCE/APVTS, and Svelte; a bounded string-free runtime lookup keyed by stable ID serves the audio thread. |
| [0005](0005-coalesced-parameter-publication.md) | Accepted; amended by 0007 | Continuous parameters use a generation-bound normalized MPSC atomic store with bounded RT mapping/application and applied-target confirmation. |
| [0006](0006-fixed-quantum-smoothing-delivery.md) | Superseded by 0007 | Rejected fixed-quantum Engine smoothing design retained for history. |
| [0007](0007-simplified-coalesced-application.md) | Accepted | Keep latest-value coalescing, map/apply targets once per block, and defer smoothing to explicit DSP-local implementations when product need is demonstrated. |
| [0008](0008-portable-state-envelope.md) | Accepted | Use one canonical versioned project envelope with tagged composition/routing payloads, source-preserving migration diagnostics, and NRT-prepared block-boundary restore. |
