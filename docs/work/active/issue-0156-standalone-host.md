---
title: Task Packet — Issue 156 Standalone Host Boundary
summary: Active context and handoff for feature-isolating CPAL/OSC and simplifying the standalone threading model.
status: current
updated: 2026-07-19
issue: 156
owner: jpalvarezl
branch: issue/156-standalone-host
---

# Task Packet — Issue 156: Standalone Host Boundary

## Goal

Contain CPAL, OSC, queue/callback adaptation, metering transport, logging, and temporary Tokio use behind an explicit standalone feature/module while host-free tracker/offline rendering remains buildable.

## Read first

1. [Standalone host domain](../../domains/standalone-host.md)
2. [System boundaries](../../architecture/system-boundaries.md)
3. Issue #156 and M2 follow-up #161
4. `audio_backend/src/standalone/` entry points only

## Dependencies

- Parent: #130
- Depends on: #155 (complete)
- Blocks: #157
- Tokio removal: #161 (M2)

## Scope

- Feature-gate/move standalone CPAL/OSC/Tokio modules and targets.
- Make standalone dependencies optional.
- Switch Tokio to current-thread runtime.
- Prove no-default tracker/offline builds/tests exclude standalone targets.
- Enforce feature/runtime rules and update focused docs/examples.

No OSC protocol redesign, synchronous Tokio replacement, composition semantics, or frontend work.

## Plan

- [x] Add `standalone` feature and optional host dependencies/target requirements.
- [x] Move CPAL/OSC/meter/processor/frontend modules under a standalone namespace.
- [x] Use Tokio current-thread runtime and remove multi-thread feature.
- [x] Add host-free CI checks and architecture validation.
- [x] Update thread/feature docs and smoke instructions.
- [x] Run full CI-equivalent validation.

## Verification

- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace --all-targets` — 52 tests
- [x] `cargo clippy -p audio_backend --no-default-features --all-targets -- -D warnings`
- [x] `cargo test -p audio_backend --no-default-features --all-targets` — 10 tests
- [x] `scripts/check_audio_backend_osc.sh`
- [x] `LISTEN_SECONDS=1 scripts/smoke_meter_streaming.sh` — 30.8 Hz, 4-float meter
- [x] `scripts/smoke_osc_standalone.sh` — load/gain/play/meter/stop passed
- [x] `python3 scripts/check_architecture.py`
- [x] `python3 scripts/docs/check_docs.py`
- [x] `python3 scripts/docs/sync_roadmap.py --stdout > /dev/null`
- [x] `git diff --check`

## Handoff

- Completed: optional/default standalone feature, isolated standalone module tree, target-gated binary/examples, current-thread Tokio runtime, host-free offline/tracker build and CI checks, architecture enforcement, docs, and manual OSC/meter validation.
- Remaining: hosted Linux/macOS CI and Copilot/human review.
- Known risks: Tokio remains a temporary standalone implementation detail until #161; OSC smoke output can interleave meter telemetry before request-specific responses, which remains protocol work under #104/#120.
- Next action: open PR; after merge #157 performs final M0 dependency/docs reconciliation.
