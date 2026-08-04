---
title: "Task Packet — Issue 209: Typed instance IDs"
summary: Active context for replacing interchangeable DSP/engine identity aliases with typed newtypes.
status: current
updated: 2026-08-03
issue: 209
---

# Task Packet — Issue 209: Typed instance IDs

## Identity

- Issue: [#209](https://github.com/jpalvarezl/blight-synth/issues/209)
- Owner: jpalvarezl
- Status: in-progress
- Branch: `issue/209-typed-instance-ids`
- Worktree: `/Users/jpalvarezl/code/blight-209`
- Base branch/SHA: `main` / `5ba3241`
- Head: branch tip at handoff
- Last handoff: 2026-08-03

## Goal

Replace interchangeable instance/resource ID aliases with compact typed newtypes while preserving numeric project compatibility, deterministic ordering, and RT-safe use, as scoped by #209.

## Read first

1. [Audio engine domain](../../domains/audio-engine.md)
2. [System boundaries](../../architecture/system-boundaries.md) and [RT contract](../../architecture/realtime-contract.md)
3. `dsp/src/id.rs`, `engine/src/commands.rs`, `engine/src/events.rs`, factories and hydration call sites

## Dependencies and blockers

- Depends on: none
- Blocks: #210 and parent #135
- Current blocker: none

## Scope and non-goals

### In scope

- Typed IDs and migration of current engine/DSP/host/project call sites.
- Explicit raw access/conversion and persisted numeric compatibility.
- Focused type-safety, serde, workspace, golden, and RT tests.

### Out of scope

- Versioned node definition schemas/registry (#210), hydration migration (#211), routing (#136).

## Ownership and touch set

Expected paths: `dsp/src/id.rs`, engine/DSP command/factory APIs, audio-backend hydration/adapters/examples/tests, affected project model fields only when needed for compatibility, and this packet.

Shared contracts touched: public Rust ID types; #209 is the sole owner. #212 is documentation-only and must not change ID APIs.

Potential parallel conflicts: #179 and other ID consumers remain backlog until this contract lands.

## Plan

- [x] Inventory aliases and persistence boundaries.
- [x] Add compact typed IDs with explicit raw conversion and model-adapter behavior.
- [x] Migrate engine/DSP/audio-backend callers and tests.
- [x] Run full verification and independent review.

## Progress and decisions

- 2026-08-03 — Split from #135 as the critical-path identity contract owner.
- 2026-08-03 — Inventory complete before implementation: all six scoped IDs are interchangeable `u32` aliases in `dsp/src/id.rs`. `InstrumentId`, `EffectId`, `VoiceId`, and `EnvelopeId` cross DSP factories/traits/commands; engine commands/events/parameter targets use instrument/effect IDs; `SampleId` keys `audio_backend::ResourceManager`; `EffectChainId` currently has no consumers. Tracker GUI, examples, and tests construct IDs from literals/casts.
- 2026-08-03 — Persistence inventory: project JSON/bincode stores `sequencer::models::Instrument::id` as `usize` and tracker `Event::instrument_id` as `u8`; hydration/player are the model-to-runtime adapter boundaries. No runtime effect, chain, envelope, sample, or voice ID is currently persisted. Project model field shapes remain unchanged, but hydration now rejects instrument-bank IDs above `u8::MAX` because current tracker events/UI cannot address them consistently; focused JSON/model-adapter tests lock the numeric shape. Runtime IDs deliberately do not gain serde until they directly cross a persistence boundary.
- 2026-08-03 — ID contract decision: six distinct `#[repr(transparent)]` `u32` newtypes with `Copy`/equality/order/hash, explicit const `from_raw`/`raw`, and no cross-ID conversions. `NoteId` remains unchanged.
- 2026-08-03 — End-to-end migration completed across DSP factories/nodes, engine commands/events/parameter targets, tracker hydration/playback, resources, device-host tests, GUI, and examples. Compile-fail docs prove cross-domain rejection; unit tests prove size/alignment/order/hash/copy behavior and project numeric JSON compatibility.
- 2026-08-03 — Independent review verdict: APPROVE, with no critical findings or warnings. Addressed the optional observability suggestion by making the tracker UI's narrowing conversion fail explicitly instead of silently dropping updates.

## Verification

- [x] focused ID/model-adapter/type-safety tests and `cargo test -p dsp --doc`
- [x] `cargo test --workspace --all-targets`
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] `cargo clippy -p audio_backend --no-default-features --all-targets -- -D warnings`
- [x] `cargo test -p audio_backend --no-default-features --all-targets`
- [x] `cargo test -p audio_backend --no-default-features --test offline_golden`
- [x] `cargo fmt --all -- --check` and `git diff --check`
- [x] architecture, RT logging, docs, reconciliation, and reconciliation-unit checks

## Handoff

- Completed: implementation, compatibility tests, full verification, generated burndown reconciliation, and independent review.
- Remaining: none in the requested local implementation scope; GitHub metadata/PR work was intentionally not performed.
- Known risks: tracker project instrument IDs remain addressable only in the existing `u8` event range even though the model field is `usize` and runtime newtypes are `u32`; hydration and UI reject IDs above 255 explicitly. Runtime IDs intentionally have no serde until a runtime ID directly crosses persistence.
- Next smallest action: review the focused commit and decide external PR/issue workflow separately.
- Files a new agent should read next: this packet, `dsp/src/id.rs`, and `audio_backend/src/song_hydration.rs`.
