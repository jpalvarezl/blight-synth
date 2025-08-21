use std::collections::HashMap;

use log::debug;
use sequencer::models::MAX_TRACKS;

use crate::{
    id::InstrumentId, player::commands::PlayerCommand, InstrumentTrait, MonoEffect, StereoEffect,
    VoiceTrait,
};

/// Specific implementation of a synthesizer for `tracker` mode.
/// Instruments are pre-allocated to prevent RT contract violations.
pub struct Synthesizer {
    pub instrument_bank: HashMap<InstrumentId, Box<dyn InstrumentTrait>>,
    pub track_instrument: HashMap<usize, InstrumentId>, // cache the active instrument for each track
}

impl Synthesizer {
    pub fn new() -> Self {
        Self {
            instrument_bank: HashMap::with_capacity(64),
            track_instrument: HashMap::with_capacity(MAX_TRACKS),
        }
    }

    pub fn handle_command(&mut self, command: PlayerCommand) {
        match command {
            PlayerCommand::PlayNote {
                instrument_id,
                note,
                velocity,
            } => {
                debug!("Playing note: {} on instrument: {}", note, instrument_id);
                debug!("Available instruments: {:?}", self.instrument_bank.keys());
                if let Some(instrument) = self.instrument_bank.get_mut(&instrument_id) {
                    instrument.note_on(note, velocity);
                }
            }
            PlayerCommand::StopNote { instrument_id } => {
                if let Some(instrument) = self.instrument_bank.get_mut(&instrument_id) {
                    instrument.note_off();
                }
            }
        }
    }

    pub fn process(&mut self, left: &mut [f32], right: &mut [f32], sample_rate: f32) {
        for instrument in self.instrument_bank.values_mut() {
            instrument.process(left, right, sample_rate);
        }
    }

    pub fn add_instrument(&mut self, instrument: Box<dyn InstrumentTrait>) {
        self.instrument_bank.insert(instrument.id(), instrument);
    }

    pub fn add_effect_to_instrument(
        &mut self,
        instrument_id: InstrumentId,
        effect: Box<dyn StereoEffect>,
    ) {
        if let Some(instrument) = self.instrument_bank.get_mut(&instrument_id) {
            instrument.add_effect(effect);
        }
    }

    pub fn stop_all_notes(&mut self) {
        for instrument in self.instrument_bank.values_mut() {
            instrument.note_off();
        }
    }
}
