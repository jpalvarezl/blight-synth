---
title: ADR 0005 — Coalesced parameter publication and lifecycle
summary: Continuous parameters use a generation-bound, normalized MPSC atomic store; RT maps and latches engine-owned smoothing targets once per render block with eventual-latest and applied-target confirmation semantics.
status: accepted
updated: 2026-08-03
issues: [101, 121, 201, 212]
supersedes: []
amends: ["0004"]
---

# ADR 0005 — Coalesced parameter publication and lifecycle

## Status

Accepted

Deciding issue: [#212](https://github.com/jpalvarezl/blight-synth/issues/212).

This decision adds the publication, replacement, application, and host-state
contract intentionally left open by [ADR 0004](0004-parameter-manifest.md). Where
ADR 0004 described OSC or another NRT adapter mapping a `ControlCoalesced` value
before submission, this ADR replaces that narrower statement: the coalesced
store carries normalized values and the RT consumer maps them through the
prepared runtime table. ADR 0004 remains authoritative for manifest metadata,
validation, stable `ParameterId`, and mapping math.

## Context

The accepted [real-time audio contract](../architecture/realtime-contract.md)
requires continuous controls to coalesce without filling the structural command
queue, but it did not define enough of the mechanism to implement one shared
path safely:

- the standalone host currently serializes OSC through one NRT control worker,
  while a future MIDI bridge, editor bridge, or JUCE/APVTS adapter may publish
  concurrently;
- ADR 0004 said both that adapters submit mapped engine values and that smoothing
  is engine-owned, leaving the mapping/application boundary ambiguous;
- `RuntimeParamKey` is only a dense index in one prepared table, so a bare key can
  silently address a different parameter after table replacement;
- a value atomic plus a dirty bit needs an explicit ordering protocol to avoid a
  lost final write;
- queue acceptance is not proof that RT has applied a parameter target, while the
  product topology requires desired, pending, and engine-confirmed state; and
- the implemented `SampleEvent` lane already carries mapped engine values at an
  exact sample offset and must not become a second route for coalesced controls.

The decision must preserve bounded callback work, prepared-state retirement, and
one mapping and smoothing owner without requiring a transport-specific store.

## Decision

### 1. Producer cardinality and arbitration

One engine instance exposes one prepared coalesced store generation. The store
is **multi-producer, single-consumer (MPSC)**:

- The first-party standalone host continues to have one NRT control owner. OSC,
  editor, and first-party MIDI requests are serialized there before publication,
  preserving that host's deterministic accepted-request order.
- Generation-bound publisher handles may be cloned for future non-audio-thread
  adapters such as editor/APVTS message-thread changes or independently
  scheduled MIDI. Concurrent publishers are supported; they do not need an
  extra RT-facing queue or lock. DAW automation delivered with sample offsets
  is not such a producer and remains on the event lane.
- An atomic value store is the publication linearization point. Concurrent writes
  to one slot are ordered by that atomic's modification order; the later value in
  that order wins. Completion order of racing calls is not a priority rule. An
  adapter needing source priority or deterministic cross-source arbitration must
  perform it on NRT before publication.
- Publishers are NRT/control-side objects. An audio callback does not publish
  back into its own store. A plugin receiving sample-offset automation uses the
  `SampleEvent` lane; block/control-rate host values may be staged by its NRT host
  adapter under this contract.

This supports the real first-party single-writer topology without freezing a
single-writer API that future adapters would have to bypass.

### 2. Generation-bound prepared state

NRT prepares and hands off one indivisible `PreparedParameterState` containing:

- a validated `RuntimeParameterTable`;
- concrete engine target bindings for its `ControlCoalesced` entries;
- a prepared `RuntimeParamKey` → compact coalesced-slot binding plus one packed
  normalized-value/publication-revision slot, one packed applied-target
  confirmation slot, and one dirty bit per active coalesced entry;
- engine-owned smoothing state; and
- a nonzero, monotonically increasing `ParameterTableGeneration` unique for the
  engine instance.

`RuntimeParamKey` remains the unchanged dense table index from ADR 0004. During
preparation, each active `ControlCoalesced` key is also assigned a compact
`CoalescedSlotIndex` in stable runtime-key order; this internal ordinal sizes the
atomic slots/dirty bitmap and is never a manifest or cross-generation identity.
Every public publish/bind/confirmation operation pairs the runtime key and slot
with its generation, usually inside a generation-bound handle. A bare key or bare
slot is never a cross-generation API. Generation values do not wrap or get
reused; exhaustion rejects preparation and requires a new engine instance.

A replacement or semantic reset follows this lifecycle:

1. NRT closes the old generation with `accepting.store(false, Release)`. A
   publisher checks `accepting` with `Acquire` before its slot CAS and again after
   setting dirty. A racing call may conservatively return `StaleGeneration` even
   if it touched the physically separate old slot; no close operation waits or
   spins for a preempted publisher.
2. NRT resolves stable `ParameterId` values again, prepares a complete new table,
   slots, bindings, initial normalized snapshot/defaults, and smoothers, and
   assigns a new generation. State is migrated by `ParameterId`, never by old
   dense-key position.
3. RT swaps the complete state at a render-block boundary. Every initial
   coalesced value is considered dirty and is latched before that block renders.
4. RT returns the complete displaced state as one `RetiredState::Prepared`
   owner through the existing bounded RT-to-NRT retirement path. If that ring is
   full, the callback retains the owner in its preallocated pending-retirement
   storage and pauses further structural installation until NRT headroom returns;
   it never drops table/store/smoother ownership. NRT-only publisher handles may
   retain shared old-store ownership, and their last release must also occur on
   NRT.
5. NRT publishes the generation transition, rebinds adapters by stable ID, and
   replays any still-desired values that were not represented by the prepared
   initial snapshot.

An old-generation write completed before closure may be accepted. A call racing
closure may return `StaleGeneration`/`Closed` even if its old slot briefly
changed. In both cases it can mutate only the physically separate old store and
can never address the new table. A write accepted immediately before closure may
be coalesced away by replacement rather than confirmed; the observable
generation transition cancels that pending state and tells the adapter to
rebind/replay. Removed IDs produce an observable rebind failure instead of being
reinterpreted at the same numeric key.

There is no in-place dirty-bit clear while publishers are active. Reset uses the
replacement protocol above, making its ordering, initial values, and
observability deterministic.

### 3. The store carries normalized values

Each publication slot is one lock-free `AtomicU64` packing a generation-local,
nonzero `u32` revision and the canonical finite normalized `f32::to_bits()`.
Finite input is clamped to `0.0..=1.0`, and the canonical clamped value is
returned to the publisher with its revision. NaN and infinities are rejected as
`InvalidValue`; they are not silently converted into a parameter change. A slot
at `u32::MAX` rejects further publication as `RevisionExhausted`; NRT replaces
the generation before reuse, so revisions never wrap or become ambiguous.

Normalized storage is selected because it:

- is the common OSC, UI, MIDI-mapping, and APVTS boundary;
- lets desired and confirmed state compare in the same host-visible domain;
- prevents each adapter from duplicating mapping logic;
- ensures a value is interpreted by the exact runtime table generation that owns
  its dense key; and
- keeps mapped engine values and smoother state together on the RT application
  side.

`RuntimeParameterTable::normalized_to_engine` (and the underlying canonical
`Mapping`) is the **sole normalized-to-engine mapping owner**. For a coalesced
value, RT calls it after observing a dirty slot. Adapters, the store, engine
bindings, and DSP setters do not duplicate conversion. The conversion is
bounded, nonallocating prepared-table work already permitted by the RT contract.

The engine parameter application layer is the **sole smoothing owner**. It owns
one prepared smoother for each smoothed coalesced binding, receives the mapped
engine target, advances the ramp while rendering, and sends engine values to the
DSP setter. Adapters and the store never smooth. A DSP node migrated to this path
must not apply a second de-zipper to the same manifest policy.

`SmoothingPolicy::Smoothed` is valid only for `ControlCoalesced` parameters.
`SampleEvent` and `Structural` descriptors must use `SmoothingPolicy::None`;
manifest preparation rejects the contradictory combination. This reflects the
implemented sample-event binding, which contains no smoother.

### 4. Exact publication and dirty-bit memory ordering

This is the implementation target for #213–#215, not a claim about the current
transitional command path. The initial implementation uses one lock-free
`AtomicU64 publication_word` per coalesced slot, one lock-free
`AtomicU64 applied_word` per slot, and a fixed array
of `AtomicU64` dirty words. Packing revision and value makes both publication and
confirmation coherent under concurrent writers. Platforms where these atomics
are not lock-free reject strict RT preparation. The following operations are
contractual (stronger orders may be used only with justification):

**Publisher for prepared `(key k, coalesced slot s)`:**

1. validate active generation, the prepared key→slot binding, traffic class,
   writability, normalized input, and `accepting.load(Acquire)`;
2. use a `compare_exchange_weak` loop with `Relaxed` success/failure ordering to
   replace `publication_word[s]` with `(old_revision + 1, canonical_bits)`; the
   successful CAS is the publication linearization point and the loop runs only
   on NRT;
3. `dirty[word(s)].fetch_or(mask(s), Release)`;
4. recheck `accepting.load(Acquire)` and return `StaleGeneration` if closure won
   the race; otherwise return `Accepted { generation, key, revision,
   canonical_normalized, replaced_pending }`, where `replaced_pending` is derived
   from the previous dirty bit.

**RT consumer at the start of every engine render block:**

1. visit every fixed dirty word exactly once and execute
   `word.swap(0, Acquire)`;
2. for every returned set bit/slot `s`, resolve its prepared runtime key/target
   and load `publication_word[s]` with `Relaxed` ordering;
3. map its normalized value, latch/apply the smoothing target, then store that
   exact packed `(revision, normalized_bits)` to `applied_word[s]` with `Release`
   ordering; and
4. let NRT observers load `applied_word[s]` with `Acquire` ordering.

The release/acquire dirty RMW publishes the preceding successful slot CAS. The
slot atomic's modification order defines the winner among concurrent publishers.
The dirty operation must remain an RMW and consumption must remain one acquire
RMW clear; replacing either with a plain load/store invalidates this proof and
requires a new analysis. Clearing the bit before loading the slot is intentional:

- a publication before the swap is represented by the returned bit;
- a publication after the swap sets the bit for the following block;
- a publication between the swap and value load may be observed immediately and
  still leaves a redundant bit for the following block; and
- a publication that finds an already-set bit may be coalesced, but the consumer
  loads the slot only after acquiring that bit and therefore observes that value
  or a later one.

Consequently, after publishers quiesce, the final successfully published word
is applied by the first control boundary whose acquire swap observes the final
publisher's release or a later dirty-word RMW in its release sequence. If a
boundary's swap raced before that release, the bit remains set and the following
boundary observes it; a racing boundary may already load the final slot word and
make the later bit redundant. Intermediate values may be omitted. Continuous
publication may keep replacing a slot indefinitely, so the contract guarantees
**eventual latest after quiescence**, not that every value is applied.

The M1 hard capacity is at most `MAX_PARAMETER_COUNT` (16,384) table keys and at
most **1,024 `ControlCoalesced` entries**. Compact coalesced slots mean RT scans
at most 16 dirty words and performs at most 1,024 applications once per render
block. Preparation
may configure a lower coalesced limit but rejects a higher one. Raising either
limit requires renewed callback-budget measurements and a contract update; there
is no queue drain or RT retry loop. The atomic protocol receives a concurrency
model test (for example Loom) in addition to stress tests so the load-bearing
release-sequence assumptions cannot silently regress.

### 5. Host-visible desired, pending, and applied-confirmed state

Host adapters maintain three distinct concepts, all normalized and keyed by
stable `ParameterId` outside the prepared runtime:

- **Desired** is the adapter's newest local/user intent. It may exist before a
  binding is available and remains adapter-owned. After successful publication,
  comparison uses the returned canonical clamped value and revision, never the
  raw input.
- **Pending** means the desired publication revision is newer than the active
  generation's applied revision and has not been superseded. Equal revision is
  confirmed; a greater applied revision means another publication won and the
  local attempt terminates as **superseded**, not indefinitely pending. A
  generation transition, rejection, revision exhaustion, or disconnection
  cancels that publication attempt but not desired intent; the adapter rebinds
  and republishes or reports failure.
- **Applied-confirmed** is the packed revision and normalized target that RT successfully
  mapped and latched into the engine application layer for the active generation.
  RT writes confirmation only after target application succeeds. It is engine
  authority for the target, but it does **not** mean a nonzero smoothing ramp has
  reached its final audible value. Settled/ramp-progress telemetry is a separate
  future feature if needed.

Because the contract is latest-value-wins, confirmation is the current target,
not an acknowledgement queue for every intermediate request. If another producer
wins, its greater revision and normalized value appear in applied-confirmed
state; the losing attempt becomes superseded. Whether the adapter keeps a
separate unsatisfied desired intent is adapter policy. It must not create an
automatic feedback war by blindly republishing every observed external change.

Transport receipt, atomic publication, and RT application are separate events.
In particular, the standalone `/param/echo` migration must represent
applied-confirmed state, not merely control-worker or structural-queue
acceptance. An optional immediate protocol response may report publication
acceptance, but must use a different meaning/name.

### 6. Invalid input, overload, reset, and application failure

Publication returns a compact NRT-visible result. Rejections include at least:

- `StaleGeneration`/`Closed`;
- `InvalidKey` or key not classified `ControlCoalesced`;
- `ReadOnly`;
- `InvalidValue` or `RevisionExhausted`; and
- `Disconnected`/retired engine instance.

Validation rejections before the slot CAS do not set a dirty bit. A closure-race
call may conservatively return `StaleGeneration` after setting only its physically
separate old-generation bit, as defined in the replacement protocol; it cannot
dirty the new generation. Invalid/stale/disconnected counts are available as
bounded atomic counters or equivalent NRT telemetry so transports that cannot
return a synchronous rich error remain diagnosable. Rich strings and logging
stay NRT.

Coalescing is normal load handling, not queue overflow. Publication does not
return `Full`: a valid non-exhausted slot accepts a newer value. `Accepted`
reports whether it replaced an already-dirty value, and a saturating
coalesced-write counter makes sustained producer pressure observable.
Manifest/store capacity failure occurs during NRT preparation. The complete
dirty set still has the fixed worst-case callback cost described above.

Prepared binding/application failure is not silently confirmed. RT records a
compact per-generation/key failure status or saturating counter, leaves the
applied slot unchanged, and continues bounded rendering. NRT formats and exposes
the diagnostic. A bad prepared target is a preparation defect to fix, not a
reason for RT allocation, logging, panic, or retry.

Reset is observable as an old-generation/new-generation transition plus the
initial applied confirmations. Closing a generation makes its pending values
stale; installation seeds all new coalesced slots from the prepared authoritative
snapshot or descriptor defaults, resets each smoother deterministically to the
mapped seed (no inherited dense-key state and no startup ramp), marks the seeds
dirty, and confirms them when RT latches the new state.

### 7. Relationship to `SampleEvent`

The traffic classes remain disjoint:

- `ControlCoalesced` values use this normalized, generation-bound store. They
  have no sample offset, may omit intermediate values, and are consumed once at
  the start of each engine render block.
- `SampleEvent` values are converted through the same generation's canonical
  `RuntimeParameterTable` during NRT event preparation, then carry an
  already-mapped engine value in `EngineEvent::SampleParameter`. They retain
  #201–#204 admission, offset, total ordering, and whole-block rejection rules.
  They bypass coalesced dirty storage and smoothing.
- `Structural` values use reliable prepared-state replacement and neither lane.

A descriptor has exactly one `AutomationRate`, so one parameter cannot race
through both the coalesced and sample-event lanes. Changing that classification
remains a breaking manifest compatibility change. Coalesced targets are latched
before processing offset-zero timestamped events for a block, without changing
#201's ordering among admitted events.

## Consequences

### Positive

- The current one-writer standalone path stays simple while the shared API safely
  supports future concurrent adapter publishers.
- Dirty publication has a small, testable memory-order protocol and a precise
  eventual-latest guarantee without RT retry, allocation, locking, or queue
  flooding.
- Stable IDs own rebinding while dense keys remain efficient and cannot be
  accidentally reused across generations.
- Mapping and smoothing each have one owner, and host state can distinguish
  accepted publication from engine-applied target confirmation.
- Sample-accurate automation remains on the already-implemented event path with
  unchanged ordering and overload behavior.

### Costs and risks

- Every prepared coalesced parameter needs a compact slot, packed publication,
  packed applied, and dirty atomic storage plus engine binding/smoother state.
- Up to 16 fixed dirty-word scans are paid every render block even when no values
  change; the 1,024 active-coalesced hard capacity must be benchmarked and
  audited.
- Concurrent writes to the same parameter are linearizable but intentionally do
  not define source priority. Adapters needing priority must add NRT arbitration.
- Applied-confirmed describes the accepted smoothing target, not completion of
  the audible ramp; UI/protocol language must preserve that distinction.
- Generation replacement requires adapter rebind/replay and retirement of the
  complete old state rather than independently swapping a table or store.
- A continuously hot slot can exhaust its `u32` revision (about 50 days at 1 kHz
  or 2.3 years at 60 Hz), forcing an observable prepared generation replacement;
  revision exhaustion is rejected rather than wrapped.

## Alternatives considered

### Keep a single prepared writer forever

Rejected. It matches today's standalone worker but forces future APVTS/MIDI/editor
adapters through a transport-specific serializer or a second competing store.
MPSC publication costs no RT contention because RT remains the sole consumer.

### Put mapped engine values in the atomic store

Rejected. Every adapter would need access to and correct use of the active
mapping generation, host confirmation would need inverse conversion, and stale
mapped values could outlive the table that interpreted them. Normalized storage
keeps the boundary stable and mapping next to RT application/smoothing.

### Map and smooth in every adapter

Rejected. It directly contradicts the canonical manifest and produces different
sound and state semantics across OSC, UI, MIDI, and plugin hosts.

### Use a bounded MPSC queue for continuous values

Rejected. A queue spends capacity and callback work on obsolete intermediate
values, requires overflow policy, and can delay the final knob position behind a
backlog. Timestamped and structural traffic already have separate mechanisms for
cases where intermediate order is meaningful.

### Reuse bare dense keys across table replacement

Rejected. Descriptor order may change, so stale writes could control a different
parameter. Pairing keys with non-reused generations and replacing the complete
prepared state makes stale behavior explicit.

### Use only relaxed atomics

Rejected. Atomicity alone does not publish the value store through the dirty bit
or prove that the final completed write is eventually observed. The release dirty
RMW and acquire clear establish the required handoff.

### Clear dirty bits in place during reset

Rejected. A concurrent publisher can lose its notification. Closing and replacing
the generation gives reset a single lifecycle order and reuses prepared-state
retirement.

## Validation and revisit triggers

The implementation is accepted when hardware-free tests prove:

- single- and multi-producer publication, same-slot races, already-dirty
  coalescing, and eventual latest after quiescence under the exact atomic orders;
- invalid values/keys/classes, stale/closed generations, replacement races,
  stable-ID rebinding, removed IDs, generation reset, publication-revision
  exhaustion, and nonwrapping generation exhaustion behavior;
- fixed-capacity dirty scanning, one application per dirty key per block,
  applied-target confirmation, application-failure telemetry, and no RT heap,
  lock, log, panic, or unbounded retry;
- canonical mapping and engine-owned smoothing, including deterministic reset,
  no duplicate DSP/adapter smoothing, manifest rejection of `Smoothed` outside
  `ControlCoalesced`, and compatibility checks for manifests accepted under ADR
  0004;
- unchanged `SampleEvent` offset/order/admission behavior; and
- standalone OSC normalized desired/pending/applied-confirmed behavior without
  the legacy hard-coded gain conversion or structural command.

Revisit with a superseding ADR if measurements show the fixed dirty-word scan
cannot meet callback budgets at the required prepared capacity; an adapter truly
needs deterministic cross-source priority inside the store; a platform cannot
provide the required atomic operations lock-free; or a product requirement needs
per-publication acknowledgement or smoothing-settled telemetry rather than
latest-target confirmation.

## Related

- [ADR 0004 — Canonical parameter manifest and host bindings](0004-parameter-manifest.md)
- [Real-time audio contract](../architecture/realtime-contract.md)
- [Product and host topology](../architecture/product-topology.md)
- [#101 — Coalesced real-time parameter pipeline](https://github.com/jpalvarezl/blight-synth/issues/101)
- [#121 — Canonical parameter manifest](https://github.com/jpalvarezl/blight-synth/issues/121)
- [#201 — Timestamped engine events](https://github.com/jpalvarezl/blight-synth/issues/201)
- [#212 — Coalesced parameter decision](https://github.com/jpalvarezl/blight-synth/issues/212)
