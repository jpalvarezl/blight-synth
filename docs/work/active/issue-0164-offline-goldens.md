---
title: Task Packet — Issue 164 Offline Golden Renders
summary: Active context and handoff for deterministic end-to-end JSON song rendering and M0 engine-boundary completion.
status: current
updated: 2026-07-19
issue: 164
owner: jpalvarezl
branch: issue/164-offline-goldens
---

# Task Packet — Issue 164: Offline Golden Renders

## Goal

Turn the supported current-schema JSON synth/drum songs into deterministic, hardware-free end-to-end PCM references, add playable WAV output, and finalize parent #155. The historical `drum_crap.json` is deliberately excluded rather than retrofitted into the current schema.

## Read first

1. [Offline render contract](../../architecture/offline-render-contract.md)
2. [Audio engine domain](../../domains/audio-engine.md)
3. Issue #164
4. `audio_backend/src/offline.rs`, hydration, and tracker Player only

## Dependencies

- Parent: #155
- Depends on: #162 and #163 (complete)
- Blocks: #156

## Scope

Expected paths:

- offline renderer, golden integration test/manifest, CLI examples, render script
- reusable hydration command construction
- deterministic engine mix ordering
- legacy JSON compatibility required by repository fixtures
- tracker adapter naming cleanup
- focused architecture/docs/work state

No sample-accurate scheduling, transport-independent tails, external sample packaging, or final composition API.

## Plan

- [x] Add canonical offline render and PCM/reference APIs.
- [x] Render supported current-schema reference songs through shared hydration/Player/Engine/DSP.
- [x] Enforce deterministic mix/random/termination behavior.
- [x] Add exact hashes, metrics, mutation sensitivity, and explicit `--update-reference` workflow.
- [x] Add playable WAV CLI/script and direct engine harness.
- [x] Document clipping/known timing/tail limitations.
- [x] Finalize architecture enforcement and tracker adapter naming.
- [ ] Run hosted Linux/macOS CI to verify cross-platform exact hashes.

## Verification

- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace --all-targets` — 51 tests
- [x] `cargo test -p audio_backend --test offline_golden`
- [x] `cargo run -p engine --example offline_render`
- [x] `scripts/render_reference_songs.sh target/offline-renders-script`
- [x] `python3 scripts/check_architecture.py`
- [x] `python3 scripts/docs/check_docs.py`
- [x] `python3 scripts/docs/sync_roadmap.py --stdout > /dev/null`
- [x] `git diff --check`

## Handoff

- Completed: shared offline hydration, bounded tracker renderer, canonical PCM/WAV path, deterministic sorted `Vec<InstrumentSlot>` mixing, SHA-256 references/metrics for supported reference songs, mutation sensitivity, explicit reference update tool, direct engine harness, scripts/docs, and tracker adapter rename.
- Remaining: hosted cross-platform hash validation, Copilot/human review, and one listening review of the initial WAV baseline.
- Known risks: the characterization records block-boundary timing/tail limitations and exposes clipping in `ending_theme_no_effect`; #132/#134/#136 must intentionally update references when correcting them. Historical `drum_crap.json` remains load-incompatible by design until a real migration policy exists.
- Next action: open PR and inspect Ubuntu/macOS exact hash checks.
