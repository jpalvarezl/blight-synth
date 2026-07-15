---
title: ADR 0001 — Product and Host Priorities
summary: Prioritize a standalone experimental composition environment and keep complete-engine plugins optional.
status: accepted
updated: 2026-07-14
issues: [129, 144]
supersedes: []
---

# ADR 0001 — Product and Host Priorities

## Context

Blight Synth began as a tracker and later explored Svelte, OSC, JUCE, VST3/AU, and AUv3 plans. Mature trackers already cover the conventional workflow well, while the current instruments alone are not a compelling initial plugin product. Committing early to a tracker clone or plugin runtime would constrain experimentation without proving personal value.

## Decision

The primary product is a standalone experimental composition environment over a modular Rust audio engine and Svelte/TypeScript UI.

The final composition interaction remains open. The current tracker is retained as a supported event source and regression fixture. An ORCA-like character grid, hybrid, or another focused model may become the production interface after prototype-driven evaluation.

VST3, desktop AU, and AUv3 are optional downstream goals. If pursued, a plugin hosts one complete composition/audio engine and project—not separate wrappers for the current instruments. Release plugin UI uses embedded compiled assets and an in-process host bridge; it does not spawn Bun/Vite or a standalone DSP process.

## Consequences

### Positive

- M0/M1 engine work proceeds without waiting for UI selection.
- Composition experiments can share instruments, effects, parameters, state, and rendering.
- Standalone usefulness determines whether plugin investment is justified.
- Existing tracker work remains valuable without defining the product.

### Costs

- A composition event-source contract and extensible state payload are required.
- M3 begins with interaction/runtime spikes rather than a direct tracker port.
- Optional plugin host-sync and automation details remain deferred.

## Detailed topology

[Product and Host Topology](../architecture/product-topology.md) defines the accepted standalone and optional-plugin component diagrams, target matrix, state/parameter authority, runtime constraints, and explicit deferred variants.

## Guardrails

- Audio engine crates do not depend on tracker document types.
- Plugin requirements do not block M0–M3.
- Revisit this decision only with a superseding ADR based on working prototypes or product evidence.
