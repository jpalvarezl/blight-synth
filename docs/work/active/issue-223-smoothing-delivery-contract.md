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
- Base: `main` / `6b0916b`

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

Expected paths: ADR 0005 amendment/routing and this packet. #221 code is disjoint.

## Plan

- [ ] Inventory current DSP setter/smoother constraints.
- [ ] Compare block-rate, fixed quantum, and ramp-aware alternatives.
- [ ] Record accepted precise semantics and update #224 if necessary.
- [ ] Run docs checks and decision review.

## Verification

- [ ] docs/reconciliation checks
- [ ] independent decision review

## Handoff

- Completed: claimed and packet created.
- Remaining: decision/PR.
- Risk: avoid designing a smoother primitive with no viable DSP delivery path.
