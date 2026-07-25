---
title: "Task Packet — Issue 145: Event-source contract (design)"
summary: Define the engine/composition/host boundary so the audio engine consumes timestamped events instead of owning the tracker document model.
status: current
issue: 145
updated: 2026-07-24
---

# Task Packet — Issue 145: Event-source contract (design)

## Identity
- Issue: 145 · Owner: jpalvarezl · Status: in-progress
- Branch: `issue/145-event-source-contract` · Worktree: `../blight-145-eventsource`
- Base: origin/main @ a711c5c

## Goal
Record an ADR + architecture contract defining the boundary between the audio `Engine`, a
composition runtime/event source, and the host/control layer, so the engine is driven by
timestamped musical/control events rather than owning the tracker `Song → Chain → Phrase → Player`
model. Design only — no engine refactor this issue. See [#145](https://github.com/jpalvarezl/blight-synth/issues/145).

## Read first
1. [Audio engine domain](../../domains/audio-engine.md), [Composition domain](../../domains/composition.md)
2. [System boundaries](../../architecture/system-boundaries.md), [Real-time audio contract](../../architecture/realtime-contract.md) ("Control traffic classes", "Prepared-state rule")
3. [ADR template](../../templates/adr.md), existing ADRs 0001/0002
4. Read-only code: `audio_backend/src/player/mod.rs`, `audio_backend/src/player/tracker_engine_adapter.rs`, `engine/src/lib.rs`, `sequencer/src/`

## Scope
### In scope (docs/design)
- ADR defining: engine event consumer contract (timestamped note/control/transport events, sample-offset + same-offset ordering, bounded capacity), the event-source role (versioned composition doc + clock → events), and the host/control role.
- Answer the design questions: pull vs push + bounded lookahead; clock sources (internal/host/MIDI/external); fixed-memory RT eval vs deterministic NRT lookahead for generative programs; seeded randomness/determinism.
- Position the current tracker `Player` as ONE adapter/event-source implementation.
- Migration/impact notes + which follow-up issues implement it (#134 scheduling, #132 lifecycle, composition adapter extraction).
### Out of scope
- Implementing the event API or extracting the tracker adapter (later issues).
- Editing engine/DSP code or Cargo files.

## Ownership / touch set
Expected: `docs/decisions/NNNN-*.md`, maybe `docs/architecture/*.md`, `docs/domains/composition.md`, this packet.
Coordination: parallel #121 (params) and #137 (polyphony) — do NOT edit `engine/` code or their ADRs.

## Verify
- [x] `python3 scripts/docs/check_docs.py` — `documentation check passed: 28 pages`.
- [x] `python3 scripts/docs/reconcile_work.py --check` — only pre-existing unrelated errors from parallel branches (#137/#121 packets absent in this worktree; stale burndown/index). No new errors introduced by this task.
- [x] ADR follows template; registered in `docs/decisions/README.md`.

## Handoff
- ADR: [`docs/decisions/0003-event-source-contract.md`](../../decisions/0003-event-source-contract.md) (Status: Proposed).
- Routing page: [`docs/architecture/event-source-contract.md`](../../architecture/event-source-contract.md) (status: draft).
- Cross-links updated: `docs/decisions/README.md` (index row), `docs/architecture/README.md` (read-first + contract-ownership row → Proposed/ADR 0003), `docs/domains/composition.md` (read-first + open-direction).
- Design answers recorded in the ADR: pull-based generation with bounded lookahead (§3); internal/host/MIDI/external clock sources unified behind `Transport` events (§4); fixed-memory RT eval vs deterministic NRT lookahead for generative programs (§5); seeded, position-driven RNG for reproducible offline renders and save/restore (§6); live edits via immutable prepared snapshots + host-routed non-audio side effects (§7); tracker `Player` = one adapter (§8).
- Engine event-consumer contract: `Note`/`Control`/`Transport` events with `sample_offset` in `[0, block_len)`, deterministic same-offset ordering, fixed capacity set at `prepare` (#132), explicit producer-visible non-reordering overflow consistent with RT traffic class 2.
- Named follow-ups: #134 (sample-accurate scheduling), #132 (engine lifecycle), composition-adapter extraction (coordinated with #138).
- Constraint honored: docs only — no engine/DSP/Cargo edits; `realtime-contract.md` untouched; #121/#137 ADRs untouched.
- Do NOT push / no PR / no merge (design task). Committed locally as `Add event-source contract ADR (#145)`.
- Open follow-up: the M1 timestamped-event *types* are still owned by #134; this ADR is Proposed and should move to Accepted once #134/#132 land the concrete API.
