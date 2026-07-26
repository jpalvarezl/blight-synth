---
title: "Task Packet — Issue 145: Event-source contract (design)"
summary: Define the engine/composition/host boundary so the engine consumes bounded, already-offset events instead of owning tracker document or clock semantics.
status: current
issue: 145
updated: 2026-07-26
---

# Task Packet — Issue 145: Event-source contract (design)

## Identity

- Issue: [#145](https://github.com/jpalvarezl/blight-synth/issues/145)
- Owner: jpalvarezl
- Status: in-progress — PR review revisions complete; awaiting review/merge
- Branch: `issue/145-event-source-contract`
- Worktree: `../blight-145-eventsource`
- Base branch/SHA: `origin/main` @ `a711c5c`
- Head SHA: [PR #198](https://github.com/jpalvarezl/blight-synth/pull/198) (this packet is updated in the review-fix commit)
- Last handoff: 2026-07-26

## Goal

Record a proposed ADR and draft routing contract that separate the audio engine,
composition event sources, and host/control/clock adapters. The engine consumes
bounded current-block events with producer-computed sample offsets; the tracker
becomes one adapter, and a second runtime needs no DSP-engine changes. This is a
documentation-only design task.

## Read first

1. [Audio engine domain](../../domains/audio-engine.md)
2. [Real-time audio contract](../../architecture/realtime-contract.md), then
   [ADR 0003](../../decisions/0003-event-source-contract.md)
3. `audio_backend/src/player/mod.rs`,
   `audio_backend/src/player/tracker_engine_adapter.rs`, `engine/src/lib.rs`,
   `engine/examples/offline_render.rs`, and `sequencer/src/timing/mod.rs`

## Dependencies and blockers

- Depends on: accepted [real-time audio contract](../../architecture/realtime-contract.md)
- Blocks: #134 concrete scheduling, #132 lifecycle, and the #145 tracker/composite adapter extraction
- Coordinates with: #138 snapshot/generation semantics, #121 stable parameter IDs, #174 reclamation
- Landing coordination: #134 owns/lands the shared event and recovery schema first;
  #132 and the #145 adapter extraction consume it rather than changing the same
  public surface in parallel
- Current blocker: none

## Scope and non-goals

### In scope

- Define one clock/timestamp owner and host-orchestrated current-block pull.
- Separate fixed-capacity current-block output from bounded NRT lookahead
  publication, coverage, stale invalidation, and underflow recovery.
- Define cross-producer ordering, ordinary overflow admission, and reserved
  all-notes-off/transport-stop recovery.
- Specify tracker RT-state requirements, seeded seek/loop/restart behavior,
  semantic versus render determinism, and outbound side-effect restrictions.
- Name the implementation work owned by #134, #132, #138, and the composition
  adapter extraction.

### Out of scope

- Rust, DSP, engine, sequencer, or Cargo changes.
- Implementing the event schema/API, extracting the tracker, or selecting the
  final composition language/UI.
- Accepting the ADR; it remains Status: Proposed while issue/PR review is open.

## Ownership and touch set

Expected paths:

- `docs/decisions/0003-event-source-contract.md`
- `docs/architecture/event-source-contract.md`
- `docs/decisions/README.md`
- `docs/architecture/README.md`
- `docs/domains/composition.md`
- `docs/work/active/README.md`
- `docs/work/active/issue-145-event-source-contract.md`

Shared contracts/schemas touched: proposed event-source contract only; the
accepted RT contract is read-only.

Potential parallel conflicts: #121/#137 may change adjacent docs but must not be
folded into this branch. A parallel active-packet reconciliation warning is not
a blocker.

## Plan

- [x] Verify current issue, PR, branch, and tracker/engine/timing behavior.
- [x] Redesign clock authority and split current pull from NRT lookahead.
- [x] Define total ordering, deterministic overflow, and reserved recovery.
- [x] Correct tracker RT-state, determinism, outbound effects, and current-path
  claims.
- [x] Update routing/index text and this PR-aware packet.
- [x] Run documentation and work-state checks.
- [x] Commit and push review revisions to PR #198.

## Progress and decisions

- 2026-07-24 — Initial ADR/routing draft opened as
  [PR #198](https://github.com/jpalvarezl/blight-synth/pull/198).
- 2026-07-26 — Independent review returned REVISE. The contract was redesigned
  rather than patched: the host/producer side is now the sole clock mapping
  authority, while the engine only applies already-offset events.
- 2026-07-26 — Current-block pull now uses caller-owned prepared storage;
  absolute-frame NRT lookahead has separate bounded publication and coverage.
  Deadline miss, edit/seek/clock invalidation, and explicit resume are defined.
- 2026-07-26 — The total merge key and retained-prefix overflow policy are
  paired with a reserved coalescing recovery barrier, so stop/all-notes-off
  cannot be crowded out.
- 2026-07-26 — Direct RT tracker evaluation now requires structurally fixed
  indexed state; the existing callback `HashMap::insert` is explicitly not
  cured by its current adapter placement. Otherwise evaluation runs on NRT.
- 2026-07-26 — Determinism is split into semantic event equality and the offline
  contract's platform-qualified PCM policy. RNG seek/checkpoint and repeat/evolve
  loop rules are explicit. Direct-RT outbound side-effect generation is
  prohibited under this ADR.
- 2026-07-26 — Design challenge added bidirectional bounded clock segments,
  two-cursor lookahead slot reuse, engine-global M1 recovery scope, and explicit
  #134-first ownership of the shared event/recovery schema.

## Verification

- [x] `python3 scripts/docs/check_docs.py` — documentation check passed: 28 pages
- [x] `python3 scripts/docs/reconcile_work.py --check` — #145 packet/index are
  consistent; reports only parallel work-state drift (burndown differs and
  #137/#121 packets are absent in this worktree), which was not chased

## Handoff

- Completed: Review findings addressed in the proposed ADR, routing page,
  registrations, and task packet; committed and pushed to PR #198.
- Remaining: Independent re-review and merge/acceptance decision. Concrete Rust
  implementation remains in the named follow-up issues.
- Known failures/risks: Fail-closed overflow/underflow intentionally stops
  composition playback; implementation must expose counters and tune prepared
  capacities. The ADR remains proposed.
- Next smallest action: Re-review PR #198 against the 11 findings and the
  accepted RT contract.
- Files a new agent should read next:
  `docs/decisions/0003-event-source-contract.md`,
  `docs/architecture/event-source-contract.md`, and
  `docs/architecture/realtime-contract.md`.
