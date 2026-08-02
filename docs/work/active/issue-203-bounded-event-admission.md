---
title: "Task Packet — Issue 203: Bounded event admission"
summary: Focused implementation and handoff context for fixed-capacity current-block event admission and recovery.
status: current
updated: 2026-08-01
issue: 203
issues: [203]
---

# Task Packet — Issue 203: Bounded current-block event admission

## Identity

- Issue: [#203](https://github.com/jpalvarezl/blight-synth/issues/203)
- Owner: jpalvarezl
- Status: in-progress
- Branch: `issue/203-bounded-event-admission`
- Worktree: `/Users/jpalvarezl/code/blight-203`
- Base branch/SHA: `origin/main` / `078d4f25977bb68ee93f4fedbe1a742428ab52f0`
- Head: branch tip at handoff; intentionally not duplicated as an in-file SHA
- Last handoff: 2026-08-01

## Goal

Implement the host-independent, NRT-prepared fixed-capacity current-block admission and merge layer described by [#203](https://github.com/jpalvarezl/blight-synth/issues/203), consuming #201's canonical event ordering while providing bounded producer/work limits, compact observable failure, fail-closed blocks, capacity-independent recovery, reset/reuse, and zero-heap RT behavior.

## Read first

1. [Audio engine domain](../../domains/audio-engine.md)
2. [Target system boundaries](../../architecture/system-boundaries.md) and [real-time audio contract](../../architecture/realtime-contract.md)
3. `engine/src/events.rs`, `engine/src/lib.rs`, and focused engine event/allocation tests

## Dependencies and blockers

- Depends on: #201 (canonical `TimestampedEvent::order_key()` / `EventOrderKey`)
- Blocks: #204 and first-party event-source integration
- Current blocker: none

## Scope and non-goals

### In scope

- Explicit NRT-prepared ordinary-event and stable-producer capacities.
- Bounded current-block producer admission, source sequence/order validation, and deterministic canonical merge.
- Compact producer-visible malformed/overflow status and fail-closed block behavior.
- Out-of-band stop/all-notes-off recovery, reset/reuse, and hardware-free zero-allocation tests.

### Out of scope

- Continuous latest-value parameters and structural prepared-state swaps.
- Lookahead, generation handoff, lifecycle negotiation, tracker/OSC/plugin integration, and composition-document semantics.
- Any event comparator other than `TimestampedEvent::order_key()` / `EventOrderKey`.

## Ownership and touch set

Expected paths:

- `engine/src/events.rs`
- `engine/src/event_admission.rs` (new)
- `engine/src/lib.rs`
- `engine/tests/bounded_event_admission.rs` (new)
- `engine/tests/rt_allocations.rs`
- `docs/architecture/realtime-contract.md`
- `docs/work/active/issue-203-bounded-event-admission.md`
- `docs/work/active/README.md` (generated reconciliation index)
- `docs/work/burndown.md` (generated live-state reconciliation)

Shared contracts/schemas touched: new engine admission API only; canonical #201 event schema and ordering remain unchanged.

Potential parallel conflicts: #204 must consume this API rather than edit it concurrently.

## Plan

- [x] Define a prepared fixed-capacity admission object, compact statuses, and out-of-band recovery.
- [x] Implement bounded per-producer validation and canonical deterministic merge using only `order_key()`.
- [x] Enforce fail-closed no-partial output and explicit reset/reuse semantics.
- [x] Add capacity, interleaving, malformed sequence/order, recovery, reset, and zero-heap RT tests.
- [x] Update the durable RT contract only for mechanisms resolved by this implementation.
- [x] Run focused/workspace tests, strict clippy/fmt, RT/static checks, and docs reconciliation/checks.

## Progress and decisions

- 2026-08-01 — #201's `TimestampedEvent::order_key()` / `EventOrderKey` is the sole ordering comparator; semantic precedence intentionally overrides source emission sequence across event kinds.
- 2026-08-01 — Recovery uses one separately prepared physical slot and a distinct stable producer identity, so all-notes-off remains admissible at exact ordinary capacity and after ordinary rejection.
- 2026-08-01 — Each configured active producer may stage one complete bounded slice per block; silent configured producers may omit submission. Source sequence must increase across successfully finalized blocks, while `reset` explicitly clears sequence history.
- 2026-08-01 — Ordinary failures are fail-closed and expose only a valid recovery event, never an ordinary prefix. Admission still validates later producer submissions; final failure selection is independent of call interleaving, prioritizing malformed/protocol failures over capacity and then stable producer identity. Rejected events are cleared at finalization and are never carried forward implicitly.
- 2026-08-01 — Slices no larger than total prepared capacity are validated before aggregate capacity accounting. A larger slice is intrinsically overflow without an unbounded RT scan. Overflow attribution uses the lowest stable identity among non-empty capacity contributors.
- 2026-08-01 — Rejected blocks do not commit ordinary sequence baselines. Recovery commits independently when finalized, and `reset` clears both ordinary and recovery baselines.
- 2026-08-01 — Final merge still uses allocation-free `sort_unstable_by_key(TimestampedEvent::order_key)` as the sole event comparator.
- 2026-08-01 — An independent REVISE review found call-order-dependent final rejection and incomplete fallible-result/allocation coverage; the correction and focused regressions are included in this handoff.

## Verification

- [x] `cargo test -p engine --all-targets`
- [x] `cargo test --workspace --all-targets`
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] `cargo clippy -p audio_backend --no-default-features --all-targets -- -D warnings`
- [x] `cargo fmt --all -- --check`
- [x] `python3 scripts/check_architecture.py`
- [x] `python3 scripts/check_rt_logging.py`
- [x] `python3 scripts/docs/reconcile_work.py --check`
- [x] `python3 scripts/docs/check_docs.py`
- [x] `python3 -m unittest scripts.docs.test_reconcile_work`

## Handoff

- Completed: host-independent bounded admission/merge/recovery API; interleaving-independent final rejection; malformed-before-aggregate-capacity validation; fail-closed baseline handling; must-use statuses; focused/workspace and expanded zero-heap coverage; durable contract updates.
- Remaining: first-party host/composition integration in #204; no #203 implementation work remains.
- Known failures/risks: none in the #203 verification set. #204 must inspect every fallible admission/finalization result, pass the same frame count to `begin_block` and `Engine::process_with_events`, retain the admission owner for NRT destruction, and explicitly decide whether rejected producer events are dropped or resubmitted.
- Next smallest action: #204 consumes `BoundedEventAdmission`; do not add another event comparator or queue rejected current-block events implicitly.
- Files a new agent should read next: `engine/src/event_admission.rs`, `engine/src/events.rs`, `engine/tests/bounded_event_admission.rs`, `engine/tests/rt_allocations.rs`.
