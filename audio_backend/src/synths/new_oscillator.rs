// ⚙️ Optimization Ideas (Later)
//   - Use lookup tables (LUTs) for sine/trig functions
//   - SIMD waveform generation
//   - PolyBLEP for band-limited oscillators (avoids aliasing)

use std::f32::consts::PI;

use crate::synths::waveform::Waveform;

pub struct Oscillator {
    waveform: Waveform,
    frequency: f32,
    sample_rate: f32,
    phase: f32,
}

impl Oscillator {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            waveform: Waveform::Sine,
            frequency: 440.0,
            sample_rate,
            phase: 0.0,
        }
    }

    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.waveform = waveform;
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
    }

    pub fn reset(&mut self) {
        self.phase = 0.0;
    }

    pub fn next_sample(&mut self) -> f32 {
        let phase_inc = self.frequency * 2.0 * PI / self.sample_rate;
        let sample = match self.waveform {
            Waveform::Sine => (self.phase).sin(),
            Waveform::Square => if self.phase < PI { 1.0 } else { -1.0 },
            Waveform::Sawtooth => 2.0 * (self.phase / (2.0 * PI)) - 1.0,
            Waveform::Triangle => {
                2.0 * (2.0 * (self.phase / (2.0 * PI)) - 1.0).abs() - 1.0
            }
        };

        self.phase = (self.phase + phase_inc) % (2.0 * PI);
        sample
    }
}
