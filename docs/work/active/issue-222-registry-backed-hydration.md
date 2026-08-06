---
title: "Task Packet — Issue 222: Registry-backed hydration"
summary: Switch tracker/offline/standalone hydration to the built-in node registry.
status: current
updated: 2026-08-06
issue: 222
---

# Task Packet — Issue 222: Registry-backed hydration

## Identity

- Issue: [#222](https://github.com/jpalvarezl/blight-synth/issues/222)
- Owner: jpalvarezl
- Status: in-progress
- Branch/worktree: `issue/222-registry-backed-hydration` / `/Users/jpalvarezl/code/blight-222`
- Base: `main` / `3137a53`

## Goal

Replace active tracker hydration type switches with the merged legacy adapter plus NRT node registry, preserving command order, envelope compatibility, retirement, and audio behavior.

## Scope and reviewability

One coherent integration concept. Aim for roughly 500–800 meaningful lines including focused tests; pause/re-split above ~1,000. Do not change schemas, routing, UI, or node definitions.

Touched paths: `audio_backend/src/song_hydration.rs`, focused hydration tests, the factory sample-rate accessor, architecture dependency enforcement, and packet/generated docs.

## Plan

- [x] Map prepared registry owners into existing structural commands.
- [x] Preserve explicit legacy envelope commands and effect ordering/IDs.
- [x] Add multi-effect/DFAM/unknown diagnostics and retirement tests.
- [x] Run workspace/golden/RT/strict checks and independent review.

## Acceptance evidence

- [x] Standalone/offline tracker hydration adapts legacy definitions and prepares owners through `BuiltInRegistry` on NRT.
- [x] Instrument, effect, kick pitch-envelope, and amplitude-envelope command order remains explicit and tested.
- [x] Repeated same-kind effects and DFAM ladder plus user effect retain ordered, independently addressable IDs.
- [x] Unsupported Sample/Synth owners and unknown/invalid definitions retain structured adapter/registry diagnostics.
- [x] Prepared effect rejection uses the existing RT-to-NRT retirement path.
- [x] Existing offline PCM goldens are unchanged; strict workspace, all-feature, host-free, RT, architecture, formatting, and docs gates pass.

## Verification

- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `RUSTFLAGS='-D warnings' cargo test --workspace --all-targets --all-features`
- `cargo clippy -p audio_backend --no-default-features --all-targets -- -D warnings`
- `RUSTFLAGS='-D warnings' cargo test -p audio_backend --no-default-features --all-targets`
- `cargo test --workspace --all-targets`
- `cargo test -p audio_backend --no-default-features --test offline_golden -- --nocapture`
- `cargo test -p engine --test rt_allocations -- --nocapture`
- `cargo test -p audio_backend --all-features --test rt_player_allocations -- --nocapture`
- `cargo fmt --all -- --check`
- `python3 scripts/check_architecture.py`
- `python3 scripts/check_rt_logging.py`
- `python3 scripts/docs/reconcile_work.py --check`
- `python3 scripts/docs/check_docs.py`

## Handoff

- Completed: registry-backed hydration integration, structured diagnostics, effect/envelope/order/retirement tests, architecture enforcement, full gates, and independent review with no blocking findings.
- Remaining: push/PR/merge and GitHub status changes were explicitly not performed.
- Risk: existing PCM goldens do not contain effects or DFAM; focused owner/command equivalence tests cover those paths, while the reviewed golden manifest remains byte-for-byte unchanged.
