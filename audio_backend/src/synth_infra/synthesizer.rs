use std::collections::HashMap;

use crate::{
    id::InstrumentId, synth_infra::voice::VoiceManager, Command, InstrumentTrait,
    StereoEffectChain, SynthCommand,
};

pub struct Synthesizer {
    // pub sample_rate: f32,
    voice_manager: VoiceManager,
    instruments: HashMap<InstrumentId, Box<dyn InstrumentTrait>>,
    // Add more fields as needed for synthesizer state.
    master_effect_chain: StereoEffectChain,
}

impl Synthesizer {
    pub fn new() -> Self {
        Self {
            instruments: HashMap::with_capacity(64), // pre-alloc 64 instrument slots
            voice_manager: VoiceManager::new(),
            master_effect_chain: StereoEffectChain::new(10), // Pre-allocate space for 10 effects.
        }
    }

    pub fn handle_command(&mut self, command: Command) {
        match command {
            Command::PlayNote {
                voice,
                note,
                velocity,
            } => {
                // Create a new voice and add it to the manager.
                let voice_id = voice.id(); // Assuming Voice has an id field.
                self.voice_manager.add_voice(voice);
                self.voice_manager
                    .find_voice_mut(voice_id)
                    .expect("Voice should be added")
                    .note_on(note, velocity);
            }
            Command::StopNote { voice_id } => {
                // Find the voice by ID and stop it.
                self.voice_manager.find_voice_mut(voice_id).map(|voice| {
                    voice.note_off();
                });
            }
            Command::ChangeWaveform { voice_id, waveform } => {
                // Find the voice and change its waveform.
                if let Some(voice) = self.voice_manager.find_voice_mut(voice_id) {
                    voice.try_handle_command(&SynthCommand::SetWaveform(waveform));
                }
            }
            Command::AddMasterEffect { effect } => {
                // Add a new effect to the master effect chain.
                self.master_effect_chain.add_effect(effect);
            }
            Command::AddVoiceEffect { voice_id, effect } => {
                // Find the voice by ID and add the effect.
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
