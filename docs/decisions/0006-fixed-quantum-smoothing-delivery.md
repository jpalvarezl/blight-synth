---
title: ADR 0006 — Fixed-quantum smoothing delivery
summary: Engine-owned coalesced smoothers use an absolute 16-frame control phase and existing scalar DSP setters.
status: accepted
updated: 2026-08-06
issues: [214, 223, 224, 234]
supersedes: []
amends: ["0005"]
---

# ADR 0006 — Fixed-quantum smoothing delivery

## Status

Accepted. Deciding issue: [#223](https://github.com/jpalvarezl/blight-synth/issues/223).

## Context

[ADR 0005](0005-coalesced-parameter-publication.md) makes the engine the sole
coalesced-smoothing owner but leaves delivery to block-oriented
`set_parameter(index, value)` DSP nodes open. Callback and event segmentation
must not change a latched trajectory, and migrated nodes must not smooth it again.

## Decision

### Delivery and callback bound

Use a fixed **16-frame control quantum**. Prepared engine state owns a phase in
`0..16`, reset to zero on installation or semantic reset and advanced only by
rendered frames. Rendering is split at the union of quantum boundaries and event
offsets; a boundary sends one scalar value through each active binding's existing
DSP setter before rendering that frame. A coincident scalar delivery precedes
timestamped events at that offset. Existing and migrated nodes must remain
slice-continuous across these subdivisions.

For a successful top-level call, `Engine::process` latches coalesced targets once
before rendering. `process_with_events` first validates the complete event slice,
then performs the same one latch before offset-zero events; it uses a private
renderer rather than recursively relatching through public `process`. A
zero-frame successful call may latch and confirm targets but does not advance the
phase or deliver a quantum value. Async publication still determines *which*
block latches a target; from that latch cursor onward, callback and event
partitions do not affect the trajectory or delivery phase.

With `F` common-prefix frames, at most `1 + ceil(F / 16)` boundary sweeps occur.
Each worst-case sweep examines at most 16 fixed active-ramp bitmap words and calls
at most 1,024 scalar setters. This is in addition to ADR 0005's once-per-top-level
block scan of 16 dirty words and at most 1,024 mappings/latches; there is no
allocation, lock, retry, or unbounded loop. Inactive ramps may be skipped.
Sixteen frames gives the current shortest built-in ramp (15 ms) at least 41
quantum steps at 44.1 kHz; callback and audio-quality measurements remain
required by #214.

### Exact smoother semantics

Preparation requires a finite positive sample rate, finite sign-unconstrained
seed/target values, and a finite non-negative `duration_ms`. Negative or non-finite durations are rejected
with a compact preparation error and never reach callback processing. For a
positive duration, let `N = max(1, ceil(duration_ms * sample_rate / 1000))`;
preparation rejects an unrepresentable frame count. Either zero duration or
`SmoothingPolicy::None` independently causes an immediate jump and delivery at
the latch cursor.

A new target starts from current value `s` at its latch cursor, sets elapsed
`e = 0`, and uses the full `N` frames. Republishing the already-latched target
does not restart its trajectory; a target equal to current settles immediately.
For `0 <= e < N`:

- linear: `x(e) = s + (target - s) * e / N`;
- exponential: `x(e) = target + (s - target) * 10^(-5 * e / N)`.

At `e >= N`, both return exactly `target` and settle. Thus linear duration is
exactly `N` elapsed rendered frames. The underlying exponential curve reaches a
`1e-5` residual of the initial step (−100 dB) at `e = N`; every pre-snap point
`e < N` has a larger residual, and snapping exactly at `N` is the settle
criterion.
Implementations derive values from integer total elapsed frames, not repeated
per-call accumulation, so advancing `a + b` frames equals advances of `a` then
`b` for a fixed latch. Scalar DSP delivery is piecewise constant: if settlement
falls between control boundaries, the setter receives the exact target at the
next boundary.

Target confirmation retains ADR 0005's meaning: RT confirms immediately after a
successful map and latch, not after scalar delivery or settlement. Installation
or semantic reset maps the authoritative seed, sets current and target to that
seed, clears elapsed/ramping state, and invokes the DSP setter before the first
rendered sample; there is no startup ramp.

### One smoothing owner and migration

Migration of a manifest-bound coalesced parameter is atomic: install its engine
binding/smoother and remove or bypass any de-zipper for that same parameter in
the DSP node. In particular, reverb mix must no longer pass engine-delivered
values through its current per-sample `dsp::Smoother`. That utility may remain
for DSP-internal modulation and explicitly unmigrated legacy controls, never as
a second stage for a migrated manifest policy.

## Consequences

The scalar setter contract remains usable and render behavior after a latch is
independent of host/event partitioning. Costs are staircase delivery and up to a
16-frame control delay plus the bounded repeated setter work above.

## Alternatives considered

- **One scalar per host block:** rejected because host block size and event-driven
  segmentation change both audible steps and duration.
- **Ramp-aware DSP API:** rejected for this delivery because it widens every
  effect/voice process contract and either duplicates curve state in nodes or
  requires a larger parameter-buffer protocol. Revisit if quantum staircasing or
  setter cost fails measurement.
- **Per-sample scalar setters:** rejected because 1,024 bindings times maximum
  block size is an unnecessarily large dispatch/conversion budget.

## Validation and revisit triggers

#224 tests the closed-form primitive, retarget/reset, finite edges, and partition
equivalence. #234 integrates the absolute phase into both process entries and
tests offset-zero ordering, coincident event/quantum ordering, arbitrary event
and callback partitions, bounded setter work, gain delivery, and zero heap work.
#235 owns representative duplicate reverb-smoother removal. Supersede this ADR
if those measurements require a ramp-aware API or a different fixed quantum.
