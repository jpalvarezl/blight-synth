use crate::MonoEffect;

/// Simple digital delay with feedback
///
/// Signal flow:
/// input -> [+] -> output
///           |
///           v
///      [delay line] -> [feedback] --+
///                                   |
///                                   +-> back to input
pub struct Delay {
    buffer: Vec<f32>,
    write_index: usize,
    delay_samples: usize,
    feedback: f32,
    mix: f32,
}

impl Delay {
    pub fn new(sample_rate: f32, delay_ms: f32, feedback: f32, mix: f32) -> Self {
        let max_delay_samples = (sample_rate * 2.0) as usize; // 2 seconds max
        let delay_samples = ((delay_ms / 1000.0) * sample_rate) as usize;

        Self {
            buffer: vec![0.0; max_delay_samples],
            write_index: 0,
            delay_samples: delay_samples.min(max_delay_samples - 1),
            feedback: feedback.clamp(0.0, 0.95), // Prevent runaway feedback
            mix: mix.clamp(0.0, 1.0),
        }
    }

    pub fn set_delay_ms(&mut self, delay_ms: f32, sample_rate: f32) {
        self.delay_samples = ((delay_ms / 1000.0) * sample_rate) as usize;
        self.delay_samples = self.delay_samples.min(self.buffer.len() - 1);
    }
}

impl MonoEffect for Delay {
    fn process(&mut self, buffer: &mut [f32], _sample_rate: f32) {
        for sample in buffer.iter_mut() {
            let input = *sample;

            // Read from delay line
            let read_index =
                (self.write_index + self.buffer.len() - self.delay_samples) % self.buffer.len();
            let delayed = self.buffer[read_index];

            // Write to delay line with feedback
            self.buffer[self.write_index] = input + delayed * self.feedback;
            self.write_index = (self.write_index + 1) % self.buffer.len();

            // Mix dry and wet signals
            *sample = input * (1.0 - self.mix) + delayed * self.mix;
        }
    }

    fn set_parameter(&mut self, _index: u32, _value: f32) {
        // not implemented yet
    }
}
