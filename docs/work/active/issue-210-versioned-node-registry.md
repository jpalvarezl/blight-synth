---
title: "Task Packet — Issue 210: Versioned node registry"
summary: Active context for versioned instrument/effect definitions and the built-in NRT registry.
status: current
updated: 2026-08-04
issue: 210
---

# Task Packet — Issue 210: Versioned node registry

## Identity

- Issue: [#210](https://github.com/jpalvarezl/blight-synth/issues/210)
- Owner: jpalvarezl
- Status: in-progress
- Branch: `issue/210-versioned-node-registry`
- Worktree: `/Users/jpalvarezl/code/blight-210`
- Base branch/SHA: `main` / `fb67163`
- Head: branch tip at handoff
- Last handoff: 2026-08-04

## Goal

Define stable kind IDs, versioned serializable instrument/effect definitions, and an NRT-only built-in registry/factory over the typed IDs from #209.

## Read first

1. [Audio engine domain](../../domains/audio-engine.md)
2. [System boundaries](../../architecture/system-boundaries.md) and [ADR 0004](../../decisions/0004-parameter-manifest.md)
3. `dsp/src/factories/`, sequencer instrument/effect models, and audio-backend hydration

## Dependencies and blockers

- Depends on: #209 (merged)
- Blocks: #211 and parent #135
- Current blocker: none

## Scope

In scope: versioned definitions, stable kind IDs, ordered same-kind effect instances, built-in NRT registry/factory, unknown kind/version diagnostics, deterministic JSON/compatibility tests.

Out of scope: current tracker hydration migration (#211), full engine snapshots (#138), routing (#136), runtime third-party modules.

## Ownership and touch set

Expected: new `node_registry/` contract/factory crate, workspace dependency metadata, architecture checker/dependency graph, focused fixtures/tests, audio-domain routing docs, and this packet. Existing DSP factory APIs are consumed without changing tracker hydration. #213 owns only param_manifest coalesced storage and must not edit node definitions.

## Inventory

- DSP instrument factories: monophonic oscillator (waveform), polyphonic oscillator, hi-hat, kick, snare, Moog DFAM, one-shot sample player, and loop sample player.
- DSP effect factories: mono/stereo reverb, mono delay, mono/stereo gain, mono Moog ladder, plus the existing deprecated mono distortion and filter.
- Current tracker JSON is a separate unversioned enum shape (`InstrumentData` and ordered `Vec<AudioEffect>`); current hydration assigns effect ID `1`. It remains unchanged for #211.

## Plan

- [x] Inventory built-in kinds/current serialized shapes.
- [x] Define versioned schemas and compatibility errors.
- [x] Implement NRT registry/factory.
- [x] Test same-kind instances, round-trip, unknown kinds/versions, and run full gates.

## Verification

- [x] `cargo test -p node_registry --all-targets`
- [x] `cargo test --workspace --all-targets`
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] `cargo clippy -p audio_backend --no-default-features --all-targets -- -D warnings`
- [x] `cargo test -p audio_backend --no-default-features --all-targets`
- [x] `cargo test -p audio_backend --no-default-features --test offline_golden`
- [x] `RUSTFLAGS='-D warnings' cargo test --release -p dsp diagnostics::tests::argument_evaluation_matches_diagnostic_build_mode`
- [x] `cargo fmt --all -- --check`
- [x] `python3 scripts/check_architecture.py`
- [x] `python3 scripts/check_rt_logging.py`
- [x] `python3 scripts/docs/check_docs.py`
- [x] `python3 scripts/docs/reconcile_work.py --check`
- [x] `python3 -m unittest scripts.docs.test_reconcile_work`
- [x] `python3 scripts/docs/sync_roadmap.py --stdout > /dev/null`
- [x] independent code review (approved; suggestions addressed and pre-existing reverb limitation recorded)

## Handoff

- Completed: versioned definitions, full static built-in inventory/resolution, sample-resource validation, ordered same-kind effect preparation, structured diagnostics, deterministic v1 fixture/tests, NRT boundary enforcement/docs, acceptance checklist update, and all verification gates.
- Remaining: review/merge workflow only; intentionally not performed in this task.
- Known risks: the existing DSP reverb applies decay and damping from independent base feedback values, so current hydration order makes damping overwrite decay; the registry deliberately preserves that behavior instead of changing DSP/golden output in #210. Deprecated distortion/filter factories remain registered and discoverable as deprecated for compatibility. Tracker hydration, snapshots, and routing remain #211/#138/#136.
- Next: hand off the clean issue commit without changing issue status or assignee.
