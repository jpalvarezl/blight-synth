---
title: Documentation Tooling
summary: Dependency-free commands for validating the vault and mirroring GitHub roadmap status.
status: current
updated: 2026-07-14
---

# Documentation Tooling

Run from the repository root:

```bash
python3 scripts/docs/sync_roadmap.py
python3 scripts/docs/check_docs.py
```

`sync_roadmap.py` queries issues carrying `roadmap-task` and writes `docs/work/burndown.md`. GitHub remains canonical. `--check` fails when the committed snapshot differs; `--stdout` previews output.

`check_docs.py` requires frontmatter (`title`, `summary`, `status`) and validates local relative Markdown links. It deliberately has no YAML or Markdown package dependency.

Issue [#131](https://github.com/jpalvarezl/blight-synth/issues/131) owns CI integration.
