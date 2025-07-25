use crate::Command;

pub struct Synthesizer {
    pub sample_rate: f32,
    // Add more fields as needed for synthesizer state.
    oscillator: TestOscillator,
}

struct TestOscillator {
    sample_rate: f32,
    frequency: f32,
    phase: f32,
}

impl TestOscillator {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            frequency: 0.0, // Default frequency
            phase: 0.0,     // Initial phase
        }
    }

    pub fn next_sample(&mut self) -> f32 {
        let sample = (self.phase * std::f32::consts::TAU).sin();
        self.phase += self.frequency / self.sample_rate;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
        sample
    }
}

impl Synthesizer {
    pub fn new(sample_rate: f32) -> Self {
        let oscillator = TestOscillator::new(sample_rate);
        Self {
            sample_rate,
            oscillator,
        }
    }

    pub fn handle_command(&mut self, command: Command) {
        match command {
            Command::PlayNote { note, velocity } => {
                // midi note translation should be extracted.
                self.oscillator.frequency = 440.0 * 2f32.powf((note as f32 - 69.0) / 12.0);
            }
            Command::StopNote { note } => {
                // Handle stopping a note.
            }
            Command::SetParameter { param, value } => {
                // Handle setting a parameter.
            }
        }
    }

    pub fn process(&mut self, left_buf: &mut [f32], right_buf: &mut [f32]) {
        // Process audio and fill the output buffer.
        for (left, right) in left_buf.iter_mut().zip(right_buf.iter_mut()) {
            let sample = self.oscillator.next_sample();
            *left = sample;
            *right = sample;
        }
    }
}
