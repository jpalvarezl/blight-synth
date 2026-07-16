---
title: Task Packet — Issue 154 Resource Boundary
summary: Active context and handoff for moving file/platform sample loading out of DSP.
status: current
updated: 2026-07-15
issue: 154
owner: jpalvarezl
branch: issue/154-resource-boundary
---

# Task Packet — Issue 154: Resource Boundary

## Goal

Make `dsp` portable by moving `ResourceManager`, WAV loading, macOS DLS loading, and I/O-specific errors into the current non-RT `audio_backend` host adapter while preserving immutable `SampleData` and playback APIs.

## Read first

1. [Audio engine domain](../../domains/audio-engine.md)
2. [Standalone host domain](../../domains/standalone-host.md)
3. [Product topology](../../architecture/product-topology.md)
4. `dsp/src/resources/mod.rs`, crate manifests, and direct references only

## Dependencies

- Parent: #130
- Depends on: #129 (complete)
- Blocks: #155

## Scope

Expected paths:

- `dsp/Cargo.toml`, `dsp/src/lib.rs`, old resource/result modules
- `audio_backend/Cargo.toml`, `audio_backend/src/lib.rs`, new resource/result modules
- `scripts/check_architecture.py`
- focused docs and tests

No asset-registry/state redesign, engine extraction, protocol, or UI work.

## Plan

- [x] Move non-RT resource manager/loaders and I/O error conversion to `audio_backend`.
- [x] Remove `hound`/`os_dls` dependencies and file-loading exports from `dsp`.
- [x] Preserve `audio_backend::ResourceManager`, `Result`, and existing examples.
- [x] Add focused manager/WAV tests.
- [x] Tighten dependency enforcement and domain docs.
- [x] Run full CI-equivalent validation.

## Verification

- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace --all-targets` — 41 tests
- [x] `cargo test -p audio_backend resources::tests -- --nocapture`
- [x] `python3 scripts/check_architecture.py`
- [x] `python3 scripts/docs/check_docs.py`
- [x] `python3 scripts/docs/sync_roadmap.py --stdout > /dev/null`
- [x] `git diff --check`

## Handoff

- Completed: resource manager, WAV/DLS loading, and hound-specific errors now live in `audio_backend`; DSP retains immutable sample data and no longer depends on hound/os_dls; focused tests and dependency enforcement were added; malformed/truncated WAV samples now return an error instead of silently substituting zeroes.
- Remaining: Copilot/human review and hosted Linux/macOS CI.
- Known risks: removing `dsp::Result`/`dsp::ResourceManager` is an intentional crate-boundary API change; repository consumers continue using the preserved `audio_backend` exports.
- Next action: open PR and inspect cross-platform checks.
