---
title: ADR 0002 — Shared device host versus OSC standalone transport adapter
summary: Separate a reusable in-process CPAL device host from the OSC-controlled standalone process, so OSC is a transport adapter over the same typed control boundary rather than an owner of device-host semantics.
status: proposed
updated: 2026-07-24
issues: [185, 181, 182, 139, 161, 101, 134, 145]
supersedes: []
---

# ADR 0002 — Shared device host versus OSC standalone transport adapter

## Status

Proposed

Deciding issue: [#185](https://github.com/jpalvarezl/blight-synth/issues/185).
This ADR records a target boundary and migration plan only. It does **not**
rename or split modules, change Cargo features, or alter engine/DSP semantics;
that implementation is scheduled follow-up work (see
[Validation and revisit triggers](#validation-and-revisit-triggers)).

## Context

`audio_backend`'s optional-default `standalone` feature currently bundles two
concerns that have different owners and lifecycles:

1. A reusable **in-process device host** that owns the CPAL output device, the
   real-time callback adapter, the bounded command/retirement rings, metering,
   and the NRT factories/resources used to prepare state. This is the layer a
   Rust client submits control to.
2. An **OSC-controlled standalone process** that decodes UDP/OSC packets,
   streams meters, translates protocol messages into engine commands, and owns
   process readiness/shutdown plus a temporary current-thread Tokio runtime.

Today both live under `audio_backend/src/standalone/` behind one `standalone`
feature. Evidence for the boundary:

- `standalone/audio_frontend/blight_audio.rs` — `BlightAudio` builds the CPAL
  stream, the command ring (`SharedRb<Command>`), the retirement ring, meter,
  and factories, and exposes the typed submission API
  (`try_send_command` / `send_command` / `send_command_until` / `reclaim_retired`
  / `meter_state` / factory accessors).
- `standalone/audio_processor/mod.rs` — the RT callback adapter and bounded
  per-block command budget; host-neutral (has a `channels == 0` path for
  "future non-CPAL hosts").
- `standalone/meter.rs` — `MeterState` / `MeterLevels`, RT-safe atomics; the
  callback writes, a reader streams.
- `standalone/osc.rs` — `OscServer`, packet dispatch, `/param/set` gain
  normalization, `/song/load`, `/transport/*`, `/meter/level` dBFS encoding.
  This is transport mapping onto already host-neutral command types
  (`MixerCmd`, `TransportCmd`).
- `standalone/control_worker.rs` — `StandaloneControlWorker`, the dedicated NRT
  owner of `BlightAudio` (CPAL streams are intentionally not `Send`), with an
  OSC-specific request enum and protocol-acknowledgement policy.
- `bin/dsp-core.rs` — the process: spawns the control worker, binds OSC, prints
  the `READY` stdout contract, and awaits ctrl-c.
- `Cargo.toml` — one `standalone` feature pulls `cpal`, `ringbuf`, `rosc`,
  `tokio`, `env_logger`, the `dsp-core` binary, and every example.

The tension: the Rust tracker only needs concern (1). It already runs its own
dedicated NRT worker (`tracker_gui/src/audio.rs::AudioManager`) that owns a
`BlightAudio` and submits `Command`s. Yet to reach `BlightAudio` it must compile
`rosc` and `tokio` because they share one feature. Meanwhile `#161` (Tokio
removal) and `#101`/`#134` (command-traffic replacement) will churn the control
plane; without a recorded boundary there is a real risk that OSC-specific
lifecycle logic accretes shared device-host semantics.

The intended architecture is analogous to SuperCollider's separation between the
synthesis server / control core and its clients: there is one shared
synthesis/control core, an in-process typed client (the tracker), and a
network/OSC client that is a *transport adapter* to the same control boundary —
not a second, privileged owner of device-host behavior.
[System boundaries](../architecture/system-boundaries.md) ("Dependency direction",
"Parallelization boundary"), the [standalone host domain](../domains/standalone-host.md)
("Threading/runtime decision", "Feature boundary"), and the
[real-time audio contract](../architecture/realtime-contract.md) ("Thread roles",
"Backpressure and overload") already point this direction; this ADR makes the split
explicit and names it.

## Decision

Adopt a two-layer target boundary inside `audio_backend`, with OSC positioned
strictly as a transport adapter over the shared control interface.

### 1. Shared layer name: `device_host`

The reusable in-process host is named **`device_host`** (host-neutral role),
not `cpal_host`.

Rationale: the module names the *role* — owning the audio device, the RT
callback, and the command/control boundary — not the backend vendor. CPAL is one
device backend; a future JACK, JUCE-embedded, or offline "device" could back the
same host without a misleading module name. This mirrors SuperCollider's
"server/control core" being independent of any single driver.

### 2. Typed in-process control interface: `BlightAudio` over the `Command` envelope

The **typed in-process client/control interface** that both the tracker and the
OSC adapter target is `BlightAudio` plus the already host-neutral `Command`
envelope (which re-exports `engine::InstrumentCmd`, `engine::MixerCmd`, and
`audio_backend::SequencerCmd` / `TransportCmd`). Submission uses the existing
nonblocking/reliable/cancellable API and the bounded SPSC command ring, with
`MeterState` and the factories as the accompanying handoff surfaces.

This is the project's Rust-side analog of the UI's `EngineClient`: a typed,
host-neutral control handle. Both first-party clients are NRT owners that submit
through it:

- the tracker's `AudioManager` worker (in-process typed client), and
- the standalone `StandaloneControlWorker` (owned by the OSC adapter).

OSC decodes packets to the *same* `Command`s; it introduces no control semantics
of its own.

### 3. Type ownership

| Concern | Owner | Notes |
|---|---|---|
| `BlightAudio` (device/stream setup, submission API, factory/meter accessors) | `device_host` | The typed control handle |
| RT callback adapter (`AudioProcessor`, per-block command budget, retirement) | `device_host` | Already host-neutral |
| Command ring + retirement ring (`SharedRb`, `CommandSender`, `CommandSubmission*`) | `device_host` | Bounded RT/NRT handoff |
| `MeterState` / `MeterLevels` (RT-safe telemetry atomics) | `device_host` | Callback writes; any reader consumes |
| Factories (`InstrumentFactory`, `VoiceFactory`, `EffectFactory`) + `ResourceManager` handoff | `device_host` | Preparation surfaces |
| Dedicated-NRT-owner *pattern* for a non-`Send` stream | `device_host` concern | Concretely realized per client |
| `OscServer`, packet dispatch, protocol mapping, gain/dBFS conversions | `osc`/standalone-process | Transport encoding only |
| `StandaloneControlWorker` request enum + protocol-acknowledgement policy | standalone-process | OSC-specific NRT translation |
| Readiness (`READY` stdout), ctrl-c/shutdown, process wiring (`dsp-core`) | standalone-process | Process lifecycle |
| Temporary Tokio current-thread runtime (UDP, meter cadence, response poll) | standalone-process | Contained; see §5 |

`commands`, `SequencerCmd`/`TransportCmd`, `resources`, and `song_hydration`
already compile without the `standalone` feature and are shared with offline
rendering; this split does not move them. It only draws a line *within* the
current `standalone` module between the reusable host and the OSC process.

### 4. Cargo feature split

Replace the single `standalone` feature with a layered pair, keeping
`standalone` as a compatibility alias:

- **`device-host`** — enables `cpal` + `ringbuf` and compiles the `device_host`
  layer (`BlightAudio`, `AudioProcessor`, `MeterState`, factories). No `rosc`,
  no `tokio`. Rust clients (tracker, most examples) depend on this.
- **`standalone-process`** — depends on `device-host` and additionally enables
  `rosc` + `tokio` + `env_logger`, compiling `OscServer`, the OSC control
  worker, the `dsp-core` binary, and the OSC/device network examples.
- **`standalone` = `["standalone-process"]`** — retained alias so
  `default = ["standalone"]` and existing invocations keep working unchanged.

Naming: `standalone-process` is chosen over the narrower `osc-host` because the
layer owns process readiness/shutdown, not only OSC transport
(see [standalone host domain](../domains/standalone-host.md), "Standalone process
lifecycle"). `osc-host` remains an acceptable later synonym if a non-process OSC
embedding appears.

`audio_backend --no-default-features` semantics are unchanged: neither
`device-host` nor `standalone-process` compiles, so shared hydration, resources,
and deterministic offline rendering remain host-free.

#### Migration steps and compatibility impact

- **Tracker (`tracker_gui`)**: today `tracker_gui/Cargo.toml` depends on
  `audio_backend` with default features, so it transitively compiles `rosc` and
  `tokio` through `standalone`. The migration sets that dependency to
  `default-features = false` and enables only `device-host`. The tracker keeps
  its own NRT `AudioManager` worker and the `BlightAudio` submission path
  unchanged; only the compiled dependency surface shrinks. A `cargo tree`/build
  check on the tracker path must prove `rosc` and `tokio` are absent.
- **Examples**: examples that use only `BlightAudio`/factories/meter
  (`simple_setup`, `simple_song`, `cycle_waveforms`, `envelope`, `master_gain`,
  `voice_effects`, `sample_playback_from_file`,
  `sample_playback_from_gl_instruments`) move to
  `required-features = ["device-host"]`. Two device-host examples
  (`polyphonic_song`, `play_song_file`) additionally call `env_logger::init()`;
  because `env_logger` is owned exclusively by `standalone-process` (§4), they
  cannot compile under `device-host` alone. To keep §4 coherent, these two move
  to `required-features = ["standalone-process"]` (they want logging, so they are
  gated with the layer that owns `env_logger`). If a future change wants them on
  `device-host`, they must first drop or relocate their `env_logger::init()` call
  (e.g. behind a `standalone-process`-gated helper). OSC/device-network examples
  (`osc_control`, `meter_listen`) require `standalone-process`.
- **Example feature validation (for #190)**: after re-gating, each example must
  build under its declared feature set, e.g.
  `cargo build -p audio_backend --example simple_setup --features device-host`,
  `cargo build -p audio_backend --example polyphonic_song --features standalone-process`,
  `cargo build -p audio_backend --example play_song_file --features standalone-process`,
  and `cargo build -p audio_backend --example osc_control --features standalone-process`.
  A `device-host`-only build of `polyphonic_song`/`play_song_file` is expected to
  fail until `env_logger` is removed from them.
- **`dsp-core` binary**: requires `standalone-process` (unchanged behavior;
  `required-features` updates from `standalone` to `standalone-process`, which
  the `standalone` alias also satisfies).
- **`--no-default-features`/offline builds**: unchanged. No device or network
  module compiles; offline golden renders and shared hydration/resources are
  untouched.
- Implementation of these steps is tracked by
  [#190](https://github.com/jpalvarezl/blight-synth/issues/190).

### 5. Evolution under #161 and #101/#134

- **#161 (Tokio removal)** is contained entirely within `standalone-process`.
  Tokio only backs the OSC UDP loop, meter cadence, and shutdown poll. The
  `device_host` submission API is already runtime-agnostic (nonblocking +
  reliable-with-backpressure + cancellable), so removing or replacing the
  runtime touches only the adapter's I/O topology. `rosc` encoding is
  independent of the runtime choice and stays regardless.
- **#101/#134 (command-traffic replacement)** evolve the control plane *behind*
  the `device_host` interface. The transitional mixed `Command` queue and its
  64-item/block budget are replaced by coalesced continuous parameters (#101)
  and bounded timestamped events (#134). Because OSC only maps protocol messages
  onto whatever typed control classes the device host exposes, these changes are
  a `device_host`/engine concern; the OSC adapter updates its mapping without
  gaining semantics. This is the concrete guarantee that **OSC does not own
  shared device-host semantics**.

### Non-goals

- No module rename/split, feature rename, or example re-gating in this ADR's
  branch. The split above is the *target* the follow-up implements.
- No change to engine/DSP semantics, RT rules, or protocol behavior.
- No second player/synthesizer; the device host keeps delegating rendering to
  `engine` per [ADR 0001](0001-product-and-host-priorities.md).

## Consequences

### Positive

- Rust clients (tracker, in-process examples) can depend on the device host
  without compiling `rosc`/`tokio`, shrinking their build and dependency
  surface.
- The SuperCollider-style relationship is explicit: one control core, an
  in-process typed client, and OSC as a peer transport adapter — preventing OSC
  lifecycle logic from silently becoming shared engine semantics.
- `#161` and `#101`/`#134` get a recorded containment boundary, reducing
  coordination risk between the runtime and control-plane owners.
- Offline/`--no-default-features` builds are provably unaffected because the
  shared command envelope and hydration already sit outside the feature gate.

### Costs and risks

- A later mechanical change must actually move modules and re-gate examples; the
  alias mitigates breakage but the follow-up still touches `Cargo.toml`,
  `standalone/mod.rs`, and per-example `required-features`.
- Two features increase the build matrix; CI should cover `device-host` alone,
  `standalone-process`, and `--no-default-features`.
- `StandaloneControlWorker` currently constructs `BlightAudio` and installs the
  master gain effect on its worker thread. Cleanly separating the "own a
  non-`Send` stream on a dedicated NRT thread" pattern (shared) from the
  OSC-specific request/response policy (adapter) needs care so the tracker and
  OSC workers do not diverge in backpressure behavior.
- Feature naming (`standalone-process` vs `osc-host`) may need revisiting if a
  non-process OSC embedding is later required.

## Alternatives considered

### Name the shared layer `cpal_host`

Rejected: it leaks the backend vendor into the role name and would misdescribe
future JACK/JUCE/offline device backings. The host's job is device + callback +
control boundary ownership, not "CPAL".

### Keep one `standalone` feature and only document the intent

Rejected as insufficient: the tracker would keep compiling `rosc`/`tokio`, and
nothing structurally prevents OSC lifecycle code from accreting shared
semantics. The value of the split is the compile-time and ownership boundary.

### Make OSC a first-class control owner (its own command types/state)

Rejected: it duplicates control semantics, contradicts the single-control-core
architecture, and would force `#101`/`#134` to be re-implemented per transport.
OSC must map onto the shared `Command`/event surface.

### Split the device host into a separate crate now

Deferred, not chosen now: a crate boundary may be justified once the feature
split proves the module boundary is clean, but doing it immediately risks
churning the workspace dependency graph (a contract change under
[system boundaries](../architecture/system-boundaries.md)) before the internal
seams are settled. Revisit as a superseding decision if a second binary/host
needs the device host without `audio_backend`'s other modules.

## Validation and revisit triggers

The decision is validated when the follow-up implementation
([#190](https://github.com/jpalvarezl/blight-synth/issues/190), recommended
milestone **M2** per issue #185 metadata) lands and:

- `tracker_gui`'s `audio_backend` dependency is `default-features = false` with
  `device-host`, and `cargo tree -p tracker_gui` (or an equivalent build check)
  shows no `rosc`/`tokio` on the tracker path;
- `cargo build -p audio_backend --no-default-features --features device-host`
  compiles the in-process device host without `rosc`/`tokio`;
- `cargo build -p audio_backend` (default → `standalone-process`) and the
  `dsp-core` binary/OSC examples still build and pass existing tests;
- `cargo test -p audio_backend --no-default-features` retains offline/golden
  render behavior unchanged;
- the tracker and OSC adapter both submit exclusively through the `BlightAudio`
  typed interface, with no OSC-owned control state.

Revisit with a superseding ADR if: a device backend other than CPAL requires a
different host shape; a non-process OSC embedding makes `standalone-process` the
wrong name; or `#101`/`#134` reveal that continuous-parameter/timestamped-event
traffic cannot be expressed behind the current typed control interface without
transport-specific semantics.

## Related

- Owning issue: [#185](https://github.com/jpalvarezl/blight-synth/issues/185)
- Implementation follow-up: [#190](https://github.com/jpalvarezl/blight-synth/issues/190) (recommended milestone M2)
- Related issues: #181, #182 (dedicated NRT control ownership), #139 (standalone
  lifecycle), #161 (Tokio removal), #101/#134 (command-traffic classes), #145
  (event API)
- [Device host boundary contract (draft)](../architecture/device-host-boundary.md)
- [Standalone host domain](../domains/standalone-host.md),
  [Audio engine domain](../domains/audio-engine.md)
- [Target system boundaries](../architecture/system-boundaries.md),
  [Real-time audio contract](../architecture/realtime-contract.md)
