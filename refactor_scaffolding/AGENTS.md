# AGENTS.md — Project Context & Decision Log

This file documents the architectural decisions made for this project, their
rationale, and the constraints that shaped them. A future agent (or developer)
picking up this project should read this before making changes.

---

## Project Summary

A modular audio DSP application with a clean separation between:
- A **Rust DSP core** (audio processing, state, OSC server)
- A **Bun + Svelte GUI** (TypeScript frontend, OSC client)
- An optional **JUCE C++ plugin wrapper** (VST3/AU shell, embeds a webview)

The owner's goals: start as a personal desktop app, with a credible path to
distributing as a VST3/CLAP/AU plugin without rewriting the core.

---

## Key Decisions & Rationale

### 1. GUI in TypeScript, not Rust

**Decision:** Use TypeScript for the GUI layer.

**Rationale:** The owner explicitly ruled out native Rust GUI frameworks and
Tauri. The Rust GUI ecosystem (egui, iced, etc.) was considered immature and
poorly suited to the kind of polished, real-time UI typical of audio tools.
TypeScript/web GUI libraries (Svelte, React, D3, Canvas, WebGL) are
significantly more mature.

---

### 2. OSC over UDP for IPC, not WebSocket

**Decision:** Use OSC (Open Sound Control) over UDP between the DSP core and
the GUI host process.

**Rationale:** OSC is the established standard in professional audio software.
Hardware controllers, DAWs, Max/MSP, SuperCollider, VCV Rack, and most serious
audio tools speak OSC natively. WebSocket is a web technology with no foothold
in the audio world. Using OSC means the DSP core is a first-class citizen of
the audio ecosystem from day one — it can be driven by external tools without
any additional work.

WebSocket was initially suggested for browser compatibility, but this became
moot once we moved away from a browser-hosted GUI (see decision 3).

Note: a UDP-to-WebSocket relay is NOT needed and should NOT be added. OSC/UDP
is spoken directly between the Rust process and the Bun host process.

---

### 3. Bun as the GUI host process, not a browser

**Decision:** Use Bun (not a browser, not Electron, not Deno) as the host
runtime for the TypeScript GUI layer.

**Rationale:**
- Browsers have no UDP socket API, which would have blocked OSC.
- Electron was ruled out — too heavy, bundles all of Chromium.
- Deno was evaluated but its desktop/webview tooling is less mature than
  Bun's, and its strengths are more relevant to server/edge contexts.
- Node.js was considered (solid, mature, `dgram` for UDP) but carries legacy
  baggage (complex tsconfig setup, separate compilation step, slow installs).
- Bun wins on ergonomics: first-class TypeScript, fast installs, no
  node_modules sprawl, native UDP via `Bun.udpSocket`, npm-compatible so the
  full ecosystem is available.

The owner's view on npm: "npm is kind of bad in my experience" — Bun's package
manager was a meaningful quality-of-life factor in this decision.

Risk: Bun is newer and occasionally has compatibility edge cases with npm
packages that assume Node internals. For this project's dependency surface
(OSC library, Svelte, Vite) this risk is considered low.

---

### 4. Svelte for the UI framework

**Decision:** Use Svelte (not React) for the UI components.

**Rationale:** Svelte is leaner than React, has less boilerplate, and its
reactive store model maps naturally onto real-time parameter updates from OSC.
For an audio GUI with many knobs, meters, and continuously-updating values,
Svelte's fine-grained reactivity is a better fit than React's VDOM diffing.
Vite + Svelte also gives fast hot reload without restarting the DSP core.

---

### 5. JUCE as the plugin format wrapper only

**Decision:** Use JUCE solely as a thin shell for VST3/AU/CLAP format
compliance. The DSP and GUI logic live entirely outside JUCE.

**Rationale:**
- The owner's primary concern about nih-plug was maturity (single maintainer,
  breaking API changes) and licensing (VST3 SDK requires either GPL or a paid
  Steinberg commercial license).
- JUCE itself is dual-licensed: free under AGPL (requires open-source release),
  or paid commercial license (Indie: ~$40/month or $800 perpetual for <$500k
  revenue). This was noted as a cost consideration.
- JUCE's own GUI system was evaluated and rejected: no CSS/flexbox, manual
  coordinate math, dated default look-and-feel, full recompile per tweak. Many
  serious plugin shops end up embedding a webview inside JUCE anyway — which
  is exactly what this architecture does.
- By keeping JUCE as a format adapter only, the DSP core and GUI are developed
  and tested independently of any plugin format. The JUCE wrapper is only
  needed at distribution time.

---

### 6. Rust DSP core as a standalone process (standalone mode) or static lib (plugin mode)

**Decision:** In standalone mode the Rust core runs as a child process spawned
by Bun. In plugin mode it is intended to be compiled as a static library and
called via FFI from the JUCE PluginProcessor.

**Rationale:** VST/CLAP plugins are loaded as shared libraries into the DAW's
process. There is no separate process, no spawning, and some DAWs sandbox
plugins and block local network sockets entirely. This means the standalone
"two processes talking over OSC" model cannot be used directly in a plugin.

The FFI path (Rust static lib → C ABI → JUCE C++ caller) is the standard
approach used by studios that want Rust DSP inside a C++ plugin host. The OSC
layer is still used between the JUCE plugin and the Bun GUI process; the JUCE
OscBridge class handles keeping DAW automation state in sync.

**TODO:** Add a `[lib]` target to `dsp-core/Cargo.toml` that exposes a C ABI
(`#[no_mangle] pub extern "C" fn ...`) for the plugin build path.

---

### 7. Port assignments

| Port | Owner    | Protocol | Purpose                        |
|------|----------|----------|--------------------------------|
| 9000 | dsp-core | OSC/UDP  | Receives messages from GUI/JUCE |
| 9001 | Bun      | OSC/UDP  | Receives messages from dsp-core |
| 5173 | Bun/Vite | HTTP     | Serves Svelte UI (dev mode)    |

---

## What Was Explicitly Ruled Out

| Option              | Reason rejected                                              |
|---------------------|--------------------------------------------------------------|
| Tauri               | Owner ruled out explicitly                                   |
| Native Rust GUIs    | Owner ruled out explicitly (egui, iced, etc.)               |
| Electron            | Too heavy, bundles full Chromium                             |
| WebSocket for IPC   | Not an audio-world standard; OSC is the right choice here   |
| OSC relay/bridge    | Never needed — Bun has native UDP, no browser involved      |
| JUCE for GUI        | Dated, verbose, manual layout; most shops embed webview anyway |
| nih-plug            | Single maintainer, unstable API, VST3 licensing issues      |
| Deno                | Weakest desktop/webview story of the JS runtimes evaluated  |
| Node.js             | Legacy tooling complexity; Bun is strictly better here      |

---

## Open TODOs at Handoff

These are the most important unimplemented stubs in the scaffolding:

- `dsp-core/src/engine.rs` — cpal stream setup and audio callback
- `dsp-core/src/osc.rs` — UDP socket bind, OSC packet decode/dispatch loop
- `dsp-core/src/state.rs` — replace placeholder structs with actual atomics
- `dsp-core/src/audio.rs` — implement DSP math utilities and ParamSmoother
- `gui/src/lib/OscBridge.ts` — Bun UDP socket, OSC encode/decode
- `gui/src/lib/DspProcess.ts` — Bun.spawn, stdout readiness detection
- `gui/src/lib/stores.ts` — wire setParam to OscBridge instance
- `gui/src/components/Meter.svelte` — dB to percent mapping, peak hold
- `plugin/Source/WebviewManager.cpp` — proper stdout-based readiness detection
  (currently uses a naive fixed sleep)
- `plugin/Source/OscBridge.cpp` — UDP sockets, background receive thread
- `plugin/Source/PluginProcessor.cpp` — FFI calls into Rust static lib
- `plugin/CMakeLists.txt` — set JUCE_PATH, link dsp-core static lib

---

## Suggested Next Steps

1. Get the standalone app working end-to-end first. Don't touch JUCE until the
   Rust + Bun + OSC loop is proven.
2. Implement `OscBridge.ts` in Bun — this is the critical path.
3. Implement the OSC server in `dsp-core/src/osc.rs`.
4. Wire a single parameter (gain) end-to-end before adding more.
5. Add the `[lib]` crate target to Cargo.toml when ready for the plugin path.
6. Only bring in JUCE when the standalone app is stable.
