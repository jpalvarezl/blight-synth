---
title: "Task Packet — Issue 105: Svelte workspace and EngineClient boundary"
summary: Create the launchable production Svelte workspace and minimal mockable current-slice client contract.
status: current
updated: 2026-08-10
issue: 105
---

# Task Packet — Issue 105: Svelte workspace and EngineClient boundary

## Identity

- Issue: [#105](https://github.com/jpalvarezl/blight-synth/issues/105)
- Owner/status: jpalvarezl / in-progress
- Branch/worktree: `issue/105-svelte-workspace` / `/Users/jpalvarezl/code/blight-105`
- Base: `main` at `ba9065b8b01e75ea01ef31513ddb0e75c0027864`
- Head: the final issue commit at branch `HEAD` (the immutable SHA is reported in the implementation handoff because recording a commit's own SHA inside that commit is not possible)

## Goal

Create a launchable/buildable `gui/` Svelte/TypeScript workspace and a minimal typed/mockable EngineClient boundary for the current transport/gain/meter slice, without host or backend integration.

## Read first

1. [NOW](../../NOW.md)
2. [Frontend domain](../../domains/frontend.md)
3. Live issue #105
4. Existing root CI/README conventions; inspect tracker GUI only for visual/reference context if needed

## Scope / non-goals

In scope: Bun/Vite/Svelte/strict TypeScript workspace, static production output, minimal current-slice EngineClient types/interface, fake client, one mock parameter/meter development/test view, build/typecheck/test/lint commands, CI checks, and custom-base asset behavior.

Out of scope and unchanged: process supervision (#106), OSC/network (#107), production stores (#108), finished transport/gain/meter components (#110/#254/#112), desktop packaging (#109), Rust/backend code, plugins, MIDI, state migration, and archive work.

## Decisions

- Pin Bun 1.2.22 in `packageManager` and CI; commit Bun's text lockfile and use `--frozen-lockfile` for reproducible installs.
- Keep `EngineClient` limited to connection status/read-subscribe, async play/stop, normalized master-gain write, and stereo peak/RMS subscription. No generic request bus or snapshot/resync method was added because live #105 does not require one.
- Put host-neutral types in `engine-client.ts`; make the deterministic in-memory fake a separate implementation with fake-only event-driving methods.
- Inject the client as an `App.svelte` prop. The launch entry point chooses the fake; components do not construct or discover a host implementation.
- Use Vite `base: "./"` for custom-scheme/non-root embedding and separately exercise `/embedded/blight/` through a build command. A build verifier rejects development-server references and incorrect/missing assets.
- Use jsdom only for rendered component tests. Browser source has no Node/Bun process, UDP, CPAL, filesystem, or Rust dependency; the sole Bun API is in the build-verification tooling script.

## Exact touched paths

- `.github/workflows/ci.yml`
- `.gitignore`
- `docs/work/active/issue-105-svelte-workspace.md`
- `gui/.prettierignore`
- `gui/.prettierrc`
- `gui/README.md`
- `gui/bun.lock`
- `gui/eslint.config.js`
- `gui/index.html`
- `gui/package.json`
- `gui/scripts/verify-static-build.ts`
- `gui/src/App.svelte`
- `gui/src/App.test.ts`
- `gui/src/app.css`
- `gui/src/lib/engine-client.ts`
- `gui/src/lib/fake-engine-client.test.ts`
- `gui/src/lib/fake-engine-client.ts`
- `gui/src/main.ts`
- `gui/src/test-setup.ts`
- `gui/src/vite-env.d.ts`
- `gui/svelte.config.js`
- `gui/tsconfig.app.json`
- `gui/tsconfig.json`
- `gui/tsconfig.node.json`
- `gui/vite.config.ts`

## Plan / acceptance

- [x] Scaffold strict Svelte/TypeScript workspace and exact Bun commands.
- [x] Define minimal current-slice EngineClient and deterministic fake client.
- [x] Add a launchable injected mock view and focused browser/unit tests.
- [x] Verify static relative-base and custom-base builds with no runtime Vite dependency.
- [x] Add a lockfile-based, pinned-Bun frontend CI job without changing Rust jobs.
- [x] Run focused checks, docs validation, and independent scope/neutrality/reproducibility review.

## Verification

Run from repository root unless a leading `cd gui` is shown:

- `cd gui && bun install` — passed; generated `bun.lock` with Bun 1.2.22.
- `cd gui && bun install --frozen-lockfile` — passed with no changes across 327 packages.
- `cd gui && bun run dev -- --host 127.0.0.1 --port 4175` plus HTTP probe — passed; served the mock view with title `Blight`.
- `cd gui && bun run check` — passed; zero Svelte diagnostics, strict TypeScript, ESLint, and Prettier checks clean.
- `cd gui && bun run test` — passed; 2 files and 10 tests.
- `cd gui && bun run build` — passed; emitted `dist/index.html`, CSS, and JS, then verified two relative `./assets/` references.
- `cd gui && bun run build:custom-base` — passed; emitted and verified two `/embedded/blight/assets/` references.
- `python3 scripts/docs/check_docs.py` — passed for 24 documentation pages.
- `git diff --check` — passed.
- Browser-source forbidden-dependency search — no implementation dependency found (one explanatory contract comment mentions host-owned filesystem work).
- Live issue #105 acceptance checklist — all eight boxes marked only after the checks and review passed.
- Rust checks were not run because no Rust, workspace manifest, or shared architecture code changed; existing Rust CI steps are byte-for-byte unchanged.

## Review and remaining risks

An independent read-only peer review found no BLOCK or REVISE items. It identified relative asset handling and hidden host imports as the concrete acceptance gates. The implementation already enforces both: Vite owns all runtime asset references, default and custom-base output pass the post-build verifier, CSS has no external font/image URL, and a source search found no host/runtime import or global. Exact pins, `bun.lock`, frozen install, and the CI working directory address its remaining reproducibility questions. No speculative changes were made. (Preferred alternate-family peer attempts timed out; the completed review used an independent same-family peer.)

Remaining integration risks are intentionally owned by later approved issues: there is no process/OSC adapter yet, production connection stores do not exist, and an eventual desktop shell must load the verified static output. The numeric normalized-range contract is documented on the interface and enforced by the fake; a future production adapter must preserve it.

## Handoff

All #105 implementation and local verification are complete on `issue/105-svelte-workspace`. The branch starts at exact base `ba9065b8b01e75ea01ef31513ddb0e75c0027864`; the final head SHA and line-count calculation are included in the worker handoff. No #106–#112/#254 implementation, Rust code, NOW goal/non-goal, label, assignee, PR, push, merge, or issue closure was performed.
