use crate::{synth_infra::voice::VoiceManager, Command, StereoEffectChain, Voice};

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
                    if let Some(oscillator_node) = voice
                        .as_any_mut()
                        .downcast_mut::<Voice<crate::OscillatorNode>>()
                    {
                        oscillator_node.node.set_waveform(waveform);
                    }
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
