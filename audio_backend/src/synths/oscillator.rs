use std::f32::consts::TAU;

// Defines the essential behavior of any oscillator shape.
// Send + Sync + 'static bounds are important for use in threads (like CPAL's callback)
pub trait Waveform: Send + Sync + 'static {
    /// Calculates the waveform sample for a given phase [0.0, 1.0).
    /// Returns sample value, typically [-1.0, 1.0].
    fn shape(phase: f32) -> f32;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Sine;
impl Waveform for Sine {
    #[inline]
    fn shape(phase: f32) -> f32 {
        (phase * TAU).sin()
    }
}

/// Naive Sawtooth Wave generator (aliasing).
#[derive(Clone, Copy, Debug, Default)]
pub struct Saw;
impl Waveform for Saw {
    #[inline]
    fn shape(phase: f32) -> f32 {
        2.0 * phase - 1.0
    }
}

/// Naive Square Wave generator (aliasing).
#[derive(Clone, Copy, Debug, Default)]
pub struct Square;
impl Waveform for Square {
    #[inline]
    fn shape(phase: f32) -> f32 {
        if phase < 0.5 {
            1.0
        } else {
            -1.0
        }
    }
}

/// Naive Triangle Wave generator (aliasing).
#[derive(Clone, Copy, Debug, Default)]
pub struct Triangle;
impl Waveform for Triangle {
    #[inline]
    fn shape(phase: f32) -> f32 {
        1.0 - 4.0 * (phase - 0.5).abs()
    }
}

// Manages state and uses a Waveform type W.
#[derive(Debug, Clone)]
pub struct Oscillator<W: Waveform> {
    sample_rate: f32,
    phase: f32,
    phase_increment: f32,
    _waveform: std::marker::PhantomData<W>,
}

// --- Oscillator Implementation ---
// (Selected implementation block remains the same)
impl<W: Waveform> Oscillator<W> {
    pub fn new(sample_rate: f32, frequency: f32) -> Self {
        assert!(sample_rate > 0.0, "Sample rate must be positive.");
        let osc = Oscillator {
            sample_rate,
            phase: 0.0,
            phase_increment: frequency.max(0.0) / sample_rate,
            _waveform: std::marker::PhantomData,
        };
        osc
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        let freq = frequency.max(0.0);
        self.phase_increment = freq / self.sample_rate;
    }

    pub fn get_frequency(&self) -> f32 {
        self.phase_increment * self.sample_rate
    }

    pub fn get_sample_rate(&self) -> f32 {
        self.sample_rate
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        assert!(sample_rate > 0.0, "Sample rate must be positive.");
        let current_frequency = self.get_frequency();
        self.sample_rate = sample_rate;
        self.set_frequency(current_frequency);
    }

    #[inline]
    pub fn next_sample(&mut self) -> f32 {
        let output = W::shape(self.phase);
        self.phase += self.phase_increment;
        self.phase = self.phase.fract();
        output
    }

    pub fn reset_phase(&mut self, phase: f32) {
        self.phase = phase.fract().abs();
    }
}
