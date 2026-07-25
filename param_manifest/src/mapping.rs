//! Normalized `0..1` <-> engine-value mapping.
//!
//! The mapping is the single owner of unit conversion between the host-facing
//! normalized control value (always `0..1`, the VST/AU/APVTS convention) and the
//! engine-facing value a DSP node consumes (dB, Hz, seconds, a linear factor,
//! etc.). Every host adapter — OSC, APVTS, Svelte — converts through this type so
//! the conversion is defined once. The mapping is `Copy` and string-free, so it
//! lives in both the descriptor and the real-time [`RuntimeParameter`].

use serde::{Deserialize, Serialize};

/// How a normalized `0..1` control value maps to the engine value and back.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "curve", rename_all = "snake_case")]
pub enum Mapping {
    /// `engine = min + t * (max - min)`; `t = (engine - min) / (max - min)`.
    Linear { min: f32, max: f32 },

    /// Perceptual/exponential mapping between two positive endpoints:
    /// `engine = min * (max / min)^t`. Useful for frequency and time controls.
    Exponential { min: f32, max: f32 },

    /// The normalized value is a linear amplitude (`0..1`) and the engine value
    /// is decibels: `engine = clamp_floor(20 * log10(t))`. This is the master
    /// gain convention shared with the OSC `/param/set gain` mapping.
    AmplitudeDecibel { floor_db: f32 },
}

impl Mapping {
    /// Convert a normalized `0..1` value to the engine value.
    ///
    /// The input is clamped to `0..1` first; the output stays within the
    /// mapping's engine range. This is `Copy`/branch-only arithmetic and is safe
    /// to call on the audio thread.
    #[must_use]
    pub fn to_engine(self, normalized: f32) -> f32 {
        let t = normalized.clamp(0.0, 1.0);
        match self {
            Mapping::Linear { min, max } => min + t * (max - min),
            Mapping::Exponential { min, max } => {
                // Endpoints are expected positive; guard against non-positive
                // configuration by falling back to linear interpolation.
                if min > 0.0 && max > 0.0 {
                    min * (max / min).powf(t)
                } else {
                    min + t * (max - min)
                }
            }
            Mapping::AmplitudeDecibel { floor_db } => {
                if t <= 0.0 {
                    floor_db
                } else {
                    (20.0 * t.log10()).max(floor_db)
                }
            }
        }
    }

    /// Convert an engine value back to a normalized `0..1` value.
    ///
    /// The result is clamped to `0..1`. This is the inverse of [`to_engine`]
    /// within the mapping's representable range.
    ///
    /// [`to_engine`]: Mapping::to_engine
    #[must_use]
    pub fn to_normalized(self, engine: f32) -> f32 {
        let t = match self {
            Mapping::Linear { min, max } => {
                if (max - min).abs() <= f32::EPSILON {
                    0.0
                } else {
                    (engine - min) / (max - min)
                }
            }
            Mapping::Exponential { min, max } => {
                if min > 0.0 && max > 0.0 && (max / min - 1.0).abs() > f32::EPSILON && engine > 0.0 {
                    (engine / min).log10() / (max / min).log10()
                } else if (max - min).abs() > f32::EPSILON {
                    (engine - min) / (max - min)
                } else {
                    0.0
                }
            }
            Mapping::AmplitudeDecibel { floor_db } => {
                if engine <= floor_db {
                    0.0
                } else {
                    10.0_f32.powf(engine / 20.0)
                }
            }
        };
        t.clamp(0.0, 1.0)
    }

    /// The finite-checkable numeric endpoints/parameters of this mapping, used by
    /// manifest validation to reject non-finite configuration.
    #[must_use]
    pub fn endpoint_values(self) -> [f32; 2] {
        match self {
            Mapping::Linear { min, max } | Mapping::Exponential { min, max } => [min, max],
            Mapping::AmplitudeDecibel { floor_db } => [floor_db, floor_db],
        }
    }
}
