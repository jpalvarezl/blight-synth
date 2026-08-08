---
title: "Task Packet — Issue 238: Simplified coalesced application"
summary: Apply mapped coalesced targets once per valid block and delete unused Engine smoothing infrastructure.
status: current
updated: 2026-08-07
issue: 238
---

# Task Packet — Issue 238: Simplified coalesced application

## Identity

- Issue: [#238](https://github.com/jpalvarezl/blight-synth/issues/238)
- Owner/status: jpalvarezl / in-progress
- Branch/worktree: `issue/238-simplified-coalesced-application` / `/Users/jpalvarezl/code/blight-238`
- Base: `main` / `8a35bd7`

## Goal

Keep useful generation-bound coalescing/mapping/confirmation, implement first block-start target delivery, and remove unused generic Engine smoother/fixed-quantum infrastructure per ADR 0007.

## Scope / reviewability

One deletion-heavy simplification/integration. Target 500–800 meaningful lines excluding removed tests; pause/re-split if new production logic exceeds ~1,000.

In scope: remove PreparedSmoother/per-binding smoother state/tests/dependencies; generic bindings accept None and reject unsupported Smoothed; map and resolve/invoke concrete scalar setters once at valid top-level process start; confirm only successful resolution/invocation; install/reset seed application; preserve event validation/order and DSP-local smoothing; master gain policy None.

Out of scope: device-host generation lifecycle (#215), OSC (#216), new DSP smoothing.

Expected touched paths: `engine/src/{lib,coalesced_bindings}.rs`, Engine focused/RT tests, DSP internal effect-target resolution, `param_manifest` built-in descriptor/tests, and ADR 0004/0005/0007 status wording. Delete only the unused Engine smoother module/tests.

## Plan

- [x] Remove unused smoother and simplify binding preparation/application.
- [x] Add minimal constructor-time Engine coalesced state and one private non-relatching renderer.
- [x] Apply/confirm dirty targets before offset-zero events; preserve invalid-event pending state and zero-frame behavior.
- [x] Prove failures/seed/reset/sample-event/zero-heap behavior and measure code deletion.
- [x] Run full gates and independent review; no PR opened by request.

## Handoff

- Completed: implementation, focused/full/host-free/golden/RT/docs gates, and independent complete-diff review.
- Remaining: no code work identified; device-host installation/replacement remains #215.
- Risk: scalar effect setters remain infallible and cannot report semantic rejection of a valid parameter index/value. Applied confirmation therefore means exact effect resolution plus setter invocation; custom instruments must opt into precise coalesced resolution through `try_set_effect_parameter`.
- Verification: `cargo test --workspace --all-targets --all-features`; `cargo clippy --workspace --all-targets --all-features -- -D warnings`; host-free audio-backend test/Clippy; `cargo fmt --all -- --check`; architecture, RT logging, docs, and reconciliation scripts.
