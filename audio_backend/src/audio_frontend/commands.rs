#![cfg(feature = "tracker")]

use std::sync::Arc;

use sequencer::models::Song;

use crate::{MonoEffect, VoiceTrait};

pub enum TrackerCommand {
    AddTrackInstrument {
        track_id: usize,
        instrument: Box<dyn VoiceTrait>,
    },
    AddEffectToTrack {
        track_id: usize,
        effect: Box<dyn MonoEffect>,
    },
    PlaySong {
        song: Arc<Song>,
    },
    StopSong,
}
