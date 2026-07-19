use crate::id::EnvelopeId;
use crate::synth_infra::{Envelope, PitchEnvelope};
use crate::{instruments::OscillatorNode, SynthCmd, SynthNode};
use utils::note::midi_to_frequency;

pub struct KickDrumVoice {
    osc: OscillatorNode, // sine or triangle wave
    amp_env: Envelope,
    pitch_env: PitchEnvelope,
}

// Provide a convenience constructor for the concrete Envelope+PitchEnvelope combo:
impl KickDrumVoice {
    pub fn new(sample_rate: f32) -> Self {
        let mut env = crate::Envelope::new(sample_rate);
        env.set_parameters(0.01, 0.1, 0.0, 0.1);
        let pitch_env = crate::PitchEnvelope::new(50.0, env.clone());
        let osc = OscillatorNode::new();
        KickDrumVoice {
            osc,
            amp_env: env,
            pitch_env,
        }
    }
}

// // Generic impl for audio path — no vtable calls
impl SynthNode for KickDrumVoice {
    fn process(&mut self, mono_buf: &mut [f32], sample_rate: f32) {
        for sample in mono_buf.iter_mut() {
            if !self.is_active() {
                *sample = 0.0;
                continue;
            }
            let freq = self.pitch_env.next_freq();
            self.osc.set_frequency(freq);
            *sample = self.osc.next_sample(sample_rate) * self.amp_env.process();
        }
    }

    fn note_on(&mut self, note: u8, _velocity: u8) {
        let start = midi_to_frequency(note);
        self.pitch_env.note_on(start);
        self.amp_env.gate(true);
    }

    fn note_off(&mut self) {
        self.amp_env.gate(false);
        self.pitch_env.note_off();
    }

    fn is_active(&self) -> bool {
        self.osc.is_active() || self.amp_env.is_active() || self.pitch_env.is_active()
    }

    fn try_handle_command(&mut self, command: &crate::commands::SynthCmd) -> bool {
        match command {
            SynthCmd::EnvelopeCommand {
                envelope_id,
                command,
            } => match envelope_id {
                Some(0) => self.amp_env.handle_command(command),
                Some(1) => self.pitch_env.handle_command(command),
                Some(_) => false,
                None => {
                    crate::rt_debug_log!("No Envelope ID defined; command ignored");
                    false
                }
            },
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KickDrumEnvelope {
    Oscillator = 0,
    Noise,
}

impl From<KickDrumEnvelope> for EnvelopeId {
    fn from(env: KickDrumEnvelope) -> Self {
        env as EnvelopeId
    }
}
