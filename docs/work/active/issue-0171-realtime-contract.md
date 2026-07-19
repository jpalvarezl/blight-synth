---
title: Task Packet — Issue 171 Real-Time Contract
summary: Active context and handoff for defining callback safety and mapping enforcement work.
status: current
updated: 2026-07-18
issue: 171
owner: jpalvarezl
branch: issue/171-rt-contract
---

# Task Packet — Issue 171: Real-Time Contract

## Goal

Define the reviewed RT/NRT contract before Engine lifecycle implementation, inventory callback-reachable violations, and assign enforcement to focused #172–#175 work.

## Read first

1. [Current crate graph](../../architecture/crate-dependency-graph.md)
2. [Audio engine domain](../../domains/audio-engine.md)
3. Parent #133 and issue #171
4. Callback chain only: standalone AudioProcessor, Player/adapter, Engine, DSP effects/instruments

## Dependencies

- Parent: #133
- Blocks: #172, #173, #174, #175
- Informs/blocks implementation of #132

## Scope

- Thread roles and callback-reachable ownership.
- Allocation/deallocation, locks, I/O/logging, panic, and bounded-work rules.
- Control traffic classes, overload/backpressure, and deferred reclamation.
- Path-specific current violation inventory and verification plan.

No runtime enforcement implementation or final event/parameter/state API.

## Plan

- [x] Trace callback-reachable code and current ownership.
- [x] Define hard callback rules and prepared-state policy.
- [x] Define parameter/event/structural traffic classes.
- [x] Define bounded-work, backpressure, telemetry, and reclamation policy.
- [x] Inventory current violations and map each to an owner.
- [x] Reconcile Engine lifecycle prerequisites.
- [ ] Receive human/Copilot review and incorporate accepted decisions.

## Verification

- [ ] `python3 scripts/docs/check_docs.py`
- [ ] `python3 scripts/docs/sync_roadmap.py --check`
- [ ] `git diff --check`

## Handoff

- Completed: proposed RT contract, current violation inventory, enforcement split, and #132 prerequisite mapping.
- Remaining: human/Copilot contract review before unblocking implementation issues.
- Known risks: initial command/event budgets and retirement overflow strategy are intentionally specified as requirements but not numeric/implemented until #173/#174.
- Next action: review policy choices, then mark #172/#173/#175 ready for parallel work and sequence #174 with ownership decisions.
