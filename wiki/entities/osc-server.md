---
tags: [module, osc]
sources: ["[[concepts/osc-address-space]]"]
last-updated: 2025-05-11
source-file: audio_backend/src/osc.rs
source-sha: e83e74922501e08d3445a548e7e38a002181134f
source-mtime: 1781291718051
last-synced: 2026-06-12
---
# osc.rs — OscServer

UDP OSC server for the standalone DSP core. Translates inbound OSC into existing `Command` variants (transport adapter; see [[concepts/m1-dsp-core-standalone]]) and streams meter levels outbound.

## Surface

- `OscServer { socket: UdpSocket, send_addr: SocketAddr }`.
- `bind()` → binds `OSC_LISTEN_ADDR` (`127.0.0.1:9000`), responses → `OSC_SEND_ADDR` (`127.0.0.1:9001`).
- `bind_to(listen, send)` → resolves `send` to a `SocketAddr` **once** at bind time (avoids per-packet re-parse).
- `run(&mut BlightAudio)` → grabs `audio.meter_state()` then delegates to `run_with_meter` (back-compat).
- `run_with_meter(&mut BlightAudio, &MeterState)` → `tokio::select!` loop over `recv_from` (cancel-safe) and a `tokio::time::interval(METER_INTERVAL)` meter timer (`MissedTickBehavior::Skip`).
- `apply_dispatch` runs song loads, forwards commands to the audio thread, sends responses via `send_packet`.

## Dispatch

`dispatch_packet` → `OscDispatch { commands, song_loads, responses }`. Address handling: see [[concepts/osc-address-space]]. `/param/set` accepts `OscType::Float` and `OscType::Int`. Master gain via `MASTER_GAIN_EFFECT_ID = 0`, `MASTER_GAIN_PARAM_INDEX = 0`.

## Meter streaming (#103)

- `METER_RATE_HZ = 30`; `METER_INTERVAL = 1_000_000/METER_RATE_HZ µs`.
- Each tick: `meter.take_levels()` → `meter_level()` → `/meter/level [peak_l, peak_r, rms_l, rms_r]` in dBFS.
- `amp_to_db(amp)`: `20*log10(amp)`, floors non-finite/≤0 at `METER_FLOOR_DB = -120.0`.
- See [[concepts/rt-nrt-metering]] and [[entities/meter-state]].

## Tests

15 unit tests across `meter`/`osc` modules: param-set translation (incl. int), transport, song-load recording, unknown-address ignore, `amp_to_db` flooring, `/meter/level` shape.
