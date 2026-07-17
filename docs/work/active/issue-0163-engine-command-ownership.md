---
title: Task Packet — Issue 163 Engine Command Ownership
summary: Active context and handoff for separating engine control commands from tracker/host commands.
status: current
updated: 2026-07-18
issue: 163
owner: jpalvarezl
branch: issue/163-engine-commands
---

# Task Packet — Issue 163: Engine Command Ownership

## Goal

Move instrument and master-mixer command types/dispatch to `engine`, keep transport/song concerns in `audio_backend`, and correct effect command ownership so instrument effects are never addressed through `MixerCmd`.

## Read first

1. [Audio engine domain](../../domains/audio-engine.md)
2. [System boundaries](../../architecture/system-boundaries.md)
3. Issue #163
4. `audio_backend/src/commands.rs` and `engine/src/lib.rs`

## Dependencies

- Parent: #155
- Depends on: #162 (complete)
- Blocks: #164

## Scope

Expected paths:

- `engine/src/commands.rs`, `engine/src/lib.rs`
- `audio_backend/src/commands.rs` and tracker adapter dispatch
- hydration/examples/tracker GUI command call sites
- focused docs/tests and roadmap packets

No timestamped event schema, process trait, scheduling, routing implementation, parameter manifest, protocol, or Tokio changes.

## Taxonomy

- `InstrumentCmd`: instrument creation, notes/synth control, instrument/voice effect installation, instrument effect parameters.
- `MixerCmd`: master effects only; no instrument IDs.
- `SequencerCmd`: song loading/playback only.
- `TransportCmd`: adapter transport only.

## Plan

- [x] Add engine-owned command types and dispatch.
- [x] Move instrument effect operations out of mixer/sequencer commands.
- [x] Preserve audio_backend type re-exports and top-level command envelope.
- [x] Migrate all repository call sites.
- [x] Exercise command dispatch in engine tests and update docs.
- [x] Run full CI-equivalent validation.

## Verification

- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace --all-targets` — 44 tests
- [x] `python3 scripts/check_architecture.py`
- [x] `python3 scripts/docs/check_docs.py`
- [x] `python3 scripts/docs/sync_roadmap.py --stdout > /dev/null`
- [x] `git diff --check`

## Handoff

- Completed: engine-owned command types/dispatch, master-only mixer taxonomy, instrument-owned effect operations, compatibility re-exports, repository call-site migration, and current API documentation.
- Remaining: Copilot/human review and hosted Linux/macOS CI.
- Known risks: `EngineCommand` remains transitional and non-timestamped; master effect removal/reordering remain no-ops until #136 defines graph mutation semantics.
- Next action: open PR; after merge #164 completes the parent boundary with an offline harness and final cleanup.
