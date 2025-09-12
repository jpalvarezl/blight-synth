use crate::{Envelope, NoiseGenerator, OscillatorNode, SynthNode};

pub struct SnareDrumVoice {
    osc: OscillatorNode,   // triangle or square "body"
    osc_env: Envelope,  // quick decay for body part
    noise: NoiseGenerator, // medium decay for sizzle part
    noise_env: Envelope,
}

impl SnareDrumVoice {
    pub fn new(sample_rate: f32) -> Self {
        let mut osc_env = Envelope::new(sample_rate);
        osc_env.set_parameters(0.01, 0.05, 0.0, 0.1); // quick attack, short decay

        let mut noise_env = Envelope::new(sample_rate);
        noise_env.set_parameters(0.01, 0.2, 0.0, 0.1); // very quick attack, very short decay

        SnareDrumVoice {
            osc: OscillatorNode::new(),
            osc_env,
            noise: NoiseGenerator::default(),
            noise_env,
        }
    }
}

impl SynthNode for SnareDrumVoice {
    fn process(&mut self, mono_buf: &mut [f32], sample_rate: f32) {
        for sample in mono_buf.iter_mut() {
            if !self.is_active() {
                *sample = 0.0;
                continue;
            }
            let tone = self.osc.next_sample(sample_rate) * self.osc_env.process();
            let noise = self.noise.next_sample() * self.noise_env.process();
            *sample = 0.3 * tone + 0.7 * noise; // mix and scale down
        }
    }

    fn note_on(&mut self, note: u8, _velocity: u8) {
        let freq = utils::note::midi_to_frequency(note);
        self.osc.set_frequency(freq);
        self.osc_env.gate(true);
        self.noise_env.gate(true);
    }

    fn note_off(&mut self) {
        self.osc_env.gate(false);
        self.noise_env.gate(false);
    }

    fn is_active(&self) -> bool {
        self.osc_env.is_active() || self.noise_env.is_active()
    }

    fn try_handle_command(&mut self, _command: &crate::synth_infra::SynthCommand) -> bool { false }
}