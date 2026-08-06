---
title: Generated Roadmap Burndown
summary: Offline Obsidian snapshot generated from GitHub roadmap issue metadata.
status: generated
source-updated: 2026-08-06T16:44:21Z
generator: scripts/docs/sync_roadmap.py
---

# Generated Roadmap Burndown

> [!warning] Generated file
> GitHub Issues are canonical. Do not edit this page manually. Run `python3 scripts/docs/sync_roadmap.py`.

Data snapshot through `2026-08-06T16:44:21Z`.

## Summary

| Milestone | Open | Done | Ready | In progress | Blocked | Backlog | Sized points done/total | Unsized |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| M0: Architecture & Repository Baseline | 0 | 14 | 0 | 0 | 0 | 0 | 27/27 | 3 |
| M1: Host-Independent Engine & RT Contracts | 13 | 24 | 1 | 1 | 8 | 3 | 59/75 | 12 |
| M2: Standalone Host & Control Protocol | 6 | 2 | 0 | 0 | 1 | 5 | 4/7 | 5 |
| M3: JS/TS Composition UI & Standalone App | 9 | 0 | 0 | 0 | 0 | 9 | 0/0 | 9 |
| M4 (Optional): Desktop Plugins (VST3/AU) | 7 | 0 | 0 | 0 | 0 | 7 | 0/0 | 7 |
| M5 (Optional): AUv3 & Distribution | 3 | 0 | 0 | 0 | 0 | 3 | 0/0 | 3 |

## Current milestone

### M1: Host-Independent Engine & RT Contracts

- [ ] [#101](https://github.com/jpalvarezl/blight-synth/issues/101) Implement the coalesced real-time parameter pipeline — `blocked`, `epic`, unassigned
- [ ] [#132](https://github.com/jpalvarezl/blight-synth/issues/132) Define the host-independent `Engine` lifecycle and offline render harness — `blocked`, `epic`, unassigned
- [ ] [#135](https://github.com/jpalvarezl/blight-synth/issues/135) Introduce typed instance IDs and versioned instrument/effect definitions — `blocked`, `epic`, unassigned
- [ ] [#136](https://github.com/jpalvarezl/blight-synth/issues/136) Define and implement the scalable audio routing graph — `backlog`, `unsized`, unassigned
- [ ] [#138](https://github.com/jpalvarezl/blight-synth/issues/138) Define versioned engine state snapshots and migrations — `backlog`, `unsized`, unassigned
- [ ] [#179](https://github.com/jpalvarezl/blight-synth/issues/179) Expand RT allocation audit across existing instrument and effect paths — `backlog`, `M`, unassigned
- [ ] [#211](https://github.com/jpalvarezl/blight-synth/issues/211) Migrate tracker hydration to versioned node definitions — `blocked`, `epic`, unassigned
- [ ] [#214](https://github.com/jpalvarezl/blight-synth/issues/214) Apply coalesced parameters with engine-owned smoothing — `blocked`, `epic`, unassigned
- [ ] [#215](https://github.com/jpalvarezl/blight-synth/issues/215) Integrate coalesced parameter generations with the device host — `blocked`, `M`, unassigned
- [ ] [#216](https://github.com/jpalvarezl/blight-synth/issues/216) Migrate OSC parameters to applied-confirmed coalescing — `blocked`, `S`, unassigned
- [ ] [#221](https://github.com/jpalvarezl/blight-synth/issues/221) Adapt legacy tracker models to versioned node definitions — `in-progress`, `M`, @jpalvarezl
- [ ] [#222](https://github.com/jpalvarezl/blight-synth/issues/222) Switch tracker hydration to the built-in node registry — `blocked`, `M`, unassigned
- [ ] [#224](https://github.com/jpalvarezl/blight-synth/issues/224) Implement the deterministic engine smoother primitive — `ready`, `M`, unassigned

## All roadmap tasks

### M0: Architecture & Repository Baseline

- [x] [#128](https://github.com/jpalvarezl/blight-synth/issues/128) Integrate the modular-refactor branch into `main` and remove duplicate scaffolding — `done`, `unsized`, @jpalvarezl
- [x] [#129](https://github.com/jpalvarezl/blight-synth/issues/129) Record the primary product, open composition model, and optional plugin topology — `done`, `S`, @jpalvarezl
- [x] [#130](https://github.com/jpalvarezl/blight-synth/issues/130) Refactor workspace boundaries around a host-independent engine — `done`, `epic`, @jpalvarezl
- [x] [#131](https://github.com/jpalvarezl/blight-synth/issues/131) Establish CI and a repository quality baseline — `done`, `M`, @jpalvarezl
- [x] [#146](https://github.com/jpalvarezl/blight-synth/issues/146) Establish the human/LLM knowledge base, context packets, and generated burndown — `done`, `M`, @jpalvarezl
- [x] [#149](https://github.com/jpalvarezl/blight-synth/issues/149) Make `utils` and `os_dls` strict-Clippy clean — `done`, `S`, @jpalvarezl
- [x] [#150](https://github.com/jpalvarezl/blight-synth/issues/150) Make DSP and remaining workspace targets strict-Clippy clean — `done`, `M`, @jpalvarezl
- [x] [#154](https://github.com/jpalvarezl/blight-synth/issues/154) Move sample and platform resource loading out of `dsp` — `done`, `M`, @jpalvarezl
- [x] [#155](https://github.com/jpalvarezl/blight-synth/issues/155) Extract the host-independent engine crate mechanically — `done`, `epic`, @jpalvarezl
- [x] [#156](https://github.com/jpalvarezl/blight-synth/issues/156) Isolate CPAL/OSC and simplify the standalone host threading model — `done`, `M`, @jpalvarezl
- [x] [#157](https://github.com/jpalvarezl/blight-synth/issues/157) Finalize and enforce the M0 crate dependency graph — `done`, `S`, @jpalvarezl
- [x] [#162](https://github.com/jpalvarezl/blight-synth/issues/162) Extract the host-independent engine render core — `done`, `M`, @jpalvarezl
- [x] [#163](https://github.com/jpalvarezl/blight-synth/issues/163) Separate engine command ownership from standalone transport commands — `done`, `M`, @jpalvarezl
- [x] [#164](https://github.com/jpalvarezl/blight-synth/issues/164) Add deterministic golden offline renders and finalize #155 — `done`, `M`, @jpalvarezl

### M1: Host-Independent Engine & RT Contracts

- [ ] [#101](https://github.com/jpalvarezl/blight-synth/issues/101) Implement the coalesced real-time parameter pipeline — `blocked`, `epic`, unassigned
- [x] [#121](https://github.com/jpalvarezl/blight-synth/issues/121) Design the canonical parameter manifest and host bindings — `done`, `unsized`, @jpalvarezl
- [ ] [#132](https://github.com/jpalvarezl/blight-synth/issues/132) Define the host-independent `Engine` lifecycle and offline render harness — `blocked`, `epic`, unassigned
- [x] [#133](https://github.com/jpalvarezl/blight-synth/issues/133) Enforce and test the real-time safety contract — `done`, `epic`, @jpalvarezl
- [x] [#134](https://github.com/jpalvarezl/blight-synth/issues/134) Implement sample-accurate event scheduling across block sizes — `done`, `epic`, unassigned
- [ ] [#135](https://github.com/jpalvarezl/blight-synth/issues/135) Introduce typed instance IDs and versioned instrument/effect definitions — `blocked`, `epic`, unassigned
- [ ] [#136](https://github.com/jpalvarezl/blight-synth/issues/136) Define and implement the scalable audio routing graph — `backlog`, `unsized`, unassigned
- [x] [#137](https://github.com/jpalvarezl/blight-synth/issues/137) Complete polyphony, note identity, and voice-allocation semantics — `done`, `unsized`, @jpalvarezl
- [ ] [#138](https://github.com/jpalvarezl/blight-synth/issues/138) Define versioned engine state snapshots and migrations — `backlog`, `unsized`, unassigned
- [x] [#145](https://github.com/jpalvarezl/blight-synth/issues/145) Decouple composition runtimes from the audio engine through an event-source contract — `done`, `unsized`, @jpalvarezl
- [x] [#171](https://github.com/jpalvarezl/blight-synth/issues/171) Specify the real-time contract and inventory current violations — `done`, `S`, @jpalvarezl
- [x] [#172](https://github.com/jpalvarezl/blight-synth/issues/172) Add an allocation/deallocation audit harness for engine processing — `done`, `M`, @jpalvarezl
- [x] [#173](https://github.com/jpalvarezl/blight-synth/issues/173) Bound callback control work and make queue backpressure observable — `done`, `M`, unassigned
- [x] [#174](https://github.com/jpalvarezl/blight-synth/issues/174) Defer reclamation for structural engine and song updates — `done`, `L`, @jpalvarezl
- [x] [#175](https://github.com/jpalvarezl/blight-synth/issues/175) Remove callback logging and panic paths, then stress the RT hot path — `done`, `S`, @jpalvarezl
- [ ] [#179](https://github.com/jpalvarezl/blight-synth/issues/179) Expand RT allocation audit across existing instrument and effect paths — `backlog`, `M`, unassigned
- [x] [#181](https://github.com/jpalvarezl/blight-synth/issues/181) Move tracker audio control off the egui UI thread — `done`, `M`, unassigned
- [x] [#182](https://github.com/jpalvarezl/blight-synth/issues/182) Move standalone DSP command submission off the current-thread Tokio executor — `done`, `M`, unassigned
- [x] [#186](https://github.com/jpalvarezl/blight-synth/issues/186) Add the RT-to-NRT retirement primitive and instrument replacement slice — `done`, `M`, unassigned
- [x] [#187](https://github.com/jpalvarezl/blight-synth/issues/187) Route engine and effect structural rejections through deferred retirement — `done`, `M`, unassigned
- [x] [#188](https://github.com/jpalvarezl/blight-synth/issues/188) Retire replaced songs and finalize reclamation shutdown stress coverage — `done`, `M`, @jpalvarezl
- [x] [#201](https://github.com/jpalvarezl/blight-synth/issues/201) Define and apply bounded timestamped engine events — `done`, `L`, @jpalvarezl
- [x] [#202](https://github.com/jpalvarezl/blight-synth/issues/202) Emit sample-accurate tracker tick offsets across block partitions — `done`, `M`, @jpalvarezl
- [x] [#203](https://github.com/jpalvarezl/blight-synth/issues/203) Implement bounded current-block event admission and recovery — `done`, `M`, @jpalvarezl
- [x] [#204](https://github.com/jpalvarezl/blight-synth/issues/204) Adapt tracker and live playback to the timestamped event path — `done`, `L`, @jpalvarezl
- [x] [#209](https://github.com/jpalvarezl/blight-synth/issues/209) Introduce typed DSP and engine instance IDs — `done`, `L`, @jpalvarezl
- [x] [#210](https://github.com/jpalvarezl/blight-synth/issues/210) Define versioned node definitions and the built-in registry — `done`, `L`, @jpalvarezl
- [ ] [#211](https://github.com/jpalvarezl/blight-synth/issues/211) Migrate tracker hydration to versioned node definitions — `blocked`, `epic`, unassigned
- [x] [#212](https://github.com/jpalvarezl/blight-synth/issues/212) Decide the coalesced parameter ownership and lifecycle contract — `done`, `S`, @jpalvarezl
- [x] [#213](https://github.com/jpalvarezl/blight-synth/issues/213) Implement the generation-bound coalesced parameter store — `done`, `M`, @jpalvarezl
- [ ] [#214](https://github.com/jpalvarezl/blight-synth/issues/214) Apply coalesced parameters with engine-owned smoothing — `blocked`, `epic`, unassigned
- [ ] [#215](https://github.com/jpalvarezl/blight-synth/issues/215) Integrate coalesced parameter generations with the device host — `blocked`, `M`, unassigned
- [ ] [#216](https://github.com/jpalvarezl/blight-synth/issues/216) Migrate OSC parameters to applied-confirmed coalescing — `blocked`, `S`, unassigned
- [ ] [#221](https://github.com/jpalvarezl/blight-synth/issues/221) Adapt legacy tracker models to versioned node definitions — `in-progress`, `M`, @jpalvarezl
- [ ] [#222](https://github.com/jpalvarezl/blight-synth/issues/222) Switch tracker hydration to the built-in node registry — `blocked`, `M`, unassigned
- [x] [#223](https://github.com/jpalvarezl/blight-synth/issues/223) Decide coalesced smoothing delivery to block-oriented DSP — `done`, `S`, @jpalvarezl
- [ ] [#224](https://github.com/jpalvarezl/blight-synth/issues/224) Implement the deterministic engine smoother primitive — `ready`, `M`, unassigned

### M2: Standalone Host & Control Protocol

- [ ] [#104](https://github.com/jpalvarezl/blight-synth/issues/104) Prove one confirmed parameter round-trip through the standalone host — `backlog`, `unsized`, unassigned
- [ ] [#120](https://github.com/jpalvarezl/blight-synth/issues/120) Define the versioned public control protocol and OSC mapping — `backlog`, `unsized`, unassigned
- [ ] [#122](https://github.com/jpalvarezl/blight-synth/issues/122) Implement portable project I/O and standalone load/save operations — `backlog`, `unsized`, unassigned
- [ ] [#123](https://github.com/jpalvarezl/blight-synth/issues/123) Implement standalone process lifecycle, discovery, and recovery — `backlog`, `unsized`, unassigned
- [ ] [#139](https://github.com/jpalvarezl/blight-synth/issues/139) Migrate the standalone CPAL/OSC host onto the shared engine — `backlog`, `unsized`, unassigned
- [ ] [#161](https://github.com/jpalvarezl/blight-synth/issues/161) Remove Tokio from the standalone host control loop — `blocked`, `M`, unassigned
- [x] [#185](https://github.com/jpalvarezl/blight-synth/issues/185) Separate the shared device host from OSC standalone transport adapters — `done`, `S`, @jpalvarezl
- [x] [#190](https://github.com/jpalvarezl/blight-synth/issues/190) Split audio_backend device-host infrastructure from the OSC standalone process adapter — `done`, `M`, @jpalvarezl

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
