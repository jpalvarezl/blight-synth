---
title: "Task Packet — Issue 185: Separate shared device host from OSC standalone transport adapters"
summary: Record an ADR/architecture contract for splitting the shared CPAL device host from the OSC standalone process adapter, without changing engine/DSP semantics.
status: current
issue: 185
updated: 2026-07-24
---

# Task Packet — Issue 185: Separate shared device host from OSC standalone transport adapters

## Identity

- Issue: 185
- Owner: jpalvarezl
- Status: in-progress
- Branch: `issue/185-device-host-osc-split`
- Worktree: `../blight-185-devicehost-split`
- Base branch/SHA: origin/main @ ac9627d
- Head SHA: TBD
- Last handoff: 2026-07-24

## Goal

Define and plan a clear split between the reusable CPAL/device host and the OSC-controlled
standalone process adapter, recorded as an ADR (+ architecture contract update), **without
changing engine/DSP semantics or renaming/splitting modules yet**. See issue
[#185](https://github.com/jpalvarezl/blight-synth/issues/185).

## Read first

1. [Standalone host domain](../../domains/standalone-host.md) ("Threading/runtime decision", "Feature boundary")
2. [System boundaries](../../architecture/system-boundaries.md) ("Dependency direction", "Parallelization boundary"), [Real-time audio contract](../../architecture/realtime-contract.md) ("Thread roles", "Backpressure and overload")
3. [ADR template](../../templates/adr.md) and existing [ADR 0001](../../decisions/0001-product-and-host-priorities.md)
4. Code entry points (read-only, for boundary evidence):
   - `audio_backend/src/standalone/audio_frontend/blight_audio.rs`
   - `audio_backend/src/standalone/audio_processor/mod.rs`
   - `audio_backend/src/standalone/osc.rs`, `control_worker.rs`, `meter.rs`
   - `audio_backend/src/bin/dsp-core.rs`
   - `audio_backend/Cargo.toml` (feature layout)

## Dependencies and blockers

- Depends on: #181 (closed), #182 (closed) findings
- Blocks: implementation follow-ups
- Current blocker: NONE

## Scope and non-goals

### In scope (documentation only)

- ADR recording the target dependency/module/feature split (shared `device_host` vs OSC/process adapter).
- Migration steps + compatibility impact for tracker, examples, `dsp-core`, and `--no-default-features`/offline builds.
- Confirm OSC stays a transport adapter over the same typed control semantics used by Rust clients.
- Name the typed in-process client/control interface both tracker and OSC adapters target.
- Slot the implementation work into the right roadmap milestone.

### Out of scope

- Any code rename/module split/feature change (implementation is a later issue).
- Changing engine/DSP semantics.
- Doing #181/#182's module reshaping opportunistically.

## Ownership and touch set

Expected paths (documentation only):

- `docs/decisions/0002-device-host-osc-split.md` (new ADR)
- `docs/decisions/README.md` (ADR index)
- `docs/architecture/device-host-boundary.md` (new draft contract)
- `docs/architecture/README.md` (index + contract ownership table)
- `docs/domains/standalone-host.md` (Read-first routing)
- `docs/work/active/README.md`, this packet

Shared contracts/schemas touched: proposes a target Cargo feature/module split
(`device-host` + `standalone-process`) as an ADR only; no code/feature change in
this branch. Implementation is issue #190.

Potential parallel conflicts: NONE (no Rust code touched). #190 will own the
mechanical split.

## Questions to settle (from the issue)

- Shared layer name: `device_host` vs `cpal_host` vs other host-neutral term.
- Which types are shared (`BlightAudio`, callback adapter, FIFO, meter, factories/resources) vs OSC-only (`OscServer`, protocol mapping, readiness/shutdown, temporary Tokio runtime).
- Cargo feature split: `device-host` vs `osc-host`/`standalone-process`.
- How the boundary evolves under #161 (Tokio removal) and #101/#134 (command traffic replacement).

## Verification

- [x] `python3 scripts/docs/check_docs.py`
- [x] `python3 scripts/docs/reconcile_work.py --check`
- [x] ADR follows `docs/templates/adr.md`; links to owning issue #185 and related #181/#182/#139/#161/#101/#145.

## Handoff

- Completed: Added [ADR 0002 — device host vs OSC split](../../decisions/0002-device-host-osc-split.md)
  (Status: Proposed), registered it in the [decisions index](../../decisions/README.md),
  and added the draft [device host boundary contract](../../architecture/device-host-boundary.md).
  Resolved every "Question to settle": shared layer named `device_host` (host-neutral, not
  `cpal_host`); typed in-process control interface is `BlightAudio` over the host-neutral
  `Command` envelope (both tracker and OSC target it); type ownership table splits shared
  device-host types from OSC-only transport/lifecycle types; Cargo split is
  `device-host` + `standalone-process` with `standalone` kept as a compatibility alias;
  and #161/#101/#134 evolution is contained behind the device-host interface so OSC never
  owns device-host semantics. Recommended implementation milestone: M2. Also added the
  required `issue: 185` frontmatter key so `reconcile_work.py --check` passes.
- Remaining: Implementation follow-up (module/feature split, example re-gating) is
  tracked by [#190](https://github.com/jpalvarezl/blight-synth/issues/190) (M2);
  this task is documentation-only per the acceptance criteria.
- Known failures/risks: None from this change. No Rust code changed.
  `check_docs.py` passes; `reconcile_work.py --check` passes for #185 (the only
  remaining error is pre-existing and unrelated: in-progress issue #188 has no
  active packet, owned elsewhere).
- Next smallest action: Schedule #190 to perform the mechanical
  `device_host`/`standalone-process` split described in ADR 0002.
