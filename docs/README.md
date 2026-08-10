---
title: Knowledge Base Home
summary: Entry point and targeted-reading map for the committed Obsidian-compatible project vault.
status: current
updated: 2026-08-10
---

# Blight Synth Knowledge Base

Open `docs/` as an Obsidian vault, or browse it directly in GitHub. The vault uses standard relative Markdown links and YAML frontmatter; no Obsidian plugin is required.

## Read by intent

| Intent | Start here | Then read |
|---|---|---|
| Understand current work | [NOW](NOW.md) | [Current product spec](spec/current-product.md) |
| Change DSP/audio processing | [Audio engine domain](domains/audio-engine.md) | [System boundaries](architecture/system-boundaries.md) |
| Change tracker/generative composition | [Composition domain](domains/composition.md) | Issue [#145](https://github.com/jpalvarezl/blight-synth/issues/145) |
| Change standalone CPAL/OSC | [Standalone host domain](domains/standalone-host.md) | [OSC spec](osc-spec.md) |
| Change Svelte/TypeScript UI | [Frontend domain](domains/frontend.md) | Current slice [#253](https://github.com/jpalvarezl/blight-synth/issues/253) |
| Consider plugin work | [Plugins domain](domains/plugins.md) | [ADR 0001](decisions/0001-product-and-host-priorities.md) |
| Pick up or hand off work | [NOW](NOW.md) | [Work system](work/README.md) |
| Reconcile after a merge | [Post-merge transition](work/post-merge.md) | [Active packets](work/active/README.md) |
| Record a decision | [Decision index](decisions/README.md) | [ADR template](templates/adr.md) |

## Sections

- [NOW](NOW.md) — human-approved current slice and agent-maintained execution state.
- [Specification](spec/current-product.md) — long-term product commitments and deliberately open questions.
- [Architecture](architecture/README.md) — boundaries and cross-domain contracts.
- [Domains](domains/README.md) — focused context packets and code entry points.
- [Decisions](decisions/README.md) — durable ADR history.
- [Work](work/README.md) — task status, context packets, parallel workflow, and burndown.
- [Templates](templates/task-packet.md) — repeatable task and decision structure.

## Freshness model

Pages declare a frontmatter `status`:

- `current` — canonical current guidance.
- `accepted` — durable decision.
- `draft` — intended direction owned by an open issue; not implemented fact.
- `generated` — produced from another source and never manually edited.
- `historical` — retained only for background.

Code and tests remain the authority for current runtime behavior. A draft architecture page describes the target and must link to its owning issue.
