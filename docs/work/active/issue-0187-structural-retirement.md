---
title: Task Packet — Issue 187 Structural Retirement
summary: Active context for routing engine and effect structural ownership through deferred NRT retirement.
status: current
updated: 2026-07-23
issue: 187
owner: jpalvarezl
branch: issue/187-structural-retirement
---

# Task Packet — Issue 187: Structural Retirement

## Identity

- Issue: #187
- Owner: jpalvarezl
- Status: in-progress
- Branch: `issue/187-structural-retirement`
- Base branch/SHA: `main` / `86e4228`
- Head SHA: see branch head
- Last handoff: 2026-07-23

## Goal

Extend the `RetireSink` primitive from #186 across current Engine/DSP structural ownership sites without taking over #136 effect mutation semantics or #137 capacity policy.

## Read first

1. [Audio engine domain](../../domains/audio-engine.md)
2. [Real-time contract](../../architecture/realtime-contract.md)
3. Issue #187
4. `engine/src/lib.rs`
5. `dsp/src/synth_infra/effects.rs`
6. `dsp/src/synth_infra/instruments.rs`
7. `audio_backend/src/standalone/audio_processor/mod.rs`

## Dependencies and blockers

- Parent: #174
- Depends on: #186 (complete)
- Blocks: #188
- Coordinates with: #136, #137
- Current blocker: none

## Scope and non-goals

### In scope

- Instrument clear/bulk retirement.
- Mono/master effect rejection or overflow retirement.
- Voice-effect remainder/rejection retirement, including boxed inline-container elements.
- Exactly-once NRT destruction and focused RT drop/allocation probes.

### Out of scope

- Effect remove/reorder semantics (#136).
- Instrument hard-capacity/stealing policy (#137).
- Song replacement and final shutdown stress (#188).

## Ownership and touch set

Expected paths:

- `engine/src/lib.rs`
- `engine/src/commands.rs`
- `dsp/src/synth_infra/effects.rs`
- `dsp/src/synth_infra/instruments.rs`
- `dsp/src/instruments/`
- `audio_backend/src/player/`
- `audio_backend/src/standalone/audio_processor/mod.rs`
- `engine/tests/rt_allocations.rs`
- `docs/architecture/realtime-contract.md`
- `docs/work/active/`

Shared contract: extend `RetiredState`/`RetireSink`; do not replace the #186 handoff or independently define #136/#137 policy.

## Plan

- [x] Inventory every structural boxed-object/rejection in this slice.
- [x] Extend retired-object variants and sink-aware Engine/DSP methods.
- [x] Implement bounded bulk-clear retirement semantics.
- [x] Route effect/voice-effect overflow ownership to the callback sink.
- [x] Add focused exactly-once/drop-thread/allocation tests.
- [x] Run complete local validation.
- [x] Run independent review and address findings.
- [ ] Request Copilot review and address findings.

## Progress and decisions

- 2026-07-23 — #186 merged the reverse retirement ring, fixed callback pending bound, `RetireSink`, offline drop policy, and duplicate-instrument replacement slice.
- 2026-07-23 — This issue extends that established contract only; #188 owns song and complete shutdown stress.
- 2026-07-23 — Added `MonoEffect`/`StereoEffect` retired variants; chain overflow returns ownership instead of dropping it.
- 2026-07-23 — `clear_instruments` drains every slot through `RetireSink`; missing-instrument effects and polyphonic voice-effect remainders are retired individually without allocation.
- 2026-07-23 — Production factories/types pass zero-allocation/deallocation tests for clear, missing-effect rejection, and master-chain overflow.
- 2026-07-23 — Independent review caught the #186 one-retired-object-per-command pending bound becoming stale. Increased fixed callback pending capacity to 4096 (64 commands × 64 objects) and added a worst-case ring-full test proving no reallocation.
- 2026-07-23 — Final review approved. Documented/asserted the temporary coupling to Engine's soft 64-instrument capacity (#137), confirmed monophonic batches now follow their documented first-effect behavior, and documented intentionally unordered LIFO retirement flushing.
- 2026-07-24 — Human review clarified terminology and failure semantics. Renamed callback sizing from “owners” to “retired objects.” Kept non-panicking polyphonic rejection: the returned Box prevents RT deallocation; a separate NRT command-result path must communicate UnsupportedOperation to applications (#136 coordination).

## Verification

- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace --all-targets` — 76 tests plus examples
- [x] `cargo test -p audio_backend --no-default-features --all-targets`
- [x] `cargo clippy -p audio_backend --no-default-features --all-targets -- -D warnings`
- [x] `python3 scripts/check_rt_logging.py`
- [x] `python3 scripts/check_architecture.py`
- [x] `python3 scripts/docs/check_docs.py`
- [x] `git diff --check`

## Handoff

- Completed: full structural-object inventory, sink-aware clear/effect/voice retirement, focused drop/allocation tests, and local validation.
- Remaining: Copilot review, hosted CI, and human review.
- Known risks: effect-chain method signatures overlap future #136 work; hard insertion/capacity behavior remains #137.
- Next action: push the reviewed branch, open the PR, and request Copilot review.
