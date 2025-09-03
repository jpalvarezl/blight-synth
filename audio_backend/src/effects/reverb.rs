use log::warn;

use crate::{MonoEffect, StereoEffect};

// The main Reverb effect
pub struct Reverb {
    // Comb filter delays (in samples)
    comb_buffers: [Vec<f32>; 4],
    comb_indices: [usize; 4],
    comb_feedback: [f32; 4],

    // Allpass filter delays
    allpass_buffers: [Vec<f32>; 2],
    allpass_indices: [usize; 2],
    allpass_feedback: f32,

    // Wet/dry mix
    wet_gain: f32,
    dry_gain: f32,
}

impl Reverb {
    /// Creates a new Reverb instance.
    /// All memory allocation for the delay lines happens here, making it real-time safe.
    pub fn new(sample_rate: f32) -> Self {
        // Delay times in milliseconds for different room sizes
        let comb_delays_ms = [29.7, 37.1, 41.1, 43.7];
        let allpass_delays_ms = [5.0, 1.7];

        // Convert to samples
        let comb_delays: [usize; 4] = comb_delays_ms
            .iter()
            .map(|&ms| (ms * sample_rate / 1000.0) as usize)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        let allpass_delays: [usize; 2] = allpass_delays_ms
            .iter()
            .map(|&ms| (ms * sample_rate / 1000.0) as usize)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        // Initialize buffers
        let mut comb_buffers = [(); 4].map(|_| Vec::new());
        for i in 0..4 {
            comb_buffers[i] = vec![0.0; comb_delays[i]];
        }

        let mut allpass_buffers = [(); 2].map(|_| Vec::new());
        for i in 0..2 {
            allpass_buffers[i] = vec![0.0; allpass_delays[i]];
        }

        Self {
            comb_buffers,
            comb_indices: [0; 4],
            comb_feedback: [0.84, 0.82, 0.79, 0.76], // Different feedback for each comb

            allpass_buffers,
            allpass_indices: [0; 2],
            allpass_feedback: 0.7,

            wet_gain: 0.3,
            dry_gain: 0.7,
        }
    }

    fn comb_filter(&mut self, input: f32, index: usize) -> f32 {
        let buffer = &mut self.comb_buffers[index];
        let buf_index = &mut self.comb_indices[index];
        let feedback = self.comb_feedback[index];

        let delayed = buffer[*buf_index];
        let output = delayed;

        // Write new value with feedback
        buffer[*buf_index] = input + delayed * feedback;

        // Update circular buffer index
        *buf_index = (*buf_index + 1) % buffer.len();

        output
    }

    fn allpass_filter(&mut self, input: f32, index: usize) -> f32 {
        let buffer = &mut self.allpass_buffers[index];
        let buf_index = &mut self.allpass_indices[index];
        let feedback = self.allpass_feedback;

        let delayed = buffer[*buf_index];
        let output = -input * feedback + delayed;

        // Write new value
        buffer[*buf_index] = input + delayed * feedback;

        // Update circular buffer index
        *buf_index = (*buf_index + 1) % buffer.len();

        output
    }
}

impl MonoEffect for Reverb {
    fn process(&mut self, buf: &mut [f32], _sample_rate: f32) {
        for sample in buf.iter_mut() {
            let input = *sample;
            let mut output = 0.0;

            // Sum all comb filter outputs
            for i in 0..4 {
                output += self.comb_filter(input, i);
            }

            // Scale comb output
            output *= 0.25;

            // Process through allpass filters in series
            output = self.allpass_filter(output, 0);
            output = self.allpass_filter(output, 1);

            // Mix wet and dry signals
            *sample = self.dry_gain * input + self.wet_gain * output;
        }
    }

    fn set_parameter(&mut self, index: u32, value: f32) {
        match index {
            0 => self.wet_gain = value,
            1 => self.dry_gain = value,
            _ => warn!("Invalid parameter index for reverb effect"),
        }
    }

    fn reset(&mut self) {
        // Clear all buffers and reset indices
        for i in 0..4 {
            self.comb_buffers[i].fill(0.0);
            self.comb_indices[i] = 0;
        }
        for i in 0..2 {
            self.allpass_buffers[i].fill(0.0);
            self.allpass_indices[i] = 0;
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

    fn reset(&mut self) {
        self.left.reset();
        self.right.reset();
    }
}
