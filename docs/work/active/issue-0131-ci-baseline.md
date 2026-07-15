---
title: Task Packet — Issue 131 CI Baseline
summary: Active context and handoff for hardware-free CI, lint, and dependency-boundary checks.
status: current
updated: 2026-07-14
issue: 131
owner: jpalvarezl
branch: issue/131-ci-baseline
---

# Task Packet — Issue 131: CI Baseline

## Goal

Make `main` continuously verifiable with hardware-free Rust, documentation, and architecture checks plus documented local equivalents.

## Read first

1. [Work system](../README.md)
2. [Audio engine domain](../../domains/audio-engine.md)
3. [System boundaries](../../architecture/system-boundaries.md)

## Dependencies and blockers

- Depends on: none.
- Blocks: safe M0/M1 refactoring.
- Current blocker: none.

## Scope and non-goals

### In scope

- GitHub Actions baseline.
- Strict Clippy cleanup or explicit baseline.
- Current dependency-direction guard.
- Hardware-free local command documentation.

### Out of scope

- TypeScript CI before `gui/` exists.
- Audio-device smoke tests in hosted CI.
- Final post-#130 crate rules.

## Ownership and touch set

- `.github/workflows/`
- `scripts/check_architecture.py`
- lint-only fixes identified by strict Clippy
- `README.md` and focused tooling docs

Shared contracts touched: CI policy only; no engine API/schema.

## Plan

- [x] Inventory and split strict Clippy failures into #149 and #150.
- [x] Add dependency-boundary checker.
- [x] Add hardware-free GitHub Actions jobs.
- [x] Document local commands and future TypeScript extension.
- [x] Rebase onto merged #149/#150 and run the complete strict CI-equivalent set.

## Verification

- [x] `cargo fmt --all -- --check`
- [x] `cargo test --workspace --all-targets`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `python3 scripts/check_architecture.py`
- [x] `python3 scripts/docs/check_docs.py`
- [x] `git diff --check`

## Handoff

- Completed: CI workflow, current dependency checker, local command documentation, strict Clippy, hardware-free workspace tests, docs checks, and roadmap-generator exercise.
- Remaining: Copilot/human review and hosted Ubuntu/macOS workflow validation.
- Known failures/risks: the Ubuntu package list must be proven by the first GitHub Actions run; roadmap generation intentionally uses `--stdout` to avoid live-metadata flakiness.
- Next smallest action: open the integration PR and inspect hosted checks.
- Files a new agent should read next: this packet, `.github/workflows/ci.yml`, and `scripts/check_architecture.py`.
