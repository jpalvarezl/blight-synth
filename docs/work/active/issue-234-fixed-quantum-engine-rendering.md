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
- Base: `main` / pending task-packet commit

## Goal

Latch prepared coalesced bindings once per top-level Engine process call and deliver active smoothers through an absolute 16-frame phase using a representative gain target with no duplicate DSP smoothing.

## Scope / reviewability

One renderer-integration concept. Aim for 600–800 meaningful lines; 800–1,000 is acceptable for tightly coupled tests. Stop and re-split above ~1,000.

In scope: minimal prepared state access, one private renderer, one latch, union of event/quantum boundaries, gain scalar delivery, bounded work and RT tests.

Out of scope: reverb migration (#235), device-host lifecycle (#215), OSC (#216), broad target migration.

## Checkpoint plan

1. Planner-only worker: exact state/renderer/API plan, no edits.
2. Review plan and touched paths.
3. Implementation worker: core renderer + focused tests only.
4. Inspect diff/line count before full verification.
5. Separate review/fix pass, then PR.

## Verification

- [ ] focused process/phase/order tests
- [ ] engine/workspace strict gates and host-free tests
- [ ] RT allocation/setter-bound measurements
- [ ] fmt, architecture, RT logging, docs/reconcile checks

## Handoff

- Completed: split/claim/packet.
- Remaining: checkpointed planning and implementation.
- Risk: public process_with_events must validate before latch and never recursively relatch.
