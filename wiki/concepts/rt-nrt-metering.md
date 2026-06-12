---
tags: [audio, rt-nrt, metering]
sources: ["[[entities/meter-state]]", "[[entities/audio-processor]]"]
last-updated: 2025-05-11
---

# RT/NRT Metering

Level metering crosses the realtime (audio callback) → non-realtime (OSC server) boundary lock-free. Implemented by [[entities/meter-state|MeterState]].

## Roles

- **Writer (RT):** the audio callback ([[entities/audio-processor|AudioProcessor::process]]) calls `MeterState::record_block(left, right)` once per processed block, after master effects are applied (post-master point).
- **Reader (NRT):** the OSC server's meter timer calls `MeterState::take_levels()` at ~30 Hz (`METER_RATE_HZ`) and streams `/meter/level`.

## Lock-free encoding

State is four `AtomicU32`s holding IEEE-754 bit patterns of **non-negative** `f32`s (peak amplitude + mean-square, per channel). For non-negative floats the bit pattern is monotonic with the value, so peak-hold is a plain `fetch_max` on the `AtomicU32` with `Ordering::Relaxed`.

- **Peak:** peak-hold (max since last read). `take_levels` does `swap(0)` to reset the window.
- **RMS:** stored as the latest block's mean-square; reader does `sqrt`. Latest-block (not accumulated) so the reader always sees a recent level rather than a stale running sum.

Non-finite (NaN/Inf) samples never update the peak and are skipped from the sum, keeping stats finite. Empty blocks read as silence.

## dBFS conversion

The OSC server converts linear amplitude → dBFS via `amp_to_db` (`20*log10(amp)`), flooring silence and non-finite values at `METER_FLOOR_DB = -120.0`.
