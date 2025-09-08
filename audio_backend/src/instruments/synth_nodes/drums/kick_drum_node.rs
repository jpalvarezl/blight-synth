use utils::note::midi_to_frequency;

use crate::{Envelope, OscillatorNode, PitchEnvelope, SynthNode};

pub struct KickDrumVoice {
    osc: OscillatorNode, // sine or triangle osc
    amp_env: Envelope,
    pitch_env: PitchEnvelope,
}

impl KickDrumVoice {
    pub fn new(sample_rate: f32) -> Self {
        let mut envelope = Envelope::new(sample_rate);
        envelope.set_parameters(0.0, 0.1, 0.0, 0.1); // quick attack and decay
        let pitch_env = PitchEnvelope::new(100.0, envelope.clone());

        let osc = OscillatorNode::new();

        KickDrumVoice {
            osc,
            amp_env: envelope,
            pitch_env,
        }
    }
}

impl SynthNode for KickDrumVoice {
    fn process(&mut self, mono_buf: &mut [f32], sample_rate: f32) {
        for sample in mono_buf.iter_mut() {
            *sample = self.next_sample(sample_rate);
        }
    }

    fn note_on(&mut self, note: u8, _velocity: u8) {
        let start_freq = midi_to_frequency(note);
        self.pitch_env.note_on(start_freq);
        self.amp_env.gate(true);
    }

    fn note_off(&mut self) {
        self.amp_env.gate(false);
        self.pitch_env.note_off();
    }

    fn is_active(&self) -> bool {
        self.osc.is_active() || self.amp_env.is_active() || self.pitch_env.is_active()
    }

    fn try_handle_command(&mut self, _command: &crate::synth_infra::SynthCommand) -> bool {
        false
    }
}

impl KickDrumVoice {
    pub fn next_sample(&mut self, sample_rate: f32) -> f32 {
        if !SynthNode::is_active(self) {
            return 0.0;
        }

        let freq = self.pitch_env.next_freq();
        self.osc.set_frequency(freq);

        let s = self.osc.next_sample(sample_rate) * self.amp_env.process();

        s
    }
}
