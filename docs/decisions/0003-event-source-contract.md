---
title: ADR 0003 — Event-source contract between composition, engine, and host
summary: Make the audio engine a consumer of bounded, timestamped musical/control/transport events produced by pluggable composition runtimes, with the tracker player as one adapter, so a second composition model needs no DSP-engine changes.
status: proposed
updated: 2026-07-24
issues: [145, 134, 132, 138, 113, 101, 121, 136, 174]
supersedes: []
---

# ADR 0003 — Event-source contract between composition, engine, and host

## Status

Proposed

Deciding issue: [#145](https://github.com/jpalvarezl/blight-synth/issues/145).
This ADR records a target boundary and its contracts only. It does **not**
change engine/DSP code, Cargo files, or the [real-time audio
contract](../architecture/realtime-contract.md); it depends on and refines that
contract's [control traffic classes](../architecture/realtime-contract.md) and
[prepared-state rule](../architecture/realtime-contract.md). Implementation is
scheduled follow-up work (see
[Validation and revisit triggers](#validation-and-revisit-triggers)).

## Context

The audio `Engine` today is reached only through the tracker document model.
`audio_backend/src/player/mod.rs` (`Player`) owns an `Arc<Song>`, a
`TimingState`, and a `PlayerPosition`, and in `process()` it advances ticks and
walks `Song → arrangement → Chain → Phrase → Event` to call
`note_on`/`note_off` on `TrackerEngineAdapter`
(`audio_backend/src/player/tracker_engine_adapter.rs`). That adapter wraps
`engine::Engine` and keeps tracker-only state (a `track_last_instrument`
`HashMap`). So the *only* path that drives the engine is a tracker interpreter
that hard-codes a 16×16 grid, tracker sentinels (`NoteSentinelValues`),
per-tick effect slots, and a `TimingState` tempo model
(`sequencer/src/timing/mod.rs`).

This blocks two accepted directions:

- [ADR 0001](0001-product-and-host-priorities.md) keeps the composition
  interaction open: the tracker is "one supported event source", and an
  ORCA-like character grid, hybrid, or generative model may become the product
  after prototype-driven evaluation ([#113](https://github.com/jpalvarezl/blight-synth/issues/113)).
  A second runtime must not require DSP/mixer/routing/device changes
  ([composition domain](../domains/composition.md), "Key invariant").
- The [real-time audio contract](../architecture/realtime-contract.md) already
  reserves a distinct traffic class — "Timestamped musical/control events —
  ordered and bounded" — owned by
  [#134](https://github.com/jpalvarezl/blight-synth/issues/134)/#145, separate
  from coalesced continuous parameters (#101/#121) and structural prepared-state
  updates (#174/#138). Its violation inventory names the tracker active-instrument
  `HashMap` and the note/row logging as concerns this contract must resolve
  structurally, and the "structural command work mixes notes, parameters, and
  graph updates" row is explicitly deferred to this separation.

The tension: without a recorded event-source boundary, every new composition
model either forks the engine or grows more tracker-shaped coupling, and the
timestamped-event traffic class stays theoretical. This ADR names the boundary,
the engine's event-consumer contract, and how generative/clock/randomness
concerns sit on the correct side of it.

## Decision

Adopt a three-role boundary in which the engine consumes bounded, timestamped
events and never interprets a composition document. Rendering direction follows
[system boundaries](../architecture/system-boundaries.md): `document + clock →
composition runtime → timestamped events → engine → DSP → audio`.

### 1. Three roles and their ownership

**Audio `Engine` (event consumer).** Owns instruments, voices, routing,
parameters, transport clock conversion (musical time → sample offsets), and
rendering over caller-provided buffers. It consumes a bounded slice of
timestamped events plus a block descriptor and produces audio and telemetry. It
has **no** dependency on `Song`, `Chain`, `Phrase`, tracker `Event`, sentinels,
or any UI/document type (acceptance criterion in #145).

**Composition runtime / event source (event producer).** Turns a *versioned
composition document* plus *clock and input state* into timestamped
note/control/transport events for a requested time window. It owns document
semantics, interpreter/runtime state, seeded randomness, replay, live-edit
snapshots, and composition-specific migrations
([composition domain](../domains/composition.md), "Owns"). It performs **no**
audio-device, socket, or filesystem I/O and never mutates the audio-thread
object graph directly.

**Host / control layer.** Owns live edits, external MIDI/OSC I/O, filesystem,
device lifecycle, and process wiring. It selects the clock source, delivers
prepared document snapshots to the runtime, and routes the runtime's abstract
outbound events to real MIDI/OSC transports off the audio thread. This is the
same typed control boundary described in
[ADR 0002](0002-device-host-osc-split.md); OSC/MIDI are transports over it, not
owners of event semantics.

### 2. Engine event-consumer contract

The engine consumes an ordered, fixed-capacity buffer of events scoped to the
current render block. This realizes traffic class 2 of the
[real-time audio contract](../architecture/realtime-contract.md).

- **Event kinds.** `Note` (on/off/choke with stable target instrument/voice
  identity and velocity), `Control` (sample-accurate automation keyed by a
  stable parameter ID from the #121 manifest), and `Transport`
  (start/stop/continue, tempo/position, loop boundary). Continuous
  latest-value-wins knob traffic stays on the coalesced parameter path
  (#101/#121), *not* here; `Control` events are the sample-accurate automation
  variant that must land at a specific offset.
- **Timestamp.** Each event carries a `sample_offset` within the current block,
  in `[0, block_len)`. The engine applies each event at its offset while
  rendering; it does not itself interpret musical/tempo units except through the
  `Transport` events and its own clock-conversion.
- **Deterministic ordering.** Events are delivered sorted by `sample_offset`.
  Events sharing an offset carry a stable, producer-assigned sequence order that
  the engine preserves exactly (Hard Rule 8, "no randomized execution order").
  Two runs with the same document, seed, and clock produce byte-identical event
  streams and therefore identical renders (offline-render determinism per
  [offline render contract](../architecture/offline-render-contract.md)).
- **Fixed capacity + explicit overflow.** The per-block event buffer has a
  configured maximum established at `prepare` (#132). Overflow has an explicit,
  producer-visible, deterministic policy: excess events are **rejected at the
  producer boundary** (not silently dropped or reordered inside RT), a bounded
  overflow counter/status is incremented (strict builds), and a defined recovery
  path — including all-notes-off/transport-stop — remains reachable. Overflow
  never reorders accepted events and never allocates on RT.
- **No allocation / bounded work.** The engine reads the event slice, applies a
  bounded number of events per block, and renders. Applying an event may not
  allocate, log (outside the compile-gated developer wrapper), or run
  parsing/factory/migration work; all such work is prepared on NRT
  (prepared-state rule).

### 3. Pull-based generation with bounded lookahead

The engine **pulls** events; it does not accept an unbounded push stream. Before
(or at the start of) a block, the host/scheduler asks the active event source to
*fill* the block's event buffer for the window `[now, now + block_len)`, plus a
bounded lookahead horizon for scheduling headroom. Rationale:

- Pull gives the RT side a hard, per-block work bound and a natural place to
  apply the fixed-capacity/overflow policy, instead of an external producer
  racing the callback with unbounded volume.
- Bounded lookahead lets a runtime that must think ahead (chords, ratcheting,
  generative decisions) do so within a declared horizon without unbounded
  buffering or unbounded RT work.
- The tracker's current pattern already matches pull: `TimingState::advance`
  computes how many ticks fall in a block and emits their notes. This ADR
  generalizes that into "fill the event buffer for this window".

The *evaluation* that produces events may still run on NRT (see §5); "pull"
describes the demand-driven contract at the block boundary, not the thread the
evaluator runs on.

### 4. Clock sources

The event source is parameterized by a clock; the host selects it. Four sources
are in scope, all reduced to the same `Transport`-event + sample-offset contract
before reaching the engine:

- **Internal clock.** Free-running tempo/transport owned by the host
  (today's `TimingState` role), driving pull windows from the sample clock.
- **Host clock.** A plugin/embedding host supplies transport/tempo/position;
  the adapter maps it into the same `Transport` events. This is where optional
  plugin host-sync (deferred in ADR 0001) attaches.
- **MIDI clock.** External MIDI clock/MTC arrives on the host/control layer,
  is smoothed/estimated on NRT, and is delivered as `Transport` events — never
  read via RT I/O.
- **External clock.** Other sync sources (e.g. OSC transport, link-style) are
  treated like MIDI clock: host-side ingestion, NRT estimation, engine sees
  only `Transport` events.

The engine's transport clock conversion consumes `Transport` events; it does not
know which physical source produced them.

### 5. Fixed-memory RT evaluation vs deterministic NRT lookahead

Composition runtimes fall on a spectrum, and the contract supports both without
changing the engine:

- **Fixed-memory RT evaluation.** A runtime whose per-window evaluation is
  provably bounded in memory and work (e.g. the tracker: read a row, emit
  notes) *may* fill the event buffer directly on the RT/callback path, subject
  to all hard callback rules. No allocation, no unbounded loops, fixed capacity.
- **Deterministic NRT lookahead.** A runtime whose evaluation cannot be bounded
  on RT (arbitrary generative programs, graph rewriting, ORCA-like propagation)
  runs its evaluation on NRT ahead of playback, producing a bounded, prepared
  buffer of timestamped events for upcoming windows. RT consumes that prepared
  buffer under the same event contract; the evaluator never runs on RT. This is
  the prepared-state rule applied to event *generation*.

The engine cannot tell which strategy produced its event buffer — that is the
point. The choice is per-runtime and is a property of the adapter, not of the
engine contract.

### 6. Seeded randomness and determinism

Any nondeterminism in a runtime is driven by an explicit seed that is part of
the versioned composition document / runtime snapshot state (#138). Given the
same document, seed, and clock/input trace, a runtime produces an identical
event stream, so offline renders are reproducible
([offline render contract](../architecture/offline-render-contract.md)) and
state save/restore is exact. RNG state advances deterministically with musical
position so that seek/loop/restart land on a defined stream position rather than
wall-clock entropy. The engine itself contains no RNG that affects event
application order (Hard Rule 8).

### 7. Live edits, snapshots, and non-audio side effects

- **Live edits** replace the runtime's document/program through immutable
  prepared snapshots/revisions installed via the structural traffic class
  (#174/#138): NRT prepares the new snapshot, RT swaps at a safe boundary, and
  the displaced snapshot is retired to NRT for destruction — never dropped on
  RT. This matches how `Player::set_song` already retires the displaced
  `Arc<Song>` as `RetiredState::Prepared`.
- **Infinite/generative behavior under seek/loop/save/restart** is defined by
  the runtime's snapshot + seeded RNG state, not by engine state, so these
  operations are runtime concerns expressed through prepared snapshots and the
  clock, leaving the engine contract unchanged.
- **Non-audio side effects** (outbound MIDI/OSC) are emitted by the runtime as
  *abstract outbound events* and routed by the host to real transports off the
  audio thread. The audio thread performs no MIDI/OSC I/O
  ([composition domain](../domains/composition.md), "Abstract outbound events").

### 8. The tracker player is one adapter

The current `Song → Chain → Phrase → Player` interpreter becomes **one**
event-source adapter implementing the producer contract above. `Player` +
`TrackerEngineAdapter`'s tracker-only state (`PlayerPosition`,
`track_last_instrument`, tracker sentinels, per-tick effect slots) moves behind
the adapter; the engine keeps only generic instrument/voice/rendering. Its
existing deterministic playback tests remain the tracker adapter's regression
fixture (#145 acceptance criterion), and a minimal synthetic/generative event
source proves a second model needs no DSP-engine changes.

### Non-goals

- No engine/DSP/Cargo change and no code refactor in this ADR's branch; this is
  the target the follow-ups implement.
- No selection of the final composition UI/language — that is
  [#113](https://github.com/jpalvarezl/blight-synth/issues/113).
- No new synthesizer/player; the engine keeps delegating rendering to `engine`
  per [ADR 0001](0001-product-and-host-priorities.md).
- No change to the coalesced continuous-parameter path (#101/#121) or the
  structural reclamation path (#174/#138); this ADR consumes them.

## Consequences

### Positive

- A second composition model (ORCA-like/generative) can drive the engine with
  zero DSP/mixer/routing/device changes, satisfying the composition domain's key
  invariant and ADR 0001's openness.
- The timestamped-event traffic class from the real-time contract gains a
  concrete producer/consumer contract, and the tracker `HashMap`/logging
  inventory rows get a structural target.
- Determinism is a first-class, seed-driven property, so offline golden renders
  and state restore remain exact across runtimes.
- RT work is bounded by construction: pull + fixed capacity + explicit overflow,
  with generative cost pushed to NRT lookahead.
- Clock sources unify behind `Transport` events, isolating plugin/MIDI/external
  sync in host adapters.

### Costs and risks

- Follow-up implementation must actually extract the tracker interpreter into an
  adapter and define the concrete event types/capacities; until then the tracker
  path remains the only producer and the inventory rows stay open.
- A bounded lookahead horizon plus NRT event prebuffering adds scheduling
  latency and buffering state the current direct-tick path lacks; the horizon
  must be tuned against overflow behavior.
- Deterministic NRT lookahead for open-ended generative programs constrains what
  such programs may express (they must be evaluable ahead of a bounded window),
  which the #113 spikes must validate.
- Two producer strategies (RT-eval vs NRT-lookahead) increase the number of
  paths to test for allocation-safety and determinism.
- Overflow at the producer boundary means a pathological runtime can drop its
  own excess events; the recovery/all-notes-off path must be proven reachable.

## Alternatives considered

### Keep the engine driven by the tracker document model

Rejected: it hard-couples the only engine entry path to a 16×16 grid and tracker
sentinels, contradicts ADR 0001's open interaction model and the composition
domain's key invariant, and leaves the timestamped-event traffic class
unrealized. Every new model would fork the engine.

### Push-based event stream from an external producer

Rejected as the primary contract: an unbounded external push races the callback,
has no natural per-block work bound, and makes fixed-capacity/overflow policy
awkward. Pull gives RT a hard bound and a single place to apply overflow. (A
host may still *feed* NRT-prepared buffers that RT pulls from — that is §5, not
an unbounded push.)

### Evaluate all runtimes directly on RT

Rejected: arbitrary generative/graph-rewriting programs cannot be bounded on the
callback (Hard Rules 1/6/7). Forcing RT evaluation would either forbid such
runtimes or reintroduce allocation/unbounded work. NRT lookahead keeps the
engine contract identical while allowing unbounded-shape evaluators off RT.

### One undifferentiated command/event queue

Rejected: it conflates continuous parameters, timestamped events, and structural
updates with different overload semantics — exactly the conflation the real-time
contract's traffic classes forbid. Notes need ordered bounded delivery; knobs
need latest-value-wins; snapshots need reliable reclamation.

### Engine owns tempo/clock and RNG

Rejected: musical-time interpretation and nondeterminism belong to the
composition runtime so that seek/loop/save/restore and reproducibility are
document/snapshot properties. The engine only converts `Transport` events to
sample offsets and must have no order-affecting RNG (Hard Rule 8).

## Validation and revisit triggers

The decision is validated when the follow-up implementation lands and:

- The engine's public processing path takes a bounded timestamped-event buffer
  plus a block descriptor and has **no** dependency on `Song`/`Chain`/`Phrase`/
  tracker `Event`/sentinel types (#134 + #145 acceptance criteria; a
  `cargo tree`/type-visibility check on the engine crate proves the absence).
- Events carry `sample_offset` and stable target/parameter identities from the
  #121 parameter manifest, with deterministic same-offset ordering and a
  fixed-capacity, explicit, producer-visible overflow policy exercised by tests.
- The tracker interpreter is reachable only as one adapter and retains its
  deterministic playback tests, and a minimal synthetic/generative event source
  renders correctly without DSP-engine changes.
- Seeded runs reproduce identical event streams and offline renders across
  seek/loop/save/restore, per the offline render contract.
- Abstract outbound MIDI/OSC events are routed by the host with no RT I/O.

Named follow-up implementation issues:

- [#134](https://github.com/jpalvarezl/blight-synth/issues/134) — sample-accurate
  timestamped-event scheduling (the concrete event types, capacities, ordering,
  and overflow behavior).
- [#132](https://github.com/jpalvarezl/blight-synth/issues/132) — engine
  lifecycle (`prepare` establishes event-buffer capacity/block layout before
  `process`).
- Composition-adapter extraction — move the tracker `Player`/`TrackerEngineAdapter`
  interpreter behind the event-source contract and add the minimal
  synthetic/generative event source (tracked under the composition domain and
  #145 follow-up; coordinated with
  [#138](https://github.com/jpalvarezl/blight-synth/issues/138) snapshots).

Revisit with a superseding ADR if: the #113 spikes show a viable runtime that
cannot be expressed as a pull-based, bounded-lookahead, seed-deterministic event
source; plugin host-sync requires the engine to own musical time; or the
timestamped-event contract cannot be reconciled with the #121/#101 parameter
paths without transport-specific semantics.

## Related

- Owning issue: [#145](https://github.com/jpalvarezl/blight-synth/issues/145)
- Follow-up implementation: [#134](https://github.com/jpalvarezl/blight-synth/issues/134),
  [#132](https://github.com/jpalvarezl/blight-synth/issues/132), composition-adapter extraction
- [ADR 0001](0001-product-and-host-priorities.md),
  [ADR 0002](0002-device-host-osc-split.md)
- [Event-source contract (draft)](../architecture/event-source-contract.md)
- [Composition domain](../domains/composition.md),
  [Audio engine domain](../domains/audio-engine.md)
- [System boundaries](../architecture/system-boundaries.md),
  [Real-time audio contract](../architecture/realtime-contract.md),
  [Offline render contract](../architecture/offline-render-contract.md)
