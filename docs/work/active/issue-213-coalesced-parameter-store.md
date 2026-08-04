---
title: "Task Packet — Issue 213: Coalesced parameter store"
summary: Active context for the generation-bound atomic normalized-value store from ADR 0005.
status: current
updated: 2026-08-04
issue: 213
---

# Task Packet — Issue 213: Coalesced parameter store

## Identity

- Issue: [#213](https://github.com/jpalvarezl/blight-synth/issues/213)
- Owner: jpalvarezl
- Status: in-progress
- Branch: `issue/213-coalesced-parameter-store`
- Worktree: `/Users/jpalvarezl/code/blight-213`
- Base branch/SHA: `main` / `072ade3`
- Head: branch tip at handoff
- Last handoff: 2026-08-04

## Goal

Implement ADR 0005's generation-bound normalized atomic store and manifest validation without adding engine smoothing or host integration.

## Read first

1. [Audio engine domain](../../domains/audio-engine.md)
2. [ADR 0005](../../decisions/0005-coalesced-parameter-publication.md), [ADR 0004](../../decisions/0004-parameter-manifest.md), and [RT contract](../../architecture/realtime-contract.md)
3. `param_manifest/src/runtime.rs` and its allocation tests

## Dependencies and blockers

- Depends on: #212 (merged)
- Blocks: #214
- Current blocker: none

## Scope

In scope: fixed-capacity generation-bound packed atomic slots, dirty words, publication/drain/reset/close status, invalid/stale/pressure/revision observability, ControlCoalesced manifest validation, concurrency/model and zero-heap tests.

Out of scope: engine mapping/smoothing (#214), device-host lifecycle (#215), OSC (#216), typed node definitions (#210).

## Ownership and touch set

Expected: `param_manifest` runtime/store/validation modules and focused tests/docs. #210 owns node definitions/registry only.

## Plan

- [ ] Translate ADR invariants into compact public types.
- [ ] Implement publication, bounded drain, generation close/reset.
- [ ] Add manifest class/smoothing validation.
- [ ] Add concurrency/eventual-latest/model and RT allocation tests.
- [ ] Run full gates and review.

## Verification

- [ ] param_manifest focused/model/allocation tests
- [ ] workspace/strict Clippy/host-free tests
- [ ] fmt, architecture, RT logging, docs/reconcile checks

## Handoff

- Completed: claimed and packet created.
- Remaining: implementation through PR.
- Known risks: lock-free target support and revision exhaustion semantics must match ADR exactly.
- Next: encode ADR store state machine in tests/types.
