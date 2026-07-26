---
title: Event-Source Contract (draft)
summary: Routing page for the host-orchestrated composition→engine boundary defined in ADR 0003.
status: draft
updated: 2026-07-26
issues: [145, 134, 132, 138, 113]
---

# Event-Source Contract (draft)

This page routes readers to the boundary proposed in
[ADR 0003](../decisions/0003-event-source-contract.md). The ADR is authoritative;
the API is not implemented yet. Owning issue:
[#145](https://github.com/jpalvarezl/blight-synth/issues/145).

## Read first

- [ADR 0003 — Event-source contract](../decisions/0003-event-source-contract.md)
- [Real-time audio contract](realtime-contract.md) — control traffic class 2,
  backpressure/overflow, and the prepared-state rule
- [Offline render contract](offline-render-contract.md) — determinism and
  platform policy
- [System boundaries](system-boundaries.md) — dependency direction
- [Composition domain](../domains/composition.md) and
  [Audio engine domain](../domains/audio-engine.md)

## Authority and flow

```text
host-selected clock/input adapters + versioned document
  -> one composite event source/scheduler
  -> caller-owned current-block events with sample offsets
  -> Engine applies events and renders DSP
```

There is one timestamp authority. The host supplies the producer side with a
per-block absolute render-frame window and bounded prepared clock segments that
map both frame→musical position and musical deadline→frame. Segments are
monotonic; stopped spans and loop wraps split the mapping, and segment/loop
crossing count is fixed at `prepare`. An over-complex map is rejected on NRT.
The composition runtime/clock adapter computes `sample_offset` in
`[0, frame_count)`. The engine applies already-offset events; it does not select
a clock or convert musical/external time. Already-offset `Transport` events
change render transport state but are not clock-conversion requests.

The current tracker/song route is the first-party composition path being
adapted, not the engine's only caller. Direct engine note, command, and process
APIs are also used by embeddings, `engine/examples/offline_render.rs`, and tests.

## Roles

| Role | Owns | Must not own |
|---|---|---|
| Host/control + clock adapters | block windows/epochs, clock selection and estimation, live edits, MIDI/OSC I/O, filesystem, side-effect routing | composition semantics or engine DSP |
| Composition runtime/child producer | versioned document semantics, cursor, seeded RNG/checkpoints, semantic events mapped to producer-side render frames | devices, sockets, filesystem, engine graph mutation |
| Composite event source/scheduler | the one bounded current-block fill, stable producer IDs, total-order merge, overflow/recovery status | DSP rendering or unbounded callback work |
| Audio `Engine` | already-offset event application, instruments, voices, routing, render transport state, DSP, telemetry | musical-time conversion, `Song`/`Chain`/`Phrase`/tracker/UI types |

## Current-block pull

The host calls one composite producer for the current half-open `BlockWindow`,
then passes the resulting ordered slice to the engine. The host owns an empty,
preallocated buffer whose ordinary and recovery capacities were fixed during
`prepare` (#132). It never contains future events. Every event at the engine
boundary has a current-block `sample_offset`; an event at the block end belongs
to the next block.

Child producer count, child scratch, bounded inputs, merge work, and event count
all have configured maxima. Fixed-memory runtimes may evaluate in this call only
when their state and worst-case work satisfy the accepted RT contract. Other
runtimes consume NRT lookahead behind the same current-block interface.

## Separate NRT lookahead

NRT lookahead uses separate prepared storage with a maximum horizon in render
frames and maximum event count. Its timestamp domain is
`(clock_epoch, publication_generation, absolute_render_frame)`. NRT publishes
plain-data fixed-capacity packets followed by a contiguous coverage watermark
through a prepared SPSC ring. RT advances the ring's atomic consumption cursor
only after it no longer needs a packet; NRT observes that cursor before slot
reuse and never overwrites unread data. A full ring rejects publication and
cannot advance coverage. The RT adapter exposes a current block only when
matching coverage reaches the block end, then converts matching absolute frames
to current-block offsets.

Partial or late coverage is never applied. A deadline miss raises the reserved
recovery barrier at offset zero, records underflow, and suspends the source until
an explicit prepared resume. Edit, seek, loop-policy change, and clock
discontinuity install a newly tagged snapshot/publication generation at a block
boundary; old packets are rejected by tag and displaced ownership is reclaimed
on NRT. This prevents stale lookahead from crossing a revision or clock epoch.

## Ordering, overflow, and recovery

The final ordinary lane contains already-offset `Note`, sample-accurate
`Control`, and `Transport` events. One composite merger orders all child
producers by:

```text
(sample_offset, semantic_phase, stable_producer_id, producer_sequence)
```

Phases are transport stop/seek barriers, note releases, transport start/state,
controls, then note attacks. Same-offset control applies to existing target
state; initial per-note expression belongs on the attack event. The composite
validates and strips generation tags before the engine boundary. The full phase
table lives in ADR 0003 and must be extended for every new event kind.

At capacity, the merger retains the earliest `C` events by that key and rejects
the suffix with a producer-visible count/first key. A separate one-slot
coalescing recovery latch is reserved at preparation and cannot be consumed by
ordinary events. Any rejection produces an engine-global compound transport-stop +
all-notes-off barrier at the first rejected offset, after the retained prefix;
later current-block events are ignored and the composite scheduling generation
suspends. Multiple panic requests coalesce to the earliest offset. Global scope
is intentional because M1 voices do not have producer ownership; recovery may
release another producer's notes but cannot leave shared-instrument voices
stuck. This is the defined traffic-class-2 overload behavior, including when a
note-off was rejected.

## Determinism and outbound effects

Event-stream determinism means equal semantic event values and total order for
the same versioned document/runtime, seed or exact checkpoint, clock/input
trace, and declared loop policy. It is not a byte-identity claim without a
versioned canonical event serialization. A runtime uses either
position/counter-addressed RNG or exact checkpoint replay, and declares whether
a loop repeats loop-entry RNG state or evolves through a persisted loop count.

Render determinism is separate. Exact PCM equality follows the
[offline render contract](offline-render-contract.md): exact references on the
canonical platform, with that page's repeated-render and metric policy
elsewhere.

Abstract outbound MIDI/OSC is not an engine event. Direct-RT evaluators are
prohibited from producing outbound side effects under this ADR. Runtimes that
need them evaluate that output on NRT and use a bounded host side-effect
scheduler with deterministic suffix rejection and destination panic recovery;
actual I/O remains on NRT.

## Tracker migration truth

`track_last_instrument` already lives behind `TrackerEngineAdapter`, but it is a
callback-reachable `HashMap` and `insert` may allocate. Before direct RT tracker
evaluation, the extraction must replace it with structurally fixed indexed
state and prove tick/event bounds; otherwise tracker evaluation must use NRT
lookahead. Moving or preallocating the same map is not a fix.

Likewise, `TimingState::advance` returns only an elapsed tick count. The current
player demonstrates demand-driven block evaluation, not sample-accurate event
placement; #134 must calculate each event's absolute frame/offset.

## Follow-up implementation

- [#134](https://github.com/jpalvarezl/blight-synth/issues/134) first — public
  ordinary/recovery schema, phase/clock rules, and engine event application
- [#132](https://github.com/jpalvarezl/blight-synth/issues/132) — prepare/process
  lifecycle and caller-owned capacities using #134's schema
- Composition-adapter extraction under #145 — composite bounded merge,
  tracker/lookahead adapters, two-cursor publication ring, and a second source
- [#138](https://github.com/jpalvarezl/blight-synth/issues/138) — versioned
  snapshots, exact saved runtime state, and generation handoff
