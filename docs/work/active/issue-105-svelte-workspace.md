---
title: "Task Packet — Issue 105: Svelte workspace and EngineClient boundary"
summary: Create the launchable production Svelte workspace and minimal mockable current-slice client contract.
status: current
updated: 2026-08-10
issue: 105
---

# Task Packet — Issue 105: Svelte workspace and EngineClient boundary

## Identity

- Issue: [#105](https://github.com/jpalvarezl/blight-synth/issues/105)
- Owner/status: jpalvarezl / in-progress
- Branch/worktree: `issue/105-svelte-workspace` / `/Users/jpalvarezl/code/blight-105`
- Base: `main` / `dda6e0a`

## Goal

Create a launchable/buildable `gui/` Svelte/TypeScript workspace and a minimal typed/mockable EngineClient boundary for the current transport/gain/meter slice, without host or backend integration.

## Read first

1. [NOW](../../NOW.md)
2. [Frontend domain](../../domains/frontend.md)
3. Live issue #105
4. Existing root CI/README conventions; inspect tracker GUI only for visual/reference context if needed

## Scope / non-goals

In scope: Bun/Vite/Svelte/strict TypeScript workspace, static production output, minimal current-slice EngineClient types/interface, fake/mock implementation, one mock parameter/meter development/test view, build/typecheck/test/lint commands, CI checks, custom-base asset behavior.

Out of scope: process supervision (#106), OSC/network (#107), production stores (#108), final transport/gain/meter components, desktop packaging, Rust changes, plugins/MIDI/state/archive work.

## Expected touch set

- `gui/`
- `.github/workflows/ci.yml`
- `.gitignore`
- `README.md` and focused frontend/work docs only if required
- this packet and generated burndown

## Plan

- [ ] Scaffold strict Svelte/TypeScript workspace and commands.
- [ ] Define minimal current-slice EngineClient and fake client.
- [ ] Add launchable mock view and browser/unit tests.
- [ ] Verify static custom-base build and CI integration.
- [ ] Run focused/full checks and independent review.

## Handoff

- Completed: live verification, claim, NOW transition, packet.
- Remaining: implementation, tests, PR.
- Risk: do not expand the interface for future hosts or build #106–#112 early.
