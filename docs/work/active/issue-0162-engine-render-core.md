---
title: Task Packet — Issue 162 Engine Render Core
summary: Active context and handoff for extracting the generic host-independent render runtime.
status: current
updated: 2026-07-16
issue: 162
owner: jpalvarezl
branch: issue/162-engine-render-core
---

# Task Packet — Issue 162: Engine Render Core

## Goal

Add the `engine` crate and move generic instrument dispatch, planar mixing, and master effects out of the tracker-specific synthesizer while keeping tracker document/track state in its adapter.

## Read first

1. [Audio engine domain](../../domains/audio-engine.md)
2. [System boundaries](../../architecture/system-boundaries.md)
3. [Product topology](../../architecture/product-topology.md)
4. Issue #162 and current tracker synthesizer only

## Dependencies

- Parent: #155
- Depends on: #154 (complete)
- Blocks: #163

## Scope

Expected paths:

- new `engine/` crate and workspace manifests
- `audio_backend/src/player/tracker_synthesizer.rs`
- `scripts/check_architecture.py`
- focused docs and tests

No command ownership migration, final event schema, sample-accurate scheduler, routing redesign, parameter manifest, persistence, CPAL/OSC, or Tokio implementation changes.

## Plan

- [x] Add portable engine crate and generic runtime.
- [x] Leave track cache and tracker semantics in adapter.
- [x] Preserve current command behavior through delegation.
- [x] Add direct planar render tests.
- [x] Enforce and document dependency boundary.
- [x] Record contained/deferred Tokio decision and #161 in docs/roadmap snapshot.
- [x] Run full CI-equivalent validation.

## Verification

- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace --all-targets` — 43 tests
- [x] `python3 scripts/check_architecture.py`
- [x] `python3 scripts/docs/check_docs.py`
- [x] `python3 scripts/docs/sync_roadmap.py --stdout > /dev/null`
- [x] `git diff --check`

## Handoff

- Completed: new `engine` crate, generic instrument/mixer/master-effects runtime, tracker-specific adapter delegation, direct planar render tests, dependency enforcement, docs, and deferred Tokio-removal roadmap item #161.
- Remaining: Copilot/human review and hosted Linux/macOS CI.
- Known risks: current engine methods are deliberately transitional and synchronous; final event, scheduling, routing, parameter, state, and RT mutation contracts remain M1 work.
- Next action: open PR, then #163 can move engine-owned command types after merge.
