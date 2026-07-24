---
title: "Task Packet — Issue 190: Split device-host infrastructure from the OSC standalone adapter"
summary: Apply ADR 0002 — split audio_backend standalone into a shared device_host layer and an OSC standalone-process adapter with explicit Cargo features.
status: current
issue: 190
updated: 2026-07-24
---

# Task Packet — Issue 190: Split device-host infrastructure from the OSC standalone adapter

## Identity

- Issue: 190
- Owner: jpalvarezl
- Status: in-progress
- Branch: `issue/190-device-host-split`
- Worktree: `../blight-190-devicehost`
- Base branch/SHA: origin/main @ ad2e035
- Last handoff: 2026-07-24

## Goal

Implement the module + Cargo-feature split **accepted by ADR 0002** so Rust in-process clients use
shared device-host infrastructure directly while foreign-language clients use OSC as a transport
adapter over the same typed control boundary. This is execution of an accepted design — do not
re-litigate the ADR. See [#190](https://github.com/jpalvarezl/blight-synth/issues/190).

## Read first

1. [ADR 0002 — device-host / OSC split](../../decisions/0002-device-host-osc-split.md) (the accepted design; follow its ownership table, names, and feature split exactly)
2. [Device-host boundary contract](../../architecture/device-host-boundary.md)
3. [Standalone host domain](../../domains/standalone-host.md)
4. Code: `audio_backend/src/standalone/**`, `audio_backend/Cargo.toml`, `audio_backend/src/bin/dsp-core.rs`, `audio_backend/src/lib.rs`, examples under `audio_backend/examples/`, `tracker_gui/src/audio.rs`

## Dependencies and blockers

- Depends on: #185 (closed — ADR 0002 accepted)
- Blocks: #161 (Tokio removal), #139
- Current blocker: NONE

## Scope and non-goals

### In scope

- Apply the accepted module names + dependency direction from ADR 0002 (shared `device_host` vs OSC `standalone-process` adapter).
- Split the `standalone` Cargo feature into `device-host` and `standalone-process`, keeping `standalone` as a compatibility alias so `default` and `--no-default-features`/offline builds are unchanged.
- Move OSC-only types (`OscServer`, protocol mapping, readiness/shutdown, temporary Tokio runtime) into the `standalone-process` adapter; keep shared types (`BlightAudio`, `AudioProcessor` callback adapter, command/retirement rings, `MeterState`, factories/resources) in `device_host`.
- Gate examples per ADR 0002's migration section (the `env_logger` examples ride with `standalone-process`).
- Keep `engine`/DSP/offline crates depending on neither layer.

### Out of scope

- Removing Tokio (#161) — only relocate it behind `standalone-process`.
- Changing engine/DSP semantics or the RT retirement behavior landed by #188.
- Editing `docs/architecture/realtime-contract.md` (owned by the parallel #133 close-out this cycle).

## Ownership and touch set

Expected paths: `audio_backend/**`, `audio_backend/Cargo.toml`, `docs/domains/standalone-host.md`, this packet.
Shared contracts touched: Cargo feature surface + `standalone` module boundary (this track owns it).
Potential parallel conflicts: must NOT edit `realtime-contract.md` (owned by #133 track).

## Verification

- [x] `cargo build -p audio_backend` (default features)
- [x] `cargo build -p audio_backend --no-default-features` (offline/tracker still compiles without OSC/CPAL/Tokio)
- [x] `cargo build -p audio_backend --features device-host --no-default-features` (shared host without OSC)
- [x] `cargo test --workspace --all-targets`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo build -p audio_backend --examples` and per-example feature builds from ADR 0002
- [x] `cargo build -p tracker_gui` (tracker compiles against `device_host`; `cargo tree` shows no `rosc`/`tokio`)
- [x] `python3 scripts/docs/check_docs.py`

## Handoff

- Completed: Applied ADR 0002. Split `audio_backend/src/standalone` into a shared
  `device_host` module (feature `device-host`: `BlightAudio`, `AudioProcessor`,
  command/retirement rings + `CommandSender`/`CommandSubmission*`, `MeterState`/
  `MeterLevels`, factories/`ResourceManager` handoff) and a `standalone_process`
  module (feature `standalone-process`, depends on `device-host`: `OscServer`,
  protocol/gain/dBFS mapping, `StandaloneControlWorker`, temporary Tokio runtime,
  `dsp-core`). `standalone = ["standalone-process"]` compatibility alias keeps
  `default` and `--no-default-features` semantics unchanged.

  Module moves (old -> new):
  - `standalone/audio_frontend/{mod,blight_audio,command_sender}.rs` -> `device_host/audio_frontend/*`
  - `standalone/audio_processor/mod.rs` -> `device_host/audio_processor/mod.rs`
  - `standalone/meter.rs` -> `device_host/meter.rs`
  - `standalone/osc.rs` -> `standalone_process/osc.rs`
  - `standalone/control_worker.rs` -> `standalone_process/control_worker.rs`

  Feature graph: `device-host = [cpal, ringbuf]`;
  `standalone-process = [device-host, rosc, tokio, env_logger]`;
  `standalone = [standalone-process]`; `default = [standalone]`.
  `engine`/`dsp`/offline depend on neither.

  Call sites updated:
  - `audio_backend/src/lib.rs`: module decls + re-exports now feature-gated per layer.
  - `audio_backend/src/song_hydration.rs`: `BlightAudio` helpers re-gated `standalone` -> `device-host`.
  - `audio_backend/Cargo.toml`: feature split + per-example/bin `required-features` re-gated per ADR 0002.
  - Root `Cargo.toml`: `audio_backend` workspace dep set `default-features = false` (needed so `tracker_gui` can select only `device-host`).
  - `tracker_gui/Cargo.toml`: `audio_backend` now `default-features = false, features = ["device-host"]`.
  - Examples: no source edits needed; only `required-features` re-gated (env_logger examples `polyphonic_song`/`play_song_file` -> `standalone-process`).
  - `dsp-core.rs`: unchanged source; `required-features` -> `standalone-process`.

- Remaining: none for #190. Unblocks #161 (Tokio now contained in `standalone_process`).
