---
title: Task Packet — Issue 182 Standalone Control Worker
summary: Active context and handoff for moving reliable standalone command submission off the current-thread Tokio executor.
status: current
updated: 2026-07-22
issue: 182
owner: jpalvarezl
branch: issue/182-standalone-control-worker
---

# Task Packet — Issue 182: Standalone Control Worker

## Identity

- Issue: #182
- Owner: jpalvarezl
- Status: in-progress
- Branch: `issue/182-standalone-control-worker`
- Base branch/SHA: `main` / `c521307`
- Head SHA: see branch head
- Last handoff: 2026-07-22

## Goal

Implement the dedicated NRT command-control ownership required by [issue #182](https://github.com/jpalvarezl/blight-synth/issues/182), preserving standalone FIFO order and accepted-only OSC responses without blocking the current-thread Tokio executor.

## Read first

1. [Standalone host domain](../../domains/standalone-host.md)
2. [Target system boundaries](../../architecture/system-boundaries.md)
3. [OSC protocol snapshot](../../osc-spec.md)
4. `audio_backend/src/bin/dsp-core.rs`
5. `audio_backend/src/standalone/osc.rs`
6. `audio_backend/src/standalone/audio_frontend/`

## Dependencies and blockers

- Depends on: #180 (merged queue API)
- Blocks: #173 closure
- Coordinates with: #161 (future Tokio removal), #104 (confirmed parameter round-trip), #174 (prepared-state ownership)
- Current blocker: none

## Scope and non-goals

### In scope

- Dedicated NRT owner for `BlightAudio`, factories/resources, and reliable command submission.
- Executor-safe typed request/response boundary for OSC command and song-load operations.
- Strict FIFO ordering, accepted-only responses, saturation recovery, and responsive shutdown.
- Hardware-free worker tests.

### Out of scope

- Final parameter coalescing/event schemas (#101/#134).
- Atomic prepared-state installation/deferred reclamation (#174/#138).
- Full Tokio removal (#161).
- OSC address-space redesign.

## Ownership and touch set

Expected paths:

- `audio_backend/src/bin/dsp-core.rs`
- `audio_backend/src/standalone/osc.rs`
- `audio_backend/src/standalone/audio_frontend/`
- `audio_backend/src/song_hydration.rs`
- `docs/architecture/realtime-contract.md`
- `docs/domains/standalone-host.md`
- `docs/work/active/`

Shared contracts/schemas touched: standalone control-worker ownership only; no OSC wire or engine command schema changes.

Potential parallel conflicts: #181 may establish a related tracker-side worker boundary; share queue semantics but do not couple standalone lifecycle to tracker state.

## Plan

- [x] Define a Tokio-agnostic typed worker request/result boundary and shutdown contract.
- [x] Move `BlightAudio` plus reliable command/hydration ownership to the worker.
- [x] Keep OSC receive, metering, and Ctrl-C polling responsive while requests wait on RT capacity.
- [x] Preserve strict FIFO and accepted-only protocol responses.
- [x] Add hardware-free saturation, ordering, disconnection, and shutdown tests.
- [x] Run the complete local validation matrix.
- [x] Request Copilot review and address all findings.

## Progress and decisions

- 2026-07-22 — PR #180 merged the 64-item callback budget and distinct `try_send_command`/reliable `send_command` APIs.
- 2026-07-22 — A blocking call on the current-thread Tokio executor is forbidden; this issue owns the standalone consumer-side threading correction.
- 2026-07-22 — The worker should remain useful after #161 removes Tokio, so command ownership and ordering must not depend on Tokio runtime internals.
- 2026-07-22 — CPAL `Stream` is intentionally non-`Send`; `BlightAudio` is therefore constructed and remains on the worker thread rather than being moved from Tokio.
- 2026-07-22 — A bounded std request channel separates Tokio from the worker. The worker owns one pending FIFO, retries only its front command with 1 ms parked backoff, and never accepts later work ahead of it.
- 2026-07-22 — Song loads are parsed/prepared entirely on the worker into one ordered load/hydration batch. `/song/loaded` and parameter echoes are emitted only after every associated command reaches the RT ring.
- 2026-07-22 — Shutdown uses an atomic cancellation flag plus channel close; pending commands are destroyed on the NRT worker, and stalled RT capacity cannot prevent join. An RAII running guard also surfaces panic/disconnection exits to the Tokio loop.
- 2026-07-22 — The bounded ingress distinguishes Full from Disconnected and returns unaccepted requests; focused tests cover ingress saturation in addition to RT-ring saturation.
- 2026-07-22 — PR #183 Copilot review covered all 11 changed files; both findings (worker-owned master-gain documentation and task-index freshness) were applied.

## Verification

- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace --all-targets` — 66 tests plus examples
- [x] `cargo clippy -p audio_backend --no-default-features --all-targets -- -D warnings`
- [x] `cargo test -p audio_backend --no-default-features --all-targets` — 10 tests plus examples
- [x] `python3 scripts/check_rt_logging.py`
- [x] `python3 scripts/check_architecture.py`
- [x] `python3 scripts/docs/check_docs.py`
- [x] `git diff --check`

## Handoff

- Completed: dedicated worker ownership, bounded request/response bridge, deterministic song batches, saturation retry, worker health/shutdown, focused tests, and protocol/threading docs.
- Remaining: final hosted CI after review fixes and human PR review.
- Known failures/risks: requests rejected before bounded worker-queue acceptance remain caller-visible through missing success/error logging; high-rate values still await #101 coalescing. The 1 ms worker poll/backoff is transitional until #161 finalizes the synchronous control loop.
- Next smallest action: push review fixes, resolve threads, and await final hosted CI.
- Files a new agent should read next: this packet and the six Read first entries above.
