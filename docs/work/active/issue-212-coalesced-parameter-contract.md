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
- Base branch/SHA: `main` / `5ba3241`
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

Touched paths: `docs/decisions/0004-parameter-manifest.md`, additive accepted ADR 0005, RT/architecture/domain/OSC routing, #101/#144/#212/#213–#216 GitHub metadata, this packet, and generated burndown.

Shared contracts touched: coalesced parameter traffic class only; #212 is owner. #209 changes Rust ID APIs only.

Potential parallel conflicts: none with #209.

## Plan

- [x] Reconcile current ADR/code/host producer truth.
- [x] Decide publication and lifecycle semantics with explicit memory-order guarantees.
- [x] Define adapter-visible state and error/reset behavior.
- [x] Create/link sized child issues and promote the first safe leaf.
- [x] Run docs/reconciliation checks and independent decision review.

## Progress and decisions

- 2026-08-03 — Split from #101 because ADR 0004 said NRT adapter mapping while engine-owned smoothing required one coherent application boundary.
- 2026-08-03 — Accepted additive ADR 0005: the first-party host serializes one writer, the shared generation-bound store supports future NRT MPSC publishers, packed revision/value atomics publish normalized values, dirty Release RMW / Acquire swap guarantees eventual latest after quiescence, RT owns mapping and engine smoothing, and applied confirmation means target latched rather than ramp settled.
- 2026-08-03 — Replacement/reset prepares a non-reused generation, rebinds by stable `ParameterId`, rejects/contains stale keys, seeds every coalesced target, and retires the complete old state to NRT. `SampleEvent` remains mapped on NRT and bypasses smoothing.
- 2026-08-03 — Created #213–#216 with non-overlapping store, engine application, device-host lifecycle, and OSC/protocol ownership. Only dependency-free #213 is `status:ready`; #214–#216 remain blocked and all are unassigned.
- 2026-08-03 — Independent review found replacement quiescence, superseded-pending termination, release-sequence wording, capacity, retirement fallback, and smoothing validation underspecified. ADR 0005 now uses non-waiting close/recheck, packed per-slot revisions, an explicit 16,384-key/1,024-coalesced cap, preallocated retirement fallback, a model-test requirement, and a named manifest validation change.
- 2026-08-03 — Independent re-review found no atomic/lifecycle blocker and requested two consistency fixes: compact coalesced slot indexing now makes the exact scan cap 16 dirty words, and implemented ADR 0004 is promoted from stale Proposed status to Accepted before accepted ADR 0005 amends it.
- 2026-08-03 — Complete-diff review found one REVISE contradiction: ADR 0004 described ADR 0005's new smoothing/class validation as implemented. ADR 0004 now explicitly records current code behavior and defers that enforcement to #213; the ADR process/template now documents additive `amends` metadata.

## Verification

- [x] `python3 scripts/docs/reconcile_work.py --check`
- [x] `python3 scripts/docs/check_docs.py`
- [x] independent decision review (REVISE findings addressed in ADR 0005)
- [x] `python3 scripts/docs/sync_roadmap.py --check`

## Handoff

- Completed: accepted decision and ADR 0004 reconciliation; docs routing; #101 split/link/status metadata; roadmap links; generated dashboard; independent review and checks.
- Remaining: integrate this local commit through the repository's normal review path, then close #212. No push or PR was requested in this task.
- Known risks: the packed `AtomicU64` protocol requires lock-free target support and a Loom/equivalent model test; the 1,024 active-coalesced callback cap needs measurement; applied confirmation is target-latched, not smoothing-settled; the `/param/echo` semantic migration is deliberately isolated in #216.
- Next smallest action: review/merge this docs-focused commit, close #212, then claim unassigned ready leaf #213 separately if desired.
- Files a new agent should read next: this packet, ADR 0005, ADR 0004, RT contract, and #213.
