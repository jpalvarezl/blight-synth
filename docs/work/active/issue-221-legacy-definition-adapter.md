---
title: "Task Packet — Issue 221: Legacy definition adapter"
summary: Pure legacy tracker-model to versioned node-definition adaptation without hydration wiring.
status: current
updated: 2026-08-06
issue: 221
---

# Task Packet — Issue 221: Legacy definition adapter

## Identity

- Issue: [#221](https://github.com/jpalvarezl/blight-synth/issues/221)
- Owner: jpalvarezl
- Status: in-progress
- Branch: `issue/221-legacy-definition-adapter`
- Worktree: `/Users/jpalvarezl/code/blight-221`
- Base: `main` / `dcfd9fb`
- Head: branch tip at handoff

## Goal

Add a pure adapter from current tracker models to versioned node definitions, with deterministic effect identity, while leaving active hydration unchanged.

## Read first

1. [Audio engine domain](../../domains/audio-engine.md)
2. `audio_backend/src/song_hydration.rs`, `sequencer/src/models/instruments.rs`, `node_registry/src/definitions.rs`

## Scope / review budget

- One adapter concept; target 200–400 changed lines including tests, hard stop/re-split near 500.
- Deterministic ordered effect IDs, DFAM implicit ladder placement, current kind/payload mapping, structured adapter errors.
- No hydration wiring, command/factory installation, UI, routing, or golden changes.

Expected paths: `audio_backend/src/legacy_definition_adapter.rs`, its crate manifest/export, the dependency graph, focused tests, and this packet. #223 is docs-only and disjoint.

Contract decisions: the adapter lives in host-free `audio_backend`, the existing bridge between tracker models and NRT preparation, and introduces the deliberate `audio_backend -> node_registry` edge. Ordered legacy effect slots use one-based IDs; DFAM's implicit ladder occupies slot/ID 1 before user effects. Finite legacy effect values are normalized exactly where the current DSP setters clamp them so registry validation does not reject currently playable data; non-finite values and legacy instrument variants without a faithful registry representation return compact structured adapter errors. Envelope commands remain outside the constructor definition payload and active hydration remains unchanged for #222.

## Plan

- [x] Inventory legacy-to-registry mapping and current clamp semantics.
- [x] Implement pure adapter and deterministic IDs.
- [x] Add multi-effect, DFAM, round-trip, invalid-value tests.
- [x] Run focused/full gates and review.

## Verification

- [x] `cargo test -p audio_backend --no-default-features --test legacy_definition_adapter`
- [x] `cargo test --workspace --all-targets`
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] host-free audio-backend tests/strict Clippy and `offline_golden`
- [x] fmt, architecture, RT-logging, docs, and reconciliation checks
- [x] independent review (approved; envelope deferral documented)

## Handoff

- Completed: pure host-free adapter, one-based ordered effect IDs, explicit DFAM ladder slot, faithful kind/payload and clamp mapping, structured unrepresentable-data errors, focused regressions, dependency documentation, and full gates.
- Remaining: review/merge workflow and #222 registry-backed hydration wiring.
- Risk: registry constructor payloads do not represent legacy amplitude or kick pitch envelopes; #222 must preserve the existing explicit envelope commands while switching preparation. `Sample` and generic `Synth` remain structurally unrepresentable and return typed adapter errors. Active hydration, factories, commands, UI, routing, DSP, and goldens are unchanged.
