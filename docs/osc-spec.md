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

- **Inbound parameters** (`/param/set`) carry a **normalized `0.0..1.0`** control
  value (the VST/AU convention). The DSP core owns the mapping from normalized
  to engine units; clients stay unit-agnostic. Values are clamped to `0..1`.
- **Outbound levels** (`/meter/level`) are **dBFS** floats. Silence /
  non-finite values floor at **-120.0**.
- `float` = OSC `f`; `int` (`i`) is accepted where noted and coerced to `f32`.
- `string` = OSC `s`.

## Inbound — GUI → DSP (port 9000)

| Address | Args | Effect | Status |
|---------|------|--------|--------|
| `/param/set` | `string id`, `float\|int value` | Set a parameter to a **normalized `0..1`** value. Currently only `id = "gain"` is handled → mapped to dB for the master `Gain` effect (`1.0` = unity/0 dB, `0.0` = mute). Emits `/param/echo` with the accepted normalized value. | ✅ implemented |
| `/transport/play` | — | Play the last loaded song (`TransportCmd::PlayLastSong`). | ✅ implemented |
| `/transport/stop` | — | Stop playback (`TransportCmd::StopSong`). | ✅ implemented |
| `/song/load` | `string path` | Load + hydrate a JSON song from `path`. Emits `/song/loaded` or `/song/error`. | ✅ implemented |

## Outbound — DSP → GUI (port 9001)

| Address | Args | Meaning | Status |
|---------|------|---------|--------|
| `/param/echo` | `string id`, `float value` | Confirms an applied `/param/set`. | ✅ implemented |
| `/song/loaded` | `string path`, `string name` | A `/song/load` succeeded. | ✅ implemented |
| `/song/error` | `string path`, `string error` | A `/song/load` failed. | ✅ implemented |
| `/meter/level` | `float peak_l`, `float peak_r`, `float rms_l`, `float rms_r` | Stereo output levels in dBFS, streamed at **~30 Hz**. Peak is peak-hold over the frame window; RMS is the latest block. A single-bar (mono) display should use `max(peak_l, peak_r)`. | ✅ implemented |

## Open decisions (settle before M2 — #120)

1. ~~**`gain` units.**~~ **Resolved: normalized `0.0..1.0`** (linear amplitude).
   `/param/set gain <0..1>` maps to dB in the core via `dB = 20·log10(value)`,
   `1.0 → 0 dB (unity)`, `0.0 → mute` (floored at -120 dB). The echo returns the
   accepted normalized value. No boost above unity for now; remap the range if
   headroom is needed later. This is the convention for *all* `/param/set` ids.
2. **`/meter/level` shape.** Issue #100 listed a single `float db`. The
   implementation sends **4 floats** (stereo peak + RMS) and a mono display uses
   `max(peak_l, peak_r)`, so the simple scaffold `Meter.svelte` stays
   compatible. Keeping stereo unless a strong reason to trim appears.
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
