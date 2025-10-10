// TODO: think of a better place for this module

use utils::note::velocity_to_amplitude;

use crate::SynthNode;

pub struct NoiseGenerator {
    state: u32,
    volume: f32,
}

impl NoiseGenerator {
    pub fn new(seed: u32) -> Self {
        Self { state: seed, volume: 0.7 }
    }

    #[inline(always)]
    pub fn next_sample(&mut self) -> f32 {
        // Xorshift PRNG
        self.state ^= self.state << 13;
        self.state ^= self.state >> 17;
        self.state ^= self.state << 5;
        let sample = (self.state & 0xFF) as f32 / 128.0 - 1.0;

        sample
    }
}

impl Default for NoiseGenerator {
    fn default() -> Self {
        Self::new(0x12345678) // Some arbitrary non-zero seed
    }
}

impl SynthNode for NoiseGenerator {
    fn process(&mut self, mono_buf: &mut [f32], _sample_rate: f32) {
        // Implementation of oscillator processing logic
        for sample in mono_buf.iter_mut() {
            *sample = self.volume * self.next_sample();
        }
    }

    fn note_on(&mut self, _note: u8, velocity: u8) {
        self.volume = velocity_to_amplitude(velocity);
    }

    fn note_off(&mut self) {
        // ignored for noise
    }

    fn is_active(&self) -> bool {
        true
    }
}
