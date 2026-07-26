---
title: ADR 0003 — Event-source contract between composition, engine, and host
summary: Make the audio engine a consumer of bounded, already-offset musical/control/transport events produced by pluggable composition runtimes, with clock mapping and event generation outside the engine.
status: proposed
updated: 2026-07-26
issues: [145, 134, 132, 138, 113, 101, 121, 136, 174]
supersedes: []
---

# ADR 0003 — Event-source contract between composition, engine, and host

## Status

Proposed

Deciding issue: [#145](https://github.com/jpalvarezl/blight-synth/issues/145).
This ADR records a target boundary and its contracts only. It does **not**
change engine/DSP code, Cargo files, or the accepted
[real-time audio contract](../architecture/realtime-contract.md). Implementation
is scheduled follow-up work under #134, #132, #138, and the composition-adapter
extraction described below.

## Context

The current first-party tracker/song playback path reaches `engine::Engine`
through `audio_backend::player::Player` and `TrackerEngineAdapter`. `Player`
owns an `Arc<Song>`, `TimingState`, and `PlayerPosition`. For each callback,
`TimingState::advance` returns only the number of elapsed ticks; `Player` walks
`Song → arrangement → Chain → Phrase → Event`, invokes note methods, and then
renders the whole block. It therefore has the useful shape of demand-driven
block evaluation, but it does **not** compute sample offsets and is not a
sample-accurate implementation of this contract.

The tracker path is not the only way to drive the engine. `Engine` exposes
public note, command, and `process` methods used directly by
`engine/examples/offline_render.rs` and tests. The coupling problem is narrower:
the current first-party *song/composition* path interprets one tracker document
model directly on the callback instead of crossing a generic event boundary.

`TrackerEngineAdapter` also already owns the tracker-only
`track_last_instrument` state. Merely saying that state moves “behind the
adapter” does not fix its real-time violation: it is a `HashMap`, and
`HashMap::insert` remains callback-reachable and can allocate despite initial
preallocation. A direct-RT tracker evaluator needs structurally fixed indexed
state, or tracker evaluation must move to NRT.

This matters for two accepted directions:

- [ADR 0001](0001-product-and-host-priorities.md) keeps the composition
  interaction open. The tracker remains supported, but an ORCA-like grid,
  hybrid, or generative model must not require DSP, mixer, routing, or device
  changes.
- The [real-time audio contract](../architecture/realtime-contract.md) defines
  timestamped musical/control events as a distinct ordered, bounded traffic
  class. It must not be conflated with latest-value continuous parameters or
  prepared structural updates, and its overload behavior must keep recovery
  reachable.

The earlier draft also assigned musical-time conversion to both the engine and
clock/event-source adapters, and mixed current-block output with future
lookahead in one buffer. Neither model has one implementable owner. This
revision chooses one clock flow and separates the current-block pull from NRT
lookahead publication.

## Decision

Adopt a host-orchestrated, demand-driven boundary:

```text
selected clock/input adapters + versioned document
  -> composite event source/scheduler
  -> current-block, already-offset event slice
  -> Engine event application + DSP rendering
```

The engine never interprets a composition document and never converts musical
or external clock time into sample offsets.

### 1. Roles and clock authority

**Host/control and clock adapters.** The host owns device/plugin callback
wiring, clock-source selection, filesystem and MIDI/OSC I/O, live edit
submission, and side-effect routing. For every render block it provides the
producer side with a bounded `BlockWindow` containing, conceptually:

- a `clock_epoch` identifying one continuous clock mapping;
- the half-open absolute render-frame interval
  `[start_frame, start_frame + frame_count)`;
- a prepared, bounded list of clock segments that is queryable in both
  directions between render frames and semantic musical positions; and
- the active document/publication generation and bounded timestamped input
  trace.

Each clock segment has a half-open frame span, a monotonic/invertible semantic
span, transport state, and an occurrence identity (including loop iteration
when needed). Stopped regions produce no musical-deadline mapping; absolute
live-input frames remain schedulable. Loop wraps and other non-invertible points
split segments rather than asking each producer to invent an inverse. The
maximum segments and loop crossings per block are fixed at `prepare`. A loop or
tempo map whose worst case exceeds that bound is rejected on NRT before
transport starts; an unexpected invalid map fails closed and starts a new epoch.
This makes both frame-to-position observation and semantic-deadline-to-frame
conversion one shared, deterministic operation.

Internal, plugin-host, MIDI, and external clocks all terminate in a clock
adapter here. MIDI/OSC ingestion and clock estimation run on NRT. A seek, clock
source change, or discontinuous remap creates a new `clock_epoch`; it is not
smuggled into an existing continuous mapping.

**Composition runtime/event source.** The runtime owns document semantics,
interpreter cursor, seeded randomness, and composition-specific replay. Given a
`BlockWindow`, it or its producer-side clock adapter maps semantic deadlines to
absolute render frames and then to `sample_offset` values in the current block.
It emits only already-offset events. It does no audio-device, socket, or
filesystem I/O and does not mutate the engine object graph.

**Composite event source/scheduler.** One host-owned composite producer merges
the active composition runtime, live input, transport, and sample-accurate
automation producers. It is the only writer of the final engine event buffer.
This gives cross-producer conflicts one deterministic ordering authority rather
than relying on callback, queue, or hash iteration order.

**Audio `Engine`.** The engine owns instruments, voices, routing, render-time
transport state, event application, DSP rendering, and telemetry. It receives a
block descriptor and a bounded sorted slice whose offsets are already in
`[0, frame_count)`. It applies events at those offsets and renders caller-owned
buffers. It does **not** receive musical timestamps, select a clock, estimate
tempo, or convert time. `Transport` events are already-offset state transitions
such as start, stop, continue, or loop boundary; they are not requests for the
engine to perform clock conversion. Stopping transport does not by itself
prevent effect/release tails from rendering (#132).

This producer-side conversion is the sole clock/timestamp flow. It intentionally
resolves the ambiguous “engine owns transport clock conversion” wording from the
issue boundary in favor of an engine API that consumes sample offsets.

### 2. Current-block pull interface

“Pull” means the host scheduler requests bounded output for the current block;
it does not mean that `Engine` calls arbitrary composition code. The conceptual
call sequence is:

```text
composite.fill_current_block(window, bounded_inputs, caller_owned_event_buffer)
engine.process(block_descriptor, caller_owned_event_buffer.as_ordered_slice())
```

The concrete Rust API belongs to #134/#132, but it must preserve these
invariants:

- The host owns and preallocates the event buffer during `prepare`. The callback
  passes it empty to exactly one composite producer and retains ownership.
- The ordinary event capacity and the separate recovery capacity are fixed by
  preparation. Filling, sorting/merging, and application perform no allocation,
  deallocation, blocking, logging, parsing, or unbounded retry.
- The buffer contains only events for the requested half-open current window.
  Every `sample_offset` is in `[0, frame_count)`. An event exactly at the end
  belongs to the next block.
- Producer count, per-producer scratch capacity, input count, event count, and
  merge work all have configured maxima. Child producers emit ordered streams;
  the composite performs a bounded merge rather than a general allocating sort.
- The fill result reports compact statuses such as complete, ordinary overflow,
  invalid clock/generation, or lookahead deadline miss. NRT formats diagnostics;
  strict RT only updates bounded counters/status.

A fixed-memory runtime may evaluate in this call only if all of its state and
worst-case work satisfy the real-time contract. Otherwise its adapter consumes
prepared NRT lookahead as described next. The engine sees the same current-block
slice in either case.

### 3. NRT lookahead is a separate publication contract

Future lookahead never shares the current-block output buffer. An NRT evaluator
uses separate storage with two limits established at `prepare`:

1. a maximum future horizon in render frames; and
2. a maximum number of stored events/coverage records.

Lookahead records use the absolute producer-side timestamp domain
`(clock_epoch, publication_generation, absolute_render_frame)`, plus ordering
metadata. They do not contain an in-block offset until the RT adapter pulls the
matching current window. NRT publishes fixed-capacity plain-data event packets
and a contiguous `covered_through_frame` marker through a prepared bounded SPSC
ring. Publication of the marker follows publication of every record it covers.

The ring protocol is explicitly two-way: its NRT write cursor is the publication
watermark, and RT advances an atomic read/consumption cursor only after it no
longer needs a packet. NRT observes that cursor before reusing a slot and never
overwrites unread data. A full ring rejects further publication with a compact
observable status; coverage cannot advance past an unpublished/full slot, so a
block eventually takes the declared deadline-miss path rather than reading torn
or overwritten data. Packet slots contain no heap ownership, and cursor
publication/consumption is bounded and nonblocking. Ownership-bearing document
or generation state uses the separate structural swap-and-retire path.

At callback time, the lookahead adapter may expose a block only if matching
records provide contiguous declared coverage through the block end. It copies
or views only events inside the current window and computes their
`sample_offset = absolute_render_frame - start_frame`. Future records remain in
the lookahead store. Thus horizon capacity provides scheduling headroom without
changing current-block buffer semantics.

A missing complete coverage watermark at the start of fill is a deterministic
deadline miss. The adapter does not consume a partial block, wait, or apply a
late packet on a later callback. It raises a recovery barrier at offset zero,
increments an underflow counter, and enters a suspended state. Recovery stops
composition transport and releases active notes while DSP tails may continue.
The source remains suspended until the host explicitly installs/resumes a
prepared generation with contiguous coverage beginning at a declared future
block. Scheduler speed therefore affects whether a deadline miss occurs, but
the response to the observed deadline trace is fixed and testable.

Document edits, seeks, loop-policy changes, and clock discontinuities invalidate
stale lookahead as follows:

- records are tagged with document revision, `clock_epoch`, and publication
  generation;
- an invalidating operation prepares a new generation on NRT and installs its
  snapshot, cursor, and publication storage together at a block boundary;
- RT accepts only the active tags and never scans an unbounded stale backlog;
- displaced snapshot/publication ownership follows the structural
  swap-and-retire rule and is reclaimed on NRT; and
- old-generation notes are reconciled by a bounded prepared release list or,
  by default, the recovery barrier. Old events never leak into the new epoch.

A predictable loop whose mapping was published ahead may remain in one epoch.
A discontinuous or edited loop mapping creates a new generation.

### 4. Event model and total ordering

The ordinary engine lane contains:

- `Note`: note-on, note-off, and choke/release operations with stable target
  identity and velocity/expression data;
- `Control`: sample-accurate automation keyed by stable target/parameter IDs;
  continuous latest-value knob traffic remains on traffic class 1; and
- `Transport`: already-offset render-state transitions (start, stop, continue,
  reset/seek notification, loop boundary), not musical-time conversion.

Every child producer receives a stable `producer_id` during preparation and
assigns a deterministic source-local `producer_sequence`. The composite merger
uses the total key:

```text
(sample_offset, semantic_phase, producer_id, producer_sequence)
```

`semantic_phase` is fixed by the event schema:

1. stop/seek/discontinuity transport barriers;
2. note-off/choke/release;
3. start/continue and other non-barrier transport state;
4. sample-accurate control;
5. note-on/attack.

The schema must define a phase for every future event kind before that kind can
enter the lane. Stable IDs, not runtime registration order or hash order, decide
cross-producer ties. Same-offset `Control` applies to already-existing target
state; expression needed to initialize a newly attacked voice travels in that
`Note` event. The composite validates generation tags and does not include them
in the final engine slice, so the engine does not track document/publication
generations. It preserves the total key exactly. A stop/seek transition closes
the render transport gate for later attacks (releases still apply) until an
ordered start/continue transition reopens it.

### 5. Capacity, rejection, and always-reachable recovery

Traffic class 2 uses two structurally separate capacities:

- an ordinary lane of `C` events; and
- one reserved, coalescing recovery latch that ordinary events can never
  consume.

The composite bounded merge retains the earliest `C` ordinary events by the
total key and rejects every later ordinary event. The fill report exposes the
rejected count and first rejected key; strict RT increments a bounded overflow
counter. It never overwrites an accepted event, silently reorders the prefix, or
uses “drop oldest/newest” based on arrival races.

Rejecting any ordinary event fails closed. At the first rejected event's offset,
the recovery latch produces one compound recovery barrier whose semantics are
engine-global transport-stop plus all-notes-off. M1 uses global scope because
voices do not yet carry producer ownership; pretending recovery were
producer-scoped could leave shared-instrument notes stuck. The conservative
barrier may therefore release notes started by another producer.

The engine applies the retained canonical prefix before that point, then applies
the barrier and ignores ordinary events at or after it for the current block.
The composite scheduling generation becomes suspended and requires the same
explicit prepared resync as an underflow. Multiple recovery requests coalesce
to the earliest offset; the global effect is already the widest scope, so the
latch remains bounded and cannot itself become `Full`. This makes
note/transport recovery reachable even when ordinary capacity is exhausted and
avoids hanging voices after a rejected note-off. A future producer-scoped
barrier first requires an accepted voice-ownership model.

Malformed ordering, an out-of-window timestamp, a child scratch overflow, and
an NRT lookahead-capacity failure use the same fail-closed recovery semantics.
Tests must cover capacity zero/one, same-offset overflow, rejected note-off, and
repeated panic requests.

### 6. Fixed-memory RT evaluation and the tracker adapter

The producer strategy is per runtime:

- A fixed-memory RT evaluator may fill the current block directly when its
  document view is immutable/prepared, all mutable state is fixed-capacity, and
  event/work bounds are proven from `prepare` limits.
- An arbitrary generative, graph-rewriting, or otherwise unbounded runtime runs
  on NRT and uses the lookahead publication contract. It never evaluates its
  program on the callback.

The current tracker becomes one event-source adapter, but the boundary alone is
not an RT fix. `track_last_instrument` already lives in
`tracker_engine_adapter.rs`; its callback-reachable `HashMap::insert` violates
Hard Rule 1. Before tracker evaluation can be classified as direct RT, the
follow-up must replace it with structurally fixed indexed state (for example, a
prepared `[InstrumentId; MAX_TRACKS]` plus explicit validity/sentinel state) and
prove all tick/event bounds. If that work is not done, tracker interpretation
must run on NRT lookahead. Preallocating a `HashMap` or moving it to another
adapter type is insufficient.

`TimingState::advance` also only returns an elapsed tick count. It is evidence
for demand-driven evaluation, not for sample-accurate timing. #134 must preserve
fractional timing and calculate each event's absolute frame/offset instead of
triggering all elapsed ticks before whole-block rendering.

The target event API becomes the first-party tracker/song processing path after
#132/#134. Public direct engine note/command/process APIs may remain for
instrument audition, embeddings, examples, and tests; this ADR does not claim
that the tracker adapter is the engine's only caller.

### 7. Seed, seek, loop, restart, and save determinism

A runtime that promises deterministic replay must declare one of these models:

- **Position/counter-addressed randomness.** Each draw is a pure function of a
  versioned algorithm ID and a key such as `(seed, stable node ID, semantic
  position, local draw index, loop iteration when applicable)`. Seeking can
  calculate the same draw without replaying wall time.
- **Checkpoint replay.** A checkpoint contains the exact document revision,
  semantic cursor, loop iteration, RNG algorithm/state, pending interpreter
  state, and any event cursor needed for replay. Seek restores a canonical
  checkpoint at or before the target and deterministically replays to the exact
  target. Replay runs on NRT when its work is not strictly bounded for RT.

Every loop declares one of two policies in versioned state:

- `repeat`: each pass restores the loop-entry cursor/RNG state, or excludes loop
  iteration from counter-addressed keys, so the loop repeats; or
- `evolve`: a persisted monotonic loop iteration participates in the RNG
  key/state, so each pass differs reproducibly.

Restart restores the document's defined initial cursor/seed/checkpoint. Seek
must include enough transport context (including loop iteration for `evolve`)
to identify one state; an ambiguous position is rejected on NRT rather than
chosen from wall-clock history. Save/restore persists the declared seed model,
algorithm version, cursor, loop policy/iteration, and exact checkpoint/RNG state
where applicable. No promised replay path draws from wall-clock or OS entropy.

This defines **semantic event-stream determinism**: for the same versioned
runtime/document, seed or checkpoint, block-window/input trace, and loop policy,
two runs produce equal event values and total order. “Equal” does not mean
byte-identical until a versioned canonical event serialization is separately
defined.

Render determinism is a separate property. Given equal semantic events, the
same engine/DSP version and configuration must be repeatable under the
[offline render contract](../architecture/offline-render-contract.md). Exact
PCM/reference equality is required only on that contract's canonical platform;
other platforms use its repeated-render, structure, clipping, and metric policy
unless they gain a separately reviewed platform hash.

### 8. Live edits and prepared generations

Live edits build immutable versioned document/runtime snapshots on NRT. The host
installs a prepared snapshot, runtime cursor/checkpoint, and (for NRT sources)
lookahead publication generation together at a block boundary. The displaced
state is retired for NRT destruction under the structural traffic class
(#174/#138). No callback path parses, migrates, allocates, or drops the last
owner.

An edit either supplies a bounded prepared reconciliation list for notes that
survive the revision or uses the default recovery barrier. In both cases the
new generation starts from an explicit cursor/clock mapping, and old lookahead
is invalid by tag. This is a semantic choice of the runtime snapshot, not an
engine document concern. This ADR does not promise a seamless live edit: the
safe default audibly releases notes, while any seamless policy must be fully
prepared and bounded before the swap.

### 9. Outbound MIDI/OSC side effects

Abstract outbound MIDI/OSC is **not** an engine event kind and never causes I/O
on the audio thread. Under this ADR, a direct-RT event evaluator is prohibited
from producing outbound side effects. A runtime that emits outbound events must
evaluate that output on NRT (it may use the same deterministic semantic plan as
its audio events) and submit abstract timestamped messages to a host-owned NRT
side-effect scheduler.

That scheduler has a configured horizon/event capacity and deterministic
admission: retain the earliest messages by their canonical timestamp/order key,
reject the suffix with an observable status, then require a destination-scoped
panic/all-notes-off before accepting later output for that destination. The host
performs actual MIDI/OSC I/O and handles transport-specific timing on NRT.

A future need to derive outbound messages on RT requires a superseding/additive
contract with a bounded RT-to-NRT handoff and deterministic overflow/recovery;
it is not permitted implicitly by this ADR.

### Non-goals

- No engine/DSP/Cargo change or tracker extraction in this documentation PR.
- No selection of the final composition UI/language; that remains #113.
- No change to latest-value continuous parameters (#101/#121) or structural
  prepared-state reclamation (#174/#138).
- No canonical binary serialization for semantic events.
- No guarantee of byte-identical PCM across platforms beyond the offline render
  contract's platform policy.

## Consequences

### Positive

- Clock ownership is singular: host/producer-side adapters define windows and
  offsets; the engine only applies already-offset events.
- Current-block work, NRT scheduling headroom, and lookahead publication have
  separate capacities and timestamp domains.
- Overload is deterministic and fail-closed, with a reserved recovery mechanism
  that cannot be crowded out by ordinary events.
- One composite merger defines cross-producer ordering.
- RT eligibility is structural and testable; the current tracker `HashMap` is
  called out as work to replace rather than hidden by an adapter rename.
- A second composition model can drive the engine without changing DSP, mixer,
  routing, or device code.

### Costs and risks

- The host must coordinate block windows, composite producer IDs, generation
  swaps, and clock epochs instead of delegating musical-time conversion to the
  engine.
- NRT sources need bounded packet storage, a coverage protocol, deadline
  telemetry, and explicit resynchronization after failure.
- Fail-closed overflow/underflow can audibly stop playback and, because M1
  recovery is engine-global, can release notes from a producer that did not
  overflow. That is preferable to nondeterministic partial evaluation or stuck
  notes without producer-owned voices, but capacities must be measurable and
  configurable.
- Position-addressed RNG constrains runtime design; checkpoint replay adds
  snapshot storage and NRT replay cost.
- Prohibiting outbound generation from direct RT evaluators may require a
  runtime with outbound behavior to use NRT even when its audio-event evaluator
  could otherwise be RT-safe.

## Alternatives considered

### Engine owns tempo/clock conversion

Rejected. It duplicates authority with composition/clock adapters, forces the
engine to understand musical and external clock domains, and makes NRT
lookahead's timestamp domain ambiguous. A producer-side `BlockWindow` gives one
mapping authority while preserving already-offset engine events.

### Put current and future events in one fill buffer

Rejected. Current-block offsets are defined only in `[0, frame_count)`, whereas
future lookahead needs an absolute timestamp, horizon, coverage, and publication
protocol. Combining them makes capacity and deadline semantics impossible to
state cleanly.

### Unbounded push stream

Rejected. It races the callback and provides neither a per-block work budget nor
deterministic overload. NRT lookahead does publish ahead, but only into bounded,
prepared storage that the current-block adapter pulls under a coverage rule.

### Evaluate every runtime on RT

Rejected. Arbitrary generative/graph programs cannot satisfy no-allocation and
bounded-work rules. RT evaluation remains an earned property of a concrete
adapter, not a requirement of the engine contract.

### One undifferentiated command/event queue

Rejected. Continuous values, timestamped events, and prepared structural swaps
have different overload and reclamation semantics under the accepted RT
contract.

### Best-effort drop on overflow or underflow

Rejected. Dropping whichever event happens to arrive last can lose note-offs,
leave voices stuck, and vary by producer race. A canonical retained prefix plus
an independent recovery barrier is bounded and testable.

## Validation and revisit triggers

The decision is validated when follow-up implementations demonstrate:

- #134/#132 expose a prepared current-block event buffer and engine processing
  API with no tracker document dependency and no engine-side musical-time
  conversion.
- Multiple block-size sequences produce the same absolute event frame positions,
  including events at boundaries, tempo changes, and multiple events per block.
- Tests cover the total cross-producer tie key, ordinary capacity boundaries,
  rejected note-off, recovery-latch coalescing, malformed offsets, and no RT
  allocation/deallocation.
- Clock tests cover stopped spans, invertible tempo segments, loop wraps, the
  configured crossing/segment bound, and rejection before transport when a map
  cannot fit.
- NRT tests cover exact-end coverage, partial publication, late publication,
  full-ring write rejection, consumption-cursor slot reuse, bounded horizon
  overflow, explicit resume, and stale edit/seek/clock packets.
- The tracker adapter retains its playback regressions and either uses proven
  fixed indexed callback state or NRT evaluation. A minimal synthetic/generative
  source renders without DSP-engine changes.
- Determinism tests compare semantic event values/order across restart, seek,
  save/restore, and both loop policies. Offline PCM assertions follow the
  canonical-platform qualification rather than claiming universal byte identity.
- Outbound MIDI/OSC is produced and routed on NRT with bounded admission and no
  callback I/O, or a later accepted contract explicitly adds RT-to-NRT handoff.

Named implementation owners and landing order:

- [#134](https://github.com/jpalvarezl/blight-synth/issues/134) lands the public
  ordinary/recovery event schema, semantic phase table, clock-segment timing
  rules, and engine event application first.
- [#132](https://github.com/jpalvarezl/blight-synth/issues/132) owns `prepare`
  capacities, caller-owned block buffers/descriptors, process/reset/suspend/
  resume, and the direct API surface, using #134's schema.
- Composition-adapter extraction under #145 owns the composite bounded merge,
  fixed tracker state or NRT tracker evaluation, two-cursor lookahead ring
  protocol, and synthetic second source, using #134/#132 rather than defining a
  competing recovery type.
- [#138](https://github.com/jpalvarezl/blight-synth/issues/138), under #174's
  accepted rules, owns snapshot/publication-generation handoff, exact saved
  runtime state, and NRT reclamation.

These issues share one public event/recovery surface and must follow that order
or add an explicit coordination note before parallel changes.

Revisit with a superseding ADR if a #113 prototype cannot be expressed by a
bounded current-block pull plus bounded NRT lookahead; if a required clock
cannot provide a prepared block mapping; if fail-closed recovery is unsuitable
for a proven use case; or if direct-RT outbound generation becomes a product
requirement.

## Related

- Owning issue: [#145](https://github.com/jpalvarezl/blight-synth/issues/145)
- [Event-source contract routing page](../architecture/event-source-contract.md)
- [Real-time audio contract](../architecture/realtime-contract.md)
- [Offline render contract](../architecture/offline-render-contract.md)
- [Composition domain](../domains/composition.md)
- [Audio engine domain](../domains/audio-engine.md)
- [System boundaries](../architecture/system-boundaries.md)
- [ADR 0001](0001-product-and-host-priorities.md)
- [ADR 0002](0002-device-host-osc-split.md)
