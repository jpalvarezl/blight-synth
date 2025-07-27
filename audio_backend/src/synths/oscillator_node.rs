use utils::note::midi_to_frequency;

use crate::SynthNode;

/// Represents a simple oscillator node for audio synthesis.
/// It can generate different waveforms such as sine, square, sawtooth, and triangle
#[derive(Debug, Clone, Copy)]
pub struct OscillatorNode {
    waveform: Waveform,
    frequency: f32,
    phase: f32,
}

/// Represents the different waveforms that the oscillator can generate.
#[derive(Debug, Clone, Copy)]
pub enum Waveform {
    Sine,
    Square,
    Sawtooth,
    Triangle,
}

impl OscillatorNode {
    pub fn new() -> Self {
        OscillatorNode {
            waveform: Waveform::Sine,
            frequency: 0.0,
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

    pub fn next_sample(&mut self, sample_rate: f32) -> f32 {
        let phase_inc = self.frequency * std::f32::consts::TAU / sample_rate;

        let sample = match self.waveform {
            Waveform::Sine => (self.phase).sin(),
            Waveform::Square => {
                if self.phase < std::f32::consts::PI - f32::EPSILON {
                    1.0
                } else {
                    -1.0
                }
            }
            Waveform::Sawtooth => 2.0 * (self.phase / std::f32::consts::TAU) - 1.0,
            Waveform::Triangle => {
                let normalized_phase = self.phase / std::f32::consts::TAU; // 0 to 1
                if normalized_phase < 0.5 {
                    // Rising: -1 to 1
                    4.0 * normalized_phase - 1.0
                } else {
                    // Falling: 1 to -1
                    3.0 - 4.0 * normalized_phase
                }
            }
        };

        self.phase += phase_inc;
        if self.phase >= std::f32::consts::TAU {
            self.phase -= std::f32::consts::TAU;
        }

        sample
    }
}

impl SynthNode for OscillatorNode {
    fn process(&mut self, mono_buf: &mut [f32], sample_rate: f32) {
        // Implementation of oscillator processing logic
        for sample in mono_buf.iter_mut() {
            *sample = self.next_sample(sample_rate);
        }
    }

    fn note_on(&mut self, note: u8, _velocity: u8) {
        self.frequency = midi_to_frequency(note)
    }

    fn note_off(&mut self) {
        // ignored for dumb oscillator
    }

    fn is_active(&self) -> bool {
        true
    }
}
