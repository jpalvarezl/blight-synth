---
title: "Task Packet — Issue 244: Initial coalesced device-host generation"
summary: Install one prepared coalesced generation and expose NRT stable-ID publish/confirmation access.
status: current
updated: 2026-08-08
issue: 244
---

# Task Packet — Issue 244: Initial coalesced device-host generation

## Identity

- Issue: [#244](https://github.com/jpalvarezl/blight-synth/issues/244)
- Owner/status: jpalvarezl / in-progress
- Branch/worktree: `issue/244-initial-coalesced-device-host` / `/Users/jpalvarezl/code/blight-244`
- Base: `main` / pending packet commit

## Goal

Prepare/install one initial manifest/table/store/binding generation before callback processing and expose stable-ParameterId NRT publication plus applied-confirmation queries.

## Scope / reviewability

One initial lifecycle vertical slice, generally 500–800 meaningful lines. Constructor plumbing through device host/Player/Engine, NRT facade, block-start application, failures, no queue growth, RT tests. No replacement/rebind/retirement (#245) and no OSC (#216).

## Plan

- [ ] Define NRT facade and initial preparation.
- [ ] Plumb constructor-owned state to Engine.
- [ ] Expose stable-ID publish/confirmation and failures.
- [ ] Verify queue independence, zero heap, goldens, and host behavior.

## Handoff

- Completed: claimed/packet.
- Remaining: implementation/PR.
