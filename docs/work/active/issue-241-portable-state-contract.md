---
title: "Task Packet — Issue 241: Portable state contract"
summary: Decide the minimal portable engine-state envelope, compatibility, and restore ownership.
status: current
updated: 2026-08-08
issue: 241
---

# Task Packet — Issue 241: Portable state contract

## Identity

- Issue: [#241](https://github.com/jpalvarezl/blight-synth/issues/241)
- Owner/status: jpalvarezl / in-progress
- Branch/worktree: `issue/241-portable-state-contract` / `/Users/jpalvarezl/code/blight-241`
- Base: `main` / `bfb701b60035660fcca6f75915cba2b176c59670`
- Head: final local commit (see handoff); no push/PR requested

## Goal

Accept the smallest M1 portable state envelope and compatibility/NRT-restore contract before implementation.

## Read first

1. [Composition domain](../../domains/composition.md)
2. [ADR 0008](../../decisions/0008-portable-state-envelope.md)
3. `sequencer/src/models/`, `sequencer/src/project/mod.rs`, `node_registry/src/`, `param_manifest/src/manifest.rs`, and `engine::RetiredState`

## Dependencies and boundaries

- Parent: #138; implementation successors: #242 then #243.
- #132 owns the final public Engine lifecycle; #136 owns richer routing topology.
- This issue changes persisted-state contracts only, not code or Cargo boundaries.

## Scope / non-goals

- In scope: tagged tracker/future composition payloads, versioned nodes, normalized values, fixed routing reference, portable assets, canonical bytes, migration/diagnostics, and prepared restore ownership.
- Out of scope: implementation, filesystem/JUCE adapters, routing redesign, and ephemeral DSP/voice/tail/device snapshots.

## Touch set

- `docs/decisions/0008-portable-state-envelope.md`
- `docs/decisions/README.md`
- `docs/architecture/{README.md,product-topology.md}`
- `docs/domains/composition.md`
- `docs/work/active/issue-241-portable-state-contract.md`
- Generated work docs only if reconciliation changes them.

## Plan

- [x] Inventory current tracker/project and reusable definition state.
- [x] Decide minimal envelope/non-goals and compatibility.
- [x] Define NRT/RT ownership and adapter boundaries.
- [x] Stabilize #138/#242/#243 and review docs.

## Progress and decisions

- Accepted [ADR 0008](../../decisions/0008-portable-state-envelope.md): RFC 8785 canonical V1, source-preserving failure, legacy `Song` import, and one uniquely owned prepared generation swapped/retired at a block boundary.
- Independent decision review found and the ADR fixed effect-order canonicalization, JCS numeric bounds, prior-format fixture wording, confirmation authority, failure-byte ownership, and unique mutable-generation retirement.
- #241 criteria are marked; #138/#242/#243 bodies now match the accepted slices. Workflow labels/state remain unchanged.

## Verification

- [x] `python3 scripts/docs/reconcile_work.py --fix-docs`
- [x] `python3 scripts/docs/reconcile_work.py --check`
- [x] `python3 scripts/docs/check_docs.py`

## Handoff

- Completed: contract, routing/index updates, successor issue boundaries, and independent decision review fixes.
- Remaining: #242/#243 implementation after this contract lands.
- Known risks: canonical JCS support and self-contained host asset packaging need implementation evidence; full-generation ownership replaces today's split song/Engine graph.
- Next smallest action: review and merge the contract, then transition #242 through normal GitHub workflow.
