# Blight GUI

Production Svelte/TypeScript workspace for Blight's host-neutral browser UI. The
launchable view uses a deterministic `FakeEngineClient`; process supervision and
OSC integration belong to later issues and are intentionally absent here.

## Requirements and install

Use Bun 1.2.22 (the version pinned by `package.json` and CI):

```bash
cd gui
bun install
```

After the lockfile exists, reproducible installs use:

```bash
bun install --frozen-lockfile
```

## Commands

Run these from `gui/`:

```bash
bun run dev                 # launch the Vite development view
bun run check               # strict Svelte/TypeScript, lint, and format checks
bun run test                # unit and rendered component tests
bun run build               # relative-base static production build in dist/
bun run build:custom-base   # verify an absolute /embedded/blight/ base build
```

`bun run build` uses relative `./assets/...` references, so a desktop shell can
load `dist/index.html` below a non-root path or through a custom URL scheme
without a Vite server. `build:custom-base` separately proves Vite's explicit
base override. Both builds run `scripts/verify-static-build.ts`, which rejects
development-server references and missing or incorrectly based assets.

## Browser boundary

Browser/Svelte code depends only on `src/lib/engine-client.ts`. It exposes the
current slice's connection status, play/stop requests, normalized master-gain
write, and stereo peak/RMS events. `src/lib/fake-engine-client.ts` is a
predictable in-memory implementation for development and tests. It has no
process, UDP, audio-device, filesystem, Rust, Node, or Bun dependency.
