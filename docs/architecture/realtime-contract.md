---
title: Real-Time Audio Contract
summary: Proposed callback safety, bounded-work, ownership, overload, and verification rules for M1.
status: draft
updated: 2026-07-18
issues: [101, 132, 133, 134, 138, 171, 172, 173, 174, 175]
---

# Real-Time Audio Contract

This document defines the contract M1 will enforce. It is intentionally stricter than the current implementation; the violation inventory below is work to remove, not permission to retain it.

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

A small `rt_debug_log!`-style wrapper should make the exception visible and remove the complete call site/argument evaluation when disabled. We should not build a lock-free diagnostic subsystem until release callback telemetry is a demonstrated requirement.

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

High-rate knob/automation values use coalesced fixed storage keyed by stable parameter ID. RT consumes dirty/latest values at a documented control rate. Intermediate values may coalesce; the final submitted value must be observable. Owned by #101/#121.

### 2. Timestamped musical/control events — ordered and bounded

Notes and sample-accurate automation carry an offset within the current block and deterministic same-offset ordering. Capacity is fixed; overflow has an explicit producer-visible policy and cannot silently reorder events. Owned by #134/#145.

### 3. Structural prepared-state updates — reliable and reclaimable

Instrument/effect/song/routing replacement uses infrequent prepared objects or snapshots. Installation work per block is bounded. Replaced state is returned to NRT for destruction. Owned by #174/#138.

## Work budget and fairness

Queue capacity alone is not a callback budget. The RT loop processes at most a documented number of structural/control items per block, then renders. A producer burst cannot consume the complete callback deadline. Remaining work stays queued or is coalesced according to class.

#173 must define and test the initial command budget, queue fairness, producer status, and recovery behavior. #134 separately defines the event budget and timing contract.

## Backpressure and overload

- NRT submission returns an observable accepted/full/disconnected or equivalent status.
- State-changing protocol acknowledgements are emitted only after accepted semantics, never after silent rejection.
- Continuous values coalesce by contract rather than filling the structural queue.
- Structural updates are not silently dropped.
- Event overflow behavior is explicit and deterministic; all-notes-off/recovery remains possible.
- Capacity exhaustion increments a bounded counter/status in strict RT builds. Developer diagnostic builds may additionally emit a compile-time-gated callback log.

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

RT methods do not build rich errors in strict builds. They return compact status codes/counters where action is possible; NRT formats/logs them. Developer diagnostic builds may use compile-time-gated direct logs for functional debugging, accepting that those builds are not RT-performance evidence. Missing IDs and malformed buffers use documented no-op/silence/truncation behavior. Capacity/configuration errors are rejected during preparation.

A future FFI wrapper catches panics outside the RT entry and must never permit unwinding into C/C++. Panic containment is a last boundary defense, not permission for callback panics.

## Current violation inventory

| Current path/behavior | Contract gap | Owner |
|---|---|---|
| `standalone/audio_processor`: drains `while let Some(command)` | No explicit per-block command budget/fairness | #173 |
| `BlightAudio::send_command`: returns `()` and prints on full queue | Producer cannot react; protocol can imply false success | #173, #104 |
| `Engine::add_instrument`, sorted `Vec::insert`, replacement, `clear_instruments` | May allocate past capacity and drops replaced/all boxed instruments on RT | #174; hard capacity #137 |
| Instrument/effect commands carry `Box`/`ArrayVec<Box<_>>` | Consuming/rejecting/replacing can destroy heap owners on RT | #174 |
| `Player::load_song` replaces `Arc<Song>` and clears instruments | Last-owner song/graph destruction can occur on RT | #174/#138 |
| Stereo/mono effect-chain add when full | `eprintln!` and dropped effect on RT; no status | #174/#175/#136 |
| Effect-chain/master remove/reorder commands | Current no-op semantics provide no observable result | #136/#173 |
| Player/tracker adapter note/row/end logging | Formatting/logger calls reachable from callback | #175 |
| Polyphonic instrument note allocation logs and warning paths | Logging reachable from callback | #175/#137 |
| Delay/reverb/drum parameter error logging | Logging reachable from command handling on callback | #175/#101 |
| Tracker active-instrument `HashMap::insert` | Capacity is assumed rather than structurally fixed | #175/#145 |
| Voice scratch buffers are fixed at 4096 frames | Direct oversized Engine use can panic below a negotiating host adapter | #132/#175 |
| Engine/effect capacities are preallocated but not hard-rejected consistently | Allocation/overflow behavior is incomplete | #137/#136/#175 |
| Structural command work mixes notes, parameters, and graph updates | Overload semantics are conflated | #101/#134/#173/#174 |

## Existing safe foundations

- `engine` and `dsp` have no CPAL/OSC/Tokio/file dependencies.
- Standalone callback buffers are preallocated and oversized host buffers are chunked.
- Engine instrument render order uses a sorted preallocated slot vector.
- Voice effect batches use fixed-capacity `ArrayVec` containers.
- Meter handoff uses nonblocking atomics and performs network/formatting work outside RT.
- Factories and project/sample decoding already live on NRT paths by architecture.
- Offline golden renders provide end-to-end behavioral regression evidence, though they do not prove allocation safety.

## Verification plan

- #172: [`engine/tests/rt_allocations.rs`](../../engine/tests/rt_allocations.rs) provides test-only allocation/deallocation instrumentation around representative prepared processing plus a known-allocating self-test fixture; [harness details](rt-allocation-audit.md).
- #173: bounded command/burst/backpressure/fairness tests.
- #174: thread-identified drop probes and stress tests for swap/retire/shutdown ownership.
- #175: central debug-only callback logging macro, release compile-out checks, static inventory, and malformed-input/capacity/block-size hot-path stress tests.
- Existing golden renders: detect accidental sonic/timing behavior changes while enforcement lands.

## Relationship to Engine lifecycle (#132)

`prepare` must establish sample rate, maximum block/channel layout, capacities, scratch storage, and prepared state before processing. `process` accepts only bounded prepared inputs and cannot fail through allocation or rich errors. `reset/suspend/resume` must define whether they run on RT and therefore whether they may reclaim state. Public compatibility re-exports may narrow only after these ownership/lifecycle boundaries are explicit.

## Contract completion

This draft becomes the enforced M1 contract only when #172–#175 pass and parent #133 closes. Any exception requires an explicit documented rationale, bounded behavior, tests, and review; “unlikely in practice” is not an exception.
