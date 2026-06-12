# OSC Address Space — `blight-synth`

Single source of truth for the OSC protocol between the standalone DSP core
(`audio_backend` `dsp-core` binary) and any GUI host (M2 Svelte GUI, M3 JUCE
plugin). Both the Rust and TypeScript sides should reference this file.

> Status: **living document**. Owned by #120 ("Define full OSC address space").
> This first revision documents what is *implemented today* (M1) plus the open
> decisions that must be settled before M2 codes against it.

## Transport

| Role | Address | Notes |
|------|---------|-------|
| DSP core listens (inbound, GUI → DSP) | `127.0.0.1:9000` | UDP |
| GUI listens (outbound, DSP → GUI) | `127.0.0.1:9001` | UDP |

- Encoding: standard OSC 1.0 packets (`rosc` on Rust, `osc` npm on TS).
- Localhost-only for now; multi-process lifecycle is #123.
- Unknown addresses and malformed packets are logged and dropped, never fatal.

## Conventions

- Levels are **dBFS** floats. Silence / non-finite values floor at **-120.0**.
- `float` = OSC `f`; `int` (`i`) is accepted where noted and coerced to `f32`.
- `string` = OSC `s`.

## Inbound — GUI → DSP (port 9000)

| Address | Args | Effect | Status |
|---------|------|--------|--------|
| `/param/set` | `string id`, `float\|int value` | Set a parameter. Currently only `id = "gain"` is handled → master `Gain` effect. Emits `/param/echo`. | ✅ implemented |
| `/transport/play` | — | Play the last loaded song (`TransportCmd::PlayLastSong`). | ✅ implemented |
| `/transport/stop` | — | Stop playback (`TransportCmd::StopSong`). | ✅ implemented |
| `/song/load` | `string path` | Load + hydrate a JSON song from `path`. Emits `/song/loaded` or `/song/error`. | ✅ implemented |

## Outbound — DSP → GUI (port 9001)

| Address | Args | Meaning | Status |
|---------|------|---------|--------|
| `/param/echo` | `string id`, `float value` | Confirms an applied `/param/set`. | ✅ implemented |
| `/song/loaded` | `string path`, `string name` | A `/song/load` succeeded. | ✅ implemented |
| `/song/error` | `string path`, `string error` | A `/song/load` failed. | ✅ implemented |
| `/meter/level` | `float peak_l`, `float peak_r`, `float rms_l`, `float rms_r` | Stereo output levels in dBFS, streamed at **~30 Hz**. Peak is peak-hold over the frame window; RMS is the latest block. | ✅ implemented |

## Open decisions (settle before M2 — #120)

1. **`gain` units.** Currently `/param/set gain <v>` interprets `v` as **dBFS**
   (existing `Gain::set_parameter` semantics). Issue #104 wrote
   `/param/set gain 0.5`, implying a normalized `0.0..1.0`. Pick one and make
   the GUI (#111 ParameterPanel) agree. *Current behavior: dBFS.*
2. **`/meter/level` shape.** Issue #100 listed a single `float db`. The
   implementation sends **4 floats** (stereo peak + RMS). The simple scaffold
   `Meter.svelte` reads `args[0]` (= `peak_l`), so the 4-float form is
   backward-compatible. Decision: keep stereo (documented here) or trim to mono.
3. **`/preset/load` vs `/song/load`.** The scaffolding used `/preset/load`; the
   project uses `/song/load` against the existing `Song` model. Save/load
   protocol is #122.
4. **Parameter transport: Commands vs atomics (#101).** `/param/set` currently
   routes through the bounded (1024) `Command` ring buffer, which *drops* on
   overflow. Fine for low-rate control; for high-rate continuous params
   (knob drags / automation in #111) a coalescing atomic ("latest value wins",
   `AtomicU32` + `f32::to_bits`) is preferable — that pattern already exists in
   `MeterState` (DSP → GUI). Revisit when #111 adds live knob control.

## Not yet implemented

Instrument bank ops, per-instrument/effect params, envelope (ADSR/pitch) params,
broader mixer commands, and arrangement editing — mapped from the `Command` enum
as the GUI needs them (#120).
