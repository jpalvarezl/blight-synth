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
- Status: in-progress — final design-ADR review round complete; awaiting merge
- Branch: `issue/145-event-source-contract`
- Worktree: `../blight-145-eventsource`
- Base branch/SHA: `origin/main` @ `a711c5c`
- Head: [PR #198](https://github.com/jpalvarezl/blight-synth/pull/198)
- Last handoff: 2026-07-26

## Goal

Record a coherent Proposed ADR that fixes the composition/engine/host boundary
and hard constraints without prematurely specifying the implementation owned by
#134/#132. The engine consumes bounded current-block events with producer-side
sample offsets; the tracker is one adapter, and a second runtime requires no
DSP-engine changes. This is documentation-only design work.

## Read first

1. [Composition domain](../../domains/composition.md)
2. [Real-time audio contract](../../architecture/realtime-contract.md)
3. [ADR 0003](../../decisions/0003-event-source-contract.md) and its
   [routing page](../../architecture/event-source-contract.md)
4. `audio_backend/src/player/tracker_engine_adapter.rs`,
   `audio_backend/src/player/mod.rs`, and `sequencer/src/timing/mod.rs`

## Dependencies and blockers

- Depends on: accepted [real-time audio contract](../../architecture/realtime-contract.md)
- Blocks: #134 concrete scheduling, #132 lifecycle, and tracker/composite adapter extraction
- Coordinates with: #138 snapshot/state semantics, #121 parameter IDs, #174 reclamation
- Current blocker: none

## Scope and non-goals

### In scope

- Decide ownership of clock mapping and the composition→event→engine boundary.
- Set hard bounds for current-block pull, optional lookahead, ordering,
  overload/recovery, determinism, generation changes, and NRT side effects.
- Resolve seek/discontinuity activation to one next-block generation rule.
- Name mechanism questions for implementing issues without choosing protocols,
  packet layouts, ordering keys, or RNG reconstruction designs here.
- Correctly characterize the current tracker map and timing behavior.

### Out of scope

- Rust, DSP, engine, sequencer, or Cargo changes.
- Concrete API/event schema, queue/atomic protocol, tracker extraction, or final
  composition language/UI.
- Accepting the ADR; it remains Status: Proposed while issue/PR review is open.

## Ownership and touch set

Expected paths:

- `docs/decisions/0003-event-source-contract.md`
- `docs/architecture/event-source-contract.md`
- `docs/work/active/issue-145-event-source-contract.md`

Shared contracts/schemas touched: proposed event-source design only; the
accepted RT contract is read-only. Parallel packet/burndown drift is not part of
this branch.

## Plan

- [x] Confirm issue, PR, branch, current tracker caller bounds, and timing behavior.
- [x] Correct the tracker `HashMap` factual claim.
- [x] Assign future-clock mapping and define fallback to current-block pull.
- [x] Select one next-block seek/reset/discontinuity activation rule.
- [x] Reduce queue/order/RNG mechanism to constraints plus named open questions.
- [x] Add explicit implementation latitude while preserving ADR governance.
- [x] Keep semantic/render determinism, outbound side-effect, first-party-path,
  and `TimingState::advance` qualifications.
- [x] Update the routing page and this handoff.
- [x] Run documentation/work-state checks, commit, and push PR #198.

## Progress and decisions

- 2026-07-24 — Initial ADR/routing draft opened as
  [PR #198](https://github.com/jpalvarezl/blight-synth/pull/198).
- 2026-07-26 — Earlier review revision selected producer-side clock authority,
  bounded current pull/lookahead, deterministic ordering/recovery, and clarified
  determinism and NRT side effects.
- 2026-07-26 — Final review right-sized the Proposed ADR. The tracker map is now
  factual: `with_capacity(MAX_TRACKS)` plus at most `MAX_TRACKS` distinct keys
  does not currently reallocate; the gap is implicit rather than structural
  enforcement.
- 2026-07-26 — The selected host clock adapter owns prepared current mapping and
  any reliable bounded future mapping. Without a reliable future mapping, the
  runtime falls back to current-block pull if it can meet RT bounds; otherwise
  that runtime/clock combination cannot run yet.
- 2026-07-26 — Seek, reset, and clock discontinuity now activate only at offset
  zero of the next block with a new generation; the conflicting in-block seek
  barrier was removed.
- 2026-07-26 — SPSC/coverage/memory ordering/full recovery, producer identity and
  sequence, exact order/admission key, and seek/loop RNG/interpreter
  reconstruction are named implementation questions rather than prescribed
  mechanisms.
- 2026-07-26 — An explicit latitude statement delegates mechanism to #134/#132
  and allows evidence-driven refinement/replacement while requiring normal ADR
  amendment or supersession for boundary/constraint changes.

## Verification

- [x] `python3 scripts/docs/check_docs.py` — documentation check passed: 28 pages
- [x] `python3 scripts/docs/reconcile_work.py --check` — #145 packet/index are
  consistent; unrelated parallel branch work-state warnings were not chased

## Handoff

- Completed: Final ADR review round applied, verified, committed, and pushed to
  PR #198.
- Remaining: Maintainer review and merge/acceptance decision. Concrete mechanism
  and Rust implementation remain in #134/#132/#138 and adapter follow-ups.
- Known risks: A runtime that is neither direct-RT bounded nor able to obtain a
  reliable future clock window cannot run under this boundary until an accepted
  compliant strategy exists. Fail-closed recovery can audibly stop playback.
- Next smallest action: Review PR #198 for acceptance as a right-sized Proposed
  decision, not an implementation specification.
- Files to read next: `docs/decisions/0003-event-source-contract.md`,
  `docs/architecture/event-source-contract.md`, and
  `docs/architecture/realtime-contract.md`.
