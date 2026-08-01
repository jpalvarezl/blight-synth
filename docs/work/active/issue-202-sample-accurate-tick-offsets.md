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
- Head SHA: `1ba4db77ba2ed56e0a68986b55a3fcd85653f108`
- Last handoff: 2026-08-01

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
- Explicit invalid configuration and result-capacity overflow behavior.
- TPL ownership documentation and a bounded compatibility path for the current tracker caller if migration is deferred to #204.
- Durable composition timing documentation where the resolved behavior belongs.

### Out of scope

- Changes to the #201 engine event schema or processing contract.
- #203 event admission storage, merge, overload, or recovery APIs.
- Tracker row/event integration owned by #204.

## Ownership and touch set

Expected paths:

- `sequencer/src/timing/mod.rs`
- `sequencer/tests/rt_timing_allocations.rs`
- `audio_backend/src/player/mod.rs` (move TPL row progression out of the tick clock and keep the count shim bounded until #204)
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
- [x] Review the complete diff and prepare the focused commit.

## Progress and decisions

- 2026-08-01 — Confirmed the task starts from `origin/main` at `1ba4db7`; #201 is present and #203 surfaces are outside this branch.
- 2026-08-01 — Chose absolute Q64.64 tick phases with one checked interval addition per tick. Integer offsets are ceilings of exact phases, making partitioning irrelevant while retaining fractional phase.
- 2026-08-01 — `advance_ticks` is a callback cursor with a caller-prepared maximum invocation count and a compact result. This avoids cursor-drop/partial-consumption ambiguity and owns no dynamic storage.
- 2026-08-01 — A tempo directive at an emitted tick schedules the next interval from that tick's exact phase. Invalid directives, capacity exhaustion, and position exhaustion become sticky fail-closed statuses requiring deliberate recovery/reset.
- 2026-08-01 — Intervals shorter than one frame are rejected so offsets remain strictly increasing and callback work has the structural upper bound of one tick per frame.
- 2026-08-01 — Moved TPL to `Player` row state. The old TPL constructor ignores TPL, and count-only `advance` keeps historical end-inclusive behavior solely to preserve current PCM references until #204 removes both shims.
- 2026-08-01 — Independent complete-diff review returned APPROVE with no critical findings or warnings; minor allocation/comment suggestions were incorporated.

## Verification

- [x] `cargo test -p sequencer`
- [x] `cargo test -p audio_backend --lib`
- [x] `cargo test --workspace`
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] `cargo fmt --all -- --check`
- [ ] `python3 scripts/docs/reconcile_work.py --check` — generated docs reconcile, but live parallel issue #203 has no active packet in this worktree.
- [x] `python3 scripts/docs/check_docs.py`

## Handoff

- Completed: implementation, focused/adversarial/allocation tests, current-player compatibility, durable docs, generated docs reconciliation, workspace verification, and independent review.
- Remaining: commit this prepared change; #204 later consumes offsets and removes the two explicitly documented compatibility shims.
- Known failures/risks: reconciliation remains externally blocked only by missing active packet for parallel in-progress issue #203. Q64.64 derives its prepared interval from the supplied `f64`, then accumulates that quantized interval exactly.
- Next smallest action: integrate `advance_ticks` into #204's bounded event producer and remove `advance`/`new_with_bpm_tpl`.
- Files a new agent should read next: `sequencer/src/timing/mod.rs`, `audio_backend/src/player/mod.rs`, `docs/architecture/event-source-contract.md`.
