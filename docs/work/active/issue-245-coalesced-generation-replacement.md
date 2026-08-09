---
title: "Task Packet — Issue 245: Coalesced generation replacement"
summary: Add stable-ID generation replacement, block-boundary swap, NRT retirement, and shutdown coverage.
status: current
updated: 2026-08-09
issue: 245
---

# Task Packet — Issue 245: Coalesced generation replacement

## Identity

- Issue: [#245](https://github.com/jpalvarezl/blight-synth/issues/245)
- Owner/status: jpalvarezl / in-progress
- Branch/worktree: `issue/245-coalesced-generation-replacement` / `/Users/jpalvarezl/code/blight-245`
- Base: `main` / `dae80ec`

## Goal

Extend the initial static generation from #244 with stable-ID replacement/replay, callback-safe whole-generation swap, displaced-owner retirement, and deterministic shutdown.

## Scope / reviewability

One lifecycle transition concept, generally 500–800 meaningful lines including tests; pause/re-split above ~1,000.

In scope: close/rebind/replay, missing-ID diagnostics, prepared replacement command/handoff, block-boundary Engine swap, RetiredState/pending fallback, generation transition/confirmation, stale handles, shutdown/ring saturation stress.

Out of scope: OSC parsing/protocol (#216), generic UI desired-state stores, parameter schema changes.

## Plan

- [x] Define NRT replacement preparation/facade transition.
- [x] Add bounded callback swap and retirement.
- [x] Add stable-ID replay/missing/stale/confirmation behavior.
- [x] Prove ring-full/shutdown/zero-heap and run full gates/review.

## Acceptance evidence

- [x] NRT closes the old generation, allocates a monotonic generation, compiles a complete manifest/table/store/binding/snapshot, and rebinds desired stable IDs.
- [x] The existing bounded command ring installs one whole generation at a callback block boundary; the existing retirement ring/pending fallback reclaims the displaced `RetiredState::Prepared` owner.
- [x] Old facades remain closed and physically isolated; transition, removed/missing IDs, pending/applied confirmation, and deterministic desired replay are NRT-visible.
- [x] Ring saturation pauses later replacement, repeated replacement progresses after NRT drain, and shutdown disconnects outliving facades while reclaiming callback/queued/retired ownership on NRT.
- [x] Hardware-free tests cover replacement/rebind/replay, stale isolation, block-start seed application, prior confirmation, saturation/fairness, shutdown, and zero callback heap activity.

## Touched paths

- `engine/src/{lib.rs,coalesced_bindings.rs}`
- `audio_backend/src/{commands.rs,player/,device_host/}`
- `audio_backend/tests/{device_host_parameters.rs,rt_player_allocations.rs}`
- `docs/{architecture/realtime-contract.md,domains/audio-engine.md,work/active/issue-245-coalesced-generation-replacement.md}`

## Handoff

- Completed: implementation, focused/full verification, docs, independent review.
- Remaining: handoff only; no push/PR/status change requested.
