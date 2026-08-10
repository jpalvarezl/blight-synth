---
title: NOW — Standalone Svelte Vertical Slice
summary: Human-approved current product slice and agent-maintained execution state after the M0 recovery reset.
status: current
updated: 2026-08-10
issues: [105, 106, 107, 108, 109, 110, 112, 253, 254]
---

# NOW — Standalone Svelte Transport, Gain, and Meter

## Approved product slice

Deliver the first user-visible standalone Svelte vertical slice over the existing M0 backend.

The UI must launch, connect to the standalone DSP process, control play/stop and master gain, and display stereo peak/RMS levels. The existing tracker/egui path remains a reference workflow and must keep working.

GitHub parent: [#253](https://github.com/jpalvarezl/blight-synth/issues/253).

## Definition of done

- A production TypeScript/Svelte workspace builds and launches in development.
- Browser components depend on a typed, mockable `EngineClient` boundary.
- The host bridge starts/connects to the existing standalone process.
- Play and stop work end to end.
- A normalized master-gain control works end to end.
- Stereo peak/RMS values update in the UI.
- Existing tracker and standalone OSC smoke paths remain usable.
- One documented command runs the development vertical slice.

## Explicit non-goals

- New Rust engine architecture, crate extraction, or generic lifecycle work.
- Plugins, JUCE, APVTS, AUv3, or MIDI.
- Generic portable state or migration infrastructure.
- Multiple composition runtimes or final tracker/ORCA/hybrid selection.
- Arbitrary routing, mixer redesign, or speculative RT scale work.
- Restoring archived post-M0 implementations without a demonstrated blocker.

## Execution state — agent maintained

### Active

_None._

### Ready

- [#105 — Create the production TypeScript/Svelte workspace and host-neutral UI boundary](https://github.com/jpalvarezl/blight-synth/issues/105)

### Blocked sequence

- [#106 — Standalone DSP process supervision](https://github.com/jpalvarezl/blight-synth/issues/106) — depends on #105.
- [#107 — Standalone OSC EngineClient adapter](https://github.com/jpalvarezl/blight-synth/issues/107) — depends on #105/#106.
- [#108 — Connection-aware Svelte stores](https://github.com/jpalvarezl/blight-synth/issues/108) — depends on #107.
- [#110 — Transport UI](https://github.com/jpalvarezl/blight-synth/issues/110), [#112 — meter UI](https://github.com/jpalvarezl/blight-synth/issues/112), and [#254 — gain control](https://github.com/jpalvarezl/blight-synth/issues/254) — depend on #108 and may proceed in parallel.
- [#109 — packaged shell](https://github.com/jpalvarezl/blight-synth/issues/109) — only after the development vertical slice works.

## Archive

The abandoned post-M0 architecture-heavy implementation is preserved on `archive/post-m0-agent-refactor-2026-08` and indexed in [#252](https://github.com/jpalvarezl/blight-synth/issues/252). It is reference material, not current architecture.

## Governance

Agents maintain execution state, issue metadata, packets, and verification. Changing the approved slice, definition of done, or non-goals requires explicit human approval. Named PR merges require explicit human approval.
