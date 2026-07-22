---
title: Task Packet — Issue 186 Retirement Primitive
summary: Active context for the RT-to-NRT retirement primitive and duplicate-instrument replacement slice.
status: current
updated: 2026-07-23
issue: 186
owner: jpalvarezl
branch: issue/186-retirement-primitive
---

# Task Packet — Issue 186: Retirement Primitive

## Identity

- Issue: #186
- Owner: jpalvarezl
- Status: in-progress
- Branch: `issue/186-retirement-primitive`
- Base branch/SHA: `main` / `a85fd21`
- Head SHA: see branch head
- Last handoff: 2026-07-23

## Goal

Prove the bounded RT-to-NRT ownership-return topology for duplicate-ID instrument replacement as the first implementation slice of #174.

## Read first

1. [Audio engine domain](../../domains/audio-engine.md)
2. [Real-time contract](../../architecture/realtime-contract.md)
3. Issue #186
4. `engine/src/lib.rs`
5. `audio_backend/src/standalone/audio_processor/mod.rs`
6. `audio_backend/src/standalone/audio_frontend/`

## Dependencies and blockers

- Parent: #174
- Depends on: #180/#181/#182 merged FIFO and NRT-worker ownership
- Blocks: #187
- Coordinates with: #136, #137

## Scope and non-goals

### In scope

- Ringbuf-free retired-owner representation in `engine`.
- Duplicate-ID instrument replacement returns old ownership.
- Bounded reverse retirement ring in the CPAL adapter.
- NRT drain before submissions and NRT shutdown ordering.
- Focused ownership/drop tests.

### Out of scope

- Bulk clear, effect rejection, and voice-effect remainder (#187).
- Song replacement and complete shutdown stress (#188).
- Capacity/stealing policy (#137) and effect remove/reorder (#136).

## Ownership and touch set

- `engine/src/lib.rs`
- `audio_backend/src/player/`
- `audio_backend/src/standalone/audio_processor/mod.rs`
- `audio_backend/src/standalone/audio_frontend/`
- `docs/architecture/realtime-contract.md`
- `docs/domains/audio-engine.md`
- `docs/work/active/`

Shared contract: `Engine::handle_command`/`add_instrument` surface displaced ownership; coordinated follow-ups must extend rather than replace this contract.

## Plan

- [x] Surface retired instrument ownership from Engine without host dependencies.
- [x] Thread retired ownership through tracker Player/adapter command handling.
- [x] Add bounded RT-to-NRT retirement ring and fixed RT pending fallback.
- [x] Drain retired ownership on NRT command submission and after stream shutdown.
- [x] Add focused ownership/drop tests.
- [x] Complete local validation.
- [x] Run independent review and address findings.
- [x] Request Copilot review and address all findings.

## Progress and decisions

- 2026-07-23 — Split parent #174 into #186 → #187 → #188 to avoid mixing retirement topology, all structural sites, and song/shutdown stress in one large branch.
- 2026-07-23 — `engine::RetiredState` is host/ringbuf-free; the CPAL adapter owns the reverse ring transport.
- 2026-07-23 — Callback work stops consuming commands while fixed pending retirement ownership cannot reach NRT, preserving a bounded fallback.
- 2026-07-23 — Retirement ring capacity is 128 owner units; at most 64 commands execute per block and this slice retires at most one owner per command.
- 2026-07-23 — Independent review approved the ownership/bound invariants but required transport coverage. Added live-consumer replacement delivery plus forced ring-full pending/pause/resume tests.
- 2026-07-23 — Copilot reviewed all 11 changed files. Made intentional NRT retirement discards explicit at offline/test call sites and clarified that the pending-retirement gate applies on subsequent callback blocks.

## Verification

- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace --all-targets` — 71 tests plus examples
- [x] `cargo test -p audio_backend --no-default-features --all-targets`
- [x] `cargo clippy -p audio_backend --no-default-features --all-targets -- -D warnings`
- [x] `python3 scripts/check_rt_logging.py`
- [x] `python3 scripts/check_architecture.py`
- [x] `python3 scripts/docs/check_docs.py`
- [x] `git diff --check`

## Handoff

- Completed: retirement ownership primitive, replacement slice, reverse ring, NRT drain, focused drop test.
- Remaining: final hosted CI after review fixes and human review.
- Known risks: shutdown handling of RT-side pending ownership and all non-instrument retirement variants intentionally remain #188/#187.
- Next action: push the reviewed branch, open the PR, and request Copilot review.
