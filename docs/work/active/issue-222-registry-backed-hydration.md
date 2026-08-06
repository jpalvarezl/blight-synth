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
- Base: `main` / `060b11f`

## Goal

Replace active tracker hydration type switches with the merged legacy adapter plus NRT node registry, preserving command order, envelope compatibility, retirement, and audio behavior.

## Scope and reviewability

One coherent integration concept. Aim for roughly 500–800 meaningful lines including focused tests; pause/re-split above ~1,000. Do not change schemas, routing, UI, or node definitions.

Expected paths: `audio_backend/src/song_hydration.rs`, architecture dependency enforcement, focused effect-bearing equivalence tests, packet/generated docs.

## Plan

- [ ] Map prepared registry owners into existing structural commands.
- [ ] Preserve explicit legacy envelope commands and effect ordering/IDs.
- [ ] Add multi-effect/DFAM/unknown diagnostics and retirement tests.
- [ ] Run workspace/golden/RT/strict checks and review.

## Handoff

- Completed: claimed/packet.
- Remaining: implementation/PR.
- Risk: registry payloads do not own legacy envelope commands; preserve them explicitly.
