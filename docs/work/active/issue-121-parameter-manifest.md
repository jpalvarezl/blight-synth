---
title: "Task Packet — Issue 121: Canonical parameter manifest and host bindings"
summary: Define one source of truth for parameters (ADR + serializable schema) and implement Rust descriptor/runtime lookup types.
status: current
issue: 121
updated: 2026-07-24
---

# Task Packet — Issue 121: Canonical parameter manifest and host bindings

## Identity
- Issue: 121 · Owner: jpalvarezl · Status: in-progress
- Branch: `issue/121-parameter-manifest` · Worktree: `../blight-121-parammanifest`
- Base: origin/main @ a711c5c

## Goal
Define one canonical parameter manifest/schema across Rust DSP, project state, TS/Svelte, OSC, and
JUCE/APVTS, and implement the Rust descriptor + runtime lookup types, in a NEW self-contained module
(minimize edits to `engine/src/lib.rs`). See [#121](https://github.com/jpalvarezl/blight-synth/issues/121).

## Read first
1. [Audio engine domain](../../domains/audio-engine.md), [Real-time audio contract](../../architecture/realtime-contract.md) ("Control traffic classes" — continuous params coalesce; "Prepared-state rule": descriptive strings off the audio thread)
2. [ADR template](../../templates/adr.md), ADRs 0001/0002
3. Read-only code: `engine/src/lib.rs` (InstrumentCmd/MixerCmd, set_parameter paths), `dsp/src/`, `audio_backend/src/commands.rs`

## Scope
### In scope
- ADR recording the manifest design (descriptor fields, normalized 0..1 mapping, automation-rate classes sample-event/coalesced/structural, smoothing policy, host visibility flags, schema version + compatibility rules).
- A serializable manifest/schema (serde types) with compatibility/versioning rules.
- Rust descriptor + runtime lookup types in a NEW module/file, with a bounded RT representation (stable parameter IDs; descriptive strings kept off the RT path).
- Note how host adapters (OSC, APVTS, Svelte) bind to it without duplicating unit conversion/smoothing. Stable IDs across versions.
### Out of scope
- Implementing the coalesced RT parameter pipeline (#101) — this defines the manifest it consumes.
- Wiring every existing instrument/effect parameter through it (follow-up); provide the types + one representative example.
- Editing engine voice code (that is #137).

## Ownership / touch set
Expected: `docs/decisions/NNNN-*.md`, `docs/decisions/README.md`, a NEW params descriptor module (prefer a new file/module; if a new crate is cleaner, add it to the workspace), tests, this packet.
Coordination: parallel #145 (event contract) and #137 (polyphony). Prefer NEW files; keep `engine/src/lib.rs` edits minimal to avoid collisions with #137. Do NOT touch `realtime-contract.md`.

## Verify
- [x] `cargo test --workspace --all-targets`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `python3 scripts/docs/check_docs.py`
- [x] ADR registered in `docs/decisions/README.md`.

## Handoff

Status: implemented; verification green. ADR is `proposed` (awaiting acceptance).

### Delivered
- **ADR 0003** — [`docs/decisions/0003-parameter-manifest.md`](../../decisions/0003-parameter-manifest.md),
  registered in [`docs/decisions/README.md`](../../decisions/README.md). Records
  descriptor fields, normalized↔engine mapping ownership, automation-rate classes,
  smoothing policy, host-visibility flags, schema version + compatibility rules,
  stable IDs, the bounded string-free RT representation, and how OSC/APVTS/Svelte
  bind without duplicating conversion/smoothing.
- **New crate `param_manifest/`** (added to workspace `members` and
  `workspace.dependencies`; depends only on `serde`, dev-dep `serde_json`).
  - `descriptor.rs` — `ParameterDescriptor`, `ParameterId`, `NodeRef`/`NodeType`,
    `Unit`, `ValueRange`, `ParameterKind`/`DiscreteStep`, `AutomationRate`,
    `SmoothingPolicy`/`SmoothingCurve`, `Visibility`.
  - `mapping.rs` — `Mapping` (Linear / Exponential / AmplitudeDecibel) with
    `to_engine`/`to_normalized` (Copy, RT-safe).
  - `manifest.rs` — `ParameterManifest`, `MANIFEST_SCHEMA_VERSION`, `validate()`,
    `is_readable_by`, `ManifestError`.
  - `compatibility.rs` — `compatibility_against` + `CompatibilityReport`/`CompatibilityBreak`.
  - `runtime.rs` — bounded string-free RT handle `RuntimeParameterTable` (`get`
    RT O(1) slice index) split from the NRT owner `ParameterLookup`
    (`from_manifest`, `key_for` NRT, `table`/`into_table`); `RuntimeParameter`
    (Copy), `RuntimeParamKey`, `RuntimeKind`. The `ParameterId`→key resolver map
    lives only on the NRT owner and is never handed to the callback.
  - `builtin.rs` — representative wired parameter: **master gain** (`master.gain`),
    mapping `AmplitudeDecibel { floor_db: -120 }`, targeting master effect
    `set_parameter` index 0. Mirrors the OSC `/param/set gain` conversion.
- **Tests** — `param_manifest/tests/manifest.rs` (15 tests): JSON round-trip,
  duplicate-id/schema-version/numeric validation (reversed range, non-finite
  default), compatibility rules (remove/deprecate/add/automation-rate-change/
  mapping-semantics-change), master-gain mapping matches the OSC numbers
  (0.5→−6.02 dB), linear round-trip, string-free RT-table handle + bounded key
  indexing, lookup-by-ID, discrete kind→step-count collapse.

### Review-driven refinements (code_review, gpt-5.5)
- Split the NRT resolver `HashMap` from the RT handle: the callback receives a
  string-free `RuntimeParameterTable`.
- `validate()` now rejects non-finite/reversed/out-of-range descriptor data and
  bad mapping/smoothing/discrete config before it can reach the RT `clamp` path.
- Compatibility now flags meaning-bearing changes (mapping/range/unit/kind/engine
  slot) under a stable ID as breaking.
- The representative descriptor uses the existing public OSC id `"gain"` (not a
  new `master.gain`) to avoid breaking current clients on migration.

### Not done (by design / follow-up)
- No coalesced RT parameter pipeline (#101 consumes this manifest).
- OSC/engine dB double-conversion left in place; migrating `normalized_gain_to_db`
  to `Mapping` is a mechanical follow-up. `engine/src/lib.rs` untouched.
- Only one representative parameter wired; full catalog is follow-up.

### Verification
- `cargo test --workspace --all-targets` → all pass (param_manifest: 15 passed).
- `cargo clippy --workspace --all-targets -- -D warnings` → clean.
- `python3 scripts/docs/check_docs.py` → `documentation check passed: 27 pages`.
