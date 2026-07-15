---
title: Task Packet — Issue 149 Utility and DLS Clippy Cleanup
summary: Active context and handoff for strict lint cleanup in utils and os_dls.
status: current
updated: 2026-07-14
issue: 149
owner: jpalvarezl
branch: issue/149-utils-osdls-clippy
---

# Task Packet — Issue 149

## Goal

Make `utils` and `os_dls` pass strict Clippy through mechanical, behavior-preserving cleanup.

## Read first

1. Issue #149
2. Clippy diagnostics for the two selected crates
3. Only the flagged source files

## Dependencies

- Parent: #131
- Blocks: workspace-wide strict lint gate

## Scope

Expected paths: `utils/`, `os_dls/`, this packet.

No DSP, engine, protocol, schema, or UI changes.

## Plan

- [x] Apply idiomatic mechanical fixes.
- [x] Document the narrow DLS parser-constructor lint allowance.
- [x] Run focused tests and strict Clippy.

## Verification

- [x] `cargo fmt --all -- --check`
- [x] `cargo test -p utils -p os_dls --all-targets`
- [x] `cargo clippy -p utils -p os_dls --all-targets -- -D warnings`
- [x] `python3 scripts/docs/check_docs.py`
- [x] `git diff --check`

## Handoff

- Completed: all issue acceptance criteria; fixes are mechanical and focused on the two selected crates.
- Remaining: review and merge before #131 enables workspace-wide strict Clippy.
- Known risks: none; `Sample::new` retains a rationale-backed argument-count allowance instead of an unrelated API redesign.
- Next action: Copilot/human review.
