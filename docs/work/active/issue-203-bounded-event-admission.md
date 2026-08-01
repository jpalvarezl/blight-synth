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
- Base branch/SHA: `origin/main` / `1ba4db77ba2ed56e0a68986b55a3fcd85653f108`
- Head SHA: final focused commit on this branch (packet is included in that commit)
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
- 2026-08-01 — Ordinary failures are sticky and expose only a valid recovery event, never an ordinary prefix. Rejected events are discarded at the next `begin_block` and are never carried forward implicitly.
- 2026-08-01 — Final merge uses allocation-free `sort_unstable_by_key(TimestampedEvent::order_key)`; an independent code review approved the implementation and requested recovery error coverage, which was added.

## Verification

- [x] `cargo test -p engine --all-targets`
- [x] `cargo test --workspace --all-targets`
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] `cargo clippy -p audio_backend --no-default-features --all-targets -- -D warnings`
- [x] `cargo fmt --all -- --check`
- [x] `python3 scripts/check_architecture.py`
- [x] `python3 scripts/check_rt_logging.py`
- [ ] `python3 scripts/docs/reconcile_work.py --check` — #203 packet/index/burndown reconcile cleanly, but live GitHub reports unrelated in-progress leaf #202 has no active packet.
- [x] `python3 scripts/docs/check_docs.py`
- [x] `python3 -m unittest scripts.docs.test_reconcile_work`

## Handoff

- Completed: host-independent bounded admission/merge/recovery API, focused and workspace tests, zero-heap audit, durable contract updates, independent review, and generated-doc reconciliation.
- Remaining: first-party host/composition integration in #204; no #203 implementation work remains.
- Known failures/risks: work-state reconciliation is blocked only by unrelated live issue #202 lacking an active packet. #204 must pass the same frame count to `begin_block` and `Engine::process_with_events`, retain the admission owner for NRT destruction, and explicitly decide whether rejected producer events are dropped or resubmitted.
- Next smallest action: #204 consumes `BoundedEventAdmission`; do not add another comparator or queue rejected current-block events implicitly.
- Files a new agent should read next: `engine/src/event_admission.rs`, `engine/src/events.rs`, `engine/tests/bounded_event_admission.rs`, `engine/tests/rt_allocations.rs`.
