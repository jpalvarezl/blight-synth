# AGENTS.md — Context Router

This file is intentionally short. It routes humans and coding agents to the smallest useful context set; it is not a project encyclopedia.

## Start every task this way

When selecting rather than receiving an issue, query live GitHub state first:

```bash
python3 scripts/docs/reconcile_work.py --check
gh issue list --repo jpalvarezl/blight-synth --state open --label status:ready
```

Never select work solely from the offline burndown snapshot.

1. Read the assigned GitHub issue: `gh issue view <number> --repo jpalvarezl/blight-synth`.
2. Read [`docs/README.md`](docs/README.md).
3. Read exactly one relevant page from [`docs/domains/`](docs/domains/README.md).
4. Follow only the contracts/ADRs listed under that page's **Read first** section.
5. Inspect the named code entry points; expand outward only when imports or behavior require it.
6. If the task is active, create/update one focused context packet from [`docs/templates/task-packet.md`](docs/templates/task-packet.md).

Do not preload the entire repository, all documentation, closed issues, or the retired wiki history.

## Sources of truth

| Information | Canonical source |
|---|---|
| Product direction and accepted decisions | `docs/spec/`, `docs/decisions/` |
| Architecture contracts and invariants | `docs/architecture/`, tests |
| Current implementation behavior | Code and tests |
| Task scope and acceptance criteria | GitHub issue |
| Status, owner, milestone, dependencies | GitHub issue metadata/labels |
| Current offline dashboard | `docs/work/burndown.md` (generated) |
| In-flight branch state and handoff | `docs/work/active/issue-*.md` |

When sources disagree: tests/code describe current behavior; accepted ADRs describe intended direction. Report the contradiction in the task packet instead of silently choosing.

## Domain routing

- Audio/DSP/RT: [`docs/domains/audio-engine.md`](docs/domains/audio-engine.md)
- Composition runtimes (tracker, generative, ORCA-like): [`docs/domains/composition.md`](docs/domains/composition.md)
- Standalone CPAL/OSC host: [`docs/domains/standalone-host.md`](docs/domains/standalone-host.md)
- Svelte/TypeScript UI: [`docs/domains/frontend.md`](docs/domains/frontend.md)
- Optional VST3/AU/AUv3 work: [`docs/domains/plugins.md`](docs/domains/plugins.md)

## Parallel work rules

- One issue per branch/worktree; branch name `issue/<number>-<slug>`.
- Claim leaf work with the `status:in-progress` label, an assignee, and an active task packet.
- `size:epic` issues are tracked through child packets; do not mark an epic in progress unless it has its own branch/packet.
- Record expected touched paths in the packet before editing.
- Two tasks may not concurrently change the same public contract, schema, migration, Cargo workspace boundary, or protocol surface without an explicit coordination note.
- Implementation tasks depend on accepted contract issues; do not invent a local competing abstraction.
- Rebase/update from the agreed base before handoff. Never overwrite another task's uncommitted work.
- Finish with focused tests, packet status, unresolved questions, and exact verification commands.

## Reviewability and PR slicing

- Prefer one coherent, independently explainable concept per PR over a rigid line limit.
- As a planning signal, aim for roughly 500–800 meaningful changed lines including tests; 800–1,000 is acceptable when implementation and tests are tightly coupled.
- Generated files, lockfiles, fixtures, and mechanical call-site migration count less than architectural or behavioral logic.
- Above roughly 1,000 meaningful lines, pause and actively look for a clean stacked split before continuing.
- Avoid combining schema/contract design, core implementation, host integration, and data migration in one PR unless separating them would make a PR untestable or unsafe.
- Use stacked dependent PRs/issues when necessary. Each PR description must state its single primary behavior, explicit non-goals, and the next deferred slice.
- If review reveals that a nominal leaf still contains several concepts, split the issue again rather than optimizing only for delivery velocity.

## Documentation rules

- Use standard relative Markdown links; they work in GitHub and Obsidian.
- Add YAML frontmatter to pages under `docs/`.
- Prefer short routing/contract pages over copies of source code.
- Record durable decisions as ADRs; do not rewrite accepted history without a superseding ADR.
- GitHub owns live task status. Never manually edit generated burndown content.
- Run `python3 scripts/docs/check_docs.py` after documentation changes.
- Run `python3 scripts/docs/reconcile_work.py --fix-docs` after GitHub roadmap metadata changes or issue completion.
- Before handoff, run `python3 scripts/docs/reconcile_work.py --check` and `python3 scripts/docs/check_docs.py`.
