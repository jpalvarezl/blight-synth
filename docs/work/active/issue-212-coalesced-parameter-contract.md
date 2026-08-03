---
title: "Task Packet — Issue 212: Coalesced parameter contract"
summary: Active context for deciding ownership, publication, lifecycle, and host semantics before implementing #101.
status: current
updated: 2026-08-03
issue: 212
---

# Task Packet — Issue 212: Coalesced parameter contract

## Identity

- Issue: [#212](https://github.com/jpalvarezl/blight-synth/issues/212)
- Owner: jpalvarezl
- Status: in-progress
- Branch: `issue/212-coalesced-parameter-contract`
- Worktree: `/Users/jpalvarezl/code/blight-212`
- Base branch/SHA: `main` / `2f251a3`
- Head: branch tip at handoff
- Last handoff: 2026-08-03

## Goal

Resolve the coalesced-parameter producer, value-ownership, dirty publication, prepared-table lifecycle, and host confirmation contract, reconcile ADR 0004, and split #101 into sized implementation leaves.

## Read first

1. [Audio engine domain](../../domains/audio-engine.md)
2. [ADR 0004 — Parameter manifest](../../decisions/0004-parameter-manifest.md)
3. [RT contract](../../architecture/realtime-contract.md)
4. `param_manifest/src/runtime.rs`, current OSC parameter path, and #201 sample-event binding

## Dependencies and blockers

- Depends on: #121/#133 (merged)
- Blocks: parent #101 implementation
- Current blocker: none

## Scope and non-goals

### In scope

- Accepted decision/amendment and implementation issue split.
- Producer cardinality/memory ordering, normalized-vs-engine values, mapping/smoothing owner, RuntimeParamKey/table lifecycle, dirty/reset/error semantics, and desired/pending/confirmed behavior.

### Out of scope

- Store implementation or OSC migration.
- Typed ID APIs owned by #209.

## Ownership and touch set

Expected paths: `docs/decisions/0004-parameter-manifest.md` or a superseding/additive ADR, RT/event contract routing, #101/#144 GitHub issue metadata, this packet, and generated burndown.

Shared contracts touched: coalesced parameter traffic class only; #212 is owner. #209 changes Rust ID APIs only.

Potential parallel conflicts: none with #209.

## Plan

- [ ] Reconcile current ADR/code/host producer truth.
- [ ] Decide publication and lifecycle semantics with explicit memory-order guarantees.
- [ ] Define adapter-visible state and error/reset behavior.
- [ ] Create/link sized child issues and promote the first safe leaf.
- [ ] Run docs/reconciliation checks and review.

## Progress and decisions

- 2026-08-03 — Split from #101 because ADR 0004 currently says NRT mapping while the issue language could imply engine mapping; implementation remains blocked until ownership is explicit.

## Verification

- [ ] `python3 scripts/docs/reconcile_work.py --check`
- [ ] `python3 scripts/docs/check_docs.py`
- [ ] independent decision review

## Handoff

- Completed: issue claimed and packet created.
- Remaining: contract decision, child issues, review, PR.
- Known risks: avoid speculative MPSC complexity while preserving future APVTS/MIDI adapter viability.
- Next smallest action: map current producer threads and value conversions.
- Files a new agent should read next: this packet, ADR 0004, RT contract, `param_manifest/src/runtime.rs`.
