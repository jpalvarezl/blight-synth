---
title: Task Packet — Issue 157 Finalize M0
summary: Active context and handoff for final M0 dependency enforcement, documentation reconciliation, and milestone closure.
status: current
updated: 2026-07-19
issue: 157
owner: jpalvarezl
branch: issue/157-finalize-m0
---

# Task Packet — Issue 157: Finalize M0

## Goal

Close the mechanical architecture baseline with an accurate current dependency graph, exact portable-crate allowlists, standalone target ownership checks, documented compatibility shims, and full validation.

## Read first

1. [M0 crate dependency graph](../../architecture/crate-dependency-graph.md)
2. [System boundaries](../../architecture/system-boundaries.md)
3. Parent #130 and issue #157
4. `scripts/check_architecture.py`

## Dependencies

- Parent: #130
- Depends on: #154, #155, #156 (complete)

## Scope

- Current dependency/feature/target documentation.
- Exact portable crate allowlists and example ownership checks.
- Explicit compatibility re-export policy.
- Final README/domain path reconciliation and full tests/smokes.

No M1 lifecycle, event, scheduling, routing, parameter, state, or composition redesign.

## Plan

- [x] Document current crate/feature graph and responsibilities.
- [x] Enforce exact portable dependency allowlists.
- [x] Enforce standalone versus host-free example ownership.
- [x] Reconcile README/domain paths and compatibility shims.
- [x] Run all default, host-free, offline golden, OSC, meter, architecture, docs, and strict lint checks.
- [ ] Close #157/#130 and M0 after review/merge.

## Verification

- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace --all-targets` — 52 tests
- [x] `cargo clippy -p audio_backend --no-default-features --all-targets -- -D warnings`
- [x] `cargo test -p audio_backend --no-default-features --all-targets` — 10 tests
- [x] `scripts/check_audio_backend_osc.sh`
- [x] meter smoke — 30.2 Hz, four-float payload
- [x] OSC smoke — load/gain echo/play/meter/stop
- [x] `python3 scripts/check_architecture.py`
- [x] `python3 scripts/docs/check_docs.py`
- [x] `python3 scripts/docs/sync_roadmap.py --stdout > /dev/null`
- [x] `git diff --check`

## Handoff

- Completed: current crate/feature graph, exact portable allowlists, standalone/host-free target ownership checks, compatibility policy, docs navigation, and complete default/no-default/offline/OSC/meter validation.
- Remaining: hosted CI, Copilot/human review, then close #157/#130 and M0.
- Known risks: broad `audio_backend` compatibility re-exports remain intentionally transitional until #132; Tokio remains isolated but present until #161.
- Next action: open final M0 PR and review the documented/enforced dependency graph.
