---
title: "Task Packet — Issue 242: Portable state model"
summary: Implement ADR 0008's portable state envelope, canonical bytes, tracker round-trip, and migration fixtures.
status: current
updated: 2026-08-08
issue: 242
---

# Task Packet — Issue 242: Portable state model

## Identity

- Issue: [#242](https://github.com/jpalvarezl/blight-synth/issues/242)
- Owner/status: jpalvarezl / in-progress
- Branch/worktree: `issue/242-portable-state-model` / `/Users/jpalvarezl/code/blight-242`
- Base: `main` / pending packet commit

## Goal

Implement ADR 0008's host-neutral serializable state and deterministic migration/diagnostics without live Engine restore.

## Scope / reviewability

One data-model/migration concept, generally 500–800 meaningful lines including fixtures/tests; pause and re-split above ~1,000.

In scope: PortableStateV1 envelope, tagged tracker/future composition and routing payloads, node definitions, normalized stable-ID parameters, digest asset references, canonical JSON bytes, current tracker import/round-trip, prior-version migration, unknown/corrupt/missing diagnostics with source preservation.

Out of scope: Engine preparation/install/RT swap (#243), filesystem/JUCE adapters, routing redesign.

## Plan

- [ ] Define envelope/types and canonical bytes.
- [ ] Implement tracker import/round-trip and migration fixture.
- [ ] Add structured diagnostics/source preservation.
- [ ] Run full gates/review and prepare PR.

## Handoff

- Completed: claimed/packet.
- Remaining: implementation/PR.
