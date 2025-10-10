use crate::synth_infra::{EnvelopeLike, PitchEnvLike};
use crate::{OscillatorNode, SynthNode};
use utils::note::midi_to_frequency;

pub struct KickDrumVoice<A: EnvelopeLike, P: PitchEnvLike> {
    osc: OscillatorNode, // sine or triangle wave
    amp_env: A,
    pitch_env: P,
}

// Provide a convenience constructor for the concrete Envelope+PitchEnvelope combo:
impl KickDrumVoice<crate::Envelope, crate::PitchEnvelope> {
    pub fn new(sample_rate: f32) -> Self {
        let mut env = crate::Envelope::new(sample_rate);
        env.set_parameters(0.0, 0.1, 0.0, 0.1);
        let pitch_env = crate::PitchEnvelope::new(100.0, env.clone());
        let osc = OscillatorNode::new();
        KickDrumVoice {
            osc,
            amp_env: env,
            pitch_env,
        }
    }
}

// // Generic impl for audio path — no vtable calls
impl<A: EnvelopeLike, P: PitchEnvLike> SynthNode for KickDrumVoice<A, P> {
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
            crate::commands::SynthCmd::SetEnvAttack { envelope_id, attack } => {
                match envelope_id {
                    Some(0) | None => self.amp_env.set_attack(*attack),
                    // Some(1) => self.pitch_env.set_attack(*attack), // Commented out - not useful for pitch
                    _ => return false,
                }
                true
            }
            crate::commands::SynthCmd::SetEnvDecay { envelope_id, decay } => {
                match envelope_id {
                    Some(0) | None => self.amp_env.set_decay(*decay),
                    // Some(1) => self.pitch_env.set_decay(*decay), // Commented out - not useful for pitch
                    _ => return false,
                }
                true
            }
            crate::commands::SynthCmd::SetEnvSustain { envelope_id, sustain } => {
                match envelope_id {
                    Some(0) | None => self.amp_env.set_sustain(*sustain),
                    // Some(1) => self.pitch_env.set_sustain(*sustain), // Commented out - not useful for pitch
                    _ => return false,
                }
                true
            }
            crate::commands::SynthCmd::SetEnvRelease { envelope_id, release } => {
                match envelope_id {
                    Some(0) | None => self.amp_env.set_release(*release),
                    // Some(1) => self.pitch_env.set_release(*release), // Commented out - not useful for pitch
                    _ => return false,
                }
                true
            }
            crate::commands::SynthCmd::SetPitchEnvFreqDelta { freq_delta } => {
                self.pitch_env.set_freq_delta(*freq_delta);
                true
            }
            crate::commands::SynthCmd::SetPitchEnvDecayTime { decay_time } => {
                self.pitch_env.set_decay_time(*decay_time);
                true
            }
            _ => false,
        }
    }
}
// Should have implemented this instead of SynthNode directtly. SynthNode is akin to a ugen in supercollider.
// The VoiceTrait provides handlers for envelope triggers, whereas the SynthNode is lower level.
// impl<A: EnvelopeLike, P: PitchEnvLike> VoiceTrait for KickDrumVoice<A, P> {
//     fn id(&self) -> crate::id::VoiceId {
//         0 // Kick drum is monophonic, so ID is always 0
//     }

//     fn is_active(&self) -> bool {
//         self.osc.is_active() || self.amp_env.is_active() || self.pitch_env.is_active()
//     }

//     fn try_handle_command(&mut self, command: &crate::commands::SynthCmd) -> bool {
//         let was_handled = match command {
//             crate::commands::SynthCmd::SetWaveform {
//                 voice_id: _,
//                 waveform,
//             } => {
//                 self.osc.set_waveform(*waveform);
//                 true
//             }
//             _ => false,
//         };
//         was_handled
//     }

//     fn process(&mut self, left_buf: &mut [f32], right_buf: &mut [f32], sample_rate: f32) {
//         let frame_count = left_buf.len().min(right_buf.len());

//         for i in 0..frame_count {
//             if !self.is_active() {
//                 left_buf[i] = 0.0;
//                 continue;
//             }
//             let freq = self.pitch_env.next_freq();
//             self.osc.set_frequency(freq);
//             left_buf[i] = self.osc.next_sample(sample_rate) * self.amp_env.process();
//             right_buf[i] = left_buf[i];
//         }
//     }
    
//     fn note_on(&mut self, note: u8, velocity: u8) {
//         let start = midi_to_frequency(note);
//         self.pitch_env.note_on(start);
//         self.amp_env.gate(true)
//     }
    
//     fn note_off(&mut self) {
//         self.amp_env.gate(false);
//         self.pitch_env.note_off();
//     }
    
//     fn set_pan(&mut self, pan: f32) {
//         todo!()
//     }
    
//     fn add_effect(&mut self, effect: Box<dyn crate::MonoEffect>) {
//         todo!()
//     }
    
//     fn set_effect_parameter(&mut self, effect_index: usize, param_index: u32, value: f32) {
//         todo!()
//     }
// }