---
title: "Task Packet — Issue 234: Fixed-quantum Engine rendering"
summary: Integrate coalesced target bindings into both Engine render entry points using ADR 0006 control boundaries.
status: current
updated: 2026-08-07
issue: 234
---

# Task Packet — Issue 234: Fixed-quantum Engine rendering

## Identity

- Issue: [#234](https://github.com/jpalvarezl/blight-synth/issues/234)
- Owner/status: jpalvarezl / in-progress
- Branch/worktree: `issue/234-fixed-quantum-engine-rendering` / `/Users/jpalvarezl/code/blight-234`
- Base: `main` / `1dc3b01`

## Goal

Latch prepared coalesced bindings once per top-level Engine process call and deliver active smoothers through an absolute 16-frame phase using a representative gain target with no duplicate DSP smoothing.

## Scope / reviewability

One renderer-integration concept. Aim for 600–800 meaningful lines; 800–1,000 is acceptable for tightly coupled tests. Stop and re-split above ~1,000.

In scope: minimal prepared state access, one private renderer, one latch, union of event/quantum boundaries, gain scalar delivery, bounded work and RT tests.

Out of scope: reverb migration (#235), device-host lifecycle (#215), OSC (#216), broad target migration.

## Checkpoint plan

1. [x] Planner-only worker: exact state/renderer/API plan, no edits.
2. [x] Review plan and touched paths.
3. [x] Implementation worker: core renderer + focused tests only.
4. [x] Inspect diff/line count before full verification.
5. [x] Separate independent review/fix pass (APPROVE).
6. [x] Full verification and commit.
7. [ ] Open PR and request Copilot review.

## Verification

- [x] focused process/phase/order tests (46 passing across four targets)
- [x] engine/workspace strict gates and host-free tests
- [x] RT allocation/setter-bound measurements
- [x] fmt and diff checks
- [x] architecture, RT logging, docs/reconcile checks

## Handoff

- Completed: planner checkpoint, implementation checkpoint, independent REVISE review, bounded-work fixes, and focused re-review (APPROVE).
- Remaining: PR and Copilot review.
- Reviewability: about 431 production additions and 659 tightly coupled test additions. The total exceeds the soft 1,000-line signal because of 12 focused contract tests, not multiple production concepts; review found no clean split.
- Risk: device-host state replacement/retirement remains #215; representative reverb duplicate-smoother removal remains #235.
