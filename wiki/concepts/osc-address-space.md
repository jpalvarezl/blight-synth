---
tags: [osc, protocol]
sources: ["[[entities/osc-server]]", "[[entities/meter-state]]"]
last-updated: 2025-05-11
---

# OSC Address Space

Current OSC surface of the standalone DSP core. Full formal definition is deferred to #120 (M4). See [[entities/osc-server]].

## Sockets

- Listen (DSP receives): `127.0.0.1:9000` (`OSC_LISTEN_ADDR`).
- Send (GUI/host receives): `127.0.0.1:9001` (`OSC_SEND_ADDR`).

## Inbound (→ DSP)

| Address | Args | Effect |
|---|---|---|
| `/song/load` | `[string path]` | load JSON song, hydrate instruments/effects |
| `/param/set` | `[string "gain", float\|int db]` | `MixerCmd::SetMasterEffectParameter` on the reserved master gain effect |
| `/transport/play` | `[]` | `TransportCmd::Play` |
| `/transport/stop` | `[]` | `TransportCmd::Stop` |

Unknown addresses are ignored (logged). `/param/set` accepts both `OscType::Float` and `OscType::Int` (int converted to `f32`).

## Outbound (DSP →)

| Address | Args | When |
|---|---|---|
| `/song/loaded` | `[string path, string name]` | song load success |
| `/song/error` | `[string path, string error]` | song load failure |
| `/param/echo` | `[string param_id, float value]` | param confirmation |
| `/meter/level` | `[float peak_l, float peak_r, float rms_l, float rms_r]` (dBFS) | streamed at ~30 Hz while running, see [[entities/meter-state]] |

The reserved master gain effect uses `EffectId = 0` (`MASTER_GAIN_EFFECT_ID`), param index `0` (`MASTER_GAIN_PARAM_INDEX`); the standalone binary installs it at startup.
