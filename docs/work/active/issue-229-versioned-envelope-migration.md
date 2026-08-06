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
- Base: `main` / `d6fe65d`

## Goal

Version node payloads to carry current amplitude/kick pitch envelopes, migrate v1 definitions, prepare those values on NRT, and delete compatibility envelope hydration commands.

## Reviewability / scope

One semantic migration, generally 500–800 meaningful lines; pause/re-split above ~1,000. No full snapshot, routing, UI redesign, or generic envelope graph.

Expected paths: node_registry definitions/registry/migration fixtures, legacy adapter, song hydration compatibility deletion, focused behavior/golden tests.

## Plan

- [ ] Add versioned envelope payload/migration.
- [ ] Prepare configured envelopes via registry.
- [ ] Adapt legacy values and remove compatibility commands/type switch.
- [ ] Verify behavior/goldens/NRT/RT and review.

## Handoff

- Completed: claimed/packet.
- Remaining: implementation/PR.
- Risk: preserve current envelope sonic behavior exactly.
