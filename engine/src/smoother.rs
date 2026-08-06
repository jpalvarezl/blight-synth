//! Prepared, host-independent scalar smoothing state from ADR 0006.
//!
//! This module owns only one latched scalar trajectory. It deliberately does
//! not own control-quantum phase, parameter bindings, publication draining, DSP
//! setters, or engine process integration.

use param_manifest::{SmoothingCurve, SmoothingPolicy};

/// Why a scalar smoother could not be prepared on NRT.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmootherPrepareError {
    /// The sample rate was zero, negative, or non-finite.
    InvalidSampleRate,
    /// The initial value was non-finite.
    InvalidSeed,
    /// A smoothed duration was negative or non-finite.
    InvalidDuration,
    /// `ceil(duration_ms * sample_rate / 1000)` does not fit in `u32`.
    DurationFrameCountUnrepresentable,
}

/// A non-finite value was rejected without changing prepared smoother state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmootherValueError {
    NonFinite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PreparedCurve {
    Jump,
    Linear,
    Exponential,
}

/// One NRT-prepared, allocation-free scalar smoothing trajectory.
///
/// Positive durations are converted once to an integer frame count
/// `N = max(1, ceil(duration_ms * sample_rate / 1000))`. A latched trajectory is
/// then evaluated from its integer elapsed-frame cursor rather than accumulated
/// increments, making its result independent of how advances are partitioned.
///
/// [`Self::latch_target`], [`Self::advance`], and [`Self::reset`] are bounded
/// O(1) operations and do not allocate. The caller remains responsible for when
/// values are delivered to DSP and for owning any fixed control phase.
#[derive(Debug, Clone, Copy)]
pub struct PreparedSmoother {
    curve: PreparedCurve,
    duration_frames: u32,
    start: f32,
    current: f32,
    target: f32,
    elapsed_frames: u32,
    settled: bool,
}

impl PreparedSmoother {
    /// Validate and prepare a scalar smoother with no startup ramp.
    ///
    /// The sample rate must be finite and positive. The seed is finite but may
    /// have either sign. `SmoothingPolicy::None` and a zero smoothed duration
    /// both prepare immediate-jump behavior.
    pub fn prepare(
        policy: SmoothingPolicy,
        sample_rate: f32,
        seed: f32,
    ) -> Result<Self, SmootherPrepareError> {
        if !sample_rate.is_finite() || sample_rate <= 0.0 {
            return Err(SmootherPrepareError::InvalidSampleRate);
        }
        if !seed.is_finite() {
            return Err(SmootherPrepareError::InvalidSeed);
        }

        let (curve, duration_frames) = match policy {
            SmoothingPolicy::None => (PreparedCurve::Jump, 0),
            SmoothingPolicy::Smoothed { duration_ms, curve } => {
                if !duration_ms.is_finite() || duration_ms < 0.0 {
                    return Err(SmootherPrepareError::InvalidDuration);
                }
                if duration_ms == 0.0 {
                    (PreparedCurve::Jump, 0)
                } else {
                    // f64 avoids overflow and avoidable intermediate rounding
                    // while evaluating the formula over the exact f32 inputs.
                    let frame_count =
                        (f64::from(duration_ms) * f64::from(sample_rate) / 1_000.0).ceil();
                    if frame_count > f64::from(u32::MAX) {
                        return Err(SmootherPrepareError::DurationFrameCountUnrepresentable);
                    }
                    let duration_frames = frame_count.max(1.0) as u32;
                    let curve = match curve {
                        SmoothingCurve::Linear => PreparedCurve::Linear,
                        SmoothingCurve::Exponential => PreparedCurve::Exponential,
                    };
                    (curve, duration_frames)
                }
            }
        };

        Ok(Self {
            curve,
            duration_frames,
            start: seed,
            current: seed,
            target: seed,
            elapsed_frames: 0,
            settled: true,
        })
    }

    #[must_use]
    pub const fn current(&self) -> f32 {
        self.current
    }

    #[must_use]
    pub const fn target(&self) -> f32 {
        self.target
    }

    #[must_use]
    pub const fn is_settled(&self) -> bool {
        self.settled
    }

    /// Evaluate the current latched trajectory at an absolute elapsed cursor.
    ///
    /// This does not mutate the smoother. A cursor at or beyond `N` returns the
    /// exact target. For jump behavior every cursor returns the target.
    #[must_use]
    pub fn value_at(&self, elapsed_frames: u32) -> f32 {
        if self.curve == PreparedCurve::Jump
            || self.start == self.target
            || elapsed_frames >= self.duration_frames
        {
            return self.target;
        }
        if elapsed_frames == 0 {
            return self.start;
        }

        let elapsed = f64::from(elapsed_frames);
        let duration = f64::from(self.duration_frames);
        let start = f64::from(self.start);
        let target = f64::from(self.target);
        let value = match self.curve {
            PreparedCurve::Jump => target,
            PreparedCurve::Linear => start + (target - start) * elapsed / duration,
            PreparedCurve::Exponential => {
                let residual = 10.0_f64.powf(-5.0 * elapsed / duration);
                target + (start - target) * residual
            }
        };
        value as f32
    }

    /// Latch a finite target at the current cursor.
    ///
    /// Republishing the numerically equal target leaves the complete trajectory
    /// untouched. A changed target starts from `current` with elapsed zero and
    /// the full prepared duration. Jump behavior, or a target equal to current,
    /// settles immediately.
    pub fn latch_target(&mut self, target: f32) -> Result<(), SmootherValueError> {
        if !target.is_finite() {
            return Err(SmootherValueError::NonFinite);
        }
        if target == self.target {
            return Ok(());
        }

        self.start = self.current;
        self.target = target;
        self.elapsed_frames = 0;
        if self.curve == PreparedCurve::Jump || target == self.current {
            self.start = target;
            self.current = target;
            self.settled = true;
        } else {
            self.settled = false;
        }
        Ok(())
    }

    /// Advance the current trajectory by an integer number of rendered frames.
    ///
    /// Work is constant regardless of `frames`. The cursor is clamped without
    /// overflow, and settlement snaps `current` to the exact target at `N`.
    pub fn advance(&mut self, frames: u32) -> f32 {
        if self.settled {
            return self.current;
        }

        let remaining = self.duration_frames - self.elapsed_frames;
        self.elapsed_frames += frames.min(remaining);
        self.current = self.value_at(self.elapsed_frames);
        if self.elapsed_frames == self.duration_frames {
            self.current = self.target;
            self.settled = true;
        }
        self.current
    }

    /// Replace current and target with one finite seed and settle immediately.
    ///
    /// The prepared policy and duration remain unchanged. Invalid input leaves
    /// the complete previous state untouched.
    pub fn reset(&mut self, seed: f32) -> Result<(), SmootherValueError> {
        if !seed.is_finite() {
            return Err(SmootherValueError::NonFinite);
        }
        self.start = seed;
        self.current = seed;
        self.target = seed;
        self.elapsed_frames = 0;
        self.settled = true;
        Ok(())
    }
}
