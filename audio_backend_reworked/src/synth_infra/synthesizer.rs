use crate::{synth_infra::voice::VoiceManager, Command};

pub struct Synthesizer {
    pub sample_rate: f32,
    voice_manager: VoiceManager,
    // Add more fields as needed for synthesizer state.
}

impl Synthesizer {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            voice_manager: VoiceManager::new(),
        }
    }

    pub fn handle_command(&mut self, command: Command) {
        match command {
            Command::PlayNote { voice, note, velocity } => {
                // Create a new voice and add it to the manager.
                let voice_id = voice.id(); // Assuming Voice has an id field.
                self.voice_manager.add_voice(voice);
                self.voice_manager.find_voice_mut(voice_id)
                    .expect("Voice should be added")
                    .note_on(note, velocity);
            }
            Command::StopNote { voice_id } => {
                // Find the voice by ID and stop it.
                self.voice_manager.find_voice_mut(voice_id).map(|voice| {
                    voice.note_off();
                });
            }
            _ => {
                // TODO: support the rest
            }
        }
    }

    pub fn process(&mut self, left_buf: &mut [f32], right_buf: &mut [f32]) {
        // Process audio and fill the output buffer.
        // for (left, right) in left_buf.iter_mut().zip(right_buf.iter_mut()) {
        //     self.voice_manager.process(left, right);
        // }
        self.voice_manager.process(left_buf, right_buf);
    }
}
