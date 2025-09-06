use log::warn;

use crate::{MonoEffect, StereoEffect};

// The main Reverb effect
pub struct Reverb {
    // We need to store the sample_rate to recalculate delay sizes if room size changes
    sample_rate: f32,

    // Comb filter delays (in samples) - pre-allocated max size
    comb_buffers: [Vec<f32>; 4],
    comb_indices: [usize; 4],
    comb_delays: [usize; 4],
    comb_feedback: [f32; 4],

    // Allpass filter delays
    allpass_buffers: [Vec<f32>; 2],
    allpass_indices: [usize; 2],
    allpass_delays: [usize; 2],
    allpass_feedback: f32,

    // Single mix control (0.0 = dry only, 1.0 = wet only)
    mix: f32,
}

impl Reverb {
    /// Creates a new Reverb instance.
    /// All memory allocation for the delay lines happens here, making it real-time safe.
    pub fn new(sample_rate: f32) -> Self {
        // Base delay times in milliseconds
        const BASE_COMB_DELAYS_MS: [f32; 4] = [29.7, 37.1, 41.1, 43.7];
        const BASE_ALLPASS_DELAYS_MS: [f32; 2] = [5.0, 1.7];
        const MAX_ROOM_SIZE: f32 = 3.0; // Pre-allocate for up to 3x room size

        // Calculate current delay lengths (start at normal room size)
        let current_comb_delays: [usize; 4] = BASE_COMB_DELAYS_MS
            .iter()
            .map(|&ms| (ms * sample_rate / 1000.0) as usize)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        let current_allpass_delays: [usize; 2] = BASE_ALLPASS_DELAYS_MS
            .iter()
            .map(|&ms| (ms * sample_rate / 1000.0) as usize)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        // Pre-allocate buffers for maximum room size
        let max_comb_delays: [usize; 4] = BASE_COMB_DELAYS_MS
            .iter()
            .map(|&ms| ((ms * MAX_ROOM_SIZE) * sample_rate / 1000.0) as usize)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        let max_allpass_delays: [usize; 2] = BASE_ALLPASS_DELAYS_MS
            .iter()
            .map(|&ms| ((ms * MAX_ROOM_SIZE) * sample_rate / 1000.0) as usize)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        // Initialize buffers with maximum size
        let mut comb_buffers = [(); 4].map(|_| Vec::new());
        for i in 0..4 {
            comb_buffers[i] = vec![0.0; max_comb_delays[i]];
        }

        let mut allpass_buffers = [(); 2].map(|_| Vec::new());
        for i in 0..2 {
            allpass_buffers[i] = vec![0.0; max_allpass_delays[i]];
        }

        Self {
            sample_rate,

            comb_buffers,
            comb_indices: [0; 4],
            comb_delays: current_comb_delays, // Initialize the new field
            comb_feedback: [0.84, 0.82, 0.79, 0.76],

            allpass_buffers,
            allpass_indices: [0; 2],
            allpass_delays: current_allpass_delays, // Initialize the new field
            allpass_feedback: 0.7,

            mix: 0.3,
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

    // Adjust reverb decay time by changing comb filter feedback
    pub fn set_decay_time(&mut self, decay: f32) {
        // decay: 0.0 = very short, 1.0 = very long
        let base_feedbacks = [0.84, 0.82, 0.79, 0.76];
        for i in 0..4 {
            // Scale feedback but keep different values for each comb
            self.comb_feedback[i] = base_feedbacks[i] * decay.clamp(0.0, 0.95);
        }
    }

    // Adjust room size by scaling all delay times
    // Warning: this method calls resize which is a violation of our realtime constraints.
    pub fn set_room_size(&mut self, size: f32, sample_rate: f32) {
        // size: 0.5 = small room, 1.0 = normal, up to 3.0 = large hall
        const BASE_COMB_DELAYS_MS: [f32; 4] = [29.7, 37.1, 41.1, 43.7];
        const BASE_ALLPASS_DELAYS_MS: [f32; 2] = [5.0, 1.7];
        let clamped_size = size.clamp(0.1, 3.0);

        // Update comb filter delay lengths
        for i in 0..4 {
            let new_delay =
                ((BASE_COMB_DELAYS_MS[i] * clamped_size) * sample_rate / 1000.0) as usize;
            self.comb_delays[i] = new_delay.max(1); // Ensure at least 1 sample delay
            self.comb_indices[i] = self.comb_indices[i] % self.comb_delays[i]; // Wrap index if needed
        }

        // Update allpass filter delay lengths
        for i in 0..2 {
            let new_delay =
                ((BASE_ALLPASS_DELAYS_MS[i] * clamped_size) * sample_rate / 1000.0) as usize;
            self.allpass_delays[i] = new_delay.max(1); // Ensure at least 1 sample delay
            self.allpass_indices[i] = self.allpass_indices[i] % self.allpass_delays[i];
            // Wrap index if needed
        }
    }

    // Adjust high frequency damping (simulates air absorption)
    pub fn set_damping(&mut self, damping: f32) {
        // damping: 0.0 = bright, 1.0 = very dark
        // The constants control how much damping affects shorter vs longer delays:
        // - DAMPING_BASE (0.3): Maximum damping reduction for the shortest delay
        // - DAMPING_STEP (0.05): How much less damping each subsequent delay gets
        const DAMPING_BASE: f32 = 0.3; // Max damping factor for first comb filter
        const DAMPING_STEP: f32 = 0.05; // Damping reduction per comb filter
        let base_comb_feedback = [0.84, 0.82, 0.79, 0.76]; // Store base feedback values

        for i in 0..4 {
            // Shorter delays (higher freq content) get more damping than longer delays
            let damping_factor = 1.0 - (damping * (DAMPING_BASE - i as f32 * DAMPING_STEP));
            self.comb_feedback[i] = base_comb_feedback[i] * damping_factor.max(0.1);
        }
    }

    // Adjust diffusion (how scattered/smooth the reverb sounds)
    pub fn set_diffusion(&mut self, diffusion: f32) {
        // diffusion: 0.0 = echoey, 1.0 = very smooth
        let base_allpass_feedback = 0.7;
        self.allpass_feedback = base_allpass_feedback * diffusion.clamp(0.0, 0.95);
    }
    // Adjust wet/dry mix with a single parameter
    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }
}

impl MonoEffect for Reverb {
    fn process(&mut self, buf: &mut [f32], sample_rate: f32) {
        self.sample_rate = sample_rate;

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

            // Mix wet and dry signals using single mix parameter
            *sample = (1.0 - self.mix) * input + self.mix * output;
        }
    }

    fn set_parameter(&mut self, index: u32, value: f32) {
        match index {
            0 => self.set_mix(value),
            1 => self.set_decay_time(value),
            2 => self.set_room_size(value, self.sample_rate),
            3 => self.set_damping(value),
            4 => self.set_diffusion(value),
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
