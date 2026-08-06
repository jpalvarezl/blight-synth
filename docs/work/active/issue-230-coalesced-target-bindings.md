---
title: "Task Packet — Issue 230: Coalesced target bindings"
summary: Prepare target bindings and directly test map/latch/confirmation without render integration.
status: current
updated: 2026-08-06
issue: 230
---

# Task Packet — Issue 230: Coalesced target bindings

## Identity

- Issue: [#230](https://github.com/jpalvarezl/blight-synth/issues/230)
- Owner/status: jpalvarezl / in-progress
- Branch/worktree: `issue/230-coalesced-target-bindings` / `/Users/jpalvarezl/code/blight-230`
- Base: `main` / `d6fe65d`

## Goal

Build NRT-prepared coalesced target bindings and a directly testable store-drain application layer that maps, latches smoother targets, and confirms applied revisions.

## Reviewability / scope

One application/binding concept, generally 500–800 meaningful lines; pause/re-split above ~1,000. No render quantum, DSP setter delivery, duplicate smoother removal, host lifecycle, or OSC.

Expected paths: narrow engine binding/application module, focused mapping/confirmation/reset/error/allocation tests, docs/packet.

## Plan

- [ ] Define prepared binding table validation and reset seeds.
- [ ] Implement drain map/latch/confirm/failure path.
- [ ] Test None/linear/exponential direct application and zero allocation.
- [ ] Run full gates/review.

## Handoff

- Completed: claimed/packet.
- Remaining: implementation/PR.
- Risk: retain exact generation/runtime-table ownership and confirmation semantics.
