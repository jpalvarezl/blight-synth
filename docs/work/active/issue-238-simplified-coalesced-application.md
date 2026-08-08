---
title: "Task Packet — Issue 238: Simplified coalesced application"
summary: Apply mapped coalesced targets once per valid block and delete unused Engine smoothing infrastructure.
status: current
updated: 2026-08-07
issue: 238
---

# Task Packet — Issue 238: Simplified coalesced application

## Identity

- Issue: [#238](https://github.com/jpalvarezl/blight-synth/issues/238)
- Owner/status: jpalvarezl / in-progress
- Branch/worktree: `issue/238-simplified-coalesced-application` / `/Users/jpalvarezl/code/blight-238`
- Base: `main` / pending packet commit

## Goal

Keep useful generation-bound coalescing/mapping/confirmation, implement first block-start target delivery, and remove unused generic Engine smoother/fixed-quantum infrastructure per ADR 0007.

## Scope / reviewability

One deletion-heavy simplification/integration. Target 500–800 meaningful lines excluding removed tests; pause/re-split if new production logic exceeds ~1,000.

In scope: remove PreparedSmoother/per-binding smoother state/tests/dependencies; generic bindings accept None and reject unsupported Smoothed; map and resolve/invoke concrete scalar setters once at valid top-level process start; confirm only successful resolution/invocation; install/reset seed application; preserve event validation/order and DSP-local smoothing; master gain policy None.

Out of scope: device-host generation lifecycle (#215), OSC (#216), new DSP smoothing.

## Plan

- [ ] Remove unused smoother and simplify binding preparation/application.
- [ ] Add minimal constructor-time Engine coalesced state and one private non-relatching renderer.
- [ ] Apply/confirm dirty targets before offset-zero events; preserve invalid-event pending state and zero-frame behavior.
- [ ] Prove failures/seed/reset/sample-event/zero-heap behavior and measure code deletion.
- [ ] Run full gates/review and prepare PR.

## Handoff

- Completed: issue claimed and packet created.
- Remaining: implementation/PR.
- Risk: existing scalar setters are infallible; application success means concrete target resolution plus invocation, not DSP semantic validation.
