use crate::{NoiseGenerator, OscillatorNode, SynthNode};

pub struct MoogNode {
    vco1: OscillatorNode,
    vco2: OscillatorNode,
    noise: NoiseGenerator,
}

impl MoogNode {
    pub fn new(vco1: OscillatorNode, vco2: OscillatorNode, noise: NoiseGenerator) -> Self {
        MoogNode { vco1, vco2, noise }
    }
}

impl SynthNode for MoogNode {
    fn process(&mut self, mono_buf: &mut [f32], sample_rate: f32) {
        for sample in mono_buf.iter_mut() {
            let vco1_sample = self.vco1.next_sample(sample_rate);
            let vco2_sample = self.vco2.next_sample(sample_rate);
            let noise_sample = self.noise.next_sample();
            *sample = 0.4 * vco1_sample + 0.4 * vco2_sample + 0.2 * noise_sample;
        }
    }

    fn note_on(&mut self, note: u8, _velocity: u8) {
        let freq = utils::note::midi_to_frequency(note);
        self.vco1.set_frequency(freq);
        self.vco2.set_frequency(freq * 1.01); // slight detune
    }

    fn note_off(&mut self) {
        // Implement note off behavior if needed
    }

    fn is_active(&self) -> bool {
        true
    }

    fn try_handle_command(&mut self, _command: &crate::commands::SynthCmd) -> bool {
        false
    }
}
