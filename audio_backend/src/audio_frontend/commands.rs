#![cfg(feature = "tracker")]

use std::sync::Arc;

use sequencer::models::Song;

use crate::VoiceTrait;

pub enum TrackerCommand {
    AddTrackInstrument {
        track_id: usize,
        instrument: Box<dyn VoiceTrait>,
    },
    PlaySong {
        song: Arc<Song>,
    },
    StopSong,
}
