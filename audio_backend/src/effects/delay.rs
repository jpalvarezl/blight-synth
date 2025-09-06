use crate::MonoEffect;
use log::warn;

#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum DelayParameter {
    Time = 0,
    NumTaps = 1,
    Feedback = 2,
    Mix = 3,
}

impl DelayParameter {
    pub fn as_index(self) -> u32 {
        self as u32
    }
}

pub struct Delay {
    sample_rate: f32,
    // Circular buffer to store delayed samples
    buffer: Vec<f32>,
    // Current write position in the buffer
    write_pos: usize,
    // Buffer size (calculated from max delay time and sample rate)
    buffer_size: usize,
    // Delay parameters
    delay_time_samples: usize,
    num_taps: usize,
    // Feedback and mix parameters
    feedback: f32,
    mix: f32,
}

pub const MAX_DELAY_SECONDS: f32 = 25.0;
pub const MAX_TAPS: usize = 5;

impl Delay {
    /// Creates a new Delay effect
    ///
    /// # Arguments
    /// * `max_sample_rate` - Maximum expected sample rate (used for buffer allocation)
    /// * `delay_time_seconds` - Delay time in seconds (up to 25 seconds)
    /// * `num_taps` - Number of delay taps (up to 5)
    /// * `feedback` - Feedback amount (0.0 to 0.95, controls regeneration)
    /// * `mix` - Dry/wet mix (0.0 = dry only, 1.0 = wet only)
    pub fn new(
        sample_rate: f32,
        delay_time_seconds: f32,
        num_taps: usize,
        feedback: f32,
        mix: f32,
    ) -> Self {
        // Clamp parameters to safe ranges
        let delay_time_seconds = delay_time_seconds.min(MAX_DELAY_SECONDS).max(0.0);
        let num_taps = num_taps.min(MAX_TAPS).max(1);
        let feedback = feedback.min(0.95).max(0.0);
        let mix = mix.min(1.0).max(0.0);

        // Calculate buffer size for maximum delay time
        // Add extra samples for safety margin
        let max_delay_samples = (sample_rate * MAX_DELAY_SECONDS) as usize + 1024;
        let delay_time_samples = (sample_rate * delay_time_seconds) as usize;

        // Pre-allocate buffer with zeros
        let buffer = vec![0.0f32; max_delay_samples];

        Self {
            sample_rate,
            buffer,
            write_pos: 0,
            buffer_size: max_delay_samples,
            delay_time_samples,
            num_taps,
            feedback,
            mix,
        }
    }

    pub fn set_delay_time(&mut self, delay_time_seconds: f32) {
        let delay_time_seconds = delay_time_seconds.min(MAX_DELAY_SECONDS).max(0.0);
        self.delay_time_samples = (self.sample_rate * delay_time_seconds) as usize;
    }

    pub fn set_num_taps(&mut self, num_taps: usize) {
        self.num_taps = num_taps.min(5).max(1);
    }

    /// Sets feedback amount (0.0 to 0.95)
    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback.min(0.95).max(0.0);
    }

    /// Sets dry/wet mix (0.0 = dry only, 1.0 = wet only)
    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix.min(1.0).max(0.0);
    }
}

impl MonoEffect for Delay {
    /// Process audio buffer in-place (NO ALLOCATIONS ALLOWED)
    fn process(&mut self, buffer: &mut [f32], _sample_rate: f32) {
        // Early return if no delay or buffer is empty
        if self.delay_time_samples == 0 || buffer.is_empty() {
            return;
        }

        for sample in buffer.iter_mut() {
            let input_sample = *sample;
            let mut delayed_sample = 0.0f32;

            // Calculate multiple delay taps
            for tap in 0..self.num_taps {
                // Calculate delay offset for this tap
                let tap_delay = self.delay_time_samples * (tap + 1) / self.num_taps;

                // Calculate read position (with wraparound)
                let read_pos = if self.write_pos >= tap_delay {
                    self.write_pos - tap_delay
                } else {
                    self.buffer_size - (tap_delay - self.write_pos)
                };

                // Ensure read position is within bounds
                let read_pos = read_pos % self.buffer_size;

                // Add delayed sample (with decreasing amplitude for each tap)
                let tap_amplitude = 1.0 / (tap as f32 + 1.0);
                delayed_sample += self.buffer[read_pos] * tap_amplitude;
            }

            // Apply feedback to create regenerative delay
            let feedback_sample = input_sample + (delayed_sample * self.feedback);

            // Write to delay buffer
            self.buffer[self.write_pos] = feedback_sample;

            // Apply dry/wet mix
            *sample = input_sample * (1.0 - self.mix) + delayed_sample * self.mix;

            // Advance write position with wraparound
            self.write_pos = (self.write_pos + 1) % self.buffer_size;
        }
    }

    fn set_parameter(&mut self, _index: u32, value: f32) {
        match _index {
            0 => self.set_delay_time(value),
            1 => {
                // num_taps is an integer; ensure value is clamped to [1, MAX_TAPS] before casting
                let taps = value.clamp(1.0, MAX_TAPS as f32) as usize;
                self.set_num_taps(taps);
            }
            2 => self.set_feedback(value),
            3 => self.set_mix(value),
            _ => warn!("Invalid parameter index for delay effect"),
        }
    }

    fn reset(&mut self) {
        for s in self.buffer.iter_mut() {
            *s = 0.0;
        }
        self.write_pos = 0;
    }
}
