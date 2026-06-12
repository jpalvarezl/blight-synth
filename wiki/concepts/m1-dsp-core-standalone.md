---
tags: [milestone, roadmap, osc]
sources: ["https://github.com/jpalvarezl/blight-synth/milestone/6"]
last-updated: 2025-05-11
---

# M1: DSP Core Standalone

Milestone 6. Goal: get the Rust DSP core running as a standalone process with an OSC server, independent of any GUI.

## Design stance

OSC is a **transport adapter over the existing internal `Command` queue**, not a separate realtime state path. Incoming OSC messages translate into existing `Command` variants and are sent through the existing audio backend queue. (An earlier `SharedAudioState`/`state.rs` atomic-gain design was proposed and **dropped**.)

## Issues

- #99 — add `rosc` + `tokio` deps to `audio_backend` — done via [[entities/dsp-core-bin|PR #125]]
- #100 — implement OSC server (`osc.rs`) — done, see [[entities/osc-server]]
- #102 — standalone binary entry point — done, see [[entities/dsp-core-bin]]
- #103 — meter/level streaming (DSP → GUI) — in progress, see [[entities/meter-state]]
- #104 — wire one parameter end-to-end over OSC (gain) — done, see [[entities/osc-server]]

Scoped to **M4 (Integration & Protocol)**, not M1:
- #120 — define full OSC address space — see [[concepts/osc-address-space]]
- #122 — adapt song save/load for OSC transport

## Status

PR #125 (#99/#100/#102/#104) squash-merged as `01e3db7 "Add standalone OSC control path (#125)"` into base branch `jpalvarezl/refactor/gui_vst_infra` (NOT `main`). #103 (meter streaming) is the only remaining M1 issue, under development on branch `jpalvarezl/feature/meter-streaming`.
