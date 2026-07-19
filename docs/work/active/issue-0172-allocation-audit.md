---
title: Task Packet — Issue 172 Allocation Audit
summary: Active context and handoff for test-only RT heap allocation/deallocation instrumentation.
status: current
updated: 2026-07-19
issue: 172
owner: jpalvarezl
branch: issue/172-allocation-audit
---

# Task Packet — Issue 172: Allocation Audit

## Goal

Make the prepared steady-state Engine heap contract executable with a thread-local test allocator and a known-allocating self-test.

## Read first

1. [Real-time contract](../../architecture/realtime-contract.md)
2. [Allocation audit design](../../architecture/rt-allocation-audit.md)
3. Issue #172

## Dependencies

- Parent: #133
- Depends on: #171 (complete)
- Coordinates with #174 for structural retirement/deallocation

## Scope

- Test-binary-local global allocator wrapper.
- Thread-local allocation/reallocation/deallocation measurement scopes.
- Representative prepared note/parameter/render path.
- Intentional allocation/deallocation detection fixture.

No production allocator, structural replacement, command budget, logger migration, or final Engine lifecycle changes.

## Plan

- [x] Add scoped allocation/deallocation counters.
- [x] Warm setup/lazy state before measurement.
- [x] Cover prepared note, synth parameter, render, and note-off operations.
- [x] Add intentional allocation/deallocation self-test.
- [x] Document usage and limitations.
- [ ] Run complete CI and receive Copilot/human review.

## Verification

- [x] `cargo test -p engine --test rt_allocations -- --nocapture`
- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace --all-targets` — 55 tests
- [x] `cargo clippy -p audio_backend --no-default-features --all-targets -- -D warnings`
- [x] `cargo test -p audio_backend --no-default-features --all-targets` — 10 tests
- [x] `python3 scripts/check_architecture.py`
- [x] `python3 scripts/docs/check_docs.py`
- [x] `python3 scripts/docs/sync_roadmap.py --stdout > /dev/null`
- [x] `git diff --check`

## Handoff

- Completed: allocator harness, zero-heap representative path, intentional allocation/drop self-test, focused documentation, and full local/default/host-free validation.
- Remaining: hosted validation and Copilot/human review.
- Known risks: thread-local scope intentionally does not observe other threads; structural swap/drop ownership remains #174.
- Next action: complete validation and open PR.
