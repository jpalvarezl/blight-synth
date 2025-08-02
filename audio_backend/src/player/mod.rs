use std::sync::Arc;

use sequencer::{models::{Song, MAX_TRACKS}, timing::TimingState};

pub struct Player {
    timing_state: TimingState,
    song: Option<Arc<Song>>,
    position: PlayerPosition,
}

pub struct TrackPosition {
    /// The current step within the active Chain for this track (e.g., 0-15).
    pub chain_step: u8,
    /// The current step/row within the active Phrase for this track (e.g., 0-15).
    pub phrase_step: u8,
}

/// Holds the complete playback position state for the entire song.
pub struct PlayerPosition {
    /// The current step/row in the master `song.arrangement` grid.
    pub song_step: usize,
    /// The current tick within the current phrase_step (from 0 to TPL-1).
    pub tick_counter: u8,
    /// The individual positions for each of the MAX_TRACKS.
    pub track_positions: [TrackPosition; MAX_TRACKS],
}

impl Default for TrackPosition {
    fn default() -> Self {
        Self {
            chain_step: 0,
            phrase_step: 0,
        }
    }
}

impl Default for PlayerPosition {
    fn default() -> Self {
        Self {
            song_step: 0,
            tick_counter: 0,
            track_positions: Default::default(),
        }
    }
}

impl Player {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            timing_state: TimingState::new(sample_rate as f64),
            song: None,
            position: PlayerPosition::default(),
        }
    }

    pub fn set_song(&mut self, song: Arc<Song>) {
        self.song = Some(song);
        self.timing_state.set_bpm(self.song.as_ref().unwrap().initial_bpm as f64);
        // self.timing_state.set_tpl(self.song.as_ref().unwrap().initial_speed as u32);
        // Reset position to start of song.
        self.position = PlayerPosition::default();
    }

    pub fn advance(&mut self, num_samples: usize) -> u32{
        self.timing_state.advance(num_samples)
    }
}