---
title: "Task Packet — Issue 204: Tracker/live timestamped event integration"
summary: Active implementation and handoff context for first-party tracker and live playback through the bounded timestamped event path.
status: current
updated: 2026-08-02
issue: 204
---

# Task Packet — Issue 204: Tracker/live timestamped event integration

## Identity

- Issue: [#204](https://github.com/jpalvarezl/blight-synth/issues/204)
- Owner: jpalvarezl
- Status: in-progress
- Branch: `issue/204-tracker-live-event-integration`
- Worktree: `/Users/jpalvarezl/code/blight-204`
- Base branch/SHA: `main` / `4698f13`
- Head: branch tip at handoff
- Last handoff: 2026-08-02

## Goal

Adapt the current tracker and live audition path to produce bounded current-block `TimestampedEvent`s, feed them through `BoundedEventAdmission`, and render through `Engine::process_with_events` with sample positions independent of callback partitioning, as scoped by issue #204.

## Read first

1. [Composition domain](../../domains/composition.md)
2. [ADR 0003 — Event-source contract](../../decisions/0003-event-source-contract.md) and [event-source routing page](../../architecture/event-source-contract.md)
3. [Offline render contract](../../architecture/offline-render-contract.md)
4. `audio_backend/src/player/mod.rs`, `audio_backend/src/player/tracker_engine_adapter.rs`, `engine/src/event_admission.rs`, and `sequencer/src/timing/mod.rs`

## Dependencies and blockers

- Depends on: #201, #202, #203 (merged)
- Blocks: parent #134
- Current blocker: none

## Scope and non-goals

### In scope

- Tracker rows translated to canonical note/release events at exact offsets.
- Fixed `MAX_TRACKS` tracker state and explicit bounded producer/event work.
- Shared bounded admission and event-aware engine rendering.
- Live note/release events independent of tracker transport.
- Partition-invariant event/output tests, overflow recovery, RT allocation coverage, and reviewed golden changes if any.

### Out of scope

- Engine lifecycle/lookahead (#132), state generations/migrations (#138), coalesced controls (#101), and OSC protocol redesign (#120).

## Ownership and touch set

Expected paths:

- `audio_backend/src/player/mod.rs`
- `audio_backend/src/player/tracker_engine_adapter.rs`
- `audio_backend/src/device_host/audio_processor/mod.rs`
- `audio_backend/src/offline.rs`
- focused tests under `audio_backend/`
- `docs/architecture/event-source-contract.md`
- `docs/work/active/issue-204-tracker-live-event-integration.md`
- generated active index/burndown

Shared contracts/schemas touched: consumes #201 `TimestampedEvent`, #202 timing, and #203 admission/order APIs. One additive `EngineEvent::InstrumentAllNotesOff` variant preserves the existing instrument-wide tracker/live release semantics without misusing engine-global recovery.

Potential parallel conflicts: none currently safe; #204 is the sole integration owner and the only ready issue.

## Plan

- [x] Map current tracker/live imperative paths and configure prepared admission bounds/producers.
- [x] Replace tracker timing count application with offset-bearing event production and fixed track state.
- [x] Route live audition and tracker events through admission and `Engine::process_with_events`.
- [x] Add partition, boundary, ordering, stopped-transport, overflow/recovery, deterministic render, and RT tests.
- [x] Review/update offline references for intentional timing changes and generate review WAVs.
- [x] Run full verification and independent review; commit locally without PR/push per task instruction.

## Progress and decisions

- 2026-08-02 — #201/#202/#203 merged; #204 is the only `status:ready` issue, so no separate write-heavy task is safe to parallelize.
- 2026-08-02 — Player now prepares 4096 tick slots, a structural maximum of two events per tick/track, 64 live events, stable tracker/live/recovery producers, and one bounded admission owner. Tracker state is `[InstrumentId; MAX_TRACKS]`; ordinary state commits only after complete timing and accepted admission.
- 2026-08-02 — F01–F1F changes TPL for the row beginning at the current tick; F20–FF applies BPM to the next interval; stable ascending track order with last applicable command wins. Looping preserves the exact timing phase.
- 2026-08-02 — Queued live note/release uses offset zero while stopped. Same-block NoteOn→release coalesces zero-duration prior attacks so canonical release precedence cannot create a stuck note. Transport/end/timing/overflow recovery uses the reserved global slot.
- 2026-08-02 — Canonical references intentionally changed because events now apply at exact offsets and a song-ending block-boundary tick moves to offset zero of one additional 256-frame block. The update tool generated `target/offline-renders/calibration.wav` and `target/offline-renders/ending_theme_no_effect.wav`; no human audition was performed or claimed.

## Verification

- [x] focused Player/admission/engine/RT allocation tests
- [x] `cargo test --workspace --all-targets`
- [x] `cargo test -p audio_backend --no-default-features --all-targets`
- [x] canonical offline golden test and explicit reference-update tool
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] `cargo clippy -p audio_backend --no-default-features --all-targets -- -D warnings`
- [x] `cargo fmt --all -- --check`
- [x] `python3 scripts/check_architecture.py`
- [x] `python3 scripts/check_rt_logging.py`
- [x] `python3 scripts/docs/check_docs.py`
- [x] `python3 scripts/docs/reconcile_work.py --check`
- [x] independent code review; restored retirement/recovery regressions and added playing-tracker allocation coverage in response

## Handoff

- Completed: full #204 implementation, focused/full verification, golden update, durable docs, independent review, and local commit.
- Remaining: human audition of the two generated review WAVs; GitHub metadata/PR/push are intentionally untouched by task instruction.
- Known risks: canonical PCM intentionally changed and requires human listening review; offline post-transport tail duration remains a #132 lifecycle policy. The prepared ordinary event lane is intentionally large enough for the structural 4096 × 8 × 2 tracker worst case, trading several MiB of NRT-prepared memory for a direct-RT proof.
- Exact review artifacts: `target/offline-renders/calibration.wav` and `target/offline-renders/ending_theme_no_effect.wav` (generated, not committed).
- Key paths: `audio_backend/src/player/mod.rs`, `audio_backend/src/player/tracker_engine_adapter.rs`, `audio_backend/src/device_host/audio_processor/mod.rs`, `audio_backend/tests/rt_player_allocations.rs`, `engine/src/events.rs`, and the event/offline/RT contract pages.
