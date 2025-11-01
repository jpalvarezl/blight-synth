use std::ops::{Add, Sub, Mul};

pub trait Smoothable:
    Copy + Add<Output = Self> + Sub<Output = Self> + Mul<f32, Output = Self>
{}
impl<T> Smoothable for T
where
    T: Copy + Add<Output = T> + Sub<Output = T> + Mul<f32, Output = T>,
{}

/// A simple one-pole smoother for parameter smoothing. Improve to make generic later.
/// | Purpose             | Typical τ (seconds) | Subjective feel          |
/// | ------------------- | ------------------- | ------------------------ |
/// | Knob movement       | 0.05 – 0.1          | Natural fade without lag |
/// | Envelope modulation | 0.005 – 0.02        | Snappy, musical          |
/// | LFO modulation      | 0.001 – 0.005       | Tight and dynamic        |
#[derive(Debug, Clone, Copy)]
pub struct Smoother<T: Smoothable> {
    value: T,
    target: T,
    coeff: f32,
}

impl<T: Smoothable> Smoother<T> {
    pub fn new(sample_rate: f32, smoothing_time: f32, initial: T) -> Self {
        let coeff = 1.0 - (-1.0 / (smoothing_time * sample_rate)).exp();
        Self { value: initial, target: initial, coeff }
    }

    /// Set a new target value (e.g. from modulation or UI)
    pub fn set_target(&mut self, target: T) {
        self.target = target;
    }

    /// Advance one sample and return the smoothed value
    pub fn next_value(&mut self) -> T {
        self.value = self.value + (self.target - self.value) * self.coeff;
        self.value
    }

    /// Get the current smoothed value (without advancing)
    pub fn value(&self) -> T {
        self.value
    }

    /// Immediately jump to a value (e.g. voice reset)
    pub fn reset(&mut self, value: T) {
        self.value = value;
        self.target = value;
    }
}
