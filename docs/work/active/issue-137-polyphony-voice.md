---
title: "Task Packet — Issue 137: Complete polyphony, note identity, and voice-allocation semantics"
summary: Scale instrument behavior beyond monophonic tracker playback with note identity, targeted note-off, per-instrument polyphony limits, and deterministic voice stealing; hard-cap instrument capacity.
status: current
issue: 137
updated: 2026-07-24
---

# Task Packet — Issue 137: Polyphony, note identity, and voice-allocation semantics

## Identity
- Issue: 137 · Owner: jpalvarezl · Status: in-progress
- Branch: `issue/137-polyphony-voice` · Worktree: `../blight-137-polyphony`
- Base: origin/main @ a711c5c

## Goal
Define instrument behavior that scales beyond monophonic tracker playback: stable note/event
identity, targeted note-off, configurable per-instrument polyphony, deterministic voice stealing,
and note articulation semantics — all RT-safe. Also make Engine's instrument capacity hard/fixed,
closing the one residual disclosed when the RT contract (#133) was accepted (65th-instrument realloc).
See [#137](https://github.com/jpalvarezl/blight-synth/issues/137).

## Read first
1. [Audio engine domain](../../domains/audio-engine.md)
2. [Real-time audio contract](../../architecture/realtime-contract.md) — hard callback rules (no alloc/dealloc, bounded work), "Prepared-state rule" (per-voice inserts built off RT), and the violation-inventory rows owned by #137
3. Code: `engine/src/lib.rs` (instrument slots, `DEFAULT_INSTRUMENT_CAPACITY`, `add_instrument_with_retirement`), `dsp/src/synth_infra/`, `dsp/src/instruments/`, voice/polyphony types
4. `audio_backend/src/device_host/audio_processor/mod.rs` NOTE(#137) at the retirement-bound constants

## Scope
### In scope
- Stable note/event identity separate from MIDI note number; targeted note-off releases only the intended note/voice, not all voices.
- Configurable per-instrument polyphony limits (fixed/preallocated) + deterministic voice-stealing policy (age/level metadata as needed).
- Retrigger, legato, one-shot, sustain/release, repeated-note semantics; skip processing inactive voices.
- Make Engine instrument capacity hard/fixed-slot (preallocated, explicit rejection) instead of the current soft `Vec` that can realloc on the 65th insert. Preserve the sorted preallocated `Vec<InstrumentSlot>` render-order direction from #164 (do NOT revert to hash-based ordering).
- Build genuinely-needed per-voice inserts off the audio thread.
### Out of scope
- Parameter manifest (#121) and event-source contract (#145) — do not edit their ADRs.
- The routing graph (#136).

## Ownership / touch set
Expected: `engine/**`, `dsp/**`, tests, `audio_backend/src/device_host/audio_processor/mod.rs` (update the NOTE(#137)/bound once capacity is hard), this packet.
Coordination: parallel #121 may add a NEW params module — avoid gratuitous `engine/src/lib.rs` churn beyond the polyphony/capacity work; do NOT touch `docs/architecture/realtime-contract.md`.

## Verify
- [x] `cargo test --workspace --all-targets` (added tests: repeated pitch, overlapping notes, targeted note-off, all-notes-off, voice exhaustion/stealing, capacity rejection)
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] No RT alloc/dealloc introduced (`engine/tests/rt_allocations.rs` extended with polyphonic steal + capacity-rejection alloc probes)
- [x] `python3 scripts/docs/check_docs.py`

## Handoff

### PR #200 review follow-up (NoteEvent + RT-safe voice-allocation diagnostics)
Two maintainer-requested changes were applied on top of the implementation below
(committed, not pushed):

1. **`NoteEvent` bundling.** Added `dsp::NoteEvent { id: NoteId, pitch: u8, velocity: u8 }`
   (in `dsp/src/id.rs`, re-exported as `dsp::NoteEvent`), derives `Clone, Copy, Debug,
   PartialEq`, with `NoteEvent::from_pitch(pitch, velocity)` using `NoteId::from_pitch`.
   `InstrumentTrait::note_on` is now `note_on(&mut self, event: NoteEvent)`; `note_off`
   and `all_notes_off` are unchanged. The lower-level `SynthNode`/`Voice`
   `note_on(note, velocity)` signatures were intentionally left untouched — instruments
   still call `voice.inner.note_on(event.pitch, event.velocity)`. Updated implementors
   (`MonophonicInstrument`, `PolyphonicInstrument`, and every test stub in
   `engine/src/lib.rs`, `audio_backend/src/device_host/audio_processor/mod.rs`,
   `engine/tests/rt_allocations.rs`, and the dsp `polyphony_tests`), plus Engine call
   sites (`note_on`/`note_on_with_id` construct a `NoteEvent`). `Engine::note_on` keeps
   its `(instrument_id, note, velocity)` signature (velocity behavior preserved).
2. **RT-safe voice-allocation diagnostics.** Restored per-voice allocation logging using
   the compile-time-gated `dsp::rt_debug_log!` macro (compiles out entirely in release).
   `PolyphonicInstrument::note_on` logs incoming note (id + pitch + velocity), the chosen
   slot index, and the decision; `allocate_slot` now returns a `SlotDecision`
   (`Retrigger`/`FreeSlot`/`Steal`) so the diagnostic reports why. Empty-pool ignore is
   also logged. `MonophonicInstrument` note_on/note_off log concise id/pitch/release info.
   No raw `eprintln!`/`println!`/`format!` on the callback path — `scripts/check_rt_logging.py`
   passes. The `note_off` identity guard is unchanged.

Verification: `cargo test --workspace --all-targets` all green (incl. `offline_golden`
2/2 byte-identical and `rt_allocations` 7/7 with 0 alloc/dealloc/realloc), `cargo clippy
--workspace --all-targets -- -D warnings` clean, `check_rt_logging.py` and
`docs/check_docs.py` pass.

### Status: implementation complete on `issue/137-polyphony-voice` (committed, not pushed).

### Note/voice identity design
- New `dsp::id::NoteId(u64)` newtype: a stable per-note identity distinct from MIDI
  pitch. `NoteId::from_pitch(pitch)` maps a pitch to an identity for hosts without a
  richer event source (#145) — used by the monophonic tracker path.
- `InstrumentTrait` now takes identity: `note_on(NoteId, note, velocity)`,
  `note_off(NoteId)` (targeted), and new `all_notes_off()` (release everything).
- `VoiceSlot` gained `note_id: Option<NoteId>` and a monotonic `age: u64` (stamped
  per note-on) for deterministic stealing. Added `VoiceSlot::new` to remove the
  repeated struct literals across the instrument constructors.

### Per-instrument polyphony / stealing (`PolyphonicInstrument`)
- Fixed, preallocated voice pool (unchanged allocation site, off the audio thread).
- `allocate_slot(note_id)` policy, bounded + allocation-free: (1) retrigger the voice
  already holding `note_id`; else (2) lowest-index idle voice; else (3) steal the
  oldest active voice (smallest `age`, ties -> lowest index).
- Targeted `note_off(note_id)` releases only the matching voice(s); `all_notes_off`
  releases all. `process` skips inactive voices (silence-equivalent, saves budget).
- Removed the callback-reachable debug/warn logging from `note_on`/`add_effect` and
  the NRT construction `log::info!` spam in `PolyphonicOscillator::new`.

### Engine hard instrument capacity
- `instruments: Vec<InstrumentSlot>` is now hard-capped by an explicit
  `instrument_capacity` field (default `DEFAULT_INSTRUMENT_CAPACITY = 64`, now `pub`;
  `Engine::with_instrument_capacity` configures it). The vector is preallocated and
  never grows: a distinct id past the cap is rejected and its owner retired via the
  existing `RetireSink` path — no callback reallocation. Replacing an existing id
  still succeeds and retires the displaced owner.
- Engine note API: `note_on`/`note_off` keep pitch-derived identity for existing
  callers; added `note_on_with_id`, `note_off_id`, and `all_notes_off`.
- `clear_instruments` `debug_assert` now asserts the enforced invariant against
  `instrument_capacity`.

### audio_processor bound / NOTE(#137)
- Updated the `MAX_INSTRUMENTS_PER_CLEAR` doc and `NOTE(#137)` in
  `audio_backend/src/device_host/audio_processor/mod.rs` to state the cap is now hard
  and enforced (no callback realloc); the retirement-ring math is unchanged (still 64).

### realtime-contract.md
- Edited exactly one violation-inventory row (the first, "Instrument insertion past
  prepared capacity") from *Deferred* to *Resolved (#137)*, per the task's
  "touch ONLY that one inventory row" constraint. The intro/Contract-completion prose
  that still mentions the residual was intentionally left untouched per that constraint
  and should be reconciled by whoever closes #137/#133.

### Tests added
- `dsp/src/instruments/mod.rs` `polyphony_tests`: repeated-note-same-identity reuse,
  overlapping same-pitch distinct voices, targeted note-off, all-notes-off, oldest-first
  voice stealing on exhaustion, freed-voice reuse before stealing, empty-pool no-panic.
- `engine/src/lib.rs`: `instrument_capacity_rejects_distinct_instruments_past_the_hard_cap`.
- `engine/tests/rt_allocations.rs`: `polyphonic_note_on_steal_and_render_has_no_heap_activity`
  and `instrument_capacity_rejection_moves_owner_without_rt_heap_activity` (both assert 0
  alloc/dealloc/realloc).

### Scope note
- Targeted note-off is exposed at the Engine method level, not as a new transitional
  command-enum variant, to avoid encroaching on the timestamped event API (#134/#145).
  The tracker host keeps all-notes-off semantics; richer per-note routing belongs to #145.
- `reconcile_work.py --check` reports pre-existing errors for #121/#145 (other in-flight
  issues' packets) and a generated burndown/index drift — not owned by #137; generated
  docs deliberately left untouched.
