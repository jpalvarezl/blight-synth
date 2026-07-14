---
title: Current Product Specification
summary: Concise statement of current commitments, optional goals, and open composition questions.
status: current
updated: 2026-07-14
issues: [129, 144]
---

# Current Product Specification

## Primary goal

Build a modular real-time Rust sound engine and a standalone experimental composition environment with a Svelte/TypeScript interface.

The project should become a focused, personal composition instrument—not a general-purpose DAW or a feature-for-feature clone of Renoise or ORCA.

## Committed direction

- The Rust audio engine is independent from audio devices, networking, filesystems, UI frameworks, and composition document types.
- Composition runtimes produce timestamped events through a replaceable event-source boundary.
- The current tracker remains a supported event source and regression fixture.
- Instruments, effects, parameters, routing, and state use versioned shared definitions.
- The standalone application is the primary product.
- The release frontend uses compiled Svelte/TypeScript assets behind a host-neutral client boundary.

## Deliberately open

- Final composition model: traditional tracker, ORCA-like character grid, hybrid, or another experiment.
- Exact desktop shell.
- Whether generative runtimes execute with fixed-memory RT control-rate work or deterministic non-RT lookahead.
- Detailed external MIDI/OSC workflow.

Issue [#113](https://github.com/jpalvarezl/blight-synth/issues/113) owns prototype-driven composition exploration.

## Optional future goal

VST3, desktop AU, and AUv3 are optional. If pursued, a plugin hosts the complete composition engine/project rather than packaging each current instrument as a separate product.

Plugin work must not dictate success or block the standalone M0–M3 roadmap. See [ADR 0001](../decisions/0001-product-and-host-priorities.md).

## Definition of modular

A new composition runtime changes no DSP, mixer, device, or optional plugin code. A new generator/effect needs one DSP implementation, one versioned registry definition, shared parameter descriptors, tests, and optional custom UI—not duplicated protocol or frontend parameter tables.
