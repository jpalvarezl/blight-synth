---
title: "Task Packet — Issue 237: Simplify coalesced application"
summary: Supersede fixed-quantum Engine smoothing while retaining useful latest-value coalescing.
status: current
updated: 2026-08-07
issue: 237
---

# Task Packet — Issue 237: Simplify coalesced application

## Identity

- Issue: [#237](https://github.com/jpalvarezl/blight-synth/issues/237)
- Owner/status: jpalvarezl / in-progress
- Branch/worktree: `issue/237-simplify-coalesced-application` / `/Users/jpalvarezl/code/blight-237`
- Base: `main` / pending packet commit

## Goal

Record the product-driven simplification: keep generation-bound latest-value coalescing/mapping/confirmation, reject fixed-quantum Engine smoothing, and defer smoothing to DSP-local implementations only when audible need exists.

## Scope

Contract/docs only: ADR 0007 supersedes ADR 0006 and amends ADR 0005; current architecture/RT/domain routing; #238/#215/#216 dependencies. No code changes.

## Plan

- [ ] Record retained versus rejected parameter mechanisms and rationale.
- [ ] Define immediate block-start mapped target application and confirmation/reset semantics.
- [ ] Mark ADR 0006 superseded and reconcile ADR 0005/current routing.
- [ ] Review docs and prepare #238.

## Verification

- [ ] docs/reconciliation checks
- [ ] independent decision review

## Handoff

- Completed: issue claimed and roadmap metadata updated.
- Remaining: ADR/docs PR.
