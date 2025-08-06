use std::collections::HashMap;

use sequencer::models::MAX_TRACKS;

use crate::{player::commands::PlayerCommand, VoiceTrait};

/// Specific implementation of a synthesizer for `tracker` mode.
/// Instruments are pre-allocated to prevent RT contract violations.
pub struct Synthesizer {
    pub track_instruments: HashMap<usize, Box<dyn VoiceTrait>>,
}

impl Synthesizer {
    pub fn new() -> Self {
        Self {
            track_instruments: HashMap::with_capacity(MAX_TRACKS),
        }
    }

    pub fn handle_command(&mut self, command: PlayerCommand) {
        match command {
            PlayerCommand::PlayNote {
                track_id,
                note,
                velocity,
            } => {
                if let Some(instrument) = self.track_instruments.get_mut(&track_id) {
                    instrument.note_on(note, velocity);
                }
            }
            PlayerCommand::StopNote { track_id } => {
                if let Some(instrument) = self.track_instruments.get_mut(&track_id) {
                    instrument.note_off();
                }
            }
            _ => {
                // Handle other commands as needed.
            }
        }
    }

    pub fn process(&mut self, left: &mut [f32], right: &mut [f32], sample_rate: f32) {
        for instrument in self.track_instruments.values_mut() {
            instrument.process(left, right, sample_rate);
        }
    }

    pub fn add_track_instrument(&mut self, track_id: usize, instrument: Box<dyn VoiceTrait>) {
        self.track_instruments.insert(track_id, instrument);
    }
}
