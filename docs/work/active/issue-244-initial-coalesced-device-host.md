---
title: "Task Packet — Issue 244: Initial coalesced device-host generation"
summary: Install one prepared coalesced generation and expose NRT stable-ID publish/confirmation access.
status: current
updated: 2026-08-08
issue: 244
---

# Task Packet — Issue 244: Initial coalesced device-host generation

## Identity

- Issue: [#244](https://github.com/jpalvarezl/blight-synth/issues/244)
- Owner/status: jpalvarezl / in-progress
- Branch/worktree: `issue/244-initial-coalesced-device-host` / `/Users/jpalvarezl/code/blight-244`
- Base: `main` / `bfb701b`

## Goal

Prepare/install one initial manifest/table/store/binding generation before callback processing and expose stable-ParameterId NRT publication plus applied-confirmation queries.

## Scope / reviewability

One initial lifecycle vertical slice, generally 500–800 meaningful lines. Constructor plumbing through device host/Player/Engine, NRT facade, block-start application, failures, no queue growth, RT tests. No replacement/rebind/retirement (#245) and no OSC (#216).

## Expected touched paths

- `audio_backend/src/device_host/` — NRT preparation/facade, callback constructor plumbing, static shutdown.
- `audio_backend/src/player/` — constructor-owned state handoff to Engine.
- `audio_backend/src/standalone_process/` — remove duplicate startup gain installation; preserve OSC wire behavior through the dedicated master-gain command.
- `audio_backend/tests/` — facade/device-host/RT allocation coverage.
- `engine/src/coalesced_parameters.rs` — NRT generation close/disconnect controls used by the facade.
- `engine/src/{events,commands,lib}.rs` — dedicated Engine-owned master-gain target/command/state outside the user effect-ID namespace.
- Focused architecture/domain/task documentation and dependency metadata.

## Plan

- [x] Define NRT facade and initial preparation.
- [x] Plumb constructor-owned state to Engine.
- [x] Expose stable-ID publish/confirmation and failures.
- [x] Verify queue independence, zero heap, goldens, and host behavior.

## Acceptance evidence

- [x] The built-in manifest default, runtime table, generation store, concrete master-gain binding, stable-ID resolver, and `PreparedCoalescedParameterState` are prepared before callback construction.
- [x] Stable `ParameterId("gain")` publication reaches the block-start mapped setter and exact applied revision without entering or growing the structural command ring.
- [x] Unknown ID, invalid value, closed generation, disconnected engine, latest value, pending/applied confirmation, last application failure, and counters are observable through compact facade results.
- [x] Device-host allocation instrumentation covers initial seed and published-value drains with zero callback allocation, reallocation, or deallocation.
- [x] Static shutdown pauses the stream, disconnects publication, then drops callback state before the facade owner on NRT; an outliving NRT facade clone observes `Disconnected`.
- [x] Host-free tracker/offline goldens and existing device-host behavior remain covered.

## Handoff

- Completed: implementation, focused tests, full verification, documentation, and independent review (approved with no blocking findings).
- Deferred: live generation replacement/rebind/retirement (#245), OSC migration (#216), smoothing, and desired-state UI stores.
- Review decision: canonical master gain is a dedicated Engine system target, not a reserved user `EffectId`. User effects retain the complete ID namespace (including 0); transitional OSC uses `MixerCmd::SetMasterGain`, while the facade uses `ParameterTarget::MasterGain`.
- Coordination risk: until #216 removes the transitional OSC structural setter, the facade and legacy OSC command can both target master gain. The standalone process uses only its legacy OSC producer; adapters must not drive both paths without NRT arbitration.
- Verification:
  - `cargo test --workspace --all-targets --all-features`
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  - `cargo clippy -p audio_backend --no-default-features --all-targets -- -D warnings`
  - `cargo test -p audio_backend --no-default-features --all-targets`
  - `RUSTFLAGS='-D warnings' cargo test --release -p dsp diagnostics::tests::argument_evaluation_matches_diagnostic_build_mode`
  - `cargo fmt --all -- --check`
  - `python3 scripts/check_architecture.py`
  - `python3 scripts/check_rt_logging.py`
  - `python3 scripts/docs/reconcile_work.py --check`
  - `python3 scripts/docs/check_docs.py`
- Remaining: none.
