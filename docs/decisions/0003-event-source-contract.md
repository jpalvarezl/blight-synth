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
This ADR records a target boundary and its hard constraints. It does **not**
change engine/DSP code, Cargo files, or the accepted
[real-time audio contract](../architecture/realtime-contract.md). Implementation
belongs to #134, #132, #138, and the composition-adapter extraction under #145.

## Scope of this decision

This decision fixes ownership, dependency direction, and the constraints that
all implementations must satisfy. It intentionally does not freeze queue
protocols, packet layouts, ordering-key fields, RNG reconstruction algorithms,
or concrete Rust APIs. Those mechanisms and the named open questions below are
delegated to the implementing issues, especially #134 and #132.

Implementation may refine or replace a mechanism suggested here when evidence
shows a better approach, provided it preserves the decision and hard
constraints. A change to those boundaries or constraints must supersede or amend
the relevant part of this ADR through the normal ADR process. This latitude is
intentional: a Proposed design must not pre-empt what sample-accurate scheduling
and engine-lifecycle implementation teach us.

## Context

The current first-party tracker/song playback path reaches `engine::Engine`
through `audio_backend::player::Player` and `TrackerEngineAdapter`. `Player`
owns an `Arc<Song>`, `TimingState`, and `PlayerPosition`. For each callback,
`TimingState::advance` returns only the number of elapsed ticks; `Player` walks
`Song → arrangement → Chain → Phrase → Event`, invokes note methods, and then
renders the whole block. It has the useful shape of demand-driven block
evaluation, but it does **not** compute sample offsets and is not a
sample-accurate implementation of this contract.

The tracker path is not the only way to drive the engine. `Engine` exposes
public note, command, and `process` methods used directly by
`engine/examples/offline_render.rs` and tests. The coupling problem is narrower:
the current first-party *song/composition* path interprets one tracker document
model directly on the callback instead of crossing a generic event boundary.

`TrackerEngineAdapter` already owns tracker-only `track_last_instrument` state.
It constructs that `HashMap` with `HashMap::with_capacity(MAX_TRACKS)`, and the
current caller inserts at most `MAX_TRACKS` distinct keys (all in
`0..MAX_TRACKS`); Rust guarantees the table can hold that many entries without
reallocating. The current path therefore does **not** demonstrate a Hard-Rule-1
allocation violation from these inserts. The gap is structural: the
bound is implicit in the caller and collection usage rather than enforced by
the state representation or its type. Direct-RT tracker evaluation must harden
that invariant and prove its other work bounds; this is preventative structural
work, not remediation of an observed allocation.

The boundary must support the existing tracker and future ORCA-like, hybrid, or
generative models without making the engine understand any composition
document. It must also honor traffic class 2 of the
[real-time audio contract](../architecture/realtime-contract.md): timestamped
events have bounded work, deterministic ordering and explicit overload behavior,
and remain separate from latest-value controls and prepared structural swaps.

## Decision

Adopt a host-orchestrated, demand-driven boundary:

```text
host-selected clock/input adapters + versioned document
  -> composition runtime and host-owned composite scheduler
  -> bounded current-block events with sample offsets
  -> Engine event application + DSP rendering
```

The engine never interprets a composition document and never converts musical
or external clock time into sample offsets.

### 1. Roles and clock authority

**Host/control and clock adapters.** The host owns device/plugin callback
wiring, clock-source selection, filesystem and MIDI/OSC I/O, live-edit
submission, and side-effect routing. Its selected clock adapter is the sole
producer-side authority for mapping clock/musical positions to render frames.
It supplies a bounded prepared mapping for the current block. When NRT
lookahead is used, that same adapter also owns provision of a bounded, prepared
future-clock window or horizon on which future timestamps can reliably be
based.

If a selected clock cannot provide a reliable future mapping, the runtime falls
back to the current-block pull path; future lookahead must not extrapolate an
unreliable mapping. A runtime may execute in that path only when its callback
state and worst-case work satisfy the RT contract. Otherwise that runtime/clock
combination cannot run until an accepted implementation provides a compliant
strategy. Exact future-window preparation and ownership mechanics are an open
implementation question.

A clock-source change or discontinuous remap starts a new clock epoch. Clock
adapters may represent internal, plugin-host, MIDI, or external clocks, but
clock estimation and external I/O remain NRT responsibilities.

**Composition runtime/event source.** The runtime owns document semantics,
interpreter state, seeded randomness, and composition-specific replay. Using
the selected clock mapping, it produces semantic events at absolute render
frames or current-block sample offsets. It does not own audio devices, sockets,
filesystems, or the engine object graph.

**Composite event source/scheduler.** One host-owned composite scheduler
combines the active composition runtime, live input, transport, and
sample-accurate control producers. It is the final ordering authority before the
engine and exposes one bounded current-block event slice.

**Audio `Engine`.** The engine owns instruments, voices, routing, render-time
transport state, event application, DSP rendering, and telemetry. It receives a
block descriptor and events whose offsets are already in the current block. It
does not receive musical timestamps, select a clock, estimate tempo, or inspect
composition/publication generations. Stopping composition transport does not by
itself prevent effect or release tails from rendering; lifecycle details remain
with #132.

### 2. Current-block pull and optional lookahead

“Pull” means the host scheduler requests events for the current half-open render
block before calling the engine. It does not mean that `Engine` calls arbitrary
composition code. The concrete API is owned by #134/#132 and must satisfy these
constraints:

- The engine-facing slice contains only events for the current block, with every
  `sample_offset` in `[0, frame_count)`. An event at the block end belongs to the
  next block.
- Event storage, work, producer/input counts, and callback-visible status are
  bounded and prepared. Callback filling, ordering, and application obey all
  hard RT rules, including no allocation, blocking, I/O, logging, or unbounded
  retry.
- A runtime may evaluate directly during current-block pull only when its state
  and worst-case work are demonstrably fixed and bounded. Other runtimes may use
  NRT lookahead behind the same engine-facing pull boundary.
- Lookahead uses separate bounded future storage. Future events do not enter the
  engine-facing slice until their current block is pulled and their offsets are
  known.
- Lookahead records are associated with the relevant clock epoch and active
  document/publication generation so stale work cannot cross a clock,
  seek/reset, or document boundary.
- The RT↔NRT handoff is bounded, non-allocating, nonblocking, and cannot expose
  torn, overwritten, or partially declared event coverage. RT must be able to
  determine, with bounded work, whether the current block is complete.
- Missing or over-capacity lookahead follows deterministic, observable,
  fail-closed behavior. It never waits, consumes a partial block, or silently
  applies late events, and note/transport recovery remains reachable even when
  ordinary event capacity is exhausted.

These are protocol constraints, not a mandate for a particular ring, cursor,
watermark, memory ordering, or resumption sequence.

### 3. Event semantics, ordering, and overload

The engine event lane contains already-offset musical and render-control facts,
not composition instructions:

- `Note` operations such as note-on, note-off, choke, or release, with stable
  target identity;
- sample-accurate `Control` changes keyed by stable target/parameter identity;
  latest-value continuous controls remain traffic class 1; and
- already-offset `Transport` state transitions, not requests to perform clock
  conversion.

For every accepted block, ordering is deterministic and total, including events
from different producers at the same sample offset. It must not depend on hash
iteration, callback arrival races, thread scheduling, or incidental runtime
registration order. The schema must define any semantic precedence needed to
avoid ambiguous results such as same-offset release/attack behavior.

Ordinary event capacity is fixed. Overflow and malformed input are observable
and deterministic, never silently reorder accepted events, and fail closed.
Transport-stop/all-notes-off recovery must remain representable when ordinary
capacity is full. Initial recovery may be engine-global because voices do not
yet carry producer ownership; a narrower scope requires an accepted ownership
model.

The exact total-order key, producer identity/sequence model, bounded merge or
sort strategy, admission rule, and recovery signaling mechanism are deliberately
left to #134/#132 and adapter implementation.

### 4. Seek, reset, and discontinuity boundary

A seek, reset, or clock discontinuity takes effect only at sample offset zero of
the next render block. It installs a new active generation (and a new clock
epoch when the mapping changes). A request arriving during a block does not
alter that block's mapping or events, and there is no alternative
“anywhere-in-block seek barrier.” Work tagged for the old generation is not
accepted after the boundary.

Document revisions and other invalidating runtime-state changes use the same
block-boundary generation rule. Prepared reconciliation may preserve selected
notes, but the safe default is bounded stop/all-notes-off recovery. Ownership
replaced at the boundary follows the accepted structural swap-and-retire
contract and is reclaimed on NRT.

Ordinary, already-scheduled transport events that are not a seek, reset, or
clock discontinuity may still have in-block offsets when their semantics allow
it. #132/#134 own the concrete lifecycle/API expression of this rule.

### 5. Fixed-memory evaluation and tracker migration

The producer strategy is selected per runtime:

- A fixed-memory runtime may evaluate in current-block pull when its prepared
  immutable view, mutable state, event capacity, and worst-case work satisfy the
  RT contract.
- An arbitrary generative, graph-rewriting, or otherwise unbounded runtime runs
  on NRT and uses bounded lookahead when its selected clock can provide the
  required future mapping.

The current tracker becomes one event-source adapter. Its existing
`track_last_instrument` map is preallocated for the current path's at most
`MAX_TRACKS` distinct keys, so this ADR does not label it an observed allocation
violation. Before the
adapter is classified as direct RT, however, the implementation must make the
track bound structural (fixed indexed state is one possible implementation) and
prove all tick/event work limits. Otherwise tracker interpretation runs on NRT.

`TimingState::advance` returns only an elapsed tick count. It is evidence for
demand-driven evaluation, not sample-accurate timing. #134 must preserve the
timing information needed to place each event at its absolute frame/current
block offset rather than triggering all elapsed ticks before whole-block
rendering.

The event API becomes the first-party tracker/song processing path after
#132/#134. Public direct engine note/command/process APIs may remain for
instrument audition, embeddings, examples, and tests.

### 6. Determinism across replay and rendering

A runtime that promises deterministic replay must reproduce the same semantic
event values and total order from the same versioned document/runtime, declared
seed or saved state, clock/block/input trace, and loop/transport context. Seek,
loop, restart, and save/restore must preserve enough versioned interpreter and
random state to honor that promise. No promised replay path may depend on wall
clock, OS entropy, hash iteration order, or thread timing.

The contract does not choose position-addressed randomness, checkpoint replay,
or another reconstruction design. It also does not prescribe the exact saved
interpreter cursor, checkpoint cadence, or repeat/evolve loop representation.
Those choices are composition-runtime semantics and must be versioned and tested
when implemented.

This is **semantic event-stream determinism**. It is not byte identity unless a
canonical event serialization is separately defined.

Render determinism is separate. Given equal semantic events, the same engine/DSP
version and configuration must follow the
[offline render contract](../architecture/offline-render-contract.md). Exact
PCM/reference equality is required only on that contract's canonical platform;
other platforms follow its repeated-render, structure, clipping, and metric
policy unless they gain a separately reviewed platform hash.

### 7. Outbound MIDI/OSC side effects

Abstract outbound MIDI/OSC is not an engine event kind and never causes I/O on
the audio thread. Under this ADR, a direct-RT evaluator is prohibited from
producing outbound side effects. A runtime that emits them evaluates that output
on NRT and submits abstract timestamped messages to a host-owned, bounded NRT
scheduler. Admission and failure are deterministic and observable, and recovery
must not leave destination notes stuck; actual I/O remains with the host on NRT.

A future requirement to derive outbound messages on RT needs an accepted
additive or superseding contract for a bounded RT-to-NRT handoff. It is not
permitted implicitly by this ADR.

## Named open questions for implementation

These are intentional design latitude, not omissions to fill in this ADR:

1. **Future-clock window preparation and ownership (#134/#132).** What bounded
   horizon representation does each clock adapter prepare, who owns its storage,
   and how is reliability declared without duplicating clock authority?
2. **RT↔NRT lookahead handoff (#145/#132).** Does implementation use SPSC storage
   or another bounded handoff; what packet/coverage representation, memory
   ordering, full-capacity response, recovery timing, and resume rule satisfy
   the constraints above?
3. **Deterministic total order (#134).** What stable producer identity and
   source-local sequence scheme, semantic precedence, and merge/admission key
   produce a total order without depending on runtime registration accidents?
4. **Seek/loop state reconstruction (#138 and runtime adapters).** Which
   position-addressed, checkpoint, or other versioned RNG/interpreter-state
   model reproduces promised behavior, and what exact loop context is persisted?
5. **Concrete process/lifecycle surface (#132/#134).** What Rust types and
   prepare/process/reset/suspend/resume operations express current-block pull,
   bounded capacities, status, and recovery without creating a competing event
   schema?
6. **Recovery publication and scope (#134/#132).** What bounded representation
   keeps recovery available at ordinary capacity, and when can producer-owned
   voices justify narrower-than-global recovery?

## Non-goals

- No engine/DSP/Cargo change or tracker extraction in this documentation PR.
- No selection of the final composition UI/language; that remains #113.
- No change to latest-value continuous parameters (#101/#121) or structural
  prepared-state reclamation (#174/#138).
- No canonical binary serialization for semantic events.
- No guarantee of byte-identical PCM across platforms beyond the offline render
  contract's platform policy.

## Consequences

### Positive

- Clock ownership is singular: host-selected clock adapters define producer-side
  mappings, while the engine only applies already-offset events.
- Future lookahead has an explicit mapping owner and a safe fallback when that
  mapping is unavailable.
- Seek/reset/discontinuity has one block-boundary generation rule.
- Current-block work and optional NRT scheduling share one engine boundary while
  retaining bounded, non-allocating RT behavior.
- Ordering, overload, recovery, replay, and side effects have hard constraints
  without freezing premature queue or state-reconstruction mechanisms.
- The tracker claim now matches current code: its map capacity is sufficient for
  the current caller, while structural enforcement remains follow-up work.
- A second composition model can drive the engine without changing DSP, mixer,
  routing, or device code.

### Costs and risks

- The host coordinates clock selection, block-boundary generations, and the
  composite scheduler instead of delegating musical-time conversion to the
  engine.
- Some runtimes cannot operate with clocks that lack reliable future mapping
  unless they can satisfy direct current-block RT bounds.
- Fail-closed overflow/underflow can audibly stop playback and global recovery
  can release notes from a producer that did not fail.
- Important mechanism choices remain for implementation and require focused
  evidence and coordination across #134, #132, #138, and adapter work.
- Prohibiting outbound generation from direct RT evaluators may require an NRT
  runtime even when its audio-event evaluator could otherwise be RT-safe.

## Alternatives considered

### Engine owns tempo/clock conversion

Rejected. It duplicates authority with composition/clock adapters, forces the
engine to understand musical and external clock domains, and prevents a clean
already-offset event boundary.

### Lookahead extrapolates any selected clock

Rejected. A future timestamp is only meaningful when the selected clock adapter
can prepare a reliable future mapping. Current-block pull is the fallback;
lookahead may not invent one.

### Seek can take effect anywhere in the current block

Rejected for this contract. Combining arbitrary in-block seek barriers with
block-boundary generation swaps creates two conflicting activation rules. One
next-block, offset-zero generation boundary is simpler and deterministic.

### Put current and future events in one engine-facing buffer

Rejected. Future events do not yet have current-block offsets and need separate
bounded storage and stale-generation handling. They enter the engine-facing
slice only when their block is pulled.

### Unbounded push stream or evaluate every runtime on RT

Rejected. Neither provides bounded callback work. Arbitrary generative programs
must use compliant NRT preparation/lookahead, while direct RT evaluation is an
earned property of a concrete adapter.

### One undifferentiated command/event queue

Rejected. Continuous values, timestamped events, and prepared structural swaps
have different overload and reclamation semantics under the accepted RT
contract.

### Best-effort drop on overflow or underflow

Rejected. It can lose note-offs, leave voices stuck, silently reorder musical
results, and make behavior depend on producer races. Failure must be observable,
deterministic, and recoverable.

## Validation and revisit triggers

The decision is validated when follow-up implementations demonstrate:

- #134/#132 provide a bounded current-block event/process path with no tracker
  document dependency and no engine-side musical-time conversion.
- Event placement is sample-accurate across block boundaries and block-size
  sequences, with deterministic total same-offset ordering.
- Callback work and RT↔NRT handoff satisfy the accepted RT contract under normal,
  full-capacity, stale-generation, and missed-deadline cases, with recovery
  always reachable.
- Seek/reset/discontinuity activates only at the next block's offset zero and no
  old-generation event crosses that boundary.
- The tracker retains playback regressions and uses structurally bounded RT
  state or NRT evaluation; a minimal second source requires no DSP-engine
  changes.
- Promised replay is reproducible across restart, seek, loop, and save/restore;
  offline PCM assertions retain their platform qualification.
- Outbound MIDI/OSC remains bounded and NRT-routed with no callback I/O.

Mechanism-level validation belongs in the implementing issues. Their results may
refine or replace the open mechanisms above; changes to this ADR's boundary or
hard constraints require the normal ADR amendment/supersession process.

Revisit the decision if a #113 prototype cannot be expressed by bounded
current-block pull plus optional bounded lookahead; if required clocks and
runtimes have no compliant current-block or future-mapping strategy; if
fail-closed recovery is unsuitable for a proven use case; or if direct-RT
outbound generation becomes a product requirement.

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
