---
title: Task Packet — Issue 201: Timestamped engine events
summary: Active context for the canonical bounded engine event schema and offset-segmented rendering.
status: current
updated: 2026-07-26
issue: 201
related-issues: [134, 132, 101, 203]
---

# Task Packet — Issue 201: Timestamped engine events

## Identity

- Issue: [#201](https://github.com/jpalvarezl/blight-synth/issues/201)
- Owner: @jpalvarezl
- Status: in-progress
- Branch: `jpalvarezl/feature/issue_201_timestamp_event`
- Worktree: `/Users/jpalvarezl/code/blight-synth`
- Base branch/SHA: `main` / `cccced5ac5e28173a74f893e3a68e09c74a882b8`
- Head SHA: `cccced5ac5e28173a74f893e3a68e09c74a882b8`
- Last handoff: 2026-07-26

## Goal

Define the one canonical engine-facing current-block event contract and make `Engine` apply already ordered events at exact sample offsets, as scoped by [#201](https://github.com/jpalvarezl/blight-synth/issues/201), without violating the accepted RT contract.

## Read first

1. [Audio engine domain](../../domains/audio-engine.md)
2. [ADR 0003 — Event-source contract](../../decisions/0003-event-source-contract.md)
3. [Real-time audio contract](../../architecture/realtime-contract.md)
4. `engine/src/lib.rs`, `engine/src/commands.rs`, and `engine/tests/rt_allocations.rs`

## Dependencies and blockers

- Depends on: none
- Blocks: #203, #204, and parent #134
- Current blocker: none

## Scope and non-goals

### In scope

- Canonical already-offset event and ordering types in `engine`.
- Compact validation/application status.
- Offset-segmented rendering over caller-provided buffers.
- Note/release, sample-event control binding, and global recovery application.
- RT allocation and deterministic ordering/render tests.

### Out of scope

- Scheduler admission/merge and producer-visible overflow (#203).
- Tracker timing and integration (#202/#204).
- Coalesced continuous controls (#101).
- Engine lifecycle/lookahead (#132), state generations/migrations (#138), and OSC.

## Ownership and touch set

Expected paths:

- `engine/Cargo.toml`
- `engine/src/events.rs`
- `engine/src/lib.rs`
- `engine/tests/rt_allocations.rs`
- `engine/tests/timestamped_events.rs`
- `scripts/check_architecture.py`
- `docs/architecture/crate-dependency-graph.md`
- `docs/architecture/event-source-contract.md`
- `docs/architecture/realtime-contract.md`
- `docs/domains/audio-engine.md`
- `docs/work/active/issue-201-timestamped-engine-events.md`
- generated `docs/work/burndown.md`

Shared contracts/schemas touched: public engine event schema and event-consuming process API; this issue is their designated owner.

Potential parallel conflicts: #202 is safe in `sequencer/src/timing/`; #101, #132, #203, and #204 must consume rather than independently alter this event surface.

## Plan

- [x] Characterize the current imperative note/process path and parameter runtime binding.
- [x] Specify the first note/recovery event payload, order key, validation status, and process signature through red tests.
- [x] Implement event validation/application and offset-segmented rendering.
- [x] Add focused semantic, malformed-input, deterministic-render, parameter-binding, and RT-allocation tests.
- [x] Run focused and workspace verification; update contract routing only for durable resolved details.

## Progress and decisions

- 2026-07-26 — Claimed #201 and established it as the sole owner of the public engine event/process contract.
- 2026-07-26 — Tutoring mode: assistant writes executable contract tests; owner writes production implementation; review proceeds in small green checkpoints.
- 2026-07-26 — First red tranche fixed `EventProducerId`, `TimestampedEvent`, note/recovery `EngineEvent`, `EventProcessError`, and `Engine::process_with_events`; canonical order is offset → semantic precedence → producer → sequence.
- 2026-07-26 — Implementation validates the complete slice before mutation, rejects non-increasing canonical keys, and segments existing DSP processing at event offsets without changing instrument interfaces.
- 2026-07-26 — `engine` now consumes `param_manifest`: NRT binding accepts only validated `SampleEvent` runtime parameters, retains the runtime key plus concrete effect/index target, and sends already-mapped engine values on RT. Coalesced/structural rates are rejected.
- 2026-07-26 — Architecture-checker feature expectations still described the pre-#190 monolithic `standalone` feature; because this issue must add the new `engine -> param_manifest` allowlist, the checker/page were synchronized to the already-merged `device-host`/`standalone-process` split rather than preserving a failing stale baseline.

## Verification

- [x] `cargo test -p engine --all-targets`
- [x] focused strict RT allocation tests (`prepared_timestamped_event_application_and_segmented_render_has_no_heap_activity`)
- [x] `cargo test --workspace --all-targets`
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] `cargo fmt --all -- --check`
- [x] `python3 scripts/check_architecture.py`
- [x] `python3 scripts/check_rt_logging.py`
- [x] `python3 scripts/docs/reconcile_work.py --check`
- [x] `python3 scripts/docs/check_docs.py`
- [x] independent final code review — APPROVE, no warnings

## Handoff

- Completed: event/order types, atomic validation, segmented rendering, note/recovery/sample-parameter application, focused semantics tests, zero-heap event audit, contract docs, all quality gates, and independent review.
- Remaining: commit/PR and issue closure; first-party host integration remains intentionally #203/#204.
- Known failures/risks: future block-size/latency-dependent DSP must preserve semantics across segment slices; #203 must use `EventOrderKey` rather than define a second comparator.
- Next smallest action: review the local diff, commit it, and open the #201 PR.
- Files a new agent should read next: this packet, `engine/src/lib.rs`, ADR 0003, and the RT contract.
