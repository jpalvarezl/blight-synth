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
    /// 32-step, 4-bit NES triangle lookup table waveform
    NesTriangle,
}

impl OscillatorNode {
    pub fn new() -> Self {
        Self {
            waveform: Waveform::Sine,
            frequency: 0.0,
            phase: 0.0,
        }
    }

    pub fn new_with_waveform(waveform: Waveform) -> Self {
        Self {
            waveform,
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

    #[inline]
    fn nes_triangle_sample(phase: f32) -> f32 {
        // NES APU triangle: 32-step repeating sequence (4-bit amplitude)
        // Commonly represented as: 15..0, 0..15 (duplicate extremes) or phase-shifted equivalent.
        // We'll use 0..15 ascending then 15..0 descending with duplicated endpoints for exact 32 steps.
        const LUT_U4: [u8; 32] = [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 15, 14, 13, 12, 11, 10, 9, 8, 7,
            6, 5, 4, 3, 2, 1, 0,
        ];
        // Convert phase in [0, TAU) to index 0..31
        let normalized_phase = phase / std::f32::consts::TAU; // [0,1)
        let idx = (normalized_phase * 32.0) as usize; // truncation OK for [0,1)
        let v = LUT_U4[idx & 31] as f32;
        // Map 0..15 to -1..1 exactly
        (v / 15.0) * 2.0 - 1.0
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
            Waveform::NesTriangle => Self::nes_triangle_sample(self.phase),
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

    fn try_handle_command(&mut self, command: &crate::synth_infra::SynthCommand) -> bool {
        let was_handled = match command {
            crate::synth_infra::SynthCommand::SetWaveform(waveform) => {
                self.set_waveform(*waveform);
                true
            }
            _ => false,
        };
        was_handled
    }
}
