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
- focused architecture documentation only if implementation resolves an open mechanism
- `docs/work/active/issue-201-timestamped-engine-events.md`
- generated `docs/work/burndown.md`

Shared contracts/schemas touched: public engine event schema and event-consuming process API; this issue is their designated owner.

Potential parallel conflicts: #202 is safe in `sequencer/src/timing/`; #101, #132, #203, and #204 must consume rather than independently alter this event surface.

## Plan

- [x] Characterize the current imperative note/process path and parameter runtime binding.
- [x] Specify the first note/recovery event payload, order key, validation status, and process signature through red tests.
- [ ] Implement event validation/application and offset-segmented rendering.
- [ ] Add focused semantic, malformed-input, deterministic-render, and RT-allocation tests.
- [ ] Run focused and workspace verification; update contract routing only for durable resolved details.

## Progress and decisions

- 2026-07-26 — Claimed #201 and established it as the sole owner of the public engine event/process contract.
- 2026-07-26 — Tutoring mode: assistant writes executable contract tests; owner writes production implementation; review proceeds in small green checkpoints.
- 2026-07-26 — First red tranche fixes `EventProducerId`, `TimestampedEvent`, note/recovery `EngineEvent`, `EventProcessError`, and `Engine::process_with_events`; canonical order is offset → semantic precedence (recovery, release, attack) → producer → sequence. Parameters remain a later tranche.

## Verification

- [ ] `cargo test -p engine --all-targets`
- [ ] focused strict RT allocation tests
- [ ] `cargo test --workspace --all-targets`
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] `cargo fmt --all -- --check`
- [ ] `python3 scripts/docs/reconcile_work.py --check`
- [ ] `python3 scripts/docs/check_docs.py`

## Handoff

- Completed: issue claimed; branch/worktree and focused packet created.
- Remaining: design, implementation, tests, and verification.
- Known failures/risks: event control binding must use #121 runtime identities without pulling NRT strings into RT; process API must leave #132 lifecycle room.
- Next smallest action: map the existing note/process and parameter-binding types, then write the event contract examples.
- Files a new agent should read next: this packet, `engine/src/lib.rs`, ADR 0003, and the RT contract.
