---
title: ADR 0008 — Portable engine-state envelope and restore boundary
summary: Define one canonical versioned project envelope, compatibility policy, and NRT-prepared block-boundary restore shared by standalone and optional plugin hosts.
status: accepted
updated: 2026-08-08
issues: [138, 241, 242, 243, 132]
supersedes: []
amends: []
---

# ADR 0008 — Portable engine-state envelope and restore boundary

## Status

Accepted. Deciding issue: [#241](https://github.com/jpalvarezl/blight-synth/issues/241).
Implementation is split between [#242](https://github.com/jpalvarezl/blight-synth/issues/242)
and [#243](https://github.com/jpalvarezl/blight-synth/issues/243). This ADR defines
portable project state, not the complete Engine lifecycle owned by
[#132](https://github.com/jpalvarezl/blight-synth/issues/132).

## Context

The current tracker persists an unversioned `Song` directly as serde JSON or
bincode. It uses process-sized `usize` indexes and sentinels, embeds legacy
instrument variants and raw sample data, and has no envelope migration or
forward-kind behavior. `node_registry` now provides stable kind IDs, independently
versioned instrument/effect definitions, an NRT v1→v2 instrument migrator, and
structured preparation errors. ADR 0004 makes stable `ParameterId` plus normalized
`0..1` values the host/project boundary; dense runtime keys are not stable state.

Standalone project files and optional JUCE state need the same model without
making the core open files or know JUCE. Restore must also preserve the accepted
RT rule: parsing, migration, asset decoding, factory work, and destruction happen
on NRT, while RT performs only a bounded prepared-state swap and retirement.

## Decision

### 1. M1 envelope

The current envelope schema is version 1. Its logical shape is:

```text
PortableStateV1 {
  schema_version: 1,
  composition: { kind, schema_version, payload },
  instruments: [InstrumentDefinition],
  master_effects: [EffectDefinition],
  parameter_values: [{ target, parameter_id, normalized_value }],
  routing: { kind, schema_version, payload },
  assets: [{ asset_id, sha256, media_type }]
}
```

Every nested definition or tagged payload owns its version independently. The
top-level version changes only when envelope meaning or shape changes; adding a
new supported composition/node/routing kind does not by itself change it.

The sole M1 composition kind is `blight.composition.tracker`, version 1. Its
payload carries the authored current `Song` semantics: name, initial tempo and
speed, arrangement, chains, phrases, and tracker events. It references instrument
instance IDs but does not duplicate instrument definitions or sample bytes.
Empty arrangement/chain slots are JSON `null`, not `usize::MAX`, and persisted
indexes/IDs have explicit bounded integer widths. The existing direct `Song`
JSON/bincode formats are legacy import inputs, not this canonical payload.

`composition` remains a string-tagged `{kind, schema_version, payload}` rather
than a closed Rust enum. A future ORCA-like, hybrid, or other runtime adds a new
kind and independently versioned payload without changing DSP or the envelope.
The payload is opaque until dispatched to that kind's NRT decoder/migrator.

`instruments` and `master_effects` use `node_registry` definitions, including
each definition's `schema_version`, stable kind, stable instance ID, constructor
payload, and deterministic effect order. Instrument-owned effect definitions
remain nested under their instrument. Instance IDs are unique in their scope;
all references must resolve exactly once.

Saved manifest parameters are a sorted overlay keyed by a typed stable node
address plus stable `ParameterId`; runtime table keys and effect-vector indexes
are never persisted. Values are finite normalized numbers in `0..=1` (discrete
values use their manifest ordinal). ADR 0004's manifest remains the sole mapping
to engine units. A definition constructor payload may still carry the node's
required initial value; when an overlay entry exists, its mapped value is
unambiguously applied after construction and before publication. Parameters not
yet represented by the manifest remain solely in the versioned constructor
payload. Missing saved values take the current descriptor default; unknown saved
IDs are a compatibility diagnostic, not silently discarded.

M1 routing uses `blight.routing.fixed_bus`, version 1, with an empty payload. It
means each instrument's ordered inserts feed the instrument mix, all instruments
feed the master bus, and the ordered `master_effects` list follows it. This is a
versioned topology reference, not an arbitrary graph. Issue #136 may introduce a
new routing kind/payload and migration; it must not reinterpret this marker.

An asset reference contains a stable project-local ID, lowercase SHA-256 content
digest, and media type. Node payloads refer to the stable ID. The envelope never
contains an absolute path, bookmark, file descriptor, or host URI. An NRT asset
resolver supplies and hash-verifies bytes. Standalone packages may store blobs
under digest-derived relative entries; a JUCE adapter must embed required blobs
or another self-contained digest-keyed bundle rather than assume it can reopen a
standalone path. Packaging is outside the envelope schema.

There is no generic seed or checkpoint field. A composition payload includes a
versioned seed/checkpoint only when that runtime's documented deterministic
replay promise cannot reconstruct state from its document. The current tracker
v1 payload needs neither and restores stopped at its document origin.

### 2. Explicit exclusions

Portable state is authored/reconstructible project state, not a seamless DSP
memory image. Version 1 excludes:

- active voices, note ownership, oscillator/envelope phase, and DSP scratch;
- delay/reverb tails, filter history, resampler history, and smoothing progress;
- live transport/playhead, callback/event queues, pending controls, telemetry,
  and UI selection/undo state;
- sample rate, block size, channel/device selection, CPAL handles, plugin host
  handles, clocks, filesystem paths, and network state; and
- prepared factories, decoded DSP owners, runtime parameter keys, and other
  process-local caches.

After restore, the host supplies device/lifecycle configuration, the composition
starts stopped, no voice is active, and tails are silent. A later seamless-session
feature requires a new reviewed state kind/version; it is not implicit here.

### 3. Canonical bytes and ordering

Canonical core bytes are UTF-8 JSON Canonicalization Scheme bytes (RFC 8785): no
BOM, insignificant whitespace, or trailing newline. Input JSON must satisfy the
I-JSON constraints required by JCS: duplicate object keys and non-finite numbers
are invalid. This choice is inspectable, host-neutral, and defined independently
of Rust serde/bincode implementation details.

Arrays with semantic order preserve it, including tracker rows/events and every
instrument/master effect chain. Set-like arrays have required order before
encoding: instruments by typed instance ID, parameter values by typed target then
`ParameterId`, and assets by `asset_id`; duplicates are invalid. JCS orders object
keys. Persisted numeric IDs/indexes are at most `2^53 - 1`; a future wider identity
must use a canonical decimal string rather than a JSON number. A current valid
model has exactly one canonical encoding, so repeated encode and decode→encode
produce identical bytes. Asset blob bytes are hash-identified but are not part
of the core envelope bytes.

### 4. Compatibility and migration

Writers emit only the current top-level and nested versions. Readers accept the
current version and explicitly implemented prior versions; they never infer
compatibility from a lower number or best-effort deserialize a newer version.
Migration is an ordered, deterministic, NRT-only chain of one-version steps.
Each step validates its input and output, retains stable IDs, and has committed
source/canonical expected-output fixtures. Version 1 is the first envelope, so
#242 proves the prior-persistence path with a committed legacy direct-`Song` JSON
→ canonical V1 import fixture and reuses the existing node-definition v1→v2
fixture. When envelope V2 exists, support requires an explicit V1→V2 fixture.

Envelope migration dispatches nested composition, node, routing, and parameter
migrations; legacy import is a named source-format adapter, and node migrations
reuse `node_registry`. New parameter IDs default.
Removed/renamed IDs require an explicit state migration consistent with ADR
0004; IDs and kind strings are never reused for new meaning. A mapping change
that would reinterpret a normalized value is breaking and requires migration.
Migration is transactional: no active state or caller-owned source is changed
until the complete current model validates and prepares.

Known-schema unknown fields are invalid so a read/write cycle cannot silently
erase data. A newer top-level or nested schema is unsupported, even if its JSON
shape appears readable. Additive evolution therefore either uses a field already
defined as optional or introduces a new schema version and migrator.

### 5. Diagnostics and source preservation

Decode/restore failures return structured NRT diagnostics with a stable code,
stage (`decode`, `migrate`, `validate`, `resolve`, or `prepare`), JSON pointer or
byte offset where available, and relevant kind/version/instance/asset identity.
At minimum the contract distinguishes:

- unsupported envelope version;
- unknown or unsupported-version composition kind;
- unknown or unsupported-version routing or node kind;
- unknown parameter ID or invalid normalized value;
- missing asset and digest/media/decode mismatch; and
- malformed UTF-8/JSON, duplicate keys, invalid references, capacity excess, or
  otherwise corrupt payload.

The result owner retains exact immutable input bytes on every failure (for
example by accepting/returning an `Arc<[u8]>` or taking equivalent ownership), so
they outlive the restore call without depending on a borrowed caller buffer. For
syntactically valid unknown future kinds it also retains the opaque payload.
Callers can report, copy, or export the source unchanged; they must not replace
it with a lossy partially decoded model. Failed preparation leaves the active
project untouched. No unknown node, field, or composition payload is skipped to
produce partial audio.

### 6. Snapshot, preparation, and RT handoff

Snapshot construction runs on NRT from the authoritative project/composition
model; it does not inspect live DSP objects. For a saved live coalesced value,
the NRT snapshot projection reads ADR 0005's active-generation, packed
applied-confirmed normalized value/revision and resolves it back to the stable
binding. Authored sample-event automation remains in the composition document,
not a transient last-event value. Pending unconfirmed edits are not silently
serialized; #243 must take a generation-consistent read or report that snapshot
cannot yet complete. If a future runtime requires a deterministic checkpoint,
RT may publish only a bounded preallocated checkpoint token at a block boundary;
expansion and encoding remain NRT. Tracker v1 requires no such request.

Restore runs the complete byte decode, migration, validation, asset resolution
and decoding, manifest binding, node factory construction, routing compilation,
and composition-runtime preparation on NRT. Success yields one generation-tagged
`PreparedGeneration`: immutable snapshot/resources may be shared, while the live
composition/Engine graph is uniquely owned so RT can mutate it without locks or
shared-`Arc` interior mutation. After publication NRT does not mutate it.

A bounded structural handoff publishes that single owner. RT either pointer-swaps
the whole generation at the start of one top-level render block or installs none;
it never exposes a new composition with an old graph or partial parameters.
This deliberately replaces today's split `Player.song`/Engine graph ownership in
#243; #132 may later wrap the same invariant in its public lifecycle. The
displaced uniquely owned generation uses the existing `RetireSink`/`RetiredState`
RT→NRT handoff and preallocated overflow policy and is destroyed on NRT. If the
current `RetiredState::Prepared(Arc<...>)` cannot carry the unique mutable owner,
#243 adds a dedicated generation variant rather than placing mutation behind an
`Arc`; it does not add a second reclamation channel. Handoff saturation is
observable and retried/failed on NRT, never waited on RT.

No callback path opens files, parses bytes, migrates schemas, resolves or decodes
assets, allocates factories, compiles routing, formats diagnostics, or destroys a
retired owner. This is additive to #132's eventual prepare/reset/suspend/resume
API and does not define a competing Engine lifecycle.

### 7. Core and host adapters

The shared core owns model types, validation/migration, canonical byte
encode/decode, diagnostics, asset-resolver interfaces, and preparation input/output.
It has no filesystem, CPAL, OSC, UI, or JUCE dependency.

A standalone adapter chooses filenames, atomic writes, package layout, relative
asset storage, autosave, and legacy `Song` import. A future JUCE adapter copies
the same canonical core bytes (plus required digest-keyed asset bundle) to/from
`getStateInformation` memory and supplies host lifecycle configuration. Neither
adapter defines a second schema, normalization rule, migration chain, or DSP
snapshot. #243 implements only the core bytes/prepared restore boundary; concrete
standalone filesystem UX and JUCE integration remain separate work.

## Consequences

### Positive

- Tracker state has a deterministic migration path while future composition
  kinds remain additive and opaque until supported.
- Node definitions, normalized controls, routing identity, and assets have one
  portable authority shared by standalone and optional plugin hosts.
- Restore is atomic at a render-block boundary and reuses proven NRT retirement.
- Unsupported/corrupt projects remain recoverable as exact source bytes.

### Costs and risks

- RFC 8785 and duplicate-key/I-JSON validation need a deliberate implementation;
  plain `serde_json::to_vec` is not the canonical encoder.
- Asset packaging is host-specific, so adapters must prove self-contained export
  rather than treating a digest reference alone as available content.
- Constructor payload plus manifest overlay temporarily duplicates some values;
  preparation order must keep overlay precedence explicit until manifest coverage
  can remove the duplication.
- Excluding voices, tails, and playhead makes restore deterministic but not
  seamless or sample-continuous.

## Alternatives considered

### Keep direct serde JSON/bincode `Song` persistence

Rejected: it is unversioned, architecture-dependent in places, tracker-closed,
and mixes documents, legacy nodes, and sample bytes without migration behavior.

### Persist prepared DSP objects or host state

Rejected: these objects are process/device dependent, include ephemeral history,
and cannot be restored or destroyed safely as portable state.

### Let each host choose its own serialization

Rejected: standalone and plugin state would drift in IDs, normalization,
compatibility, and diagnostics. Hosts may package bytes, not redefine them.

## Validation and revisit triggers

#242 validates canonical tracker round-trip, byte equality, legacy `Song`→V1
import, nested node migration, stable ordering, and every required diagnostic
with unchanged source bytes. #243 validates hardware-free NRT preparation,
all-or-nothing block-boundary generation replacement, handoff saturation, NRT
retirement/destruction, and zero filesystem/heap/destructor work on RT.

Revisit with a superseding ADR if JCS cannot satisfy measured state-size or
cross-language requirements, #136 cannot version its topology under the routing
payload, a composition runtime proves a generic checkpoint field is necessary,
or a product requirement demands seamless voice/tail/playhead restoration.
