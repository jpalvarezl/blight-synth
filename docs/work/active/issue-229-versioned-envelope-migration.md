---
title: "Task Packet — Issue 229: Versioned envelope migration"
summary: Move legacy instrument envelope configuration into versioned definitions and registry preparation.
status: current
updated: 2026-08-06
issue: 229
---

# Task Packet — Issue 229: Versioned envelope migration

## Identity

- Issue: [#229](https://github.com/jpalvarezl/blight-synth/issues/229)
- Owner/status: jpalvarezl / in-progress
- Branch/worktree: `issue/229-versioned-envelope-migration` / `/Users/jpalvarezl/code/blight-229`
- Base: `main` / `654b4ea`

## Goal

Version node payloads to carry current amplitude/kick pitch envelopes, migrate v1 definitions, prepare those values on NRT, and delete compatibility envelope hydration commands.

## Reviewability / scope

One semantic migration, generally 500–800 meaningful lines; pause/re-split above ~1,000. No full snapshot, routing, UI redesign, or generic envelope graph.

Expected paths: node_registry definitions/registry/migration fixtures, legacy adapter, song hydration compatibility deletion, focused behavior/golden tests.

## Plan

- [x] Add versioned envelope payload/migration.
- [x] Prepare configured envelopes via registry.
- [x] Adapt legacy values and remove compatibility commands/type switch.
- [x] Verify behavior/goldens/NRT/RT and review.

## Acceptance

- [x] Tracker amplitude settings survive adapter → definition JSON → NRT registry preparation.
- [x] Kick frequency delta and pitch decay survive the same path.
- [x] The committed v1 fixture migrates deterministically to canonical v2 while retaining effects and unknown payload data.
- [x] Compatibility envelope commands and the hydration `InstrumentData` switch are deleted.
- [x] Focused behavior tests and regenerated canonical references cover the semantic change.
- [x] Factories/preparation remain NRT; structural owner handoff, order, and RT retirement are unchanged.

## Handoff

- Completed: implementation, focused/full verification, golden regeneration, and independent review.
- Remaining: handoff only; no push, PR, merge, issue close, or GitHub status mutation in this task.
- Verification: `cargo test --workspace --all-targets --all-features`; strict all-feature and host-free Clippy/tests; release RT diagnostic check; offline golden; fmt; architecture; RT logging; docs; work reconciliation.
- Intentional behavior correction: legacy kick pitch commands targeted `None` and were ignored, and pitch decay was never emitted. Registry preparation now applies both authored values to pitch envelope 1. This changes only `ending_theme_no_effect.json`'s reviewed reference; amplitude behavior and owner order remain unchanged.
- Risk: the regenerated macOS-arm64 WAVs under `target/offline-renders/` were not human-auditioned in this non-interactive run. The migration API is public and tested but has no persisted node-definition loader caller yet.
