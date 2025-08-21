#![cfg(feature = "tracker")]

use std::sync::Arc;

use sequencer::models::Song;

use crate::{id::InstrumentId, InstrumentTrait, StereoEffect};

pub enum TrackerCommand {
    AddTrackInstrument {
        instrument: Box<dyn InstrumentTrait>,
    },
    AddEffectToInstrument {
        instrument_id: InstrumentId,
        effect: Box<dyn StereoEffect>,
    },
    PlaySong {
        song: Arc<Song>,
    },
    PlayLastSong,
    StopSong,
}
