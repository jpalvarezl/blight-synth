---
title: "Task Packet — Issue 137: Complete polyphony, note identity, and voice-allocation semantics"
summary: Scale instrument behavior beyond monophonic tracker playback with note identity, targeted note-off, fixed polyphony, deterministic stealing, and hard instrument capacity.
status: current
issue: 137
updated: 2026-07-26
---

# Task Packet — Issue 137: Polyphony, note identity, and voice-allocation semantics

## Identity

- Issue: 137 · Owner: jpalvarezl · Status: in-progress
- Branch: `issue/137-polyphony-voice` · Worktree: `../blight-137-polyphony`
- Base: `origin/main` @ `a711c5c`
- Pull request: [#200](https://github.com/jpalvarezl/blight-synth/pull/200)

## Goal

Define RT-safe instrument behavior beyond monophonic tracker playback: stable note/event
identity, targeted note-off, fixed per-instrument polyphony, deterministic voice stealing,
inactive-voice skipping, and hard Engine instrument capacity. Preserve the tracker render
byte-for-byte.

## Read first

1. [Audio engine domain](../../domains/audio-engine.md)
2. [Real-time audio contract](../../architecture/realtime-contract.md)
3. `dsp/src/instruments/mod.rs`
4. `engine/src/lib.rs`
5. `audio_backend/src/device_host/audio_processor/mod.rs`

## Scope and ownership

### In scope

- `NoteId`/`NoteEvent` identity and targeted release semantics.
- Fixed, NRT-prepared voice pools with deterministic oldest-first stealing.
- Envelope-backed release behavior and inactive-voice processing skips.
- Hard, preallocated Engine instrument capacity and NRT retirement on rejection.
- Accurate callback logging enforcement, RT-contract prose, tests, and this packet.

### Out of scope

- Parameter manifest (#121), timestamped event/event-source contracts (#134/#145),
  and routing graph semantics (#136).

### Touched paths

- `dsp/src/instruments/mod.rs`
- `dsp/src/instruments/polyphonic_osc.rs`
- `engine/src/lib.rs`
- `engine/tests/rt_allocations.rs`
- `audio_backend/src/device_host/audio_processor/mod.rs`
- `scripts/check_rt_logging.py`
- `docs/architecture/realtime-contract.md`
- this packet

## Implemented contract

### Note identity and instrument API

- `dsp::NoteEvent { id: NoteId, pitch: u8, velocity: u8 }` is the bundled
  instrument note-on input: `InstrumentTrait::note_on(&mut self, event: NoteEvent)`.
- `note_off(NoteId)` targets the voice that still owns that identity;
  `all_notes_off()` releases every active voice.
- The tracker-compatible Engine methods derive identity from pitch. Explicit
  `note_on_with_id`/`note_off_id` methods support overlapping equal-pitch notes.
- The lower-level `Voice`/`SynthNode` note-on signature remains `(pitch, velocity)`.

### Voice allocation and lifecycle

- A polyphonic instrument owns a fixed, preallocated voice pool and rollover scratch
  prepared off RT.
- Allocation order is: active matching identity (retrigger), lowest-index idle slot,
  then oldest active age. Equal ages steal the lowest slot index.
- Before the `u64` age sequence can wrap, all ages are compacted to stable ranks by
  `(age, slot index)`. The rare renormalization is allocation-free and bounded by the
  configured voice count, so genuinely older voices remain older across rollover.
- Targeted note-off retains the `note_id` plus `is_active()` guard. Duplicate release
  cannot re-gate an idle envelope, while duplicate/global release during a real Release
  phase remains safe and deterministic.
- Rendering skips inactive voices.

### Callback diagnostics

- Polyphonic note-on emits exactly one compile-time-gated `rt_debug_log!` containing
  note identity, pitch, and chosen slot. There is no `SlotDecision`, no empty-pool log,
  and no monophonic note log.
- `PolyphonicInstrument::add_effect` retains the specified compile-time-gated
  `rt_warn_log!` rejection diagnostic and returns the effect for NRT retirement.
- No additional callback logging was added.
- `scripts/check_rt_logging.py` now scans
  `audio_backend/src/device_host/audio_processor` after the #190 rename and fails if
  any configured scan root is stale or missing.

### Hard Engine capacity

- Engine instrument slots remain a sorted, preallocated `Vec<InstrumentSlot>` with a
  hard capacity (default 64). Distinct over-capacity instruments are rejected and their
  new owners are handed to `RetireSink`; replacement at capacity still succeeds and
  retires the displaced old owner.
- The callback retirement bound is coupled to `DEFAULT_INSTRUMENT_CAPACITY`.
- Capacity rejection currently has no counter or typed result: both rejected new owners
  and displaced old owners surface as `RetiredState::Instrument`. The RT contract now
  states that limitation rather than claiming every capacity path increments status.

## Deterministic tests

`dsp/src/instruments/mod.rs` covers:

- same-identity retrigger and overlapping equal-pitch identities;
- targeted and global note-off;
- oldest-first exhaustion, rollover renormalization, and direct equal-age tie breaking;
- idle-slot reuse and empty pools;
- an envelope-backed duplicate targeted note-off plus `all_notes_off` during Release;
- a process counter proving inactive voices are skipped.

Engine tests cover hard-cap rejection/replacement. `engine/tests/rt_allocations.rs` covers
polyphonic steal/render and hard-cap rejection with zero callback alloc/dealloc/realloc.

## Verification

- [x] `cargo test --workspace --all-targets` (`offline_golden` 2/2 unchanged;
  `rt_allocations` 7/7 with zero measured callback heap activity)
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `python3 scripts/check_rt_logging.py`
- [x] `python3 scripts/docs/check_docs.py`
- [x] `python3 scripts/docs/reconcile_work.py --check` was run; it reports unrelated
  pre-existing #121/#145 missing-packet and generated-index/burndown drift. Generated
  docs were not edited.

## Handoff notes

- The monophonic tracker path must remain byte-identical in `offline_golden`.
- Do not weaken the note-off identity/activity guard.
- Do not regenerate golden references if rendering changes.
