use std::collections::HashMap;

use sequencer::models::MAX_TRACKS;

use crate::{id::InstrumentId, player::commands::PlayerCommand, VoiceFactory, VoiceTrait};

/// Specific implementation of a synthesizer for `tracker` mode.
/// Instruments are pre-allocated to prevent RT contract violations.
pub struct Synthesizer {
    pub track_instruments: HashMap<InstrumentId, Box<dyn VoiceTrait>>,
}

impl Synthesizer {
    pub fn new(sample_rate: f32) -> Self {
        let voice_factory = VoiceFactory::new(sample_rate);

        let mut track_instrument = HashMap::with_capacity(MAX_TRACKS);
        track_instrument.insert(
            0,
            voice_factory.create_voice(1, crate::InstrumentDefinition::Oscillator, 0.0),
        );

        track_instrument.insert(
            1,
            voice_factory.create_voice(1, crate::InstrumentDefinition::Oscillator, 0.0),
        );

        // Add more default instruments as needed.
        Self {
            track_instruments: track_instrument,
        }
    }

    pub fn handle_command(&mut self, command: PlayerCommand) {
        match command {
            PlayerCommand::PlayNote {
                instrument_id,
                note,
                velocity,
            } => {
                if let Some(instrument) = self.track_instruments.get_mut(&instrument_id) {
                    instrument.note_on(note, velocity);
                }
            }
            PlayerCommand::StopNote { instrument_id } => {
                if let Some(instrument) = self.track_instruments.get_mut(&instrument_id) {
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

    fn add_instrument(&mut self, id: InstrumentId, instrument: Box<dyn VoiceTrait>) {
        self.track_instruments.insert(id, instrument);
    }

    fn get_instrument(&self, id: &InstrumentId) -> Option<&Box<dyn VoiceTrait>> {
        self.track_instruments.get(id)
    }

    fn get_instrument_unsafe_mut(&mut self, id: &InstrumentId) -> &mut Box<dyn VoiceTrait> {
        self.track_instruments
            .get_mut(id)
            .expect("Instrument should exist")
    }
}
