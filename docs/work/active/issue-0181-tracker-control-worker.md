---
title: Task Packet — Issue 181 Tracker Control Worker
summary: Active context and handoff for moving tracker audio preparation and reliable submission off egui.
status: current
updated: 2026-07-22
issue: 181
owner: jpalvarezl
branch: issue/181-tracker-control-worker
---

# Task Packet — Issue 181: Tracker Control Worker

## Identity

- Issue: #181
- Owner: jpalvarezl
- Status: in-progress
- Branch: `issue/181-tracker-control-worker`
- Base branch/SHA: `main` / `c521307`
- Head SHA: see branch head
- Last handoff: 2026-07-22

## Goal

Move `BlightAudio`, factories, hydration, and reliable RT-queue retry to a dedicated tracker NRT worker as required by [issue #181](https://github.com/jpalvarezl/blight-synth/issues/181), leaving egui with a typed semantic request/status handle.

## Read first

1. [Composition domain](../../domains/composition.md)
2. [Target system boundaries](../../architecture/system-boundaries.md)
3. [Real-time contract](../../architecture/realtime-contract.md)
4. `tracker_gui/src/audio.rs`
5. `tracker_gui/src/instrument_manager/backend.rs`
6. `tracker_gui/src/app.rs`

## Dependencies and blockers

- Depends on: #180 (merged FIFO API)
- Blocks: #173 closure
- Coordinates with: #182 (parallel standalone worker), #101, #174
- Current blocker: none

## Scope and non-goals

### In scope

- Dedicated NRT worker owning non-`Send` CPAL/`BlightAudio` lifecycle and factories.
- Typed semantic request/event boundary for egui.
- Reliable ordered retry with cancellable shutdown.
- Hardware-free FIFO ordering and shutdown tests.

### Out of scope

- Continuous-parameter coalescing (#101).
- Final event/composition schema (#134/#145).
- Atomic prepared-state swaps (#174/#138).

## Ownership and touch set

Expected paths:

- `tracker_gui/src/audio.rs`
- `tracker_gui/src/app.rs`
- `tracker_gui/src/instrument_manager/backend.rs`
- `docs/architecture/realtime-contract.md`
- `docs/domains/composition.md`
- `docs/work/active/`

Shared contracts/schemas touched: none; consume the FIFO API merged in #180 without changing `audio_backend` public contracts.

Potential parallel conflicts: #182 owns standalone files. Both branches touch active-task docs and the realtime contract; reconcile after the first PR merges.

## Plan

- [x] Replace UI-owned `BlightAudio` with a typed worker handle.
- [x] Move initialization, factories, hydration, and command retry to the worker.
- [x] Preserve request/command FIFO order and accepted-state events.
- [x] Make saturation waits cancellable during worker shutdown.
- [x] Add hardware-free worker ordering and shutdown tests.
- [x] Run complete local validation.
- [x] Run independent review and address findings.
- [ ] Request Copilot review and address findings.

## Progress and decisions

- 2026-07-22 — `BlightAudio` cannot move across threads because CPAL `Stream` is non-`Send`; the worker creates and destroys it on the same NRT thread.
- 2026-07-22 — UI requests are semantic (`Initialize`, `Reset`, transport, hydration, envelope, command) and ordered through one std channel. Factory work and command ownership never return to egui.
- 2026-07-22 — The worker retries the exact FIFO-front command with parked 1 ms backoff and observes shutdown between attempts.
- 2026-07-22 — UI playback/loop state updates only from worker events after RT-ring acceptance.
- 2026-07-22 — Independent review approved the concurrency model and identified reset/disconnection recovery edges. Failed resets now preserve the previous engine, while fatal callback disconnection clears worker/UI initialized state so a later Initialize rebuilds audio.

## Verification

- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace --all-targets` — 65 tests plus examples
- [x] `cargo clippy -p audio_backend --no-default-features --all-targets -- -D warnings`
- [x] `cargo test -p audio_backend --no-default-features --all-targets` — 10 tests plus examples
- [x] `python3 scripts/check_rt_logging.py`
- [x] `python3 scripts/check_architecture.py`
- [x] `python3 scripts/docs/check_docs.py`
- [x] `git diff --check`

## Handoff

- Completed: worker ownership, semantic UI boundary, cancellable reliable submission, focused tests, and threading docs.
- Remaining: hosted CI, Copilot review, and human PR review.
- Known failures/risks: the UI request channel is intentionally unbounded until #101 provides class-specific coalescing; a disconnected audio callback terminates current request processing and is surfaced through logging/status events.
- Next smallest action: push the reviewed branch and open the issue PR.
- Files a new agent should read next: this packet and the six Read first entries above.
