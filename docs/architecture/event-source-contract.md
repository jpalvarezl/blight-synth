---
title: Event-Source Contract (draft)
summary: Routing/contract page for the composition→engine event boundary defined in ADR 0003.
status: draft
updated: 2026-07-24
issues: [145, 134, 132, 138, 113]
---

# Event-Source Contract (draft)

This is a short routing page for the boundary decided in
[ADR 0003](../decisions/0003-event-source-contract.md). It records where the
contract lives; the ADR is authoritative and the code is not yet built. Owning
issue: [#145](https://github.com/jpalvarezl/blight-synth/issues/145).

## Read first

- [ADR 0003 — Event-source contract](../decisions/0003-event-source-contract.md)
- [Real-time audio contract](realtime-contract.md) — "Control traffic classes"
  (class 2: timestamped events) and "Prepared-state rule".
- [System boundaries](system-boundaries.md) — dependency direction.
- [Composition domain](../domains/composition.md),
  [Audio engine domain](../domains/audio-engine.md).

## Roles

| Role | Owns | Must not own |
|---|---|---|
| Audio `Engine` (consumer) | instruments, voices, routing, parameters, transport clock conversion, rendering; consumes bounded timestamped events | `Song`/`Chain`/`Phrase`/tracker `Event`/UI types |
| Composition runtime (producer) | versioned document, interpreter state, seeded RNG, snapshots; produces timestamped note/control/transport events | audio devices, sockets, filesystem, RT graph mutation |
| Host/control | live edits, MIDI/OSC I/O, filesystem, clock selection, side-effect routing | event semantics (transports map onto the shared contract) |

## Engine event-consumer contract (summary)

- Event kinds: `Note`, `Control` (sample-accurate automation by stable #121
  parameter ID), `Transport`.
- Each event carries a `sample_offset` in `[0, block_len)`; deterministic
  same-offset ordering; identical (document, seed, clock) ⇒ identical stream.
- Fixed per-block capacity set at `prepare` (#132); overflow is explicit,
  producer-visible, deterministic, non-reordering, and never allocates on RT.
- Pull-based fill with bounded lookahead; generative cost may run as
  deterministic NRT lookahead behind the same contract.

## Follow-up implementation

- [#134](https://github.com/jpalvarezl/blight-synth/issues/134) — sample-accurate
  scheduling (concrete event types/capacities/overflow).
- [#132](https://github.com/jpalvarezl/blight-synth/issues/132) — engine
  lifecycle (`prepare` capacities/block layout).
- Composition-adapter extraction — tracker `Player` becomes one adapter; add a
  minimal synthetic/generative event source (coordinated with
  [#138](https://github.com/jpalvarezl/blight-synth/issues/138)).
