---
title: Documentation Tooling
summary: Dependency-free commands for validating the vault and mirroring GitHub roadmap status.
status: current
updated: 2026-07-25
---

# Documentation Tooling

Run from the repository root:

```bash
python3 scripts/docs/reconcile_work.py --check
python3 scripts/docs/reconcile_work.py --fix-docs
python3 scripts/docs/sync_roadmap.py
python3 scripts/docs/check_docs.py
```

`sync_roadmap.py` queries issues carrying `roadmap-task` and writes `docs/work/burndown.md`. GitHub remains canonical. `--check` fails when the committed snapshot differs; `--stdout` previews output and is used by CI to exercise the generator without coupling code PRs to unrelated live issue changes.

`check_docs.py` requires frontmatter (`title`, `summary`, `status`) and validates local relative Markdown links. It deliberately has no YAML or Markdown package dependency.

`reconcile_work.py` compares live GitHub issue state with active packets, their generated index, standardized `Depends on:` lines, and the committed burndown. Its default/`--check` mode is read-only. Packet/status contradictions are errors; ordinary point-in-time burndown drift is a warning. Live `--check` is a task-boundary command, not a CI gate, so unrelated concurrent GitHub metadata changes do not fail code PRs; CI runs the offline unit suite. `--fix-docs` removes only packets for closed issues and regenerates the index/burndown—open packets are never deleted automatically. GitHub label mutation is intentionally gated behind the explicit `--fix-github` flag. Mutations are sequential rather than transactional: if `gh` fails mid-run, rerun the read-only check, correct the reported partial state, and retry.

Run its dependency-free tests with:

```bash
python3 -m unittest scripts.docs.test_reconcile_work
```

Issue [#131](https://github.com/jpalvarezl/blight-synth/issues/131) owns CI integration.
