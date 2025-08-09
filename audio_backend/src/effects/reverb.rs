use std::f32::consts::FRAC_PI_2;

use crate::{MonoEffect, StereoEffect};

// Building Block 1: Feedback Comb Filter
struct CombFilter {
    /// The delay line buffer, implemented as a vector.
    buffer: Vec<f32>,
    /// The current write/read position in the circular buffer.
    pos: usize,
    /// The feedback gain, controlling the decay time of the echoes. Must be < 1.0 for stability.
    feedback: f32,
    /// The coefficient for a simple one-pole low-pass filter in the feedback path.
    /// This simulates high-frequency absorption in a room, making the reverb tail darker.
    damping: f32,
    /// Stores the previous output of the low-pass filter for the next sample calculation.
    lowpass_state: f32,
}

impl CombFilter {
    fn process(&mut self, input: f32) -> f32 {
        let output = self.buffer[self.pos];
        self.lowpass_state = output * (1.0 - self.damping) + self.lowpass_state * self.damping;
        self.buffer[self.pos] = input + self.lowpass_state * self.feedback;
        self.pos = (self.pos + 1) % self.buffer.len();
        output
    }

    fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback.clamp(0.0, 1.0);
    }

    fn set_damping(&mut self, damping: f32) {
        self.damping = damping.clamp(0.0, 1.0);
    }
}

// Building Block 2: All-Pass Filter
struct AllPassFilter {
    buffer: Vec<f32>,
    pos: usize,
    gain: f32,
}
impl AllPassFilter {
    fn process(&mut self, input: f32) -> f32 {
        let delayed = self.buffer[self.pos];
        let output = -input + delayed;
        self.buffer[self.pos] = input + delayed * self.gain;
        self.pos = (self.pos + 1) % self.buffer.len();
        output
    }
}

// The main Reverb effect
pub struct Reverb {
    comb_filters: [CombFilter; 4],       // Example: 4 parallel comb filters
    allpass_filters: [AllPassFilter; 2], // Example: 2 series all-pass filters
    wet_level: f32, // Gain compensation for the wet signal
    wet_dry_mix: f32,
}

impl Reverb {
    /// Creates a new Reverb instance.
    /// All memory allocation for the delay lines happens here, making it real-time safe.
    pub fn new(sample_rate: f32) -> Self {
        // Pre-defined, mutually prime delay times in milliseconds, chosen for a pleasant sound.
        const COMB_TIMES_MS: [f32; 4] = [29.7, 37.1, 41.1, 43.7];
        const ALLPASS_TIMES_MS: [f32; 2] = [5.0, 1.7]; // Shorter delays for all-pass diffusers.

        // Helper closure to create an array of CombFilter instances.
        // `map` is not available on arrays of this size by default, so we build it manually.
        let mut comb_filters: [CombFilter; 4] = std::array::from_fn(|_| CombFilter {
            buffer: Vec::new(),
            pos: 0,
            feedback: 0.0,
            damping: 0.0,
            lowpass_state: 0.0,
        });
        for (i, time_ms) in COMB_TIMES_MS.iter().enumerate() {
            let delay_samples = (time_ms / 1000.0 * sample_rate).round() as usize;
            comb_filters[i] = CombFilter {
                buffer: vec![0.0; delay_samples],
                pos: 0,
                feedback: 0.6,
                damping: 0.2,
                lowpass_state: 0.0,
            };
        }

        // Helper closure to create an array of AllPassFilter instances.
        let mut allpass_filters: [AllPassFilter; 2] = std::array::from_fn(|_| AllPassFilter {
            buffer: Vec::new(),
            pos: 0,
            gain: 0.0,
        });
        for (i, time_ms) in ALLPASS_TIMES_MS.iter().enumerate() {
            let delay_samples = (time_ms / 1000.0 * sample_rate).round() as usize;
            allpass_filters[i] = AllPassFilter {
                buffer: vec![0.0; delay_samples],
                pos: 0,
                gain: 0.7,
            };
        }

        Self {
            comb_filters,
            allpass_filters,
            wet_level: 0.25, // Default to full wet
            wet_dry_mix: 0.3, // Default to a subtle mix
        }
    }
}

impl MonoEffect for Reverb {
    fn process(&mut self, buf: &mut [f32], _sample_rate: f32) {
        // Calculate equal-power gains once per block for efficiency
        let mix_angle = self.wet_dry_mix * FRAC_PI_2;
        let dry_gain = mix_angle.cos();
        let wet_gain = mix_angle.sin();

        for sample in buf.iter_mut() {
            let dry_signal = *sample;

            // 1. Pass through parallel comb filters and sum their outputs
            let mut comb_out = 0.0;
            for filter in self.comb_filters.iter_mut() {
                comb_out += filter.process(*sample);
            }

            // 2. Pass the result through series all-pass filters
            let mut allpass_out = comb_out;
            for filter in self.allpass_filters.iter_mut() {
                allpass_out = filter.process(allpass_out);
            }

            // 3. Mix dry and wet signals
            let wet_signal = allpass_out * self.wet_level;

            *sample = (dry_signal * dry_gain) + (wet_signal * wet_gain);
        }
    }

    fn set_parameter(&mut self, index: u32, value: f32) {
        match index {
            0 => self.wet_level = value,
            1 => self.wet_dry_mix = value,
            2 => self.comb_filters.iter_mut().for_each(|f| f.set_feedback(value)),
            3 => self.comb_filters.iter_mut().for_each(|f| f.set_damping(value)),
            _ => (),
        }
    }
}

pub struct StereoReverb {
    left: Reverb,
    right: Reverb,
}

impl StereoReverb {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            left: Reverb::new(sample_rate),
            right: Reverb::new(sample_rate),
        }
    }
}

// This might be too lazy of an implementation. Need to look into this topic some more.
impl StereoEffect for StereoReverb {
    fn process(&mut self, left_buf: &mut [f32], right_buf: &mut [f32], sample_rate: f32) {
        // Process left and right channels independently
        self.left.process(left_buf, sample_rate);
        self.right.process(right_buf, sample_rate);
    }

    fn set_parameter(&mut self, index: u32, value: f32) {
        self.left.set_parameter(index, value);
        self.right.set_parameter(index, value);
    }
}
