---
title: Task Packet — Issue 175 RT Debug Logging and Hot Path
summary: Active context and handoff for compile-time-gated callback logging and malformed-input stress coverage.
status: current
updated: 2026-07-19
issue: 175
owner: jpalvarezl
branch: issue/175-rt-debug
---

# Task Packet — Issue 175: RT Debug Logging and Hot Path

## Goal

Preserve useful developer callback logging while compiling it out of strict/release builds, remove accidental direct callback logger/printing calls, and harden representative callback buffer behavior.

## Read first

1. [Real-time contract](../../architecture/realtime-contract.md)
2. Issue #175
3. Known callback paths listed in `scripts/check_rt_logging.py`

## Dependencies

- Parent: #133
- Depends on: #171 (complete)

## Scope

- Shared `dsp::rt_debug_log!` macro.
- Debug argument evaluation and release compile-out test.
- Callback log migration/static policy check.
- Existing/relevant malformed buffer and missing-ID stress behavior.

No lock-free release diagnostic queue, structural reclamation, queue backpressure, or final Engine lifecycle design.

## Plan

- [x] Add compile-time-gated callback debug macro.
- [x] Prove debug arguments evaluate and release arguments do not.
- [x] Route callback-reachable Player/adapter/DSP logs through the macro.
- [x] Replace raw effect-chain `eprintln!` paths.
- [x] Add static callback logging policy check and CI step.
- [x] Chunk oversized direct Engine buffers and add regression coverage.
- [ ] Run hosted CI and receive Copilot/human review.

## Verification

- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace --all-targets` — 54 tests
- [x] `cargo clippy -p audio_backend --no-default-features --all-targets -- -D warnings`
- [x] `cargo test -p audio_backend --no-default-features --all-targets` — 10 tests
- [x] `RUSTFLAGS='-D warnings' cargo test --release -p dsp diagnostics::tests::argument_evaluation_matches_diagnostic_build_mode`
- [x] `python3 scripts/check_rt_logging.py`
- [x] `python3 scripts/check_architecture.py`
- [x] `python3 scripts/docs/check_docs.py`
- [x] `python3 scripts/docs/sync_roadmap.py --stdout > /dev/null`
- [x] `git diff --check`

## Handoff

- Completed: macro, callback log migration, release compile-out behavior, static checker/CI, oversized Engine block hardening, and full local/default/host-free/release validation.
- Remaining: hosted validation and Copilot/human review.
- Known risks: debug logging may intentionally glitch audio and is not RT-performance evidence; release callback telemetry remains deferred until demonstrated need.
- Next action: run complete validation and open PR.
