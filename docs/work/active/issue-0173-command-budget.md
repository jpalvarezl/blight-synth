---
title: Task Packet — Issue 173 Command Budget and Backpressure
summary: Active context and handoff for bounded callback command work and observable queue submission status.
status: current
updated: 2026-07-20
issue: 173
owner: jpalvarezl
branch: issue/173-command-budget
---

# Task Packet — Issue 173: Command Budget and Backpressure

## Identity

- Issue: #173
- Owner: jpalvarezl
- Status: in-progress
- Branch: `issue/173-command-budget`
- Worktree: `/Users/jpalvarezl/code/blight-173-command-budget`
- Base branch/SHA: `origin/main` / `f9e02d6`
- Head SHA: see `issue/173-command-budget` branch head
- Last handoff: 2026-07-20

## Goal

Implement the bounded callback command work and observable producer backpressure required by [issue #173](https://github.com/jpalvarezl/blight-synth/issues/173), without defining final timestamped events or continuous-parameter coalescing.

## Read first

1. [Audio engine domain](../../domains/audio-engine.md)
2. [Real-time contract](../../architecture/realtime-contract.md)
3. `audio_backend/src/standalone/audio_processor/mod.rs`
4. `audio_backend/src/standalone/audio_frontend/blight_audio.rs`
5. `audio_backend/src/standalone/osc.rs`

## Dependencies and blockers

- Depends on: #171 (complete)
- Blocks: none
- Current blocker: none

## Scope and non-goals

### In scope

- Fixed maximum compatibility-command work per callback block.
- Accepted/full/disconnected submission status.
- OSC acknowledgement suppression after rejected submission.
- Saturation, FIFO fairness, recovery, and render-progress tests.

### Out of scope

- Continuous parameter coalescing owned by #101.
- Timestamped event schema and event budget owned by #134.
- Prepared-state retirement and callback-side destruction owned by #174.

## Ownership and touch set

Expected paths:

- `audio_backend/src/standalone/audio_processor/mod.rs`
- `audio_backend/src/standalone/audio_frontend/`
- `audio_backend/src/standalone/osc.rs`
- `audio_backend/src/song_hydration.rs`
- `docs/architecture/realtime-contract.md`
- `docs/work/active/`

Shared contracts/schemas touched: compatibility command submission API only; no final event or parameter schema.

Potential parallel conflicts: issue #175 touches callback logging and malformed-input tests near `AudioProcessor`; preserve its merged logging changes and avoid changing the logging contract.

## Plan

- [x] Add and document a fixed per-process command budget.
- [x] Return accepted/full/disconnected status from command submission.
- [x] Gate protocol success responses on accepted submission.
- [x] Add saturation, recovery, fairness, and control-load render tests.
- [x] Run workspace tests, strict Clippy, and docs checks.

## Progress and decisions

- 2026-07-20 — #171 is complete and the accepted draft contract assigns the initial compatibility-queue budget/status behavior to #173.
- 2026-07-20 — Keep the current FIFO compatibility queue; final traffic-class separation remains owned by #101/#134/#174.
- 2026-07-20 — Set the transitional budget to 64 command items per host callback, not per internal render chunk. This bounds item count; command cost remains constrained by #137/#174.
- 2026-07-20 — `Accepted` means enqueued, not yet applied. OSC parameter echoes and song-loaded responses are suppressed when queue submission reports full/disconnected.
- 2026-07-20 — Finite FIFO bursts make progress and preserve command order. Priority recovery lanes and sustained-traffic class fairness remain out of scope with final queue separation.
- 2026-07-20 — Independent diff review approved the change with no blocking findings; partial hydration and ignored GUI statuses remain documented follow-up risks.

## Verification

- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace --all-targets` — 60 tests plus examples
- [x] `cargo clippy -p audio_backend --no-default-features --all-targets -- -D warnings`
- [x] `cargo test -p audio_backend --no-default-features --all-targets` — 10 tests plus examples
- [x] `python3 scripts/check_rt_logging.py`
- [x] `python3 scripts/check_architecture.py`
- [x] `python3 scripts/docs/check_docs.py`
- [x] `git diff --check`

## Handoff

- Completed: 64-item callback budget, observable queue status, accepted-only OSC acknowledgements, hydration rejection propagation, focused stress/recovery tests, local validation, and independent diff review.
- Remaining: hosted CI and PR/human review.
- Known failures/risks: the compatibility queue still combines traffic classes; sustained traffic has no priority recovery lane. Multi-command hydration can be partially enqueued before a later rejection, although OSC reports an error rather than false success; atomic prepared-state installation remains with #174/#138.
- Next smallest action: push the branch and open the issue PR.
- Files a new agent should read next: this packet and the five Read first entries above.
