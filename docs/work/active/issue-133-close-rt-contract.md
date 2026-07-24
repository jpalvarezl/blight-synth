---
title: "Task Packet — Issue 133: Close out and enforce the RT safety contract (incl. #174)"
summary: Verify merged retirement work meets #174/#133 acceptance criteria and promote the RT contract from draft to enforced.
status: current
updated: 2026-07-24
issue: 133
---

# Task Packet — Issue 133: Close out and enforce the RT safety contract (incl. #174)

## Identity

- Issue: 133 (epic) + verifies/closes child 174
- Owner: jpalvarezl
- Status: in-progress
- Branch: `issue/133-close-rt-contract`
- Worktree: `../blight-133-rt-closeout`
- Base branch/SHA: origin/main @ ad2e035
- Last handoff: 2026-07-24

## Goal

All implementation children of the M1 RT-safety contract are merged: #172, #173, #175 (closed) and
#174's slices #186 → #187 → #188 (closed). This task **verifies** the merged code+tests actually
satisfy #174's and #133's acceptance criteria, then **promotes**
`docs/architecture/realtime-contract.md` from `status: draft` to the enforced M1 contract and closes
#174 and #133. See [#133](https://github.com/jpalvarezl/blight-synth/issues/133) and
[#174](https://github.com/jpalvarezl/blight-synth/issues/174).

## Read first

1. [Real-time audio contract](../../architecture/realtime-contract.md) (esp. "Contract completion", "Verification plan", violation inventory)
2. [Audio engine domain](../../domains/audio-engine.md)
3. Merged evidence: `engine/tests/rt_allocations.rs`, `engine/src/lib.rs` (RetiredState/RetireSink), `audio_backend/src/player/mod.rs`, `audio_backend/src/standalone/audio_processor/mod.rs`, `audio_backend/src/standalone/audio_frontend/blight_audio.rs`

## Dependencies and blockers

- Depends on: #172, #173, #175 (closed/merged) and #174 (implementation slices #186/#187/#188 merged; #174 itself pending closure by this task)
- Blocks: nothing directly; unblocks M1 milestone closure
- Current blocker: NONE

## Scope and non-goals

### In scope

- Audit #174 acceptance criteria against merged #186/#187/#188 code and tests.
- Audit #133 acceptance criteria + the "Current violation inventory" in realtime-contract.md: for each row, confirm it is resolved by merged work or is explicitly owned by a still-open non-#133 issue (e.g. #101/#134/#136/#137/#145).
- Promote `realtime-contract.md` frontmatter `status: draft` → accepted/enforced and update the "Contract completion" section to reflect closure.
- Report any genuine gap instead of closing an issue whose criteria are unmet.

### Out of scope

- Any `audio_backend/**` code refactor (that is #190, running in parallel).
- Editing Cargo features or `docs/domains/standalone-host.md` (owned by #190 this cycle).
- New RT features (#101/#134/#136/#137 remain their own issues).

## Ownership and touch set

Expected paths: `docs/architecture/realtime-contract.md`, this packet.
Shared contracts touched: `realtime-contract.md` (this track owns it this cycle).
Potential parallel conflicts: #190 must NOT edit `realtime-contract.md`.

## Verification

- [x] `cargo test --workspace --all-targets` (RT/allocation tests pass on current main — `rt_allocations.rs` 5/5, `audio_processor` retirement/backpressure suite green)
- [x] `python3 scripts/docs/check_docs.py` (28 pages pass)
- [x] `python3 scripts/docs/reconcile_work.py --check` (pre-existing, out-of-scope error for the closed-#188 packet only; see Audit results)

## Handoff

- Completed: Audited #174 and #133 acceptance criteria against merged code+tests; promoted `docs/architecture/realtime-contract.md` from `status: draft` to `status: accepted` with resolved/deferred inventory attribution. No gap in the #172/#173/#175/#174 acceptance criteria required for promotion; one bounded, #137-owned residual to Hard Rule 1 (instrument insertion past soft capacity) is disclosed, not hidden.
- Remaining: Orchestrator closes #174 and #133 on GitHub after review.
- Follow-up (not done here — outside this task's edit scope of `realtime-contract.md` + this packet): add this packet to the `docs/work/active/README.md` **Active** index, and either fix or remove the closed-#188 packet (`issue-188-retire-replaced-songs.md` lacks numeric `issue:` frontmatter, which is the sole `reconcile_work.py --check` error and predates this branch).

## Audit results

Evidence paths are on `main` @ `ad2e035`. Hard-rule children #172/#173/#175 are **closed**; #174/#133 are **open** (this task closes them). All deferred rows are owned by still-open issues.

### #174 — Defer reclamation for structural engine and song updates

| Acceptance criterion | Verdict | Evidence |
|---|---|---|
| Representative instrument/effect/song replacement performs no callback alloc/dealloc | Met | `engine/tests/rt_allocations.rs`: `structural_clear_and_effect_rejection_move_owners_without_rt_heap_activity`, `master_effect_overflow_moves_rejected_owner_without_rt_heap_activity` (0 alloc/dealloc/realloc). Song swap: `audio_backend/src/standalone/audio_processor/mod.rs::swapped_song_crosses_retirement_ring_before_nrt_drop`; `audio_backend/src/player/mod.rs::{load_song,play_song}_retires_previous_song_for_nrt_drop`. Engine routing: `engine/src/lib.rs` `handle_command_with_retirement`, `add_instrument_with_retirement`, `clear_instruments`, `add_effect_to_instrument`, `add_master_effect`; `Player::set_song` retires `Arc<Song>` as `RetiredState::Prepared`. |
| Retired state reclaimed on non-RT owner | Met | `BlightAudio::reclaim_retired` drains `retirement_rx` and drops on NRT, called from `try_send_command`/`send_command`/`send_command_until`; `engine::DropRetireSink` for offline/NRT callers. `replaced_instrument_crosses_retirement_ring_before_nrt_drop` asserts drop count stays 0 until NRT pops. |
| Retirement overflow + shutdown semantics explicit and tested | Met | Overflow: `CallbackRetireSink` retains overflow in preallocated `pending_retired` (cap `MAX_PENDING_RETIRED_OBJECTS` = 4160), never dropping on RT; `process()` pauses command consumption while pending is non-empty. Tests: `full_retirement_ring_pauses_later_commands_until_nrt_drains`, `worst_case_multi_object_commands_fit_preallocated_pending_retirement`, `load_song_clear_peak_fits_preallocated_pending_retirement`. Shutdown: `BlightAudio::stop_and_reclaim` + `Drop for BlightAudio` pause the stream then drain, with documented drop order. |
| No leak / double-drop under stress | Met | `repeated_song_and_graph_swaps_then_shutdown_reclaim_each_owner_exactly_once` (24 swaps; weak refs alive during loop, all destroyed exactly once at shutdown; `instrument_drops == swaps`); `swapped_song_crosses_retirement_ring_before_nrt_drop` asserts exactly-once via `Weak::upgrade`. |
| CI + clippy pass | Met | `cargo test --workspace --all-targets` green. CI (`.github/workflows/ci.yml`) runs `cargo clippy --workspace --all-targets -- -D warnings`, the `--no-default-features` clippy pass, and `scripts/check_rt_logging.py`. |

### #133 — Enforce and test the real-time safety contract (hard-rule children)

| Hard-rule child | Verdict | Evidence |
|---|---|---|
| #172 allocation audit harness (closed) | Met | `engine/tests/rt_allocations.rs` `TrackingAllocator` + `prepared_engine_note_parameter_and_render_path_has_no_heap_activity`; self-test `audit_harness_detects_an_intentional_allocation_and_drop`. |
| #173 bounded callback work + observable backpressure (closed) | Met | 64-command budget in `AudioProcessor::process`; tests `command_budget_is_per_host_callback_not_per_internal_render_chunk`, `command_burst_is_fifo_bounded_and_rendering_progresses_between_slices`; `BlightAudio` `CommandSubmissionResult` `Full`/`Disconnected` API. |
| #175 callback logging/panic removal + hot-path stress (closed) | Met | `dsp/src/diagnostics.rs` `rt_*_log!` macros compile out under `!debug_assertions`; `scripts/check_rt_logging.py` in CI; `process_chunks_host_buffers_larger_than_internal_scratch_space`, `process_silences_extra_channels_and_incomplete_frames`. |
| #174 deferred reclamation (closing) | Met | See #174 table above. |

### Violation-inventory rows (from `realtime-contract.md`)

| Row | Resolution |
|---|---|
| Instrument insertion past prepared capacity (#137) | Deferred to open #137. Soft 64-slot preallocated vector; installing a distinct 65th instrument can still reallocate on the callback (documented `audio_processor/mod.rs` NOTE(#137)). `debug_assert!` in `clear_instruments` guards retirement sizing. Bounded, owned exception — not a #172/#173/#175/#174 gap. |
| Instrument/effect commands carry `Box`/`ArrayVec<Box<_>>` (#174) | Resolved (#174). Owners routed to `RetireSink`; `rt_allocations.rs` proves zero dealloc. |
| `Player::load_song` replaces `Arc<Song>` + clears instruments (#174/#138) | Resolved for RT-safety (#174) via `set_song`/`clear_instruments` retirement; versioned snapshots remain open #138. |
| Effect-chain/master remove/reorder no-op semantics (#136/#173) | Partial. Rejected/overflow effects retire safely (#173/#174); observable `error.kind()` NRT result deferred to open #136. |
| Player/tracker adapter note/row/end logging (#175) | Resolved (#175). Compile-out `rt_*_log!` + `check_rt_logging.py`. |
| Polyphonic instrument note logs/warnings (#175/#137) | Logging resolved (#175); voice-allocation/capacity deferred to open #137. |
| Delay/reverb/drum parameter error logging (#175/#101) | Logging resolved (#175); coalesced parameter pipeline deferred to open #101. |
| Tracker active-instrument `HashMap::insert` (#175/#145) | Logging resolved (#175); structural active-instrument capacity deferred to open #145. |
| Voice scratch buffers fixed at 4096 (#132/#175) | Mitigated. `AudioProcessor::process` chunks oversized host buffers (tested); negotiated Engine lifecycle deferred to open #132. |
| Engine/effect capacities not hard-rejected consistently (#137/#136/#175) | Partial. Master-effect overflow rejects + retires without RT heap work (tested); hard instrument cap (#137) and routing (#136) open. |
| Structural command work mixes notes/params/graph (#101/#134/#173/#174) | Bounded budget + reclamation landed (#173/#174); traffic-class separation deferred to open #101/#134. |

### Gaps

No gap in the four hard-rule/reclamation children required for promotion (#172/#173/#175 callback hard-rules + #174 deferred reclamation): all are met and tested. One **bounded, explicitly-owned residual** to Hard Rule 1 remains and is honestly recorded rather than hidden — installing more than the soft 64-instrument capacity can reallocate on the callback (`Engine` uses a preallocated but not hard-capped `Vec`; the code documents this in `audio_processor/mod.rs` NOTE(#137)). This is the first inventory row, owned by open #137, and is out of scope for #172/#173/#175/#174: #174's criterion covers *replacement* of prepared state (reusing existing IDs), which is proven allocation-free, and #172 built the audit harness plus representative coverage rather than eliminating every allocation site.

The remaining rows extend structural/traffic-class guarantees and are owned by their own open issues (#101/#132/#134/#136/#137/#138/#145); none reintroduces a callback drop, lock, or log. The contract is therefore promoted to `status: accepted` as the authoritative M1 rulebook with an explicit inventory of owned residual gaps — not a claim that every code path already complies. If a reviewer requires zero callback allocation before promotion, #137 must land first; the draft's own completion criterion was “#172–#175 pass and #133 closes,” with #137 listed separately, so promotion now is consistent with the documented intent.

### Verification command tails

- `cargo test --workspace --all-targets`: all suites `ok`; `rt_allocations.rs` `test result: ok. 5 passed; 0 failed`; workspace exit 0.
- `python3 scripts/docs/check_docs.py`: `documentation check passed: 28 pages` (exit 0).
- `python3 scripts/docs/reconcile_work.py --check`: `error: docs/work/active/issue-188-retire-replaced-songs.md has no numeric \`issue:\` frontmatter` (exit 1). This is **pre-existing on `main`** (the closed-#188 packet) and reproduces with this packet removed; it is outside this task's editable scope (only `realtime-contract.md` and this packet). Not introduced by this change.
