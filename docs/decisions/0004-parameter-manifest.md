---
title: ADR 0004 — Canonical parameter manifest and host bindings
summary: One serializable parameter manifest is the single source of truth for parameter metadata across the Rust DSP/engine, project state, OSC, JUCE/APVTS, and the Svelte UI; a bounded string-free runtime lookup serves the audio thread.
status: accepted
updated: 2026-08-03
issues: [121, 101, 145, 137, 212]
supersedes: []
---

# ADR 0004 — Canonical parameter manifest and host bindings

## Status

Accepted

Deciding issue: [#121](https://github.com/jpalvarezl/blight-synth/issues/121).

This ADR defines the parameter manifest and the Rust descriptor/runtime-lookup
types that consume it. It does **not** implement the coalesced continuous-parameter
RT pipeline (owned by [#101](https://github.com/jpalvarezl/blight-synth/issues/101)),
does not rewire every existing instrument/effect parameter (follow-up), and does
not change engine voice code (owned by
[#137](https://github.com/jpalvarezl/blight-synth/issues/137)). It provides the
types plus one representative descriptor matching the standalone master gain.
The existing OSC/engine path is not migrated by this issue. The accepted
[ADR 0005](0005-coalesced-parameter-publication.md) adds the coalesced publication,
generation, mapping-execution, smoothing, and confirmation contract and amends
the narrower adapter-mapping description below.

## Context

Parameter metadata is currently re-derived independently at every boundary:

- The DSP nodes expose an opaque `set_parameter(index: u32, value: f32)`
  (`dsp/src/effects/*.rs`); the meaning of each index, its range, unit, and
  default live only in code comments.
- The OSC adapter (`audio_backend/src/standalone_process/osc.rs`) hard-codes the
  `/param/set gain` name, the `0..1`-linear-amplitude→dB conversion
  (`normalized_gain_to_db`), the silence floor, and the target `effect_id`/`param_index`.
- The engine `Gain` effect then re-converts dB→linear amplitude
  (`dsp/src/effects/gain.rs`).
- The (planned) Svelte UI and an optional JUCE/APVTS plugin host would each need
  the same name, range, unit, normalized mapping, default, and automation flags —
  and would duplicate the conversion a third and fourth time.

This duplication is the exact hazard the
[real-time audio contract](../architecture/realtime-contract.md) anticipates in
its "Control traffic classes" section: continuous parameters must *coalesce by a
stable ID at a control rate*, and its "Prepared-state rule" requires *parameter
descriptors and lookup tables* to be prepared off the audio thread with
descriptive strings never reaching the callback. [ADR 0001](0001-product-and-host-priorities.md)
requires composition experiments and the optional plugin to *share* instruments,
effects, parameters, and state rather than wrap per-host copies, and
[ADR 0002](0002-device-host-osc-split.md) positions OSC as a transport adapter
that *maps onto* shared control types without owning control semantics. A shared
parameter contract is the missing piece that lets all of that hold.

The tension: without one canonical descriptor, unit conversion, smoothing, and
stable-ID assignment fork per transport; #101's coalesced pipeline has no agreed
key space to coalesce on; and there is no bounded, string-free representation for
the audio thread to consume.

## Decision

Adopt a single serializable **parameter manifest** as the source of truth for
parameter metadata, plus a bounded **runtime lookup** derived from it for the
audio thread. Both live in a new dedicated workspace crate `param_manifest`.
This issue adds the contract crate and representative descriptor; future
consumer migrations will make `dsp`/`engine`/`audio_backend`, the UI export, and
a future plugin depend on it instead of re-deriving metadata.

### 1. Two tiers, matching the prepared-state rule

- **Manifest tier (NRT).** `ParameterManifest { schema_version, parameters:
  Vec<ParameterDescriptor> }` carries all descriptive strings, labels, mappings,
  and versioning metadata. It is authored/parsed/validated exclusively off the
  audio thread.
- **Runtime tier (RT).** `ParameterLookup` is prepared once on NRT from a
  validated manifest. It exposes a string-free `RuntimeParameterTable` with
  bounded boxed slices for `RuntimeParameter` entries and exact numeric discrete
  values. Parameters are addressed by a compact `RuntimeParamKey(u32)` and
  resolved with bounded O(1) slice indexing — no hashing, allocation, or
  `String` access in read/conversion methods. The `ParameterId`→key resolver
  `HashMap` stays on the NRT `ParameterLookup` owner; `RuntimeParameter` is a
  private-construction `Copy` numeric entry. It does not expose its raw `Mapping`;
  `RuntimeParameterTable::normalized_to_engine` is the single RT conversion API
  so discrete parameters cannot bypass their numeric arena. The table is
  intentionally not `Clone`: installation, replacement, and destruction are
  prepared-state lifecycle operations, and displaced tables must be retired to NRT.

### 2. Descriptor fields

Each `ParameterDescriptor` records:

| Field | Purpose |
|---|---|
| `id: ParameterId` (stable string) | Canonical cross-boundary identity (OSC arg, APVTS ID, Svelte key, project field). Stable across versions. |
| `owner: NodeRef { node_type, path, engine_param_index }` | Owning node type/stable path and the `set_parameter` index. Runtime `EffectId`/`InstrumentId` is resolved by the adapter at prepare time. |
| `display_name`, `short_name` | Full and abbreviated labels. |
| `unit` | Engine-value unit (dB, Hz, seconds, linear, percent, semitones, count, custom). |
| `range: ValueRange { min, max, default }` | Engine-value bounds and default. |
| `mapping: Mapping` | Normalized `0..1` ↔ engine-value conversion (linear / exponential / skewed / amplitude-dB). Single unit-conversion owner. |
| `kind: ParameterKind` | `Continuous` or `Discrete { steps: [{label, engine_value}] }`. |
| `automation_rate: AutomationRate` | `SampleEvent` \| `ControlCoalesced` \| `Structural` — the traffic class the value flows through. |
| `smoothing: SmoothingPolicy` | `None` or `Smoothed { duration_ms, curve }`. |
| `visibility: Visibility { host_visible, automatable, read_only }` | Host/plugin generic-list and automation flags. |
| `version_added: u32`, `deprecated: Option<String>` | Per-descriptor version/compatibility metadata. |

The manifest carries `schema_version` (`MANIFEST_SCHEMA_VERSION`), bumped only for
a change to the descriptor *shape/semantics*, never for adding or removing
individual parameters. Version 1 is the first defined schema and this initial
implementation accepts exactly the current version; supporting an older shape
requires an explicit migration.

### 3. Normalized ↔ engine mapping ownership

`Mapping`, exposed to runtime consumers only through
`RuntimeParameterTable::normalized_to_engine`, is the sole owner of unit
conversion and is `Copy`, so it lives in both the descriptor and the runtime
tier. The conversion API is shared, while traffic class determines its execution
site: NRT event preparation maps `SampleEvent` values; the RT consumer maps
normalized `ControlCoalesced` values under ADR 0005:

- `Linear { min, max }`
- `Exponential { min, max }` (perceptual frequency/time controls; valid manifests
  require finite `0 < min < max`)
- `Skewed { min, max, skew }` (power curve over linear endpoints; `skew==1` is
  linear, `skew<1` biases toward `max`, and `skew>1` biases toward `min`; valid
  manifests bound the exponent to `0.25..=4.0`. At the upper bound the lowest
  audited interior point remains `0.1^4 = 1e-4`, avoiding the endpoint collapse
  caused by exponent 64. Validation also checks each skew/range pair at normalized
  `0.1`, `0.25`, `0.5`, and `0.9`, requiring round-trip error no greater than
  `1e-4`, because endpoint spacing also affects `f32` precision.)
- `AmplitudeDecibel { floor_db }` (normalized is linear amplitude, engine value is
  dB; `1.0 → 0 dB`, `0.5 → −6.02 dB`, `0.0 → floor_db`; valid floors are below
  the implied `0 dB` maximum)

Mapping bounds must exactly match `ValueRange.min/max`. Conversion uses stable
`f64` intermediates so tiny representable spans and the full finite `f32` linear
span do not overflow intermediate subtraction/ratios. NaN normalized input falls
back to `0.0`; NaN engine input falls back to the range floor; infinities clamp.
The methods also sanitize invalid directly constructed mappings to finite ordered
bounds, but NRT validation remains mandatory for prepared state. The inverse is
promised only over the representable, non-floored portion of a mapping (all
amplitudes at/below a dB floor intentionally map back to `0.0`).

Choose `Exponential` when the perceived control should be geometric/equal-ratio
(frequency, time), where steepness is inherently tied to the `max/min` ratio and
endpoints must be positive. Choose `Skewed` when you need an arbitrary steepness
with plain linear endpoints (including zero/negative) — the `skew` exponent biases
the knob toward one end independently of where the endpoints sit.

### 4. Automation rate maps to the RT traffic classes

`AutomationRate` is the manifest-level name for the
[real-time contract](../architecture/realtime-contract.md) "Control traffic
classes":

- `SampleEvent` → timestamped, ordered, bounded events (#134/#145).
- `ControlCoalesced` → continuous latest-value-wins, coalesced by stable ID at a
  control rate (#101).
- `Structural` → infrequent prepared-state replacement (#174/#138).

This ADR classifies parameters; #101/#134 implement the queues that consume the
classification.

### 5. How host adapters bind without duplicating conversion/smoothing

- **OSC** decodes `/param/set <id> <normalized>` and calls
  `ParameterLookup::key_for(id)` on NRT. It publishes the canonical normalized
  value to ADR 0005's generation-bound coalesced store rather than hard-coding
  `normalized_gain_to_db`; the RT consumer maps through the owning runtime table.
- **JUCE/APVTS** builds `AudioParameterFloat`s from `display_name`/`short_name`,
  a normalized `0..1` range, the `default_normalized()` default, and the
  `automatable`/`read_only` flags; host automation arrives normalized and is
  converted with the same `Mapping`.
- **Svelte UI** consumes the exported manifest (the serde JSON is directly
  usable as a TS contract) for labels, ranges, defaults, and step labels, and
  sends normalized values keyed by the stable ID.

Smoothing policy lives in `SmoothingPolicy` on the descriptor and its prepared
state is owned only by the engine parameter application layer, so de-zipper
behavior is identical across every adapter. ADR 0005 adds the accepted target
that `Smoothed` is valid only for `ControlCoalesced`; sample-event and structural
descriptors use `None`. The validation enforcement is pending #213 rather than
implemented by the current manifest crate.

### 6. Stable IDs and compatibility rules

- A `ParameterId` string is **never renamed or reused** for a different meaning.
  A rename is a new ID plus a `deprecated` marker on the old one.
- `RuntimeParamKey` is a dense per-prepared-lookup index, *not* a stable
  cross-version identity; the stable identity is always the string `ParameterId`.
- `ParameterManifest::validate()` enforces the current schema version, practical
  table/discrete capacities, unique IDs, descriptor versions, finite ordered
  ranges, variant-specific mapping invariants, exact mapping/range agreement,
  finite smoothing, `version_added` in `1..=schema_version`, non-contradictory
  automation/read-only visibility, and discrete values/defaults within range.
  ADR 0005's additional automation-rate/smoothing cross-check is explicitly
  deferred to #213; current code does not yet enforce it. Discrete numeric values
  (including non-uniform sets) are copied into a flat
  string-free runtime arena rather than reconstructed from `step_count`;
  normalized discrete positions and `default_normalized()` use ordinal step indexes.
- `ParameterManifest::compatibility_against(previous)` reports breaking changes:
  removing a live ID; changing automation rate; changing mapping/range/unit/kind
  or the full owner identity (`node_type`, `path`, and engine slot); and changing
  host visibility/automatable/read-only capabilities. Adding or deprecating an ID
  is compatible. Smoothing changes are explicitly compatible tuning events: they
  change de-zipper behavior but do not reinterpret saved values or invalidate a
  host binding. A CI/review step can diff manifests against the accepted one.

### Non-goals

- No coalesced continuous-parameter RT pipeline (#101); this defines the manifest
  it consumes.
- No migration of every existing instrument/effect parameter; one representative
  descriptor (master gain) proves the shape.
- No engine voice-code changes (#137); the crate only *reads* the existing
  `set_parameter` index contract.
- No change to `engine`/`dsp` public command types; `engine/src/lib.rs` is
  untouched.

## Consequences

### Positive

- One place defines each parameter's name, unit, range, default, normalized
  mapping, smoothing, and flags; OSC/APVTS/Svelte bind to it instead of forking
  the math (removing the OSC↔engine dB double-conversion is now a mechanical
  follow-up).
- #101 gets an agreed stable-ID key space and a bounded string-free lookup to
  coalesce against, satisfying the prepared-state rule directly.
- The serde manifest is a portable contract: the same JSON drives the Rust
  runtime lookup and the TypeScript UI, honoring ADR 0001's "share parameters"
  guardrail.
- Compatibility rules are executable, so parameter changes get a diffable safety
  net rather than tribal knowledge.

### Costs and risks

- A new workspace crate widens the dependency graph (a
  [system-boundaries](../architecture/system-boundaries.md) contract surface); it
  is intentionally minimal (`serde` only) and depends on nothing else.
- Until #101 lands and the adapters migrate, the unused representative manifest
  and the ad-hoc OSC conversion coexist; tests assert that the master-gain
  descriptor reproduces the exact OSC numbers to prevent drift.
- The descriptor is expressive; authoring discipline (naming conventions, keeping
  `Mapping` in sync with `set_parameter` behavior) is required until parameters
  are generated from a single definition.

## Alternatives considered

### Put the types in `dsp` or `engine` instead of a new crate

Rejected: `dsp` currently has no `serde` dependency and the manifest must be
consumed by `audio_backend`, the UI export path, and a future plugin without
pulling engine internals. A small dedicated crate keeps the contract cheap to
depend on and avoids coupling the DSP graph to serialization.

### Keep per-adapter conversion and only document a naming convention

Rejected: it leaves the normalized↔engine math and smoothing duplicated three or
four times (the current OSC/engine double-conversion is the evidence) and gives
#101 no shared key space.

### Use the numeric `set_parameter` index as the cross-boundary ID

Rejected: `(effect_id, param_index)` is runtime/position dependent and unstable
across versions and routing changes. Automation and project state need an
identity that survives reordering and re-instantiation; that is the string
`ParameterId`. The numeric `RuntimeParamKey` is retained only as a bounded RT
handle derived at prepare time.

### One combined descriptor used directly on the audio thread

Rejected: descriptors carry `String`s and `Vec`s (labels, paths), which must not
be indexed on the callback. Splitting into a string-free `RuntimeParameter`
projection is what makes the RT lookup allocation- and string-free.

## Validation and revisit triggers

The decision is validated when:

- `cargo test -p param_manifest` proves manifest (de)serialization round-trips, a
  version/compatibility rule, normalized↔engine mapping, lookup-by-ID, and zero
  allocations/deallocations around continuous and discrete RT lookup/conversion
  (including invalid keys and special float inputs); and
- [#101](https://github.com/jpalvarezl/blight-synth/issues/101) consumes
  `ParameterLookup`/`RuntimeParamKey` under ADR 0005's generation-bound normalized
  publication contract, and the OSC adapter's `/param/set gain` path binds to the
  `gain` descriptor instead of `normalized_gain_to_db`.

Revisit with a superseding ADR if: a parameter needs a mapping/kind that does not
fit `Mapping`/`ParameterKind`; the RT lookup's dense-index model conflicts with
#137's voice/capacity model; or #101/#134 reveal that the three automation-rate
classes are insufficient to express real traffic.

## Related

- Owning issue: [#121](https://github.com/jpalvarezl/blight-synth/issues/121)
- New crate: `param_manifest/`
- [Real-time audio contract](../architecture/realtime-contract.md) ("Control
  traffic classes", "Prepared-state rule")
- [ADR 0001](0001-product-and-host-priorities.md),
  [ADR 0002](0002-device-host-osc-split.md)
- Consumers/related: #101 (coalesced continuous parameters), #145/#134
  (timestamped events), #137 (polyphony/capacity)
- Additive amendment: [ADR 0005 — Coalesced parameter publication and lifecycle](0005-coalesced-parameter-publication.md)
