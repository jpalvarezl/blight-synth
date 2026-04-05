/// Audio utility functions — DSP math, buffer helpers, etc.

/// Convert linear amplitude to dB.
pub fn linear_to_db(linear: f32) -> f32 {
    // TODO: implement 20 * log10(linear), handle -inf for 0.0
    0.0
}

/// Convert dB to linear amplitude.
pub fn db_to_linear(db: f32) -> f32 {
    // TODO: implement 10^(db/20)
    1.0
}

/// Clamp a value to a normalised 0..1 range.
pub fn clamp_normal(value: f32) -> f32 {
    // TODO: clamp
    value
}

/// A simple one-pole smoothing filter for parameter changes.
pub struct ParamSmoother {
    current: f32,
    target: f32,
    coeff: f32,
}

impl ParamSmoother {
    pub fn new(initial: f32, smoothing_ms: f32, sample_rate: f32) -> Self {
        Self {
            current: initial,
            target: initial,
            coeff: 0.0, // TODO: compute from smoothing_ms and sample_rate
        }
    }

    pub fn set_target(&mut self, target: f32) {
        self.target = target;
    }

    /// Call once per sample in the audio callback.
    pub fn next(&mut self) -> f32 {
        // TODO: one-pole IIR: current = current + coeff * (target - current)
        self.current
    }
}
