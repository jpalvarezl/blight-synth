---
title: Real-Time Audio Contract
summary: Enforced callback safety, bounded-work, ownership, overload, and verification rules for M1.
status: accepted
updated: 2026-08-03
issues: [101, 132, 133, 134, 136, 137, 138, 145, 171, 172, 173, 174, 175, 201, 203, 212]
---

# Real-Time Audio Contract

This document is the enforced M1 real-time safety contract. Its hard callback rules landed and are tested through [#172](https://github.com/jpalvarezl/blight-synth/issues/172) (allocation audit harness), [#173](https://github.com/jpalvarezl/blight-synth/issues/173) (bounded callback work and observable backpressure), [#175](https://github.com/jpalvarezl/blight-synth/issues/175) (callback logging/panic removal and hot-path stress), and [#174](https://github.com/jpalvarezl/blight-synth/issues/174) (deferred reclamation of structural/song ownership); parent [#133](https://github.com/jpalvarezl/blight-synth/issues/133) closes on their acceptance.

The contract is intentionally stricter than a single milestone can complete at once. The hard callback rules and deferred reclamation are enforced now; the remaining rows in the violation inventory below are not permission to retain unsafe behavior — each is explicitly owned by its own open issue and remains work to remove. See [Contract completion](#contract-completion) for exactly what is enforced versus deferred.

## Thread roles

### Audio callback / RT owner

The callback owns mutable prepared render state exclusively while processing:

```text
standalone::AudioProcessor::process
  -> Player timing/event interpretation (temporary tracker adapter)
  -> TrackerEngineAdapter
  -> Engine
  -> Instrument / Voice / SynthNode / Effect
```

The future plugin/offline hosts must use the same Engine rules even when their thread names differ.

### Main/control / NRT owner

The control side may allocate, parse, log, access files/network, build factories, decode projects/assets, and prepare immutable or exclusively-owned state. It submits bounded control/event data to RT and reclaims retired state returned from RT.

### File/network workers

Optional workers may perform blocking I/O and decoding. They never mutate live Engine state directly and communicate through the NRT owner/bounded handoff.

## Build and diagnostic modes

The project values ordinary debugging more than pretending every developer build is production-real-time safe.

- **Developer diagnostic builds** may enable direct, compile-time-gated callback logging for functional debugging. Such builds are explicitly not valid for timing, underrun, or allocation-performance claims; logger formatting/locking can glitch audio.
- **Strict RT validation and production builds** compile callback debug logs out and enforce the hard rules below. Allocation/stress tests run with callback diagnostics disabled.
- **NRT logging** remains available in both debug and release builds for device, project, protocol, lifecycle, and support diagnostics.

The shared `dsp::rt_{debug,info,warn,error}_log!` wrappers preserve normal severity levels in developer builds: they perform one runtime enabled check before evaluating arguments, delegate to `log`, and remove the complete call site/argument evaluation when `debug_assertions` are disabled. A release-mode unit test and static callback-path checker enforce that policy. We should not build a lock-free diagnostic subsystem until release callback telemetry is a demonstrated requirement.

## Hard callback rules

Inside callback-reachable code in strict RT builds, including destructors triggered there:

1. **No heap allocation or deallocation.** Destruction is as important as construction: dropping the last `Box`, `Vec`, `Arc`, song, effect, sample, or graph on RT violates the contract.
2. **No blocking or contended locks.** No mutexes, condition variables, waiting channels, thread joins, sleeps, or operations with unbounded retry.
3. **No file, network, terminal, device-management, or other syscall-oriented I/O.**
4. **No production/RT-validation logging or formatting.** Direct callback logs are allowed only through the explicitly compile-time-gated developer diagnostic wrapper; `println!`, `eprintln!`, formatted errors, and normal logger callbacks are otherwise NRT operations.
5. **No panic/unwind across the host boundary.** Callback inputs and capacities have defined fallback/error behavior in all build modes.
6. **No unbounded work.** Every command, event, voice, instrument, effect, and sample loop has an explicit configured maximum or bounded input slice.
7. **No parsing, factory construction, sample decoding, schema migration, or graph compilation.**
8. **No randomized execution order.** Render order and overload behavior are deterministic.

Allowed operations include bounded slice/array iteration, arithmetic/DSP, exclusive mutation of prepared state, nonblocking bounded queue operations, and documented atomics with explicit ordering.

## Prepared-state rule

Anything that can require allocation or failure is prepared on NRT before handoff:

- instruments/voices and their scratch buffers;
- effect instances and delay/reverb storage;
- decoded immutable sample data;
- composition/project snapshots;
- routing/state snapshots;
- parameter descriptors and lookup tables.

A successful NRT preparation does not by itself make installation RT-safe: replacing live state must also avoid deallocating the old state on RT.

## Control traffic classes

These classes have different overload semantics and must not be hidden in one undifferentiated queue.

### 1. Continuous parameters — latest value wins

High-rate knob/automation values use the generation-bound normalized MPSC atomic
store defined by [ADR 0005](../decisions/0005-coalesced-parameter-publication.md).
Adapters resolve stable `ParameterId` on NRT; RT storage uses a
`(ParameterTableGeneration, RuntimeParamKey)` binding so a stale dense key cannot
address a replacement table. At the start of each engine render block, RT scans
at most 16 compact coalesced-slot dirty words and applies at most the prepared
hard limit of 1,024 coalesced targets. Intermediate values may coalesce. Release
dirty publication and acquire clearing guarantee eventual latest after
publishers quiesce; mapped
engine targets, smoothing, applied-target confirmation, reset, invalid writes,
and pressure telemetry follow ADR 0005. Owned by #101/#121/#212.

### 2. Timestamped musical/control events — ordered and bounded

Notes and sample-accurate automation carry an offset within the current block and deterministic same-offset ordering. `engine::BoundedEventAdmission` prepares an explicit stable-producer set and ordinary-event capacity on NRT, accepts at most one bounded slice per active producer per block, validates source identity/sequence/order, and sorts accepted events only by the canonical `TimestampedEvent::order_key()`. Every slice that could fit the complete prepared lane is validated before aggregate-capacity effects; a slice larger than that complete lane is intrinsically overflow and is rejected without an unbounded scan. Admission continues after a producer failure so final rejection is independent of call interleaving: malformed/protocol failures precede overflow, then the lowest stable producer identity selects the recorded diagnostic. Ordinary overflow or malformed input returns compact producer-visible status and rejects the whole ordinary block. One separately prepared all-notes-off slot remains available at ordinary capacity or after ordinary rejection. The canonical event/order contract is #201; bounded current-block admission is #203; #204 integrates the fixed-memory first-party tracker/live producers, while additional composition-source extraction remains #145 under parent #134.

### 3. Structural prepared-state updates — reliable and reclaimable

Instrument/effect/song/routing replacement uses infrequent prepared objects or snapshots. Installation work per block is bounded. Replaced state is returned to NRT for destruction. Owned by #174/#138.

## Work budget and fairness

Queue capacity alone is not a callback budget. The RT loop processes at most a documented number of structural/control items per block, then renders. A producer burst cannot consume the complete callback deadline. Remaining structural/event work stays queued or is rejected according to its class; continuous controls use ADR 0005's fixed dirty-word scan and one-application-per-key bound rather than a drainable queue.

The transitional standalone compatibility queue consumes at most **64 command items per host callback block**, in FIFO order, before rendering. A backlog remains queued for later blocks, including when one callback is split into multiple internal 4096-frame render chunks. This is an item-count bound; worst-case command cost still depends on the prepared-state, capacity, and deferred-reclamation work owned by #137/#174. The initial budget is intentionally independent of queue capacity and does not define the future timestamped-event budget. It will be retired with this mixed compatibility queue when #101/#134 introduce coalesced continuous values and bounded timestamped events with traffic-specific overload behavior.

## Backpressure and overload

- `BlightAudio::try_send_command` is nonblocking and returns `CommandSubmissionResult`, an alias for `Result<(), CommandSubmissionError>`. `Ok(())` means accepted; `CommandSubmissionError::kind` reports `Full` or `Disconnected`, and `into_command` returns the original owned command.
- `BlightAudio::send_command` is the reliable NRT API. On `Full` it retains the same command and parks briefly until RT frees a slot; because the exclusive call does not return, no later command can overtake it. It returns an error only on `Disconnected`. The caller owns NRT thread placement and must not call it from RT or a latency-sensitive UI/async executor thread.
- `BlightAudio::send_command_until` provides the same unboxed reliable retry path with caller-owned NRT cancellation. Cancellation returns `Full` plus the exact command, allowing worker shutdown without allocator churn or command loss.
- Submission errors box their private kind/command payload only when returned to NRT, keeping results small. Reliable retry paths keep repeated full-queue attempts unboxed.
- #181/#182 give both first-party hosts dedicated NRT control ownership. The tracker enqueues semantic requests without owning `BlightAudio`; the standalone Tokio loop enqueues bounded OSC requests. Each worker retains its FIFO front command across RT-ring `Full` responses, performs preparation off latency-sensitive threads, and preserves accepted-only state/protocol updates.
- State-changing protocol acknowledgements are emitted only after `Ok(())`, never after a nonblocking `Full` or any `Disconnected` rejection.
- Continuous values coalesce by contract rather than filling the structural queue. #213 implements the generation-bound normalized store: valid active-generation slots do not return `Full`; overwriting a dirty value is observable normal coalescing, while invalid/stale/closed/revision-exhausted writes are rejected and counted. Its fixed RT drain confirms only successfully latched targets and records compact application failures. Engine target mapping/smoothing and host installation remain #214/#215, and today `/param/set` still enqueues a structural command whose echo means queue acceptance (#216).
- Structural updates are not silently dropped: replaced/rejected owners are retired to NRT rather than discarded on RT. *(Observable NRT result reporting for effect remove/reorder is deferred to open [#136](https://github.com/jpalvarezl/blight-synth/issues/136); those commands currently succeed as no-ops.)*
- Event overflow behavior is explicit and deterministic in `engine::BoundedEventAdmission`: fixed ordinary capacity rejects the complete block with compact producer identity/reason status, and deterministic final failure selection does not depend on which producer submits first. A separate one-event recovery slot keeps all-notes-off admissible at capacity or after rejection. Rejected ordinary events are cleared rather than queued into a later block, and their source-sequence baselines are not committed. #204 applies this policy in the first-party callback and exposes compact timing/event status to the host.
- Capacity exhaustion must have a documented bounded signal. Command-queue submission exposes producer-visible `Full`; hard instrument-capacity rejection is not yet counted or typed and currently surfaces only by retiring the rejected new owner as `RetiredState::Instrument`, the same variant used for a displaced old owner. Developer diagnostic builds may additionally emit a compile-time-gated callback log where one is explicitly specified.

## Deferred reclamation

Installing prepared state follows a swap-and-retire model:

```text
NRT constructs new state
  -> bounded NRT-to-RT handoff
RT swaps at a safe block boundary
  -> old state enters bounded RT-to-NRT retirement handoff
NRT destroys old state
```

If the retirement handoff is full, RT follows a predeclared non-allocating policy; it must not drop the object locally. Shutdown drains/reclaims all ownership exactly once. #174 owns implementation and stress tests.

## Errors, panics, and telemetry

RT methods do not build rich errors in strict builds. They return compact status codes/counters where action is possible; NRT formats/logs them. Developer diagnostic builds may use compile-time-gated direct logs for functional debugging, accepting that those builds are not RT-performance evidence. Missing IDs and malformed buffers use documented no-op/silence/truncation behavior. Capacity/configuration errors are rejected during preparation. Coalesced-parameter target-application failure leaves its applied confirmation unchanged and increments bounded NRT-readable diagnostics; generation reset/replacement is exposed as a transition and never clears dirty state concurrently in place.

A future FFI wrapper catches panics outside the RT entry and must never permit unwinding into C/C++. Panic containment is a last boundary defense, not permission for callback panics.

## Current violation inventory

The hard callback rules ([#172](https://github.com/jpalvarezl/blight-synth/issues/172)/[#173](https://github.com/jpalvarezl/blight-synth/issues/173)/[#175](https://github.com/jpalvarezl/blight-synth/issues/175)) and deferred reclamation ([#174](https://github.com/jpalvarezl/blight-synth/issues/174)) are enforced, so every row that could destroy heap owners, log, or format on the callback is resolved. The former instrument-insertion allocation exception is now also resolved: #137 made instrument slots a hard, fixed-size preallocated vector, so insertion past the 64-slot capacity is rejected and retired to NRT rather than reallocating on the callback (first row). Rows marked *Deferred*/*Partial* are structural/capacity/traffic-class concerns owned by their own still-open issues. No deferred row reintroduces a callback drop, lock, or log, and no *replacement* of prepared state allocates.

| Current path/behavior | Contract gap | Owner | Status |
|---|---|---|---|
| Instrument insertion past prepared capacity | Capacity overflow may allocate on RT | #137 | Resolved (#137). Engine's instrument slots are a hard, fixed-size preallocated vector (`DEFAULT_INSTRUMENT_CAPACITY`): a distinct instrument past the cap is rejected and its owner retired to NRT, so insertion never reallocates on the callback (see `audio_processor/mod.rs` NOTE(#137)). Per-instrument polyphony is a fixed voice pool with deterministic oldest-first stealing and targeted, identity-based note-off. |
| Instrument/effect commands carry `Box`/`ArrayVec<Box<_>>` | Consuming/rejecting/replacing can destroy heap owners on RT | #174 | Resolved (#174). `Engine::handle_command_with_retirement` routes every displaced/rejected owner to a `RetireSink` instead of dropping it; `rt_allocations.rs` proves zero callback dealloc. |
| `Player::load_song` replaces `Arc<Song>` and clears instruments | Last-owner song/graph destruction can occur on RT | #174/#138 | Resolved for RT-safety (#174). `Player::set_song` retires the displaced `Arc<Song>` as `RetiredState::Prepared` and `clear_instruments` retires each instrument; broader versioned snapshots remain #138. |
| Effect-chain/master remove/reorder commands | Current no-op semantics provide no observable result | #136/#173 | Partial. Rejected/overflow effects now retire safely (#173/#174); surfacing `error.kind()` as an observable NRT command result is deferred to open #136. |
| Player/tracker adapter note/row/end logging | Formatting/logger calls reachable from callback | #175 | Resolved (#175). Migrated to the compile-out `dsp::rt_*_log!` wrappers; a release compile-out test and `scripts/check_rt_logging.py` enforce it in CI. |
| Polyphonic instrument note allocation logs and warning paths | Logging reachable from callback | #175/#137 | Resolved. Logging migrated to compile-out wrappers (#175); voice-allocation and hard-capacity semantics resolved by #137 (fixed voice pool with deterministic oldest-first stealing, identity-based note-off, and hard instrument capacity). |
| Delay/reverb/drum parameter error logging | Logging reachable from command handling on callback | #175/#101 | Logging resolved (#175); #213 implements the host-independent coalesced store, while engine target mapping/smoothing migration remains #214. |
| Tracker active-instrument state | Capacity must be structurally fixed for direct RT evaluation | #175/#204 | Resolved (#204). `TrackerEngineAdapter` uses `[InstrumentId; MAX_TRACKS]`; Player prepares fixed tick/event/admission storage and documents the 4096-tick × eight-track × two-event worst case. |
| Voice scratch buffers are fixed at 4096 frames | Direct oversized Engine use can panic below a negotiating host adapter | #132/#175 | Mitigated. `AudioProcessor::process` chunks oversized host buffers into 4096-frame renders (tested); the negotiated `Engine` lifecycle is deferred to open #132. |
| Engine/effect capacities are preallocated but not hard-rejected consistently | Allocation/overflow behavior is incomplete | #137/#136/#175 | Partial. Master-effect overflow now rejects and retires without RT heap work (tested); hard instrument capacity is resolved (#137, distinct instruments past the cap are rejected and retired); routing-graph semantics (#136) remain open. |
| Structural command work mixes notes, parameters, and graph updates | Overload semantics are conflated | #101/#134/#173/#174/#203/#204 | Partial. Bounded command work/reclamation landed (#173/#174), host-independent admission landed (#203), and #204 routes queued live note/release plus tracker playback through that lane. #213 lands the host-independent coalesced store; engine application and host/OSC migration remain #214–#216. |

## Existing safe foundations

- `engine` and `dsp` have no CPAL/OSC/Tokio/file dependencies.
- `engine::CoalescedParameterStore` prepares stable compact bindings and at most 1,024 packed atomic slots on NRT. Generation-bound cloneable publishers use Relaxed packed-slot CAS followed by Release dirty `fetch_or`; the single RT consumer performs exactly 16 Acquire dirty-word swaps, relaxed slot loads, and Release applied confirmations. Closure/reset uses physically separate non-reused generations, and Loom/stress/allocation tests cover eventual-latest, races, fixed bounds, and zero heap work in publication/drain (#213). Target mapping/smoothing and host lifecycle remain #214/#215.
- Standalone callback buffers are preallocated and oversized host buffers are chunked.
- Engine instrument render order uses a sorted preallocated slot vector.
- `engine::TimestampedEvent`/`Engine::process_with_events` validate a current-block slice before mutation, apply canonical deterministic ordering, segment rendering at exact offsets, and have zero-heap audit coverage (#201). `engine::BoundedEventAdmission` adds an NRT-prepared fixed ordinary-event capacity and stable-producer table; bounded per-source validation, allocation-free canonical sort, interleaving-independent whole-block rejection, an out-of-band recovery slot, and reset/reuse all have hardware-free and zero-heap audit coverage (#203). #204 integrates tracker/live producers, stopped-transport rendering, compact host status, and zero-heap callback coverage.
- Voice effect batches use fixed-capacity `ArrayVec` containers.
- Meter handoff uses nonblocking atomics and performs network/formatting work outside RT.
- The transitional standalone command queue applies a 64-command FIFO prefix per callback; reliable NRT submission preserves FIFO order across saturation, nonblocking submission exposes explicit backpressure, and OSC success responses require acceptance.
- Instrument replacement/clear plus mono, voice, and master effect rejection surface `engine::RetiredState` through `RetireSink` and cross a bounded reverse RT-to-NRT ring. Replaced tracker `Arc<Song>` snapshots retire through the same path as an opaque `RetiredState::Prepared` owner. The callback preallocates 4160 pending retired-object slots (64 commands × up to 65 objects/command; the worst case is a song load that clears 64 instruments and retires the replaced song), retains ring overflow there, and pauses subsequent-block command consumption until NRT headroom returns (#186/#187/#188).
- Factories and project/sample decoding already live on NRT paths by architecture.
- Offline golden renders provide end-to-end behavioral regression evidence, though they do not prove allocation safety.

## Verification plan

- #172: [`engine/tests/rt_allocations.rs`](../../engine/tests/rt_allocations.rs) provides test-only allocation/deallocation instrumentation around representative prepared processing plus a known-allocating self-test fixture; [harness details](rt-allocation-audit.md).
- #173: bounded command/burst/backpressure/fairness tests.
- #174: thread-identified drop probes and stress tests for swap/retire/shutdown ownership.
- #175: central debug-only callback logging macro, release compile-out checks, static inventory, and malformed-input/capacity/block-size hot-path stress tests.
- #203: [`engine/tests/bounded_event_admission.rs`](../../engine/tests/bounded_event_admission.rs) covers exact/over capacity, interleaving-independent canonical order, malformed source metadata, recovery at capacity and after rejection, and reset/reuse; [`engine/tests/rt_allocations.rs`](../../engine/tests/rt_allocations.rs) measures the complete prepared admission/sort/recovery path.
- Existing golden renders: detect accidental sonic/timing behavior changes while enforcement lands.

## Relationship to Engine lifecycle (#132)

`prepare` must establish sample rate, maximum block/channel layout, capacities, scratch storage, and prepared state before processing. `process` accepts only bounded prepared inputs and cannot fail through allocation or rich errors. `reset/suspend/resume` must define whether they run on RT and therefore whether they may reclaim state. Public compatibility re-exports may narrow only after these ownership/lifecycle boundaries are explicit.

## Contract completion

This is the enforced M1 contract. The hard callback rules and deferred reclamation landed and are verified; the former #137-owned instrument-insertion allocation exception is now resolved with hard instrument capacity (disclosed in the inventory and below):

- **[#172](https://github.com/jpalvarezl/blight-synth/issues/172) (closed)** — [`engine/tests/rt_allocations.rs`](../../engine/tests/rt_allocations.rs) instruments allocation/deallocation/reallocation around the representative prepared note/parameter/render path and structural instrument/effect replacement, and includes a known-allocating self-test. It proves the no-heap rule for those representative paths; the remaining instrument-*insertion* allocation is now resolved by #137 (hard capacity, first inventory row), with a test proving no RT reallocation past the cap.
- **[#173](https://github.com/jpalvarezl/blight-synth/issues/173) (closed)** — the standalone callback applies a bounded 64-command-per-block budget with FIFO fairness and exposes nonblocking/reliable submission with observable `Full`/`Disconnected` backpressure.
- **[#175](https://github.com/jpalvarezl/blight-synth/issues/175) (closed)** — callback logging moved to the compile-out `dsp::rt_*_log!` wrappers, enforced by a release compile-out test and the `scripts/check_rt_logging.py` static checker in CI, plus malformed-input/oversized-block hot-path stress tests.
- **[#174](https://github.com/jpalvarezl/blight-synth/issues/174) (closing)** — structural instrument/effect/song replacement routes displaced owners through `engine::RetireSink`/`RetiredState` across a bounded RT-to-NRT retirement ring; overflow is retained in a preallocated pending buffer (never dropped on RT), and shutdown drains every owner exactly once. Stress tests cover swap/retire/shutdown ownership.

Parent [#133](https://github.com/jpalvarezl/blight-synth/issues/133) closes on this basis. The remaining violation-inventory rows are deferred to their own open issues — routing/observable results ([#136](https://github.com/jpalvarezl/blight-synth/issues/136)), coalesced continuous parameters ([#101](https://github.com/jpalvarezl/blight-synth/issues/101)), additional timestamped-event-source extraction ([#134](https://github.com/jpalvarezl/blight-synth/issues/134)/#145; first-party tracker/live integration is resolved by #204), engine lifecycle ([#132](https://github.com/jpalvarezl/blight-synth/issues/132)), versioned snapshots ([#138](https://github.com/jpalvarezl/blight-synth/issues/138)), and the event-source contract ([#145](https://github.com/jpalvarezl/blight-synth/issues/145)). Capacity/polyphony ([#137](https://github.com/jpalvarezl/blight-synth/issues/137)) is resolved: it delivers per-instrument polyphony with note identity, deterministic oldest-first voice stealing, targeted identity-based note-off, and a hard instrument capacity that rejects and retires distinct instruments past the cap instead of reallocating on the callback. The former bounded exception to Hard Rule 1 — installing more than the 64-instrument capacity — no longer applies: `Engine` now uses a hard, fixed-size preallocated slot vector. No deferred row reintroduces a callback drop, lock, or log, and no *replacement* of prepared state allocates. “Enforced” here means this document is the authoritative M1 rulebook with an explicit, owned inventory of the residual gaps — not that every code path already complies. Any new exception to the hard rules requires an explicit documented rationale, bounded behavior, tests, and review; “unlikely in practice” is not an exception.
