use std::collections::HashMap;

use engine::{Engine, EngineCommand};
use sequencer::models::{MAX_TRACKS, NO_INSTRUMENT};

use crate::id::InstrumentId;

/// Tracker-specific adapter around the host-independent render engine.
///
/// Track-to-last-instrument state remains here because it belongs to tracker
/// event interpretation, not to generic sound rendering.
pub struct TrackerEngineAdapter {
    engine: Engine,
    track_last_instrument: HashMap<usize, InstrumentId>,
}

impl TrackerEngineAdapter {
    pub fn new() -> Self {
        Self {
            engine: Engine::new(),
            track_last_instrument: HashMap::with_capacity(MAX_TRACKS),
        }
    }

    pub fn note_on(&mut self, instrument_id: InstrumentId, note: u8, velocity: u8) {
        dsp::rt_debug_log!("Playing note: {} on instrument: {}", note, instrument_id);
        self.engine.note_on(instrument_id, note, velocity);
    }

    pub fn note_off(&mut self, instrument_id: InstrumentId) {
        self.engine.note_off(instrument_id);
    }

    pub fn process(&mut self, left: &mut [f32], right: &mut [f32], sample_rate: f32) {
        self.engine.process(left, right, sample_rate);
    }

    pub fn clear_instruments(&mut self) {
        self.engine.clear_instruments();
        self.track_last_instrument.clear();
    }

    pub fn stop_all_notes(&mut self) {
        self.engine.stop_all_notes();
    }

    pub fn handle_engine_command(&mut self, command: EngineCommand) {
        self.engine.handle_command(command);
    }

    /// Determine if there is an instrument_id cached for the track if not specified in the event.
    pub fn cache_instrument_id_for_track(
        &mut self,
        track_index: usize,
        instrument_id: InstrumentId,
    ) -> InstrumentId {
        if instrument_id == NO_INSTRUMENT as InstrumentId {
            self.get_last_instrument_for_track(track_index)
        } else {
            self.set_last_instrument_for_track(track_index, instrument_id);
            instrument_id
        }
    }

    fn get_last_instrument_for_track(&self, track_index: usize) -> InstrumentId {
        self.track_last_instrument
            .get(&track_index)
            .copied()
            .unwrap_or(NO_INSTRUMENT as InstrumentId)
    }

    fn set_last_instrument_for_track(&mut self, track_index: usize, instrument_id: InstrumentId) {
        match self
            .track_last_instrument
            .insert(track_index, instrument_id)
        {
            Some(previous_id) => {
                dsp::rt_debug_log!(
                    "Track {}: Updated last instrument from {} to {}",
                    track_index,
                    previous_id,
                    instrument_id
                );
                self.engine.note_off(previous_id);
            }
            None => dsp::rt_debug_log!(
                "Track {}: Set last instrument to {}",
                track_index,
                instrument_id
            ),
        }
    }
}
