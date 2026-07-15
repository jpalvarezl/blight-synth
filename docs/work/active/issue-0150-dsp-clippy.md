---
title: Task Packet — Issue 150 DSP Clippy Cleanup
summary: Active context and handoff for strict lint cleanup in DSP and remaining workspace targets.
status: current
updated: 2026-07-14
issue: 150
owner: jpalvarezl
branch: issue/150-dsp-clippy
---

# Task Packet — Issue 150

## Goal

Make DSP and remaining workspace targets strict-Clippy clean without redesigning M1 contracts or DSP algorithms.

## Read first

1. Issue #150
2. [Audio engine domain](../../domains/audio-engine.md)
3. Clippy diagnostics and only the flagged files

## Dependencies

- Parent: #131
- Final workspace verification includes #149; crate-local cleanup proceeds independently.

## Scope

Expected paths: `dsp/`, additional workspace files only when flagged, this packet.

No engine API, routing, serialization, parameter, protocol, or audio algorithm redesign.

## Plan

- [x] Apply behavior-preserving idiomatic fixes.
- [x] Use narrow rationale-backed allowances where redesign belongs to M1.
- [x] Run tests and strict Clippy, including a temporary combined verification with #149.

## Verification

- [x] `cargo fmt --all -- --check`
- [x] `cargo test --workspace --all-targets`
- [x] `cargo clippy -p dsp -p audio_backend -p tracker_gui --all-targets --no-deps -- -D warnings`
- [x] `cargo clippy --workspace --all-targets -- -D warnings` with #149 applied temporarily
- [x] `python3 scripts/docs/check_docs.py`
- [x] `git diff --check`

## Handoff

- Completed: strict lint cleanup for DSP, audio backend/examples, and tracker GUI; workspace-wide strict Clippy was verified together with #149.
- Remaining: review and merge after or alongside #149; #131 then enables the CI gate.
- Known risks: large command variants and the envelope factory keep narrow rationale-backed allowances to avoid RT deallocation or premature M1 API redesign.
- Next action: Copilot/human review.
