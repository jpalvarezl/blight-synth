---
title: "Task Packet — Issue 223: Smoothing delivery contract"
summary: Decide how engine-owned smoothing reaches block-oriented DSP before implementation.
status: current
updated: 2026-08-06
issue: 223
---

# Task Packet — Issue 223: Smoothing delivery contract

## Identity

- Issue: [#223](https://github.com/jpalvarezl/blight-synth/issues/223)
- Owner: jpalvarezl
- Status: in-progress
- Branch: `issue/223-smoothing-delivery-contract`
- Worktree: `/Users/jpalvarezl/code/blight-223`
- Base: `main` / `dcfd9fb`

## Goal

Select one bounded deterministic smoother-to-DSP delivery mechanism and define precise linear/exponential/reset semantics before #224 code.

## Read first

1. [ADR 0005](../../decisions/0005-coalesced-parameter-publication.md)
2. [RT contract](../../architecture/realtime-contract.md)
3. current DSP `set_parameter`/internal smoother implementations and engine process entry points

## Scope / review budget

- Contract-only PR, preferably under 150 changed lines; no production smoother.
- Decide delivery quantum/API, partition phase, curve semantics, reset, duplicate DSP smoothing migration, and both engine process entries.
- No target binding/store integration or node migration.

Expected paths: additive ADR 0006, decision/audio routing, and this packet. #221 code is disjoint.

## Plan

- [x] Inventory current scalar effect setters, reverb's per-sample mix smoother, and both public engine process paths.
- [x] Compare block-rate, fixed quantum, and ramp-aware alternatives.
- [x] Accept ADR 0006's absolute 16-frame scalar quantum and exact curve/reset/migration semantics.
- [x] Make #224's primitive API and acceptance executable.
- [x] Run docs checks and independent decision review.

## Verification

- [x] `python3 scripts/docs/check_docs.py`
- [x] `python3 scripts/docs/reconcile_work.py --check`
- [x] independent decision review (fixed-quantum timing, quality, cost, and ADR-history challenge)

## Handoff

- Completed: ADR 0006 selects a fixed 16-frame scalar quantum; #224 now names the closed-form primitive API.
- Remaining: #224 implementation and #214 binding/render integration.
- Risk: #214 must measure worst-case setter cost and staircase quality; ramp-aware delivery is the explicit fallback.
