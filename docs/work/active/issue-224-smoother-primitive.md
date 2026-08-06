---
title: "Task Packet — Issue 224: Smoother primitive"
summary: Implement ADR 0006's standalone deterministic smoothing state.
status: current
updated: 2026-08-06
issue: 224
---

# Task Packet — Issue 224: Deterministic smoother primitive

## Identity

- Issue: [#224](https://github.com/jpalvarezl/blight-synth/issues/224)
- Owner: jpalvarezl
- Status: in-progress
- Branch/worktree: `issue/224-smoother-primitive` / `/Users/jpalvarezl/code/blight-224`
- Base: `main` / `3137a53`

## Goal

Implement the pure finite-state `None`/linear/exponential smoother selected by ADR 0006, without store, target binding, DSP, or Engine process integration.

## Scope and reviewability

One coherent math/state primitive. Aim for roughly 400–700 meaningful lines including exhaustive tests; re-split above ~1,000. No target IDs, mapping, coalesced drain, control-phase owner, DSP setter, or process wiring.

Touched paths: `engine/src/smoother.rs`, `engine/src/lib.rs`, `engine/tests/prepared_smoother.rs`, `engine/tests/rt_allocations.rs`, and this packet.

## Plan

- [x] Encode preparation/seed/target/elapsed APIs and compact errors.
- [x] Implement exact linear/exponential/none semantics.
- [x] Test partition equivalence, retarget, reset, invalid/range/rounding behavior.
- [x] Prove zero-allocation bounded operations and run full gates/review.

## Progress and decisions

- The issue API shorthand says “finite positive rate/seed,” but the accepted ADR and assigned task explicitly require a finite sign-unconstrained seed; the implementation follows the accepted ADR and accepts negative/zero seeds.
- `PreparedSmoother` stores a `u32` duration/cursor; preparation rejects `N > u32::MAX`, while clamped advance avoids cursor overflow.
- Duration conversion widens exact `f32` inputs to `f64` before evaluating ADR 0006's ordered `ceil(duration_ms * sample_rate / 1000)` formula.
- Finite latch/reset validation is transactional. Public observations stay limited to `current`, `target`, and `is_settled`; `value_at` provides non-mutating integer-cursor evaluation.
- Independent review approved with no critical findings or warnings. Control phase, target binding, publication drain, DSP setters, and process integration remain deferred to #214.

## Verification

- [x] `cargo test -p engine --all-targets`
- [x] `cargo test --workspace --all-targets`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo clippy -p audio_backend --no-default-features --all-targets -- -D warnings`
- [x] `cargo test -p audio_backend --no-default-features --all-targets`
- [x] `cargo fmt --all -- --check`
- [x] `python3 scripts/check_architecture.py`
- [x] `python3 scripts/check_rt_logging.py`
- [x] `python3 scripts/docs/reconcile_work.py --check`
- [x] `python3 scripts/docs/check_docs.py`

## Handoff

- Completed: standalone smoother implementation, 13 focused behavior tests, zero-heap audit, full gates, and independent review.
- Remaining: no implementation work; PR/push/merge and issue-status changes are intentionally not performed.
- Known failures/risks: none. `f32` output precision is expected; trajectory calculation uses `f64` intermediates and exact target snap at `N`.
- Next smallest action: consume this primitive from the separately scoped #214 binding/control-phase integration.
