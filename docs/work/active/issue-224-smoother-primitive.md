---
title: "Task Packet — Issue 224: Smoother primitive"
summary: Implement ADR 0006's standalone deterministic smoothing state.
status: current
updated: 2026-08-06
issue: 224
---

# Task Packet — Issue 224: Deterministic smoother primitive

## Identity

- Issue: [#224](https://github.com/jpalvarezl/blight-synth/issues/224)
- Owner: jpalvarezl
- Status: in-progress
- Branch/worktree: `issue/224-smoother-primitive` / `/Users/jpalvarezl/code/blight-224`
- Base: `main` / `060b11f`

## Goal

Implement the pure finite-state `None`/linear/exponential smoother selected by ADR 0006, without store, target, DSP, or Engine process integration.

## Scope and reviewability

One coherent math/state primitive. Aim for roughly 400–700 meaningful lines including exhaustive tests; re-split above ~1,000. No target IDs, mapping, coalesced drain, control-phase owner, DSP setter, or process wiring.

Expected paths: a narrow engine smoother module, focused tests/allocation audit, packet/generated docs.

## Plan

- [ ] Encode preparation/seed/target/elapsed APIs and errors.
- [ ] Implement exact linear/exponential/none semantics.
- [ ] Test partition equivalence, retarget, reset, invalid/edge behavior.
- [ ] Prove zero-allocation bounded advance and run full gates/review.

## Handoff

- Completed: claimed/packet.
- Remaining: implementation/PR.
- Risk: keep 16-frame control phase out of this primitive; later integration owns delivery.
