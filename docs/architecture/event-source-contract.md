---
title: Event-Source Contract (draft)
summary: Routing page for the host-orchestrated composition→engine boundary defined in ADR 0003.
status: draft
updated: 2026-08-01
issues: [145, 134, 132, 138, 113, 201, 202, 203, 204]
---

# Event-Source Contract (draft)

This page routes readers to the boundary proposed in
[ADR 0003](../decisions/0003-event-source-contract.md). The ADR is authoritative;
the API is not implemented yet. Owning issue:
[#145](https://github.com/jpalvarezl/blight-synth/issues/145).

## Read first

- [ADR 0003 — Event-source contract](../decisions/0003-event-source-contract.md)
- [Real-time audio contract](realtime-contract.md) — callback constraints and
  timestamped-event traffic class
- [Offline render contract](offline-render-contract.md) — determinism and
  platform policy
- [System boundaries](system-boundaries.md) — dependency direction
- [Composition domain](../domains/composition.md) and
  [Audio engine domain](../domains/audio-engine.md)

## Decision boundary

```text
host-selected clock/input adapters + versioned document
  -> composition runtime and host-owned composite scheduler
  -> bounded current-block events with sample offsets
  -> Engine applies events and renders DSP
```

The selected clock adapter is the sole producer-side clock-mapping authority.
It prepares the bounded current-block mapping and, when lookahead is used, a
reliable bounded future-clock window. If it cannot reliably map the future, the
runtime falls back to current-block pull; lookahead does not extrapolate an
unreliable clock. The exact future-window storage and ownership are deliberately
left to #134/#132.

The engine consumes already-offset events. It does not select a clock, convert
musical/external time, or interpret tracker/generative documents. The current
tracker/song route is the first-party composition path being adapted, not the
engine's only caller; direct APIs may remain for audition, embeddings, examples,
and tests.

## Roles

| Role | Owns | Must not own |
|---|---|---|
| Host/control + selected clock adapter | clock selection/mapping, block windows and epochs, live-edit submission, MIDI/OSC I/O, filesystem, side-effect routing | composition semantics or engine DSP |
| Composition runtime/child producer | versioned document semantics, interpreter/RNG state, semantic event generation | devices, sockets, filesystem, engine graph mutation |
| Composite scheduler | one bounded current-block output and deterministic total cross-producer ordering | DSP rendering or unbounded callback work |
| Audio `Engine` | already-offset event application, instruments, voices, routing, render transport state, DSP, telemetry | clock conversion or composition/UI types |

## Hard constraints

- The engine-facing slice contains only the current half-open block; every event
  has a `sample_offset` in `[0, frame_count)`.
- Storage, event/input/producer counts, and callback work are prepared and
  bounded. Callback code obeys the accepted RT rules.
- Fixed-memory runtimes may evaluate during current-block pull only after proving
  state and work bounds. Others use bounded NRT lookahead when the selected
  clock can provide a reliable future mapping.
- RT↔NRT handoff is bounded, non-allocating, nonblocking, stale-generation safe,
  and never exposes torn, overwritten, or partially declared block coverage.
- Same-offset ordering is deterministic and total. It cannot depend on hash
  order, races, thread scheduling, or incidental registration order.
- Overflow/deadline failure is deterministic, observable, and fail-closed;
  stop/all-notes-off recovery remains reachable when ordinary capacity is full.
- A seek, reset, or clock discontinuity takes effect only at offset zero of the
  next block with a new generation (and a new clock epoch when mapping changes).
  There is no anywhere-in-block seek alternative.
- Replaced snapshots/publication state use the accepted NRT
  swap-and-retire/reclamation path.

## Determinism and side effects

Promised semantic replay must reproduce equal event values and total order for
the same versioned runtime/document, declared seed or saved state,
clock/block/input trace, and loop/transport context. The concrete RNG,
checkpoint, loop-state, and interpreter reconstruction design is intentionally
open. Semantic equality is not byte identity without a canonical event
serialization.

Render determinism is separate. Exact PCM follows the
[offline render contract](offline-render-contract.md): exact references on the
canonical platform and that contract's repeated-render/metric policy elsewhere.

Abstract outbound MIDI/OSC is not an engine event. Direct-RT evaluators are
prohibited from producing outbound side effects under this ADR. Such output is
bounded, scheduled, and performed on NRT by the host. Any future RT-to-NRT
side-effect handoff requires an accepted additive or superseding contract.

## Tracker migration truth

`TrackerEngineAdapter` constructs `track_last_instrument` with capacity
`MAX_TRACKS`, and the current caller inserts at most `MAX_TRACKS` distinct keys
(all in `0..MAX_TRACKS`). Rust guarantees that prepared capacity can hold those
entries without reallocating, so the inserts do not currently demonstrate a
Hard-Rule-1 allocation. The gap is that this bound is implicit rather than
structurally or type-enforced.
Direct RT tracker evaluation must harden the bound and prove tick/event work;
otherwise it uses NRT evaluation.

`TimingState::advance` returns only elapsed tick count. It demonstrates
demand-driven block evaluation, not sample-accurate event placement; #134 must
retain the timing needed to calculate each event's absolute frame/current-block
offset.

## Implemented engine-facing slice (#201)

Issue #201 implements the canonical current-block event/application surface in
`engine/src/events.rs` and `Engine::process_with_events`:

- One render block is the exact common planar-buffer prefix passed to one engine
  process call. Offsets use the half-open `0..frame_count` interval; a boundary
  at `frame_count` belongs to the next block.
- `TimestampedEvent` wraps an engine-ready payload with a stable producer ID and
  source-local sequence. `EventOrderKey` is the shared canonical key for #203:
  sample offset, semantic precedence, producer ID, then sequence.
- Same-offset semantic precedence is global recovery, targeted release,
  sample-accurate parameter, then attack. This makes recovery/release happen
  first and lets a new attack observe a parameter changed at the same sample.
- The complete slice is validated before rendering or state mutation. Invalid
  offsets, non-increasing keys, and non-finite/out-of-range prepared parameter
  values return a compact `EventProcessError`; the engine never silently sorts
  or partially applies malformed input. Live hosts
  must inspect that result: because buffers remain untouched, they should render
  current voices/tails through the event-free `Engine::process` fallback, record
  bounded telemetry, and arrange recovery rather than emit stale buffer data.
- Rendering is segmented at event offsets while existing instrument/effect state
  and DSP interfaces remain unchanged. Instruments and the master chain observe
  each segment as a contiguous process slice; future block-size/latency-dependent
  DSP must preserve semantics across such slices or adopt a separately reviewed
  strategy. Direct imperative note/process APIs remain available for audition
  and focused embedding/tests.
- `PreparedParameterBinding` is constructed on NRT from a validated
  `param_manifest::RuntimeParameter` and rejects any rate other than
  `AutomationRate::SampleEvent`. Stable IDs are resolved and normalized values
  are mapped before RT; the event carries only a string-free runtime key,
  concrete effect target/index, and engine value.

This slice deliberately does not provide event admission storage, merge
capacity, or producer-visible overload (#203), tracker tick offsets (#202), or
first-party tracker/live integration (#204). The engine accepts an already
ordered bounded slice; those follow-ups own producing it.

## Implemented tracker tick clock (#202)

`sequencer::timing::TimingState` now owns only the musical tick clock and exposes
`advance_ticks`, a bounded allocation-free callback cursor plus compact result:

- Tick boundaries are relative to the exact half-open frame slice. Every emitted
  offset is in `0..frame_count`; a boundary equal to `frame_count` is retained
  and appears at offset zero of the next non-empty slice.
- Exact tick phases use prepared Q64.64 intervals and absolute frame state. Each
  interval is added once per tick, so fixed, alternating, single-frame, and
  oversized partitions produce identical absolute integer tick boundaries.
  Fractional phases are rounded up only when exposed as render-frame offsets;
  the fractional phase itself is retained.
- A BPM directive returned while processing a tick changes the interval after
  that tick. The new interval is added to the emitted tick's exact fractional
  phase, never to its rounded sample offset, and therefore cannot move the
  already emitted boundary. A BPM latched between ticks leaves the already
  scheduled next boundary unchanged and applies after that boundary.
- Preparation rejects non-finite/non-positive sample rate or BPM, intervals
  shorter than one frame (which guarantees strict offset order), intervals
  beyond the documented representable range, and a zero work bound.
  Per-slice callback work is capped by the prepared tick count. Capacity,
  invalid-tempo, and absolute-position failures return explicit sticky status;
  callers must treat offsets from a non-`Complete` result as failed output and
  deliberately reset at a transport boundary.
- Ticks per line (TPL) is tracker row-progression state in
  `audio_backend::player::Player`; it is not an input to tick spacing. The
  count-only `TimingState::advance` temporarily retains its historical
  end-inclusive count behavior so existing whole-block PCM references remain
  stable; it must not be mixed with the half-open offset API. That method and
  the old TPL constructor are bounded compatibility shims only until #204
  consumes offsets and removes them.

This clock has no dependency on `engine`, tracker song/document types, host
hardware, or event-admission storage. It therefore preserves the #201 event
surface and does not define any #203 admission API.

## Intentionally open implementation questions

- Future-clock window representation, preparation, and ownership (#134/#132).
- SPSC or alternative RT↔NRT handoff; coverage declaration, memory ordering,
  full-capacity response, recovery timing, and resume behavior (#145/#132).
- Stable producer identity/sequence and the exact total-order/admission
  mechanism (#134).
- Exact RNG/interpreter checkpointing and seek/loop state reconstruction (#138
  and runtime adapters).
- Concrete lifecycle/process types and recovery representation (#132/#134).

ADR 0003 fixes the boundary and constraints, not these mechanisms.
Implementation may refine or replace them when evidence shows a better approach;
changing the boundary or hard constraints uses the normal ADR
amendment/supersession process.

## Follow-up ownership

- [#134](https://github.com/jpalvarezl/blight-synth/issues/134) — event semantics,
  sample-accurate timing, deterministic ordering, and engine event application
- [#132](https://github.com/jpalvarezl/blight-synth/issues/132) — prepared
  lifecycle, process surface, caller-owned capacities, status, and recovery
- Composition-adapter extraction under #145 — tracker/composite adapter and a
  minimal second source, plus any chosen bounded lookahead handoff
- [#138](https://github.com/jpalvarezl/blight-synth/issues/138) — versioned
  snapshots, saved runtime state, generation handoff, and reclamation
