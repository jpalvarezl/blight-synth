---
title: "Task Packet — Issue 245: Coalesced generation replacement"
summary: Add stable-ID generation replacement, block-boundary swap, NRT retirement, and shutdown coverage.
status: current
updated: 2026-08-08
issue: 245
---

# Task Packet — Issue 245: Coalesced generation replacement

## Identity

- Issue: [#245](https://github.com/jpalvarezl/blight-synth/issues/245)
- Owner/status: jpalvarezl / in-progress
- Branch/worktree: `issue/245-coalesced-generation-replacement` / `/Users/jpalvarezl/code/blight-245`
- Base: `main` / pending packet commit

## Goal

Extend the initial static generation from #244 with stable-ID replacement/replay, callback-safe whole-generation swap, displaced-owner retirement, and deterministic shutdown.

## Scope / reviewability

One lifecycle transition concept, generally 500–800 meaningful lines including tests; pause/re-split above ~1,000.

In scope: close/rebind/replay, missing-ID diagnostics, prepared replacement command/handoff, block-boundary Engine swap, RetiredState/pending fallback, generation transition/confirmation, stale handles, shutdown/ring saturation stress.

Out of scope: OSC parsing/protocol (#216), generic UI desired-state stores, parameter schema changes.

## Plan

- [ ] Define NRT replacement preparation/facade transition.
- [ ] Add bounded callback swap and retirement.
- [ ] Add stable-ID replay/missing/stale/confirmation behavior.
- [ ] Prove ring-full/shutdown/zero-heap and run full gates/review.

## Handoff

- Completed: claimed/packet.
- Remaining: implementation/PR.
