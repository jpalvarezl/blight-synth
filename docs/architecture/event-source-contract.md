---
title: Event-Source Contract (draft)
summary: Routing page for the host-orchestrated composition→engine boundary defined in ADR 0003.
status: draft
updated: 2026-08-02
issues: [145, 134, 132, 138, 113, 201, 202, 203, 204]
---

# Event-Source Contract (draft)

This page routes readers to the boundary proposed in
[ADR 0003](../decisions/0003-event-source-contract.md). The ADR is authoritative.
Issues #201–#204 implement the bounded first-party current-block tracker/live
path; lifecycle, publication generations, optional lookahead, and extraction of
additional composition runtimes remain with #132/#138/#145. Owning issue:
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

Issue #204 makes direct-RT tracker evaluation structurally bounded.
`TrackerEngineAdapter` stores last-instrument state as
`[InstrumentId; MAX_TRACKS]`, so lookup/update cannot exceed the eight prepared
track slots or allocate. `Player` prepares storage for at most 4096 tick
boundaries, two tracker events per tick/track (prior-instrument release plus the
cell operation), 64 live events, and the corresponding admission lane. Callback
work is therefore bounded by 4096 ticks × eight tracks × two events, 64 live
events, one canonical bounded sort, and segmented engine rendering.

`Player` consumes `TimingState::advance_ticks`; offsets are relative to each
exact common stereo render slice. A tick at the half-open block end is retained
for offset zero of the next slice, so fixed, alternating, one-frame, and 4096
frame host-chunk partitions produce equal absolute event positions.

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

This slice deliberately does not itself provide event admission storage, merge
capacity, tracker interpretation, or live-input collection. Those are host and
composition-adapter responsibilities implemented by #202–#204.

## Implemented tracker tick clock (#202)

`sequencer::timing::TimingState` now owns only the musical tick clock and exposes
`advance_ticks`, a bounded allocation-free transaction into caller-owned
prepared storage plus a compact result:

- Tick boundaries are relative to the exact half-open frame slice. Every emitted
  offset is in `0..frame_count`; a boundary equal to `frame_count` is retained
  and appears at offset zero of the next non-empty slice.
- Exact tick phases use prepared Q64.64 intervals and per-transport-epoch frame
  state. Each interval is added once per tick, so fixed, alternating,
  single-frame, and oversized partitions produce identical integer boundaries.
  Fractional phases are rounded up only when exposed as render-frame offsets;
  the fractional phase itself is retained.
- A side-effect-free tempo planner may select the interval after each tick. The
  complete slice, all directives, fixed-point additions, and output capacity are
  validated before the staged clock is committed. A non-`Complete` result
  commits zero output ticks, and callers ignore scratch entries; producer state
  is mutated only after a complete result. The new interval is added to the
  tick's exact fractional phase, never its rounded offset. Public `set_bpm`
  between ticks similarly leaves the already scheduled boundary unchanged and
  applies after it.
- Preparation rejects non-finite/non-positive sample rate or BPM, intervals
  shorter than one frame (which guarantees strict offset order), intervals
  beyond the representable range, and a zero work bound. Capacity,
  invalid-tempo, and position failures are explicit and sticky. Invalid public
  BPM changes leave a valid clock untouched; valid tempo preparation or a
  deliberate transport reset provides the documented recovery. Reset begins a
  new frame epoch so even absolute-position exhaustion is recoverable.
- The shared audio-backend render-slice limit is 4096 frames. Player prepares a
  4096-tick bound, which covers the structural worst case of one valid tick per
  frame and valid `u16` song BPM at a maximum-size host chunk; the former 1024
  default did not. The allocation audit covers complete, failed, reset, and
  reused paths.
- Ticks per line (TPL) is tracker row-progression state in
  `audio_backend::player::Player`; it is not an input to tick spacing. #204
  consumes `advance_ticks` directly. The count-only compatibility method remains
  only for external transitional callers and is no longer in first-party
  playback.

This clock has no dependency on `engine`, tracker song/document types, host
hardware, or event-admission storage. It therefore preserves the #201 event
surface and does not define any #203 admission API.

## Implemented first-party admission and playback (#203/#204)

`Player` is the first host-owned composite scheduler for two stable ordinary
producers (tracker and queued live input) plus the reserved recovery producer.
It fills prepared tracker/live slices, submits both through
`BoundedEventAdmission`, and passes only the finalized canonical slice to
`Engine::process_with_events`. A rejected ordinary block renders no ordinary
prefix; reserved global all-notes-off remains available and current voices/tails
render through the event-free fallback if engine validation unexpectedly fails.
`PlayerProcessStatus` exposes compact timing plus event-admission/application
status to the device host and offline renderer.

Queued `InstrumentCmd::NoteOn` becomes a live note event at offset zero.
Legacy instrument-wide release becomes the additive canonical
`EngineEvent::InstrumentAllNotesOff` variant, which has targeted-release
precedence and avoids widening a live/tracker release into global recovery.
Transport stop uses the separate reserved global recovery slot. Events queued
before a stop in the same callback are cancelled; events queued after it remain
eligible. Because every live command is intentionally at offset zero and
canonical precedence orders releases before attacks, an instrument-wide release
coalesces earlier same-block attacks/releases for that instrument (their lifetime
is zero frames); later attacks remain after the release. This preserves FIFO
lifecycle meaning without inventing nonzero live offsets or a competing sort.

Tracker Fxx timing is deterministic: F01–F1F changes TPL for the row beginning
at that tick, F20–FF selects BPM for the interval after that tick (the #202
next-interval rule), and F00 is ignored. Tracks are scanned by ascending stable
index and the last applicable command wins. Looping resets tracker position but
keeps the exact timing phase continuous; non-looping end emits recovery at the
ending tick. Engine rendering is no longer gated by tracker play/stop, so live
audition, releases, voices, and effect tails continue while transport is
stopped.

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
