---
title: Generated Roadmap Burndown
summary: Offline Obsidian snapshot generated from GitHub roadmap issue metadata.
status: generated
source-updated: 2026-07-18T20:18:12Z
generator: scripts/docs/sync_roadmap.py
---

# Generated Roadmap Burndown

> [!warning] Generated file
> GitHub Issues are canonical. Do not edit this page manually. Run `python3 scripts/docs/sync_roadmap.py`.

Data snapshot through `2026-07-18T20:18:12Z`.

## Summary

| Milestone | Open | Done | Ready | In progress | Blocked | Backlog | Sized points done/total | Unsized |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| M0: Architecture & Repository Baseline | 2 | 12 | 0 | 2 | 0 | 0 | 26/27 | 3 |
| M1: Host-Independent Engine & RT Contracts | 10 | 0 | 0 | 0 | 0 | 10 | 0/0 | 10 |
| M2: Standalone Host & Control Protocol | 6 | 0 | 0 | 0 | 1 | 5 | 0/3 | 5 |
| M3: JS/TS Composition UI & Standalone App | 9 | 0 | 0 | 0 | 0 | 9 | 0/0 | 9 |
| M4 (Optional): Desktop Plugins (VST3/AU) | 7 | 0 | 0 | 0 | 0 | 7 | 0/0 | 7 |
| M5 (Optional): AUv3 & Distribution | 3 | 0 | 0 | 0 | 0 | 3 | 0/0 | 3 |

## Current milestone

### M0: Architecture & Repository Baseline

- [ ] [#130](https://github.com/jpalvarezl/blight-synth/issues/130) Refactor workspace boundaries around a host-independent engine — `in-progress`, `epic`, @jpalvarezl
- [ ] [#157](https://github.com/jpalvarezl/blight-synth/issues/157) Finalize and enforce the M0 crate dependency graph — `in-progress`, `S`, @jpalvarezl

## All roadmap tasks

### M0: Architecture & Repository Baseline

- [x] [#128](https://github.com/jpalvarezl/blight-synth/issues/128) Integrate the modular-refactor branch into `main` and remove duplicate scaffolding — `done`, `unsized`, @jpalvarezl
- [x] [#129](https://github.com/jpalvarezl/blight-synth/issues/129) Record the primary product, open composition model, and optional plugin topology — `done`, `S`, @jpalvarezl
- [ ] [#130](https://github.com/jpalvarezl/blight-synth/issues/130) Refactor workspace boundaries around a host-independent engine — `in-progress`, `epic`, @jpalvarezl
- [x] [#131](https://github.com/jpalvarezl/blight-synth/issues/131) Establish CI and a repository quality baseline — `done`, `M`, @jpalvarezl
- [x] [#146](https://github.com/jpalvarezl/blight-synth/issues/146) Establish the human/LLM knowledge base, context packets, and generated burndown — `done`, `M`, @jpalvarezl
- [x] [#149](https://github.com/jpalvarezl/blight-synth/issues/149) Make `utils` and `os_dls` strict-Clippy clean — `done`, `S`, @jpalvarezl
- [x] [#150](https://github.com/jpalvarezl/blight-synth/issues/150) Make DSP and remaining workspace targets strict-Clippy clean — `done`, `M`, @jpalvarezl
- [x] [#154](https://github.com/jpalvarezl/blight-synth/issues/154) Move sample and platform resource loading out of `dsp` — `done`, `M`, @jpalvarezl
- [x] [#155](https://github.com/jpalvarezl/blight-synth/issues/155) Extract the host-independent engine crate mechanically — `done`, `epic`, @jpalvarezl
- [x] [#156](https://github.com/jpalvarezl/blight-synth/issues/156) Isolate CPAL/OSC and simplify the standalone host threading model — `done`, `M`, @jpalvarezl
- [ ] [#157](https://github.com/jpalvarezl/blight-synth/issues/157) Finalize and enforce the M0 crate dependency graph — `in-progress`, `S`, @jpalvarezl
- [x] [#162](https://github.com/jpalvarezl/blight-synth/issues/162) Extract the host-independent engine render core — `done`, `M`, @jpalvarezl
- [x] [#163](https://github.com/jpalvarezl/blight-synth/issues/163) Separate engine command ownership from standalone transport commands — `done`, `M`, @jpalvarezl
- [x] [#164](https://github.com/jpalvarezl/blight-synth/issues/164) Add deterministic golden offline renders and finalize #155 — `done`, `M`, @jpalvarezl

### M1: Host-Independent Engine & RT Contracts

- [ ] [#101](https://github.com/jpalvarezl/blight-synth/issues/101) Implement the coalesced real-time parameter pipeline — `backlog`, `unsized`, unassigned
- [ ] [#121](https://github.com/jpalvarezl/blight-synth/issues/121) Design the canonical parameter manifest and host bindings — `backlog`, `unsized`, unassigned
- [ ] [#132](https://github.com/jpalvarezl/blight-synth/issues/132) Define the host-independent `Engine` lifecycle and offline render harness — `backlog`, `unsized`, unassigned
- [ ] [#133](https://github.com/jpalvarezl/blight-synth/issues/133) Enforce and test the real-time safety contract — `backlog`, `unsized`, unassigned
- [ ] [#134](https://github.com/jpalvarezl/blight-synth/issues/134) Implement sample-accurate event scheduling across block sizes — `backlog`, `unsized`, unassigned
- [ ] [#135](https://github.com/jpalvarezl/blight-synth/issues/135) Introduce typed instance IDs and versioned instrument/effect definitions — `backlog`, `unsized`, unassigned
- [ ] [#136](https://github.com/jpalvarezl/blight-synth/issues/136) Define and implement the scalable audio routing graph — `backlog`, `unsized`, unassigned
- [ ] [#137](https://github.com/jpalvarezl/blight-synth/issues/137) Complete polyphony, note identity, and voice-allocation semantics — `backlog`, `unsized`, unassigned
- [ ] [#138](https://github.com/jpalvarezl/blight-synth/issues/138) Define versioned engine state snapshots and migrations — `backlog`, `unsized`, unassigned
- [ ] [#145](https://github.com/jpalvarezl/blight-synth/issues/145) Decouple composition runtimes from the audio engine through an event-source contract — `backlog`, `unsized`, unassigned

### M2: Standalone Host & Control Protocol

- [ ] [#104](https://github.com/jpalvarezl/blight-synth/issues/104) Prove one confirmed parameter round-trip through the standalone host — `backlog`, `unsized`, unassigned
- [ ] [#120](https://github.com/jpalvarezl/blight-synth/issues/120) Define the versioned public control protocol and OSC mapping — `backlog`, `unsized`, unassigned
- [ ] [#122](https://github.com/jpalvarezl/blight-synth/issues/122) Implement portable project I/O and standalone load/save operations — `backlog`, `unsized`, unassigned
- [ ] [#123](https://github.com/jpalvarezl/blight-synth/issues/123) Implement standalone process lifecycle, discovery, and recovery — `backlog`, `unsized`, unassigned
- [ ] [#139](https://github.com/jpalvarezl/blight-synth/issues/139) Migrate the standalone CPAL/OSC host onto the shared engine — `backlog`, `unsized`, unassigned
- [ ] [#161](https://github.com/jpalvarezl/blight-synth/issues/161) Remove Tokio from the standalone host control loop — `blocked`, `M`, unassigned

### M3: JS/TS Composition UI & Standalone App

- [ ] [#105](https://github.com/jpalvarezl/blight-synth/issues/105) Create the production TypeScript/Svelte workspace and host-neutral UI boundary — `backlog`, `unsized`, unassigned
- [ ] [#106](https://github.com/jpalvarezl/blight-synth/issues/106) Implement standalone DSP process supervision — `backlog`, `unsized`, unassigned
- [ ] [#107](https://github.com/jpalvarezl/blight-synth/issues/107) Implement the standalone OSC `EngineClient` adapter — `backlog`, `unsized`, unassigned
- [ ] [#108](https://github.com/jpalvarezl/blight-synth/issues/108) Implement connection-aware Svelte stores over `EngineClient` — `backlog`, `unsized`, unassigned
- [ ] [#109](https://github.com/jpalvarezl/blight-synth/issues/109) Choose and implement the packaged standalone desktop shell — `backlog`, `unsized`, unassigned
- [ ] [#110](https://github.com/jpalvarezl/blight-synth/issues/110) Build the confirmed transport and position UI — `backlog`, `unsized`, unassigned
- [ ] [#111](https://github.com/jpalvarezl/blight-synth/issues/111) Build the registry-driven instrument and effect parameter UI — `backlog`, `unsized`, unassigned
- [ ] [#112](https://github.com/jpalvarezl/blight-synth/issues/112) Build the stereo peak/RMS meter component — `backlog`, `unsized`, unassigned
- [ ] [#113](https://github.com/jpalvarezl/blight-synth/issues/113) Epic: Explore and build a distinctive composition interface — `backlog`, `epic`, unassigned

### M4 (Optional): Desktop Plugins (VST3/AU)

- [ ] [#114](https://github.com/jpalvarezl/blight-synth/issues/114) Design and implement the versioned C ABI for the shared engine — `backlog`, `unsized`, unassigned
- [ ] [#115](https://github.com/jpalvarezl/blight-synth/issues/115) Create the JUCE VST3/AU wrapper and reproducible build — `backlog`, `unsized`, unassigned
- [ ] [#116](https://github.com/jpalvarezl/blight-synth/issues/116) Implement the JUCE `PluginProcessor` bridge to Rust `Engine` — `backlog`, `unsized`, unassigned
- [ ] [#117](https://github.com/jpalvarezl/blight-synth/issues/117) Embed the production Svelte UI in the JUCE editor — `backlog`, `unsized`, unassigned
- [ ] [#118](https://github.com/jpalvarezl/blight-synth/issues/118) Implement the native webview ↔ APVTS bridge — `backlog`, `unsized`, unassigned
- [ ] [#119](https://github.com/jpalvarezl/blight-synth/issues/119) Persist and migrate complete plugin engine state — `backlog`, `unsized`, unassigned
- [ ] [#140](https://github.com/jpalvarezl/blight-synth/issues/140) Validate the desktop plugin in multiple hosts and instances — `backlog`, `unsized`, unassigned

### M5 (Optional): AUv3 & Distribution

- [ ] [#141](https://github.com/jpalvarezl/blight-synth/issues/141) Define the AUv3 target constraints and architecture delta — `backlog`, `unsized`, unassigned
- [ ] [#142](https://github.com/jpalvarezl/blight-synth/issues/142) Complete plugin signing, packaging, installation, and release checks — `backlog`, `unsized`, unassigned
- [ ] [#143](https://github.com/jpalvarezl/blight-synth/issues/143) Implement and validate the selected AUv3 target(s) — `backlog`, `unsized`, unassigned

## Status rules

See [Work and Parallelization System](README.md). Open issues without a workflow label are backlog. Epics and unsized issues are excluded from point totals until split/estimated.
