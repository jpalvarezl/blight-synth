---
title: "Task Packet — Issue 221: Legacy definition adapter"
summary: Pure legacy tracker-model to versioned node-definition adaptation without hydration wiring.
status: current
updated: 2026-08-06
issue: 221
---

# Task Packet — Issue 221: Legacy definition adapter

## Identity

- Issue: [#221](https://github.com/jpalvarezl/blight-synth/issues/221)
- Owner: jpalvarezl
- Status: in-progress
- Branch: `issue/221-legacy-definition-adapter`
- Worktree: `/Users/jpalvarezl/code/blight-221`
- Base: `main` / `6b0916b`

## Goal

Add a pure adapter from current tracker models to versioned node definitions, with deterministic effect identity, while leaving active hydration unchanged.

## Read first

1. [Audio engine domain](../../domains/audio-engine.md)
2. `audio_backend/src/song_hydration.rs`, `sequencer/src/models/instruments.rs`, `node_registry/src/definitions.rs`

## Scope / review budget

- One adapter concept; target 200–400 changed lines including tests, hard stop/re-split near 500.
- Deterministic ordered effect IDs, DFAM implicit ladder placement, current kind/payload mapping, structured adapter errors.
- No hydration wiring, command/factory installation, UI, routing, or golden changes.

Expected paths: one narrow adapter module plus focused tests and this packet. #223 is docs-only and disjoint.

## Plan

- [ ] Inventory legacy-to-registry mapping and current clamp semantics.
- [ ] Implement pure adapter and deterministic IDs.
- [ ] Add multi-effect, DFAM, round-trip, invalid-value tests.
- [ ] Run focused/full gates and review.

## Verification

- [ ] focused adapter tests
- [ ] workspace/strict Clippy/host-free/golden gates
- [ ] fmt, architecture, docs/reconcile checks

## Handoff

- Completed: claimed and packet created.
- Remaining: implementation/PR.
- Risk: do not silently change hydration or sonic behavior.
