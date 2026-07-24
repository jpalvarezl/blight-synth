---
title: "Task Packet — Issue 188: Retire replaced songs and finalize reclamation shutdown coverage"
summary: Route Player song ownership and host shutdown through deferred NRT reclamation and add stress coverage.
status: current
updated: 2026-07-24
---

# Task Packet — Issue 188: Retire replaced songs and finalize reclamation shutdown coverage

## Identity

- Issue: 188
- Owner: jpalvarezl
- Status: in-progress
- Branch: `issue/188-retire-replaced-songs`
- Worktree: `../blight-188-retire-songs`
- Base branch/SHA: origin/main @ ac9627d
- Head SHA: 79ec3fea5ab38087a198cd30e583a4b669129d6e
- Last handoff: 2026-07-24

## Goal

Complete #174 by routing `Player` song ownership and host shutdown through the deferred
RT-to-NRT retirement primitive landed in #186/#187, then stress the full swap/retire/shutdown
path. See issue [#188](https://github.com/jpalvarezl/blight-synth/issues/188) and parent
[#174](https://github.com/jpalvarezl/blight-synth/issues/174).

## Read first

1. [Audio engine domain](../../domains/audio-engine.md)
2. [Real-time audio contract](../../architecture/realtime-contract.md) — "Deferred reclamation", "Prepared-state rule", "Control traffic classes / 3. Structural updates"
3. Code entry points:
   - `engine/src/lib.rs` (RetiredState / RetireSink / retirement ring)
   - `audio_backend/src/player/mod.rs` (`load_song`, `Arc<Song>` ownership)
   - `audio_backend/src/standalone/audio_processor/mod.rs` (RT callback, retirement drain)
   - `audio_backend/src/standalone/audio_frontend/blight_audio.rs` + `control_worker.rs` (CPAL stream stop, shutdown order)

## Dependencies and blockers

- Depends on: #186 (closed), #187 (closed)
- Blocks: #174
- Current blocker: NONE

## Scope and non-goals

### In scope

- Retire replaced `Arc<Song>` ownership from load/play state changes instead of dropping on RT.
- Define CPAL stream stop, retirement drain, pending ownership, and shutdown order.
- Cover callback/worker disconnection and retirement-channel saturation (documented bounded non-allocating fallback).
- Add burst/swap/drop probes and exactly-once shutdown tests.

### Out of scope

- Renaming/splitting the standalone module (that is #185).
- Replacing the transitional compatibility command queue (#101/#134).
- Removing Tokio (#161).

## Verification

- [x] `cargo test --workspace --all-targets`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] New stress tests cover repeated graph/song swaps and shutdown, exactly-once reclamation.

## Handoff

- Completed:
  - Added `engine::RetiredState::Prepared(Arc<dyn Any + Send + Sync>)`, an opaque
    prepared-state owner variant. Building it from a concrete `Arc<T>` is an
    allocation-free unsizing coercion, so the callback can retire an
    `Arc<Song>` without heap work and without engine depending on `sequencer`.
  - `Player::set_song` now `mem::replace`s the live `Arc<Song>` and routes the
    displaced owner through the existing `RetireSink`; `load_song`/`PlaySong`
    both go through it. Replaced songs are no longer dropped on RT.
  - Bumped the standalone worst-case retirement bound: `LoadSong` clears up to
    64 instruments and retires 1 song (65 objects/command), so the preallocated
    pending buffer is now `64 * 65 = 4160` slots. Updated the realtime contract
    doc accordingly.
  - Shutdown exactly-once guarantee: `BlightAudio` drop order (field order
    `_stream` before `retirement_rx`) stops the callback first, then the
    callback-owned `AudioProcessor` (its `pending_retired` + live player song +
    instruments) drops on NRT, then `retirement_rx` drains the ring on NRT. Each
    owner lives in exactly one place, so it is reclaimed exactly once on NRT.
  - Tests added (all hardware-free):
    - `engine`: `prepared_owner_reclamation_runs_on_the_receiving_nrt_thread`.
    - `audio_backend` player: `load_song_retires_previous_song_for_nrt_drop`,
      `play_song_retires_previous_song_for_nrt_drop`.
    - `audio_backend` audio_processor:
      `swapped_song_crosses_retirement_ring_before_nrt_drop`,
      `repeated_song_and_graph_swaps_then_shutdown_reclaim_each_owner_exactly_once`.
- Remaining: none for #188 scope. Parent #174 can integrate.
- Known failures/risks: The 4160-slot bound is coupled to Engine's soft
  64-instrument capacity (#137). If #137 makes instrument capacity
  hard/configurable, `MAX_INSTRUMENTS_PER_CLEAR` and the doc bound must follow.
- Next smallest action: close out #174 verification once #188 merges.
