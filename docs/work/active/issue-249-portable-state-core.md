---
title: "Task Packet — Issue 249: Portable state core"
summary: Implement ADR 0008's host-neutral envelope, canonical bytes, validation, diagnostics, and envelope-level migration.
status: current
updated: 2026-08-08
issue: 249
---

# Task Packet — Issue 249: Portable state core

## Identity

- Issue: [#249](https://github.com/jpalvarezl/blight-synth/issues/249)
- Owner/status: jpalvarezl / in-progress
- Branch/worktree: `issue/249-portable-state-core` / `/Users/jpalvarezl/code/blight-249`
- Base: `main` / pending packet commit

## Goal

Implement the host-neutral PortableStateV1 model and canonical JSON boundary without tracker Song adapters or Engine restore.

## Scope / reviewability

One core model/canonicalization concept, generally 500–800 meaningful lines; pause/re-split above ~1,000. Includes tagged source-preserving payloads, ordered definitions, normalized overlay, asset refs, RFC 8785 bytes, validation/diagnostics, and v0→v1 envelope fixture.

Out of scope: legacy Song adapter (#250), Engine restore (#243), filesystem/JUCE.

## Plan

- [ ] Define core types and canonical encoder.
- [ ] Add semantic validation/source-preserving diagnostics.
- [ ] Add asset validation and migration fixture.
- [ ] Run full gates/review and PR.

## Handoff

- Completed: #242 checkpoint split; claimed/packet.
- Remaining: implementation/PR.
