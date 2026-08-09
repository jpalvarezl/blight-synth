---
title: "Task Packet — Issue 249: Portable state core"
summary: Implement ADR 0008's host-neutral envelope, canonical bytes, validation, diagnostics, and envelope-level migration.
status: current
updated: 2026-08-09
issue: 249
---

# Task Packet — Issue 249: Portable state core

## Identity

- Issue: [#249](https://github.com/jpalvarezl/blight-synth/issues/249)
- Owner/status: jpalvarezl / in-progress
- Branch/worktree: `issue/249-portable-state-core` / `/Users/jpalvarezl/code/blight-249`
- Base: `main` / `fa0f3a0`

## Goal

Implement the host-neutral PortableStateV1 model and canonical JSON boundary without tracker Song adapters or Engine restore.

## Scope / reviewability

One core model/canonicalization concept, generally 500–800 meaningful lines; pause/re-split above ~1,000. Includes tagged source-preserving payloads, ordered definitions, normalized overlay, asset refs, RFC 8785 bytes, validation/diagnostics, and v0→v1 envelope fixture.

Out of scope: legacy Song adapter (#250), Engine restore (#243), filesystem/JUCE.

## Plan

- [x] Define core types and canonical encoder.
- [x] Add semantic validation/source-preserving diagnostics.
- [x] Add asset validation and migration fixture.
- [x] Run full gates/review and commit.

## Coordination note

Issue #249 explicitly requests an envelope-level v0→v1 fixture and simple
runtime-owned seed/checkpoint records, while ADR 0008 calls V1 the first envelope
and excludes generic top-level replay fields. This slice follows the narrower
live issue/task instruction: v0 is migration-fixture input only, replay records
can occur only inside an opaque composition payload, and neither is a top-level
V1 field. The legacy direct-`Song` adapter remains #250.

## Expected touched paths

- `portable_state/`
- root `Cargo.toml` / `Cargo.lock`
- `scripts/check_architecture.py`
- `docs/architecture/crate-dependency-graph.md`
- this packet and generated work docs

## Acceptance criteria

- [x] Canonical bytes are stable across map/construction order.
- [x] Unknown future composition/routing/node source is retained.
- [x] Duplicate IDs, invalid normalized/unsafe numbers, and asset failures are structured.
- [x] V0 fixture migrates deterministically to canonical V1.
- [x] No Engine, filesystem, tracker `Song`, UI, or host dependency was introduced.

## Verification

- `cargo test --workspace --all-features --all-targets`
- `cargo clippy --workspace --all-features --all-targets -- -D warnings`
- `cargo test -p audio_backend --no-default-features --all-targets`
- `cargo fmt --all -- --check`
- `python3 scripts/check_architecture.py`
- `python3 scripts/docs/check_docs.py`
- `python3 scripts/docs/reconcile_work.py --check`

Independent review approved the implementation; its coverage warnings for an
invalid node reference and asset media mismatch were fixed and tested. The slice
contains about 800 meaningful Rust lines (excluding blanks, comments, and
brace-only formatting), including tests, so no further split is needed.

## Handoff

- Completed: core model/canonicalization, validation/source diagnostics, asset
  resolver validation, envelope migration fixture, architecture/docs, and gates.
- Remaining: none for #249. Legacy `Song` adaptation and Engine restore remain
  isolated in #250 and #243.
