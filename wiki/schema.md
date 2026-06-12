# Wiki Schema

Conventions for this wiki. Terse reference. Update via the log.

## Directory layout

```
entities/   one page per concrete code unit (crate, module, struct, trait, fn group)
concepts/   cross-cutting ideas, architecture, domain notions (RT/NRT, voice, effect chain)
sources/    notes derived from external/non-code sources (README, design docs, papers)
raw/        unprocessed dumps, transcripts, pasted snippets awaiting triage
```

Top-level pages: `index.md` (navigation), `log.md` (chronological history), `schema.md` (this file).

## Page format

Every page begins with YAML frontmatter, then Markdown body.

Generic page:

```yaml
---
tags: [crate, audio]
sources: [README.md, "[[concepts/rt-nrt-threading]]"]
last-updated: 2025-01-15
---
```

File-tracking page (a page that mirrors a specific source file) adds:

```yaml
---
tags: [module]
sources: []
last-updated: 2025-01-15
source-file: audio_backend/src/lib.rs
source-sha: <git-blob-or-content-sha>
source-mtime: 2025-01-15T10:30:00Z
last-synced: 2025-01-15T10:30:00Z
---
```

Frontmatter keys:
- `tags` — list of lowercase keywords for grouping/filtering.
- `sources` — list of provenance refs: file paths, wikilinks, or URLs.
- `last-updated` — date (YYYY-MM-DD) the human-readable body last changed.
- `source-file` — repo-relative path this page tracks (file-tracking pages only).
- `source-sha` — content hash of `source-file` at `last-synced`.
- `source-mtime` — filesystem/git mtime of `source-file` at `last-synced`.
- `last-synced` — timestamp the page was last reconciled against `source-file`.

## Wikilinks

Link between pages with `[[Page]]` or `[[path/to/page]]`. Use the path form when ambiguous. Display text: `[[path/to/page|alias]]`.

## Callouts / flags

Contradiction between sources or between wiki and code:

```
> [!contradiction]
> README says `sequencer/` is a crate, but Cargo.toml workspace members omit it.
```

File-change flags on file-tracking pages (set during sync, cleared once reconciled):

```
> [!updated]
> source-file changed since last-synced; body may be stale.

> [!renamed]
> source-file moved from <old-path> to <new-path>.

> [!removed]
> source-file no longer exists in the repo.
```

## Log entry format

`log.md` is append-only, newest at bottom. Each entry:

```
## [YYYY-MM-DD HH:MM] <kind> | <session-id> | <one-line>
```

Kinds:
- `ingest` — pulled in content from a source.
- `scaffold` — created structural/empty pages.
- `sync` — reconciled file-tracking pages against source files.
- `lint` — consistency/link/frontmatter checks.
- `manual-flush` — human-forced write of pending notes.
- `manual-rotate` — human-forced log rotation/archival.
- `auto` — automated maintenance not covered above.

One-line is a terse summary. Detail goes in the referenced pages, not the log.
