# AGENTS.md — Product and Context Router

This file routes humans and coding agents to the smallest useful context set. It is not a project encyclopedia.

## Product execution gate

Before selecting or continuing work, read [`docs/NOW.md`](docs/NOW.md). It is the canonical **current product slice**: the approved user-visible outcome, definition of done, and explicit non-goals.

- Work may proceed autonomously only when it directly belongs to the approved slice.
- A ready GitHub issue outside NOW is not authorization to implement it.
- Backend architecture work requires a concrete blocker to the current slice and explicit user approval.
- Long-term specifications remain valid, but NOW determines temporal priority.

## Start every task this way

1. Read [`docs/NOW.md`](docs/NOW.md).
2. Read the assigned GitHub issue: `gh issue view <number> --repo jpalvarezl/blight-synth`.
3. Read [`docs/README.md`](docs/README.md).
4. Read exactly one relevant page from [`docs/domains/`](docs/domains/README.md).
5. Follow only the contracts/ADRs listed under that page's **Read first** section.
6. Inspect the named code entry points; expand outward only when imports or behavior require it.
7. If the task is active, create/update one focused context packet from [`docs/templates/task-packet.md`](docs/templates/task-packet.md).

Do not preload the entire repository, all documentation, closed issues, or archive history.

## Human governance and agent maintenance

The user approves:

- the current product slice, definition of done, and non-goals;
- any expansion beyond that slice;
- acceptance/supersession of ADRs;
- merges for the exact named PRs under discussion.

Agents maintain the process housekeeping:

- NOW's active/ready/progress sections;
- GitHub issue labels, assignments, dependencies, and review-sized splits;
- task packets, verification records, and generated docs;
- implementation, tests, PR preparation, and review-thread responses.

Splitting may redistribute already-approved acceptance criteria but must not add product requirements. “Continue” means continue the current task/slice; it does not authorize a new roadmap direction or future merges.

## Autonomous continuation across sessions

1. If NOW links an active packet, continue it.
2. Otherwise choose a `status:ready` child explicitly listed under NOW's approved slice.
3. Implement and open a PR, then stop for human review/merge.
4. If no approved ready child exists, propose options and stop; do not invent a new architecture program.

## Sources of truth

| Information | Canonical source |
|---|---|
| Current temporal priority and non-goals | `docs/NOW.md` |
| Long-term product direction | `docs/spec/` and human-approved ADRs |
| Architecture invariants | `docs/architecture/`, tests |
| Current implementation behavior | Code and tests |
| Task scope and acceptance criteria | GitHub issue, bounded by NOW |
| Status, owner, dependencies | GitHub issue metadata/labels |
| In-flight handoff | `docs/work/active/issue-*.md` |

When sources disagree: code/tests describe current behavior; NOW controls work selection; human-approved specifications/ADRs describe intended direction. Report contradictions instead of silently choosing.

## Domain routing

- Audio/DSP/RT: [`docs/domains/audio-engine.md`](docs/domains/audio-engine.md)
- Composition runtimes: [`docs/domains/composition.md`](docs/domains/composition.md)
- Standalone CPAL/OSC host: [`docs/domains/standalone-host.md`](docs/domains/standalone-host.md)
- Svelte/TypeScript UI: [`docs/domains/frontend.md`](docs/domains/frontend.md)
- Optional plugins: [`docs/domains/plugins.md`](docs/domains/plugins.md)

## YAGNI and evidence rules

- Do not implement for hypothetical plugins, APVTS, MIDI, multiple runtimes, migration formats, or scale unless the current slice is blocked without it.
- New crates, public abstractions, queues, schemas, and concurrency protocols require evidence from current code or a demonstrated feature.
- Prefer the existing bounded/simple path until measurement or user-visible behavior proves it insufficient.
- Reviewer feedback is mandatory only for correctness, data loss, security, current compatibility, or an approved contract. Defer speculative extensibility and optional hardening.
- After at most two backend-only PRs, deliver or directly enable a visible product capability unless the user explicitly approves otherwise.

## Parallel and subagent rules

- One issue per branch/worktree: `issue/<number>-<slug>`.
- Record expected touched paths before editing; do not overlap public contracts/schemas without coordination.
- Subagents may not be asked to “complete an epic.” Give one bounded implementation or read-only review task.
- Long tasks use planner → parent checkpoint → implementation → parent checkpoint. Stop/re-split when a worker stalls or scope grows.
- Never overwrite another task's uncommitted work.

## Reviewability and merge rules

- Prefer one coherent concept per PR; roughly 500–800 meaningful lines is a planning target, 800–1,000 is acceptable for tightly coupled tests.
- Above ~1,000 meaningful lines, actively seek a stacked split. Generated files, fixtures, lockfiles, and mechanical migrations count less than behavioral logic.
- Do not combine contract design, core implementation, host integration, and migration unless separation would be unsafe or untestable.
- PR descriptions state user-visible payoff/blocker removed, production lines added/deleted, public contracts, non-goals, and simpler alternative considered.
- Never merge automatically. Merge permission applies only to the named PR(s) and expires after that action.
- After an authorized merge, follow [`docs/work/post-merge.md`](docs/work/post-merge.md) exactly, report the resulting NOW state, and stop unless the user explicitly requests continued implementation.

## ADR rules

Agents may draft proposed ADRs only when the current slice is blocked by a durable cross-domain decision. Only the user may approve marking an ADR accepted. No implementation depends on a newly drafted ADR before approval.

## Archived post-M0 work

The branch `archive/post-m0-agent-refactor-2026-08` and tag `pre-recovery-2026-08` preserve the abandoned architecture-heavy implementation from before the recovery reset.

Before reimplementing a feature that may exist there, read GitHub issue [#252](https://github.com/jpalvarezl/blight-synth/issues/252).

The archive is reference material, not current architecture:

- do not preload it;
- do not cherry-pick wholesale without reviewing dependencies;
- prefer the smallest implementation required by NOW;
- archived ADRs do not override current main/spec/NOW.

Remove this section when issue #252's removal condition is met.

## Documentation rules

- Use standard relative Markdown links and YAML frontmatter under `docs/`.
- Prefer short routing/contract pages over source-code copies.
- GitHub owns live task status; never manually edit generated burndown content.
- Run `python3 scripts/docs/check_docs.py` after docs changes and regenerate roadmap docs only when intentionally changing live metadata.
