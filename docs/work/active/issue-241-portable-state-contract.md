---
title: "Task Packet — Issue 241: Portable state contract"
summary: Decide the minimal portable engine-state envelope, compatibility, and restore ownership.
status: current
updated: 2026-08-08
issue: 241
---

# Task Packet — Issue 241: Portable state contract

## Identity

- Issue: [#241](https://github.com/jpalvarezl/blight-synth/issues/241)
- Owner/status: jpalvarezl / in-progress
- Branch/worktree: `issue/241-portable-state-contract` / `/Users/jpalvarezl/code/blight-241`
- Base: `main` / pending packet commit

## Goal

Accept the smallest M1 portable state envelope and compatibility/NRT-restore contract before implementation.

## Scope / reviewability

Contract only, roughly 150–300 meaningful lines. Decide tagged composition payload, node definitions/parameters/routing/assets/seeds scope, ephemeral-state exclusion, migration/unknown diagnostics, NRT preparation and block-boundary handoff, and core bytes versus host adapters.

## Plan

- [ ] Inventory current tracker/project and reusable definition state.
- [ ] Decide minimal envelope/non-goals and compatibility.
- [ ] Define NRT/RT ownership and adapter boundaries.
- [ ] Stabilize #242/#243 and review docs.

## Handoff

- Completed: claimed/packet.
- Remaining: ADR/docs PR.
