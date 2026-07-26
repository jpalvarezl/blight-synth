//! Normalized `0..1` <-> engine-value mapping.
//!
//! The mapping is the single owner of unit conversion between the host-facing
//! normalized control value (always `0..1`, the VST/AU/APVTS convention) and the
//! engine-facing value a DSP node consumes (dB, Hz, seconds, a linear factor,
//! etc.). Every host adapter — OSC, APVTS, Svelte — converts through this type so
//! the conversion is defined once. The mapping is `Copy` and string-free, so it
//! lives in both the descriptor and the real-time [`RuntimeParameter`].

use serde::{Deserialize, Serialize};

/// Smallest supported power-curve exponent.
///
/// Smaller exponents collapse too much of a `f32` engine range onto its upper
/// endpoint and make the inverse numerically unusable. The reciprocal is also
/// deliberately bounded for predictable real-time arithmetic.
pub const MIN_SKEW: f32 = 1.0 / 64.0;

/// Largest supported power-curve exponent.
///
/// This symmetric bound prevents useful normalized positions from collapsing to
/// the lower endpoint while still allowing strongly shaped controls.
pub const MAX_SKEW: f32 = 64.0;

const FALLBACK_DB_FLOOR: f32 = -120.0;

/// How a normalized `0..1` control value maps to the engine value and back.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "curve", rename_all = "snake_case")]
pub enum Mapping {
    /// `engine = min + t * (max - min)`; `t = (engine - min) / (max - min)`.
    Linear { min: f32, max: f32 },

    /// Perceptual/exponential mapping between two positive endpoints:
    /// `engine = exp(lerp(ln(min), ln(max), t))`. Useful for frequency and time
    /// controls. Manifest validation requires `0 < min < max`.
    Exponential { min: f32, max: f32 },

    /// Power (skew) mapping over the normalized value with linear endpoints and
    /// a tunable steepness independent of the `min`/`max` ratio.
    ///
    /// `to_engine`: `engine = lerp(min, max, t.powf(skew))`; inverse:
    /// `t = inverse_lerp(engine).powf(1.0 / skew)`.
    ///
    /// `skew == 1.0` is linear, `skew < 1.0` biases toward `max`, and
    /// `skew > 1.0` biases toward `min`. Valid manifests constrain `skew` to
    /// [`MIN_SKEW`]..=[`MAX_SKEW`].
    Skewed { min: f32, max: f32, skew: f32 },

    /// The normalized value is a linear amplitude (`0..1`) and the engine value
    /// is decibels: `engine = max(20 * log10(t), floor_db)`. The implied engine
    /// range is `[floor_db, 0 dB]` and validation requires `floor_db < 0`.
    AmplitudeDecibel { floor_db: f32 },
}

impl Mapping {
    /// Convert a normalized `0..1` value to the engine value.
    ///
    /// NaN has the deterministic malformed-input fallback `0.0` (the normalized
    /// range floor); infinities clamp to the nearest endpoint. Validated mappings
    /// always return a finite value. Invalid directly-constructed mappings are
    /// sanitized to finite ordered bounds and never panic.
    #[must_use]
    pub fn to_engine(self, normalized: f32) -> f32 {
        let t = normalized_or_floor(normalized);
        let (lo, hi) = self.sanitized_engine_bounds();

        if t == 0.0 {
            return lo;
        }
        if t == 1.0 {
            return hi;
        }

        let value = match self {
            Mapping::Linear { .. } => lerp_f64(lo, hi, f64::from(t)),
            Mapping::Exponential { min, max }
                if min.is_finite() && max.is_finite() && min > 0.0 && min < max =>
            {
                let t = f64::from(t);
                let log_value = f64::from(min).ln() * (1.0 - t) + f64::from(max).ln() * t;
                log_value.exp()
            }
            Mapping::Exponential { .. } => lerp_f64(lo, hi, f64::from(t)),
            Mapping::Skewed { min, max, skew }
                if min.is_finite()
                    && max.is_finite()
                    && min < max
                    && skew.is_finite()
                    && skew > 0.0 =>
            {
                let shaped = f64::from(t).powf(f64::from(skew));
                lerp_f64(lo, hi, shaped)
            }
            Mapping::Skewed { .. } => lerp_f64(lo, hi, f64::from(t)),
            Mapping::AmplitudeDecibel { .. } => (20.0 * f64::from(t).log10()).max(f64::from(lo)),
        };

        finite_engine_value(value, lo, lo, hi)
    }

    /// Convert an engine value back to a normalized `0..1` value.
    ///
    /// NaN has the deterministic malformed-input fallback `0.0` (the engine
    /// range floor); infinities clamp to the nearest endpoint. The inverse is
    /// defined over the representable, non-floored part of a mapping. In
    /// particular, all amplitudes at or below an amplitude-dB floor map back to
    /// `0.0` by policy.
    #[must_use]
    pub fn to_normalized(self, engine: f32) -> f32 {
        let (lo, hi) = self.sanitized_engine_bounds();
        let engine = if engine.is_nan() {
            lo
        } else {
            engine.clamp(lo, hi)
        };

        if engine <= lo || lo == hi {
            return 0.0;
        }
        if engine >= hi {
            return 1.0;
        }

        let normalized = match self {
            Mapping::Linear { .. } => inverse_lerp_f64(lo, hi, engine),
            Mapping::Exponential { min, max }
                if min.is_finite() && max.is_finite() && min > 0.0 && min < max =>
            {
                let numerator = f64::from(engine).ln() - f64::from(min).ln();
                let denominator = f64::from(max).ln() - f64::from(min).ln();
                numerator / denominator
            }
            Mapping::Exponential { .. } => inverse_lerp_f64(lo, hi, engine),
            Mapping::Skewed { min, max, skew }
                if min.is_finite()
                    && max.is_finite()
                    && min < max
                    && skew.is_finite()
                    && skew > 0.0 =>
            {
                inverse_lerp_f64(lo, hi, engine).powf(1.0 / f64::from(skew))
            }
            Mapping::Skewed { .. } => inverse_lerp_f64(lo, hi, engine),
            Mapping::AmplitudeDecibel { .. } => 10.0_f64.powf(f64::from(engine) / 20.0),
        };

        finite_normalized(normalized)
    }

    /// The mapping's authored engine bounds.
    ///
    /// Manifest validation requires these bounds to be finite, strictly ordered,
    /// and exactly equal to the descriptor's `ValueRange` bounds.
    #[must_use]
    pub fn engine_bounds(self) -> (f32, f32) {
        match self {
            Mapping::Linear { min, max }
            | Mapping::Exponential { min, max }
            | Mapping::Skewed { min, max, .. } => (min, max),
            Mapping::AmplitudeDecibel { floor_db } => (floor_db, 0.0),
        }
    }

    fn sanitized_engine_bounds(self) -> (f32, f32) {
        match self {
            Mapping::AmplitudeDecibel { floor_db } => {
                let floor = if floor_db.is_finite() && floor_db < 0.0 {
                    floor_db
                } else {
                    FALLBACK_DB_FLOOR
                };
                (floor, 0.0)
            }
            _ => sanitize_bounds(self.engine_bounds().0, self.engine_bounds().1),
        }
    }
}

fn normalized_or_floor(value: f32) -> f32 {
    if value.is_nan() {
        0.0
    } else {
        value.clamp(0.0, 1.0)
    }
}

fn sanitize_bounds(a: f32, b: f32) -> (f32, f32) {
    match (a.is_finite(), b.is_finite()) {
        (true, true) if a <= b => (a, b),
        (true, true) => (b, a),
        (true, false) => (a, a),
        (false, true) => (b, b),
        (false, false) => (0.0, 0.0),
    }
}

fn lerp_f64(min: f32, max: f32, t: f64) -> f64 {
    // f64 intermediates avoid overflow for valid f32 endpoints such as
    // `-f32::MAX..=f32::MAX` and preserve tiny non-zero f32 spans.
    f64::from(min) * (1.0 - t) + f64::from(max) * t
}

fn inverse_lerp_f64(min: f32, max: f32, value: f32) -> f64 {
    (f64::from(value) - f64::from(min)) / (f64::from(max) - f64::from(min))
}

fn finite_engine_value(value: f64, fallback: f32, min: f32, max: f32) -> f32 {
    let value = value as f32;
    if value.is_finite() {
        value.clamp(min, max)
    } else {
        fallback
    }
}

fn finite_normalized(value: f64) -> f32 {
    let value = value as f32;
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}
