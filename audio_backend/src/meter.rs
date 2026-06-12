//! Lock-free stereo metering shared between the realtime audio callback and
//! the (non-realtime) OSC server.
//!
//! The audio callback is the sole writer; it calls [`MeterState::record_block`]
//! once per processed block with the final post-master stereo signal. The OSC
//! server is the sole reader; on its meter timer it calls
//! [`MeterState::take_levels`] and streams the result as `/meter/level`.
//!
//! Values are stored as the IEEE-754 bit patterns of *non-negative* `f32`s
//! (peak amplitude and mean-square). For non-negative floats the bit pattern
//! is monotonic with the value, so peak-hold is implemented with a plain
//! `fetch_max` on the underlying `AtomicU32`.

use std::sync::atomic::{AtomicU32, Ordering};

/// Linear peak + RMS levels for one read window, per channel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MeterLevels {
    /// Peak (max absolute sample) on the left channel, linear amplitude.
    pub peak_left: f32,
    /// Peak (max absolute sample) on the right channel, linear amplitude.
    pub peak_right: f32,
    /// RMS on the left channel for the most recent block, linear amplitude.
    pub rms_left: f32,
    /// RMS on the right channel for the most recent block, linear amplitude.
    pub rms_right: f32,
}

impl MeterLevels {
    /// Silence (all channels at zero).
    pub const SILENT: MeterLevels = MeterLevels {
        peak_left: 0.0,
        peak_right: 0.0,
        rms_left: 0.0,
        rms_right: 0.0,
    };
}

/// Realtime-safe shared metering state.
///
/// Cheap to clone behind an `Arc`; both the audio callback and the OSC server
/// hold their own `Arc<MeterState>`.
#[derive(Debug)]
pub struct MeterState {
    /// Peak-hold (linear amplitude bits) since the last [`take_levels`](Self::take_levels).
    peak_left: AtomicU32,
    peak_right: AtomicU32,
    /// Mean-square (linear) of the most recently recorded block.
    mean_sq_left: AtomicU32,
    mean_sq_right: AtomicU32,
}

impl Default for MeterState {
    fn default() -> Self {
        Self::new()
    }
}

impl MeterState {
    pub fn new() -> Self {
        Self {
            peak_left: AtomicU32::new(0),
            peak_right: AtomicU32::new(0),
            mean_sq_left: AtomicU32::new(0),
            mean_sq_right: AtomicU32::new(0),
        }
    }

    /// Record one processed stereo block. Realtime-safe: no allocation, no
    /// locking. Called from the audio callback.
    ///
    /// Peak is accumulated as a peak-hold (max since the last read); RMS is
    /// stored as the latest block's mean-square, so the reader always observes
    /// a recent level rather than a stale accumulation.
    pub fn record_block(&self, left: &[f32], right: &[f32]) {
        let (peak_l, mean_sq_l) = block_stats(left);
        let (peak_r, mean_sq_r) = block_stats(right);

        self.peak_left
            .fetch_max(peak_l.to_bits(), Ordering::Relaxed);
        self.peak_right
            .fetch_max(peak_r.to_bits(), Ordering::Relaxed);
        self.mean_sq_left
            .store(mean_sq_l.to_bits(), Ordering::Relaxed);
        self.mean_sq_right
            .store(mean_sq_r.to_bits(), Ordering::Relaxed);
    }

    /// Read the current peak-hold + latest RMS and reset the peak-hold for the
    /// next window. Called from the OSC server's meter timer.
    pub fn take_levels(&self) -> MeterLevels {
        let peak_left = f32::from_bits(self.peak_left.swap(0, Ordering::Relaxed));
        let peak_right = f32::from_bits(self.peak_right.swap(0, Ordering::Relaxed));
        let mean_sq_left = f32::from_bits(self.mean_sq_left.load(Ordering::Relaxed));
        let mean_sq_right = f32::from_bits(self.mean_sq_right.load(Ordering::Relaxed));

        MeterLevels {
            peak_left,
            peak_right,
            rms_left: mean_sq_left.sqrt(),
            rms_right: mean_sq_right.sqrt(),
        }
    }
}

/// Returns `(peak, mean_square)` for a block. Both are non-negative; a
/// non-finite (NaN) sample never updates the peak and is ignored so the
/// running statistics stay finite.
fn block_stats(samples: &[f32]) -> (f32, f32) {
    if samples.is_empty() {
        return (0.0, 0.0);
    }

    let mut peak = 0.0_f32;
    let mut sum_sq = 0.0_f32;
    for &sample in samples {
        if !sample.is_finite() {
            continue;
        }
        let abs = sample.abs();
        if abs > peak {
            peak = abs;
        }
        sum_sq += sample * sample;
    }

    (peak, sum_sq / samples.len() as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f32, b: f32) {
        assert!((a - b).abs() < 1e-6, "expected {a} ~= {b}");
    }

    #[test]
    fn silence_reads_as_zero() {
        let meter = MeterState::new();
        meter.record_block(&[0.0; 64], &[0.0; 64]);
        assert_eq!(meter.take_levels(), MeterLevels::SILENT);
    }

    #[test]
    fn peak_and_rms_are_computed_per_channel() {
        let meter = MeterState::new();
        // Left: constant 0.5 -> peak 0.5, rms 0.5.
        // Right: +/-1.0 square -> peak 1.0, rms 1.0.
        meter.record_block(&[0.5, 0.5, 0.5, 0.5], &[1.0, -1.0, 1.0, -1.0]);

        let levels = meter.take_levels();
        approx(levels.peak_left, 0.5);
        approx(levels.rms_left, 0.5);
        approx(levels.peak_right, 1.0);
        approx(levels.rms_right, 1.0);
    }

    #[test]
    fn peak_holds_across_blocks_until_read() {
        let meter = MeterState::new();
        meter.record_block(&[0.2, -0.2], &[0.0, 0.0]);
        meter.record_block(&[0.9, -0.1], &[0.0, 0.0]);
        meter.record_block(&[0.3, -0.3], &[0.0, 0.0]);

        // Peak-hold keeps the loudest sample seen since the last read.
        approx(meter.take_levels().peak_left, 0.9);
        // ...and resets afterwards.
        meter.record_block(&[0.1, -0.1], &[0.0, 0.0]);
        approx(meter.take_levels().peak_left, 0.1);
    }

    #[test]
    fn non_finite_samples_are_ignored() {
        let meter = MeterState::new();
        meter.record_block(&[f32::NAN, 0.5, f32::INFINITY], &[0.0; 3]);

        let levels = meter.take_levels();
        approx(levels.peak_left, 0.5);
        assert!(levels.rms_left.is_finite());
    }

    #[test]
    fn empty_block_is_safe() {
        let meter = MeterState::new();
        meter.record_block(&[], &[]);
        assert_eq!(meter.take_levels(), MeterLevels::SILENT);
    }
}
