#![cfg(feature = "tracker")]

use std::sync::Arc;

use sequencer::models::Song;

use crate::{id::InstrumentId, MonoEffect, VoiceTrait};

pub enum TrackerCommand {
    AddTrackInstrument {
        instrument_id: InstrumentId,
        instrument: Box<dyn VoiceTrait>,
    },
    AddEffectToTrack {
        instrument_id: InstrumentId,
        effect: Box<dyn MonoEffect>,
    },
    PlaySong {
        song: Arc<Song>,
    },
    PlayLastSong,
    StopSong,
}
