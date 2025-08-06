#![cfg(feature = "tracker")]

use std::sync::Arc;

use sequencer::models::Song;

pub enum TrackerCommand {
    PlaySong { song: Arc<Song> },
    StopSong,
}
