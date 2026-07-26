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

Status: implemented; independent-review findings addressed. ADR remains
`proposed` pending acceptance.

### Delivered
- **ADR 0004** — [`docs/decisions/0004-parameter-manifest.md`](../../decisions/0004-parameter-manifest.md)
  records descriptor/mapping ownership, automation and smoothing policy, stable-ID
  compatibility, strict validation, and the bounded NRT/RT split.
- **New crate `param_manifest/`** (only `serde`, plus dev-only `serde_json`):
  - complete serializable descriptors and compatibility reports;
  - stable Linear/Exponential/Skewed/AmplitudeDecibel conversion with explicit
    NaN/infinity policy and exact agreement between mapping bounds and ranges;
    skew is limited to `0.25..=4.0`, with per-range representative round-trip
    validation at a documented `1e-4` normalized tolerance;
  - current-schema and practical capacity validation;
  - private-construction `Copy` runtime entries, checked compact keys, and a
    non-`Clone` runtime table whose owning lifecycle remains NRT; raw runtime
    mappings are hidden so table conversion is the sole discrete-aware RT path;
  - exact non-uniform discrete numeric values in a flat string-free RT arena;
  - representative, **not yet wired**, master-gain descriptor with stable OSC ID
    `"gain"` and the existing `-120 dB` floor convention.
- **Compatibility policy** compares the full owner identity. Value/routing,
  automation-rate, and visibility/automatable/read-only changes are breaking;
  smoothing is documented as compatible tuning.
- **Tests** — 41 manifest integration tests, one RT allocation-audit integration
  test, one compile-fail API doctest, and one internal malformed-entry defense
  test. Coverage includes skew-bound round trips at `0.1`/`0.25`/`0.5`/`0.9`,
  collapsed skew/range rejection, descriptor version 0, contradictory visibility,
  continuous/discrete/invalid-key zero-allocation conversion (including NaN and
  infinities), exact non-uniform discrete RT values, and Copy/size assertions.

### Design deviation and rationale
- Discrete values are carried in a numeric runtime arena rather than reconstructed
  from `step_count + Mapping`. This preserves authored non-uniform choices exactly
  without strings, allocation, or unbounded work on RT.
- Conversion moved to `RuntimeParameterTable::normalized_to_engine` so discrete
  entries can access that arena. `RuntimeParameter` fields are private; its raw
  `Mapping` getter was removed, while metadata remains available through read-only
  getters. An internal malformed-entry test verifies the final finite, panic-free
  fallback.

### Not done (by design / follow-up)
- No coalesced RT parameter pipeline (#101 consumes this manifest).
- No existing crate consumes `param_manifest` yet. The current OSC conversion is
  unchanged; the representative descriptor is tested against it for future migration.
- No full instrument/effect catalog; `engine`, `dsp`, `audio_backend`, and
  `realtime-contract.md` are untouched by this issue.

### Verification
- `cargo test --workspace --all-targets` → all targets passed; `param_manifest`:
  1 unit test + 42 integration tests passed.
- `cargo clippy --workspace --all-targets -- -D warnings` → clean.
- `python3 scripts/docs/check_docs.py` → `documentation check passed: 27 pages`.
