---
title: "Task Packet — Issue 210: Versioned node registry"
summary: Active context for versioned instrument/effect definitions and the built-in NRT registry.
status: current
updated: 2026-08-04
issue: 210
---

# Task Packet — Issue 210: Versioned node registry

## Identity

- Issue: [#210](https://github.com/jpalvarezl/blight-synth/issues/210)
- Owner: jpalvarezl
- Status: in-progress
- Branch: `issue/210-versioned-node-registry`
- Worktree: `/Users/jpalvarezl/code/blight-210`
- Base branch/SHA: `main` / `072ade3`
- Head: branch tip at handoff
- Last handoff: 2026-08-04

## Goal

Define stable kind IDs, versioned serializable instrument/effect definitions, and an NRT-only built-in registry/factory over the typed IDs from #209.

## Read first

1. [Audio engine domain](../../domains/audio-engine.md)
2. [System boundaries](../../architecture/system-boundaries.md) and [ADR 0004](../../decisions/0004-parameter-manifest.md)
3. `dsp/src/factories/`, sequencer instrument/effect models, and audio-backend hydration

## Dependencies and blockers

- Depends on: #209 (merged)
- Blocks: #211 and parent #135
- Current blocker: none

## Scope

In scope: versioned definitions, stable kind IDs, ordered same-kind effect instances, built-in NRT registry/factory, unknown kind/version diagnostics, deterministic JSON/compatibility tests.

Out of scope: current tracker hydration migration (#211), full engine snapshots (#138), routing (#136), runtime third-party modules.

## Ownership and touch set

Expected: a narrow definitions/registry module or crate, DSP factory adapters, focused tests/docs, this packet. #213 owns only param_manifest coalesced storage and must not edit node definitions.

## Plan

- [ ] Inventory built-in kinds/current serialized shapes.
- [ ] Define versioned schemas and compatibility errors.
- [ ] Implement NRT registry/factory.
- [ ] Test same-kind instances, round-trip, unknown kinds/versions, and run full gates.

## Verification

- [ ] focused definition/registry tests
- [ ] workspace/golden/strict Clippy/host-free tests
- [ ] fmt, architecture, RT logging, docs/reconcile checks

## Handoff

- Completed: claimed and packet created.
- Remaining: implementation through PR.
- Known risks: avoid duplicating #211 hydration and #138 state schemas.
- Next: inventory factory/model kinds.
