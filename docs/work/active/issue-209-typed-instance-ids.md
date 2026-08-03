---
title: "Task Packet — Issue 209: Typed instance IDs"
summary: Active context for replacing interchangeable DSP/engine identity aliases with typed newtypes.
status: current
updated: 2026-08-03
issue: 209
---

# Task Packet — Issue 209: Typed instance IDs

## Identity

- Issue: [#209](https://github.com/jpalvarezl/blight-synth/issues/209)
- Owner: jpalvarezl
- Status: in-progress
- Branch: `issue/209-typed-instance-ids`
- Worktree: `/Users/jpalvarezl/code/blight-209`
- Base branch/SHA: `main` / `2f251a3`
- Head: branch tip at handoff
- Last handoff: 2026-08-03

## Goal

Replace interchangeable instance/resource ID aliases with compact typed newtypes while preserving numeric project compatibility, deterministic ordering, and RT-safe use, as scoped by #209.

## Read first

1. [Audio engine domain](../../domains/audio-engine.md)
2. [System boundaries](../../architecture/system-boundaries.md) and [RT contract](../../architecture/realtime-contract.md)
3. `dsp/src/id.rs`, `engine/src/commands.rs`, `engine/src/events.rs`, factories and hydration call sites

## Dependencies and blockers

- Depends on: none
- Blocks: #210 and parent #135
- Current blocker: none

## Scope and non-goals

### In scope

- Typed IDs and migration of current engine/DSP/host/project call sites.
- Explicit raw access/conversion and persisted numeric compatibility.
- Focused type-safety, serde, workspace, golden, and RT tests.

### Out of scope

- Versioned node definition schemas/registry (#210), hydration migration (#211), routing (#136).

## Ownership and touch set

Expected paths: `dsp/src/id.rs`, engine/DSP command/factory APIs, audio-backend hydration/adapters/examples/tests, affected project model fields only when needed for compatibility, and this packet.

Shared contracts touched: public Rust ID types; #209 is the sole owner. #212 is documentation-only and must not change ID APIs.

Potential parallel conflicts: #179 and other ID consumers remain backlog until this contract lands.

## Plan

- [ ] Inventory aliases and persistence boundaries.
- [ ] Add compact typed IDs with explicit conversion/serde behavior.
- [ ] Migrate engine/DSP/audio-backend callers and tests.
- [ ] Run full verification and independent review.

## Progress and decisions

- 2026-08-03 — Split from #135 as the critical-path identity contract owner.

## Verification

- [ ] focused ID/serde/type-safety tests
- [ ] `cargo test --workspace --all-targets`
- [ ] strict all-feature and host-free Clippy/tests
- [ ] golden, architecture, RT logging, fmt, reconciliation, and docs checks

## Handoff

- Completed: issue claimed and packet created.
- Remaining: implementation through PR.
- Known risks: broad compile-time API migration; preserve numeric JSON compatibility exactly.
- Next smallest action: inventory every alias and persisted/raw conversion.
- Files a new agent should read next: this packet and `dsp/src/id.rs`.
