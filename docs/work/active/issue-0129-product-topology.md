---
title: Task Packet — Issue 129 Product Topology
summary: Active context and handoff for the M0 product/host topology documentation task.
status: current
updated: 2026-07-14
issue: 129
owner: jpalvarezl
branch: issue/129-product-topology
---

# Task Packet — Issue 129: Product Topology

## Goal

Complete the accepted product decision with standalone and optional-plugin topology diagrams, a target matrix, state/parameter authority, runtime constraints, and navigation links.

## Read first

1. [Current product spec](../../spec/current-product.md)
2. [ADR 0001](../../decisions/0001-product-and-host-priorities.md)
3. [System boundaries](../../architecture/system-boundaries.md)

## Dependencies and blockers

- Depends on: none; the decision was accepted in #129.
- Blocks: #130.
- Current blocker: none.

## Scope and non-goals

### In scope

- Durable product topology documentation and diagrams.
- Target/priority matrix and authority tables.
- README/docs navigation.

### Out of scope

- Selecting the final composition UI or desktop shell.
- Implementing engine/host/plugin code.

## Ownership and touch set

- `docs/architecture/product-topology.md`
- `docs/architecture/README.md`
- `docs/decisions/0001-product-and-host-priorities.md`
- `README.md`
- generated roadmap snapshot

Shared contracts touched: documentation of accepted boundaries only; no runtime API.

## Plan

- [x] Add standalone and optional plugin diagrams.
- [x] Add host matrix, state/parameter authority, constraints, and non-goals.
- [x] Link topology from ADR, architecture index, and README; link the canonical roadmap after the reviewed file lands.
- [x] Validate docs and generated roadmap.

## Verification

- [x] `python3 scripts/docs/check_docs.py`
- [x] `python3 scripts/docs/sync_roadmap.py --check`
- [x] `git diff --check`

## Handoff

- Completed: topology diagrams, host matrix, authority tables, constraints/non-goals, ADR and repository navigation.
- Remaining: human/Copilot review; add the merged topology URL to canonical issue #144, then close #129.
- Known failures/risks: diagrams describe accepted target architecture, not current implementation; frontmatter marks the topology accepted and system boundaries draft accordingly.
- Next smallest action: review the topology for authority ambiguity.
- Files a new agent should read next: `docs/architecture/product-topology.md` and ADR 0001.
