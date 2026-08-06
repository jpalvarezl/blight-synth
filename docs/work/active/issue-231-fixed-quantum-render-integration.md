---
title: "Task Packet — Issue 231: Fixed-quantum render integration"
summary: Integrate coalesced bindings and ADR 0006 smoothing delivery into both Engine render entry points.
status: current
updated: 2026-08-06
issue: 231
---

# Task Packet — Issue 231: Fixed-quantum render integration

## Identity

- Issue: [#231](https://github.com/jpalvarezl/blight-synth/issues/231)
- Owner/status: jpalvarezl / in-progress
- Branch/worktree: `issue/231-fixed-quantum-render-integration` / `/Users/jpalvarezl/code/blight-231`
- Base: `main` / `b1129f0`

## Goal

Latch prepared coalesced bindings once per top-level process call and deliver smoother values through an absolute 16-frame control phase in both Engine process APIs, preserving timestamped event ordering and one smoothing owner.

## Read first

1. [Audio engine domain](../../domains/audio-engine.md)
2. [ADR 0005](../../decisions/0005-coalesced-parameter-publication.md)
3. [ADR 0006](../../decisions/0006-fixed-quantum-smoothing-delivery.md)
4. [RT contract](../../architecture/realtime-contract.md)
5. `engine/src/lib.rs`, `engine/src/coalesced_bindings.rs`, `engine/src/smoother.rs`, and the representative DSP parameter path

## Scope / reviewability

One coherent render-integration concept. Target roughly 500–800 meaningful lines including tests; 800–1,000 is acceptable for tightly coupled implementation/tests. Pause and re-split above ~1,000 meaningful lines.

In scope: one latch per top-level process call; private renderer; union of event/absolute 16-frame boundaries; smoother scalar delivery; representative duplicate DSP smoother removal/bypass; partition/order/setter-cost/RT tests.

Out of scope: device-host generation lifecycle (#215), OSC (#216), every DSP parameter, routing/lifecycle redesign.

## Ownership / touch set

Expected: `engine/src/lib.rs`, coalesced binding/render integration module, representative DSP effect setter/smoother, focused engine tests/allocation audit, minimal docs/packet. Public event ordering from #201 and atomic/store semantics from #213 must not change.

## Plan

- [ ] Define prepared state installation/access needed by Engine without host lifecycle ownership.
- [ ] Refactor both process APIs through one private renderer and one target latch.
- [ ] Segment at event and absolute control boundaries; deliver active smoother scalar values.
- [ ] Remove/bypass representative duplicate DSP smoothing.
- [ ] Prove partition/order/bounded setter/RT behavior and run full gates/review.

## Verification

- [ ] focused engine smoothing/render tests
- [ ] engine/workspace all-target tests and strict Clippy
- [ ] host-free tests/Clippy, RT allocation, goldens
- [ ] fmt, architecture, RT logging, docs/reconcile checks

## Handoff

- Completed: issue claimed and packet created.
- Remaining: implementation through PR.
- Risk: maintain one latch per public call and never recursively relatch from `process_with_events`.
