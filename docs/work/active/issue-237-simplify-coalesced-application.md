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
- Base: `main` / `ba2c866`

## Goal

Record the product-driven simplification: keep generation-bound latest-value coalescing/mapping/confirmation, reject fixed-quantum Engine smoothing, and defer smoothing to DSP-local implementations only when audible need exists.

## Scope

Contract/docs only: ADR 0007 supersedes ADR 0006 and amends ADR 0005; current architecture/RT/domain routing; #238/#215/#216 dependencies. No code changes.

## Plan

- [x] Record retained versus rejected parameter mechanisms and rationale.
- [x] Define immediate block-start mapped target application and confirmation/reset semantics.
- [x] Mark ADR 0006 superseded and reconcile ADR 0005/current routing.
- [x] Review docs and prepare #238.

## Verification

- [x] docs/reconciliation checks
- [x] independent decision review (REVISE findings addressed; final re-review timed out)

## Handoff

- Completed: canceled-issue/roadmap cleanup, accepted ADR 0007, ADR 0004/0005/0006 amendments/supersession, current routing updates, and review-finding reconciliation.
- Remaining: PR and merge.
- Decision review resolved: setter success uses target resolution plus the existing infallible scalar setter; #238 deletes unused Engine smoother code; event validation/private-render/zero-frame semantics are explicit; master gain `None` is a deliberate current-behavior limitation and future DSP-local trigger.
