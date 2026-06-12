---
tags: [module, audio, metering, rt-nrt]
sources: ["[[concepts/rt-nrt-metering]]"]
last-updated: 2025-05-11
source-file: audio_backend/src/meter.rs
source-sha: 18223cbb32e33dce984d4ab1c957bd8ca6ac29c5
source-mtime: 1781291601215
last-synced: 2026-06-12
---
# meter.rs — MeterState

Lock-free stereo metering shared between the RT audio callback (sole writer) and the NRT OSC server (sole reader). Added for #103. Design: [[concepts/rt-nrt-metering]].

## Types

- `MeterLevels { peak_left, peak_right, rms_left, rms_right }` (linear amplitude); `MeterLevels::SILENT`.
- `MeterState` — four `AtomicU32` (peak L/R as peak-hold bits; mean-square L/R latest-block bits). Cheap to clone behind `Arc`.

## API

- `record_block(left, right)` — RT-safe, no alloc/lock. Peak via `fetch_max` (Relaxed); mean-square via `store`.
- `take_levels()` — peak `swap(0)` (reset window) + load mean-square, returns `MeterLevels` with `rms = sqrt(mean_sq)`.
- `block_stats(samples)` → `(peak, mean_square)`; skips non-finite samples; empty block → `(0,0)`.

Values stored as IEEE-754 bit patterns of non-negative floats so `fetch_max` on bits == max on value.

## Wiring

Constructed in [[entities/blight-audio|BlightAudio]] (`Arc<MeterState>`), cloned into [[entities/audio-processor|AudioProcessor]]; exposed via `BlightAudio::meter_state()`. Read by [[entities/osc-server]].
