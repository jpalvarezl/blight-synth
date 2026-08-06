---
title: "Task Packet — Issue 230: Coalesced target bindings"
summary: Prepare target bindings and directly test map/latch/confirmation without render integration.
status: current
updated: 2026-08-07
issue: 230
---

# Task Packet — Issue 230: Coalesced target bindings

## Identity

- Issue: [#230](https://github.com/jpalvarezl/blight-synth/issues/230)
- Owner/status: jpalvarezl / in-progress
- Branch/worktree: `issue/230-coalesced-target-bindings` / `/Users/jpalvarezl/code/blight-230`
- Base: `main` / `654b4ea`

## Goal

Build NRT-prepared coalesced target bindings and a directly testable store-drain application layer that maps, latches smoother targets, and confirms applied revisions.

## Reviewability / scope

One application/binding concept, generally 500–800 meaningful lines; pause/re-split above ~1,000. No render quantum, DSP setter delivery, duplicate smoother removal, host lifecycle, or OSC.

Expected paths: `param_manifest/src/runtime.rs`, `engine/src/coalesced_parameters.rs`, a narrow engine binding/application module, focused mapping/confirmation/reset/error/allocation tests, docs/packet.

Shared contracts/schemas touched: additive opaque runtime-table identity and runtime node class, used to prove exact table/store binding.

Potential parallel conflicts: issue #229 is confined to versioned envelope migration; no expected path overlap.

## Plan

- [x] Define exact-table/generation prepared binding validation and reset seeds.
- [x] Implement direct drain map/latch/confirm/failure application.
- [x] Test None/linear/exponential, injection failures, reset/bounds/ownership, and zero allocation.
- [x] Run full workspace/all-target, strict Clippy, host-free, RT, architecture, and docs gates; independently review/fix.

## Progress and decisions

- 2026-08-07 — Added opaque allocation identity to runtime tables so structurally equal tables cannot satisfy exact store/binding ownership checks.
- 2026-08-07 — Prepared tables require one supported concrete target and one snapshot-seeded smoother for every writable `ControlCoalesced` entry; read-only and unsupported/mismatched node classes fail NRT preparation.
- 2026-08-07 — Direct application confirms only after exact-table mapping and successful latch. Review approved with no blocking or revision findings.

## Verification

- [x] `cargo test -p engine --all-targets`
- [x] `cargo clippy --workspace --all-features --all-targets -- -D warnings`
- [x] `cargo test --workspace --all-features --all-targets`
- [x] `cargo clippy -p audio_backend --no-default-features --all-targets -- -D warnings`
- [x] `cargo test -p audio_backend --no-default-features --all-targets`
- [x] `cargo fmt --all -- --check`
- [x] `python3 scripts/check_architecture.py`
- [x] `python3 scripts/check_rt_logging.py`
- [x] `python3 scripts/docs/check_docs.py`
- [x] `python3 scripts/docs/reconcile_work.py --check`

## Handoff

- Completed: implementation, focused/full tests, strict Clippy, allocation proof, docs/architecture/RT checks, and independent review.
- Remaining: none in #230; #231 owns render-phase and DSP delivery integration.
- Known risks: writable coalesced `Instrument`/`VoiceEffect` targets remain intentionally unsupported until a concrete target contract exists; opaque identity owners must follow prepared-state NRT retirement.
- Next smallest action: integrate this direct drain layer at the top-level render boundary in #231.
- Files a new agent should read next: `engine/src/coalesced_bindings.rs`, `engine/src/coalesced_parameters.rs`, `engine/tests/coalesced_parameter_bindings.rs`.
