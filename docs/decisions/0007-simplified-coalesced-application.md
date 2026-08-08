---
title: ADR 0007 — Simplified coalesced target application
summary: Keep generation-bound latest-value coalescing, but apply mapped targets once per render block and defer smoothing to explicit DSP-local implementations.
status: accepted
updated: 2026-08-07
issues: [101, 214, 231, 234, 235, 237, 238]
supersedes: ["0006"]
amends: ["0005"]
---

# ADR 0007 — Simplified coalesced target application

## Status

Accepted. Deciding issue: [#237](https://github.com/jpalvarezl/blight-synth/issues/237).
Implemented by [#238](https://github.com/jpalvarezl/blight-synth/issues/238).

This decision supersedes [ADR 0006](0006-fixed-quantum-smoothing-delivery.md)
and explicitly amends ADR 0005 §3 (smoothing ownership), §5 (confirmation), and
§6 (reset/application failure). ADR 0005's normalized generation-bound MPSC
store, dirty publication protocol, mapping ownership, and desired/pending state
semantics remain accepted.

## Context

Issues #213 and #230 established useful infrastructure for high-rate continuous
controls: publishers overwrite obsolete values instead of flooding the structural
queue, dense keys cannot cross prepared generations, the runtime table owns
mapping, and NRT can observe whether RT accepted a target revision.

ADR 0006 attempted to make the engine a generic smoother while retaining scalar
DSP setters. That required a persistent 16-frame phase, active/pending ramp
bitmaps, repeated setter sweeps, and rendering split at the union of event and
control boundaries. That implementation was reviewed but never merged; the
merged store/binding foundation does not yet deliver coalesced values to DSP.
The proposed scheduler was RT-safe and deterministic, but its complexity and
callback work were disproportionate to current product needs:
there is no production GUI, MIDI mapping, or plugin automation yet, and existing
DSP such as reverb already owns any smoothing it demonstrably needs.

All product hosts use the same DSP implementations. DSP-local smoothing can
therefore remain host-independent without adding a second scheduler to Engine.

## Decision

### Retain latest-value coalescing

Keep ADR 0005's prepared store and publication contract:

- adapters publish finite normalized values to generation-bound handles;
- intermediate values may coalesce and eventual latest is guaranteed after
  publishers quiesce;
- RT scans the fixed dirty words once at the start of each successful top-level
  render call;
- `RuntimeParameterTable` remains the sole normalized-to-engine mapper;
- stale, closed, invalid, exhausted, and application-failure outcomes remain
  compact and observable; and
- applied confirmation remains the exact normalized revision/value whose mapped
  target was accepted by the concrete DSP setter, not merely transport,
  publication, or queue acceptance. For `None`, confirmed means the scalar is in
  DSP state; for a future reviewed DSP-local smoother, it means the target was
  accepted, not that its ramp settled.

### Apply targets once at block start

For `Engine::process`, RT drains, maps, and applies each dirty target once before
rendering the common buffer prefix.

For `Engine::process_with_events`, the complete event slice is validated first.
Only a valid call drains and applies coalesced targets, before offset-zero events;
invalid event input leaves coalesced publication pending for a later valid call.
The two public APIs share one private non-relatching render path.

A zero-frame successful call may drain/apply/confirm targets but renders no audio.
Both public process APIs share one private renderer and never recursively drain;
`process_with_events` validates the complete event slice before any drain/apply.
Invalid events leave coalesced publication pending for a later valid call.

Prepared install/reset maps every authoritative seed, resolves its concrete
master/instrument effect target, invokes the existing infallible scalar setter
once before the first rendered sample, and confirms only if target resolution
and invocation succeeded. This does not change the DSP `set_parameter` trait
signature: "setter success" means the prepared target exists and the appropriate
setter was invoked. Missing or unsupported targets retain the prior confirmation
and record compact failure. Device-host ownership, replacement, and retirement
remain #215.

### Do not add generic Engine smoothing now

#238 deletes the merged but unused Engine `PreparedSmoother` module/tests and
removes per-binding smoother state. The fixed-quantum phase/render integration
was never merged. Specifically remove/defer:

- Engine-owned `PreparedSmoother` state;
- fixed-quantum phase and quantum/event render segmentation;
- active/pending ramp worksets;
- repeated scalar setter sweeps; and
- generic smoothing-settled telemetry.

Current DSP-local smoothing remains unchanged where it already exists. A new
parameter receives smoothing only after an audible/product requirement justifies
it. Such a migration should use a shared DSP utility and one owner; it must not
stack DSP and adapter/engine smoothers.

For the current generic coalesced binding path, `SmoothingPolicy::None` is the
only supported policy. Binding preparation—not manifest parsing—rejects
`Smoothed` with a compact unsupported-policy error unless a future target
explicitly declares a separately reviewed DSP-local smoothing capability. The
existing master-gain manifest is changed to `None`: it is the only built-in
manifest parameter, is not yet wired to the coalesced path, and its current
command/OSC path already applies gain immediately. This records existing audible
behavior rather than introducing a regression. Large jumps may click and coarse host
blocks may make fast drags step; that limitation is accepted for M1 and is the
revisit trigger for a focused DSP-local gain smoother.

`SampleEvent` remains separate, mapped during NRT event preparation, unsmoothed,
and sample-accurate.

## Consequences

### Positive

- High-rate GUI/MIDI/OSC publication can coalesce without queue backlog.
- The engine render loop gains one bounded block-start drain, not another timing
  scheduler.
- Existing DSP processing and event segmentation stay simple.
- GUI work can begin without first completing plugin-grade automation smoothing.
- Smoothing is added from measured audible need instead of preemptively for every
  manifest parameter.

### Costs and limitations

- A parameter without DSP-local smoothing changes immediately at a render-block
  boundary and may zipper if driven aggressively.
- Smoothing behavior is not yet uniformly generated from manifest metadata.
- Block-start target latch timing still depends on host callback partitioning;
  sample-accurate automation uses `SampleEvent` when that distinction matters.
- If real GUI/controller testing demonstrates audible artifacts, a focused
  DSP-local smoothing migration is required.

These are accepted M1 tradeoffs. They are simpler to observe and fix than a
speculative generic render scheduler.

## Alternatives considered

### Merge the fixed 16-frame Engine scheduler

Rejected for now. It is deterministic but adds phase, bitmaps, repeated setter
work, render segmentation, and substantial test/maintenance surface before a
measured product requirement.

### Smooth once per host callback in Engine

Rejected. Ramp shape and duration would depend on host callback size while still
adding generic engine state.

### Remove coalescing entirely

Rejected. Latest-value storage is small, already implemented, and directly useful
for near-term GUI knob traffic; replaying obsolete intermediate values through the
structural queue is the wrong overload behavior.

## Validation and revisit triggers

#238 implements the first production coalesced-to-DSP delivery and validates one
block-start drain/application in both Engine process APIs, setter-reported
confirmation/failure, unsupported `Smoothed` rejection, unchanged sample-event
ordering, constructor/reset-generation seed application, and zero callback heap
work while deleting unused smoother infrastructure.

Revisit smoothing only when a real control surface demonstrates zipper/click
artifacts, a parameter needs a specified ramp for product behavior, or a plugin
host requires smoothing beyond sample-accurate automation. Prefer a focused
DSP-local migration and shared utility before reconsidering a generic Engine
scheduler.
