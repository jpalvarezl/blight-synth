use crate::{
    instruments::synth_nodes::drums::noise_generator::NoiseGenerator, Envelope, SynthNode,
};

pub struct HiHatVoice {
    noise: NoiseGenerator,
    env: Envelope, // very short decay
    active: bool,
}

impl HiHatVoice {
    pub fn new(env: Envelope) -> Self {
        Self {
            noise: NoiseGenerator::default(),
            env,
            active: false,
        }
    }

    pub fn note_on(&mut self) {
        self.env.gate(true);
        self.active = true;
    }

    pub fn note_off(&mut self) {
        self.env.gate(false);
    }

    pub fn next_sample(&mut self) -> f32 {
        if !self.active {
            return 0.0;
        }

        let s = self.noise.next_sample() * self.env.process(); // sample_rate unused if ADSR is normalized

        if !self.env.is_active() {
            self.active = false;
        }

        s
    }
}

impl SynthNode for HiHatVoice {
    fn process(&mut self, mono_buf: &mut [f32], _sample_rate: f32) {
        for sample in mono_buf.iter_mut() {
            *sample += self.next_sample();
        }
    }

    fn note_on(&mut self, _note: u8, _velocity: u8) {
        self.note_on();
    }

    fn note_off(&mut self) {
        self.note_off();
    }

    fn is_active(&self) -> bool {
        self.active
    }
}
