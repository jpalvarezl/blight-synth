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
- Base branch/SHA: `main` / `fb67163`
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

Expected: `engine/src/coalesced_parameters.rs`, `engine/src/lib.rs`, focused `engine/tests/` store/model/allocation tests, `param_manifest/src/manifest.rs` validation/tests, `param_manifest/tests/`, the test-only Loom architecture allowlist, and narrow architecture/task-packet docs. #210 owns node definitions/registry only.

## Plan

- [x] Translate ADR invariants into compact public types.
- [x] Implement publication, bounded drain, generation close/reset.
- [x] Add manifest class/smoothing validation.
- [x] Add concurrency/eventual-latest/model and RT allocation tests.
- [x] Run full gates and review.

## Verification

- [x] param_manifest focused/model/allocation tests
- [x] workspace/strict all-feature Clippy/tests and host-free tests
- [x] fmt, architecture, RT logging, docs/reconcile checks

## Handoff

- Completed: implemented and independently reviewed the generation-bound packed atomic store, fixed 16-word/1,024-slot RT drain, compact statuses/counters/confirmation, close/reset/exhaustion behavior, lock-free target policy, manifest smoothing-class validation, Loom plus stress/capacity/zero-heap coverage, and narrow implemented-truth docs.
- Verification: `cargo test -p param_manifest`; `cargo test --workspace --all-targets`; strict `RUSTFLAGS='-D warnings' cargo test --workspace --all-targets --all-features`; all-feature workspace Clippy; host-free `audio_backend` tests/Clippy; release diagnostic compile-out; fmt, architecture, RT logging, docs, and live reconciliation checks.
- Remaining: no implementation work; PR creation/merge and #214–#216 integration are intentionally out of scope.
- Known risks: target mapping/smoothing and host installation are not yet wired; all `CoalescedParameterStore`/publisher owners must be retired and finally released on NRT as documented.
- Next: hand off the clean committed branch for PR creation by the coordinating agent.
