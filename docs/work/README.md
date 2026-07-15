---
title: Work and Parallelization System
summary: GitHub-backed status, focused context packets, ownership, handoff, and burndown rules.
status: current
updated: 2026-07-14
issues: [146]
---

# Work and Parallelization System

## Source of truth

GitHub Issues own task scope, acceptance criteria, milestone, state, labels, assignee, and dependency links. The [burndown](burndown.md) is a generated offline mirror for Obsidian; never edit it manually.

## Labels

### Membership

- `roadmap-task` — included in the current canonical roadmap/dashboard.

### Workflow state

Use at most one:

- no status label — backlog/not triaged;
- `status:ready` — dependencies satisfied and safe to claim;
- `status:in-progress` — actively owned; requires assignee and task packet;
- `status:blocked` — cannot progress; issue or packet must identify blocker;
- closed issue — done or explicitly cancelled, with closing rationale.

### Estimate

Use exactly one before implementation:

- `size:S` — 1 planning point; focused change.
- `size:M` — 3 points; normal reviewable task.
- `size:L` — 5 points; should be split if it crosses contracts/domains.
- `size:epic` — not estimable; must be split before implementation.

Points are planning signals, not time estimates or performance targets.

## Claiming work

1. Confirm dependencies and `status:ready`.
2. Assign the issue and replace status with `status:in-progress`.
3. Create `docs/work/active/issue-<number>-<slug>.md` from the [task packet template](../templates/task-packet.md).
4. Record branch/worktree, base SHA, read-first links, expected touched paths, and contract impact.
5. Create branch/worktree: `git worktree add ../blight-<number> -b issue/<number>-<slug> <base>`.
6. Update the packet at meaningful handoff points—not after every edit.

## Parallel safety

Work may proceed in parallel when touch sets and contract ownership do not overlap. Serialize changes to:

- Cargo workspace/crate boundaries;
- public engine/event/parameter APIs;
- persisted schemas/migrations;
- public OSC/protocol messages;
- shared frontend `EngineClient` contracts;
- generated FFI headers or plugin parameter IDs.

If overlap becomes necessary, nominate one contract owner and make other branches consume that branch/PR explicitly.

## Handoff and completion

A useful handoff includes:

- exact completed/incomplete acceptance criteria;
- decisions and contradictions discovered;
- changed/touched paths;
- commands run and their results;
- remaining failures/risks;
- next smallest action;
- branch and base/head SHAs.

Before closing:

- tests and format/lint policy pass;
- durable contract/decision docs are updated;
- task packet is marked complete or removed after its durable information is moved;
- PR links/closes the issue;
- GitHub status remains canonical;
- regenerate `burndown.md`.

## Commands

```bash
python3 scripts/docs/sync_roadmap.py
python3 scripts/docs/check_docs.py
```

Use `python3 scripts/docs/sync_roadmap.py --check` locally after intentional roadmap updates. CI exercises `--stdout` so unrelated live issue changes do not make an otherwise valid code PR fail; the committed page is explicitly a point-in-time offline snapshot.
