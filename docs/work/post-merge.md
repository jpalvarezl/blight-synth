---
title: Post-Merge Work Transition
summary: Deterministic agent procedure for closing one task, updating NOW/GitHub state, and exposing the next authorized work.
status: current
updated: 2026-08-10
issues: [146, 253]
---

# Post-Merge Work Transition

Use this procedure after the user explicitly authorizes merging a named PR. It performs housekeeping only: it does not change the approved product slice, accept an ADR, start another implementation, or authorize another merge.

## Required inputs

- merged PR number;
- closing issue number;
- current slice parent from [NOW](../NOW.md);
- completed packet path under `docs/work/active/`.

## Deterministic procedure

### 1. Confirm the named merge

```bash
gh pr view PR --repo jpalvarezl/blight-synth \
  --json state,mergedAt,mergeCommit,url
gh issue view ISSUE --repo jpalvarezl/blight-synth \
  --json state,stateReason,closedAt,url
```

Stop if the PR is not merged or the issue did not close as intended. Never infer that a green PR merged.

### 2. Update local `main`

```bash
git checkout main
git pull --ff-only origin main
git status --short --branch
```

Stop if `main` is dirty or cannot fast-forward. Never overwrite another worktree.

### 3. Preserve durable information and remove the packet

Move any durable contract/product information into its canonical document. Do not copy implementation transcripts into permanent docs. Then remove the completed packet:

```bash
rm docs/work/active/issue-ISSUE-*.md
```

An active packet is temporary handoff state and must not survive its closed issue.

### 4. Query the current slice live

Read the parent number from NOW, then query its subissues and each child’s live body/labels:

```bash
gh api graphql -f query='query {
  repository(owner:"jpalvarezl", name:"blight-synth") {
    issue(number:PARENT) {
      subIssues(first:50) {
        nodes { number title state labels(first:20) { nodes { name } } }
      }
    }
  }
}'

gh issue view CHILD --repo jpalvarezl/blight-synth \
  --json number,state,body,labels,assignees,url
```

Only children of the NOW parent are eligible for workflow promotion. Long-term `roadmap-task` or `deferred` issues outside NOW are untouched.

### 5. Evaluate standardized dependencies

Each child has at most one machine-readable line:

```text
Depends on: #123, #456
```

For every open NOW child:

- all listed dependencies closed → desired state `status:ready`;
- any dependency open → desired state `status:blocked`;
- already assigned with active packet → desired state `status:in-progress`;
- no status is valid only for intentionally untriaged work, not an approved ordered child.

Never promote an issue carrying `deferred`. Never add dependencies or acceptance criteria during post-merge housekeeping.

### 6. Apply only necessary workflow-label changes

```bash
gh issue edit CHILD --repo jpalvarezl/blight-synth \
  --remove-label status:blocked --add-label status:ready
```

Or the reverse when a dependency remains. Use at most one workflow label. Do not change estimates, milestones, product scope, or assignees except to clear a completed owner when necessary.

### 7. Update NOW’s agent-maintained execution section

Do not change **Approved product slice**, **Definition of done**, or **Explicit non-goals**.

Update only:

- Active — remove the completed issue/packet;
- Ready — list every NOW child whose live desired state is ready;
- Blocked sequence — keep dependency descriptions synchronized with live issue bodies;
- progress notes, if present — concise facts only.

NOW and live GitHub must name the same ready/in-progress children.

### 8. Regenerate and validate documentation

```bash
python3 scripts/docs/sync_roadmap.py
python3 scripts/docs/check_docs.py
python3 scripts/docs/sync_roadmap.py --check
git diff --check
```

Inspect the generated **Current execution snapshot**. It must point to NOW and show only ready/in-progress work, not the earliest deferred milestone.

### 9. Verify invariants

Before committing, verify:

- every `status:in-progress` NOW child has one assignee and active packet;
- every active packet maps to one open in-progress issue;
- every `status:ready` child belongs to NOW and has no unresolved dependency;
- every unresolved child is blocked;
- deferred issues were not changed;
- the completed issue is closed and absent from Active/Ready;
- the parent remains open unless its complete definition of done is explicitly satisfied.

### 10. Commit one housekeeping transition

```bash
git add docs/NOW.md docs/work/active docs/work/burndown.md
git commit -m "Reconcile work after closing #ISSUE"
git push origin main
```

Include other durable docs only if the merged PR intentionally changed them.

### 11. Report and stop

Report:

- merged PR and closed issue;
- packet removed;
- workflow labels changed;
- exact ready/in-progress/blocked NOW children;
- verification commands and results;
- next authorized issue(s).

Do not start the next issue in the merge turn unless the user explicitly requests continued implementation. Merge permission applies only to the named PR.

## State invariants summary

| State | Required conditions |
|---|---|
| In progress | Open NOW child, one assignee, one active packet, one branch/worktree |
| Ready | Open NOW child, all standardized dependencies closed, no assignee/packet required |
| Blocked | Open NOW child, at least one dependency open |
| Deferred | Outside current NOW execution; never auto-promoted |
| Done | Closed issue, no active packet |

## Failure handling

If GitHub, NOW, packet state, or dependencies disagree, do not choose whichever is convenient. Report the contradiction and stop. The repair must be a separate, reviewable housekeeping action before implementation continues.
