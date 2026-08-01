---
title: "Task Packet — Issue 202: Sample-accurate tracker tick offsets"
summary: Active implementation and handoff context for bounded offset-bearing tracker timing.
status: current
updated: 2026-08-01
issue: 202
---

# Task Packet — Issue 202: Sample-accurate tracker tick offsets

## Identity

- Issue: 202
- Owner: jpalvarezl
- Status: in-progress
- Branch: `issue/202-sample-accurate-tick-offsets`
- Worktree: `/Users/jpalvarezl/code/blight-202`
- Base branch/SHA: `origin/main` / `1ba4db77ba2ed56e0a68986b55a3fcd85653f108`
- Reviewed implementation SHA: `a9c6e3dd68a9ef5a20533376fee2e81544977205`
- Head SHA: branch tip (resolve with `git rev-parse HEAD`; the packet does not self-reference its owning commit)
- Last handoff: 2026-08-02

## Goal

Implement [issue #202](https://github.com/jpalvarezl/blight-synth/issues/202): a prepared, allocation-free tracker timing cursor that reports exact tick offsets, preserves fractional phase across arbitrary block partitions, and has bounded work with explicit invalid-input and overflow behavior.

## Read first

1. [Composition domain](../../domains/composition.md)
2. [ADR 0003 — Event-source contract](../../decisions/0003-event-source-contract.md) and [routing page](../../architecture/event-source-contract.md)
3. `sequencer/src/timing/mod.rs`

## Dependencies and blockers

- Depends on: #201 event contract (implemented; must remain unchanged)
- Blocks: tracker integration leaf #204
- Current blocker: none

## Scope and non-goals

### In scope

- Bounded offset-bearing tick timing API and focused tests.
- Exact half-open block boundaries, partition invariance, fractional timing, and BPM next-interval semantics.
- Explicit invalid configuration, transactional result-capacity overflow, and recovery behavior.
- TPL ownership documentation and an observable, correctly prepared compatibility path for the current tracker caller if migration is deferred to #204.
- Durable composition timing documentation where the resolved behavior belongs.

### Out of scope

- Changes to the #201 engine event schema or processing contract.
- #203 event admission storage, merge, overload, or recovery APIs.
- Tracker row/event integration owned by #204.

## Ownership and touch set

Expected paths:

- `sequencer/src/timing/mod.rs`
- `sequencer/tests/rt_timing_allocations.rs`
- `audio_backend/src/player/mod.rs` (move TPL row progression out of the tick clock and keep the count shim bounded/observable until #204)
- `audio_backend/src/lib.rs`, `audio_backend/src/offline.rs`, `audio_backend/src/device_host/audio_processor/mod.rs` (one enforced 4096-frame compatibility slice bound and host-visible timing status)
- `docs/architecture/event-source-contract.md`
- `docs/work/active/issue-202-sample-accurate-tick-offsets.md`
- `docs/work/active/README.md` and `docs/work/burndown.md` (generated reconciliation only)

Shared contracts/schemas touched: none; preserve #201 and do not introduce #203 APIs.

Potential parallel conflicts: `docs/architecture/event-source-contract.md` is shared documentation; code ownership remains `sequencer/src/timing/` as declared by #202.

## Plan

- [x] Characterize the current timing implementation and caller behavior.
- [x] Design and implement a prepared fixed-capacity offset result/cursor with compact status.
- [x] Add focused boundary, partition, fractional-phase, tempo-change, invalid-input, overflow, and allocation-regression tests.
- [x] Document TPL and the resolved timing rules; migrate or explicitly sunset compatibility at #204.
- [x] Run focused/workspace tests, strict clippy/fmt, and documentation reconciliation/checks.
- [x] Correct the independent BLOCK findings around compatibility status, maximum host chunks, invalid BPM, transactional producer state, and recovery.
- [x] Review the complete corrected diff and create the focused follow-up commit.

## Progress and decisions

- 2026-08-01 — Confirmed the task starts from `origin/main` at `1ba4db7`; #201 is present and #203 surfaces are outside this branch.
- 2026-08-01 — Chose absolute Q64.64 tick phases with one checked interval addition per tick. Integer offsets are ceilings of exact phases, making partitioning irrelevant while retaining fractional phase.
- 2026-08-01 — `advance_ticks` is a callback cursor with a caller-prepared maximum invocation count and a compact result. This avoids cursor-drop/partial-consumption ambiguity and owns no dynamic storage.
- 2026-08-01 — A tempo directive at an emitted tick schedules the next interval from that tick's exact phase. Invalid directives, capacity exhaustion, and position exhaustion become sticky fail-closed statuses requiring deliberate recovery/reset.
- 2026-08-01 — Intervals shorter than one frame are rejected so offsets remain strictly increasing and callback work has the structural upper bound of one tick per frame.
- 2026-08-01 — Moved TPL to `Player` row state. The old TPL constructor ignores TPL, and count-only `advance` keeps historical end-inclusive behavior solely to preserve current PCM references until #204 removes both shims.
- 2026-08-02 — Independent BLOCK review invalidated the original callback contract: callbacks could mutate producer state before a later non-complete result, Player discarded status, and the 1024 default was smaller than a valid high-BPM 4096-frame host chunk.
- 2026-08-02 — Changed `advance_ticks` to stage a complete slice into caller-owned prepared storage. Tempo planning is side-effect-free; only `Complete` exposes a committed output prefix, while invalid tempo/capacity/position returns zero committed ticks and sticky status.
- 2026-08-02 — Kept exact at-tick tempo changes in the planner and specified public between-tick `set_bpm`: it preserves the scheduled boundary and changes the following interval. Invalid public BPM is rejected without poisoning valid playback.
- 2026-08-02 — Unified the audio-backend render-slice limit at 4096 frames and prepares Player for the structural one-tick-per-frame maximum. Compatibility `advance` now returns status; Player stops/releases on failure and rejects invalid initial/replacement BPM while retaining valid-song recovery.
- 2026-08-02 — Transport reset now starts a new absolute-frame epoch, giving capacity and position overflow one tested recovery operation; invalid initial BPM first requires valid tempo preparation.

## Verification

- [x] `cargo test -p sequencer`
- [x] `cargo test -p audio_backend --lib`
- [x] `cargo test --workspace`
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] `cargo clippy -p audio_backend --no-default-features --all-targets -- -D warnings`
- [x] `cargo test -p audio_backend --no-default-features --all-targets`
- [x] `cargo fmt --all -- --check`
- [x] `python3 scripts/check_architecture.py`
- [x] `python3 scripts/check_rt_logging.py`
- [x] `python3 scripts/docs/reconcile_work.py --check`
- [x] `python3 scripts/docs/check_docs.py`

## Handoff

- Completed: corrected transactional timing/output contract, exact 4096-frame compatibility preparation, observable Player fail-closed handling, invalid-song rejection/recovery, focused/adversarial/allocation tests, and durable timing documentation.
- Remaining: commit/push/PR. #204 later consumes the prepared offsets and removes the compatibility shims.
- Known failures/risks: Q64.64 derives each prepared interval from supplied `f64`, then accumulates the quantized interval exactly. Tempo planners are contractually side-effect-free; callers mutate producer state only after `Complete`.
- Next smallest action: verify and review this correction, then hand the complete offset slice to #204's bounded event producer.
- Files a new agent should read next: `sequencer/src/timing/mod.rs`, `audio_backend/src/player/mod.rs`, `docs/architecture/event-source-contract.md`.
