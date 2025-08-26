use crate::{synth_infra::voice::VoiceManager, Command, StereoEffectChain, SynthCommand, SynthCmd, MixerCmd};

pub struct Synthesizer {
    // pub sample_rate: f32,
    voice_manager: VoiceManager,
    // Add more fields as needed for synthesizer state.
    master_effect_chain: StereoEffectChain,
}

impl Synthesizer {
    pub fn new() -> Self {
        Self {
            voice_manager: VoiceManager::new(),
            master_effect_chain: StereoEffectChain::new(10), // Pre-allocate space for 10 effects.
        }
    }

    pub fn handle_command(&mut self, command: Command) {
        match command {
            Command::Synth(SynthCmd::PlayNote { voice, note, velocity }) => {
                let voice_id = voice.id();
                self.voice_manager.add_voice(voice);
                self.voice_manager
                    .find_voice_mut(voice_id)
                    .expect("Voice should be added")
                    .note_on(note, velocity);
            }
            Command::Synth(SynthCmd::StopNote { voice_id }) => {
                if let Some(voice) = self.voice_manager.find_voice_mut(voice_id) {
                    voice.note_off();
                }
            }
            Command::Synth(SynthCmd::ChangeWaveform { voice_id, waveform }) => {
                if let Some(voice) = self.voice_manager.find_voice_mut(voice_id) {
                    voice.try_handle_command(&SynthCommand::SetWaveform(waveform));
                }
            }
            Command::Mixer(MixerCmd::AddMasterEffect { effect }) => {
                self.master_effect_chain.add_effect(effect);
            }
            Command::Synth(SynthCmd::AddVoiceEffect { voice_id, effect }) => {
                if let Some(voice) = self.voice_manager.find_voice_mut(voice_id) {
                    voice.add_effect(effect);
                }
            }
            _ => {
                // TODO: support the rest
            }
        }
    }

    pub fn process(&mut self, left_buf: &mut [f32], right_buf: &mut [f32], sample_rate: f32) {
        // 1. Get audio from all active voices.
        self.voice_manager.process(left_buf, right_buf, sample_rate);

        // 2. Process the master effect chain.
        self.master_effect_chain
            .process(left_buf, right_buf, sample_rate);
    }
}
