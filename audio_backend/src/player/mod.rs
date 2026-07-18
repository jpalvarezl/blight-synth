mod tracker_engine_adapter;

use std::sync::Arc;

use log::debug;
use sequencer::{
    models::{NoteSentinelValues, Song, DEFAULT_CHAIN_LENGTH, DEFAULT_PHRASE_LENGTH, MAX_TRACKS},
    timing::TimingState,
};

use crate::{id::InstrumentId, Command, SequencerCmd, TransportCmd};

/// Holds the playback position for a single track.
#[derive(Debug, Clone, Copy, Default)]
pub struct TrackPosition {
    /// The current step within the active Chain for this track (e.g., 0-15).
    pub chain_step: u8,
    /// The current step/row within the active Phrase for this track (e.g., 0-15).
    pub phrase_step: u8,
}

/// Holds the complete playback position state for the entire song.
#[derive(Debug, Clone, Copy)]
pub struct PlayerPosition {
    /// The current step/row in the master `song.arrangement` grid.
    pub song_step: usize,
    /// The current tick within the current phrase_step (from 0 to TPL-1).
    pub tick_counter: u32,
    /// The individual positions for each of the MAX_TRACKS.
    pub track_positions: [TrackPosition; MAX_TRACKS],
}

impl PlayerPosition {
    pub fn reset(&mut self) {
        self.song_step = 0;
        self.tick_counter = 0;
        for track_pos in self.track_positions.iter_mut() {
            track_pos.chain_step = 0;
            track_pos.phrase_step = 0;
        }
    }
}

impl Default for PlayerPosition {
    fn default() -> Self {
        Self {
            song_step: 0,
            tick_counter: 0,
            track_positions: [TrackPosition::default(); MAX_TRACKS],
        }
    }
}

/// The Player is the main "conductor" of the song. It reads the song data,
/// keeps track of time, and translates tracker state into engine operations.
pub struct Player {
    song: Arc<Song>,
    timing: TimingState,
    position: PlayerPosition,
    is_playing: bool,
    loop_enabled: bool,
    engine_adapter: tracker_engine_adapter::TrackerEngineAdapter,
}

impl Player {
    pub fn new(song: Arc<Song>, sample_rate: f64) -> Self {
        let timing = TimingState::new_with_bpm_tpl(
            sample_rate,
            song.initial_bpm as f64,   // Initial BPM
            song.initial_speed as u32, // Initial Ticks Per Line (TPL)
        );

        Self {
            song,
            timing,
            position: PlayerPosition::default(),
            is_playing: false,
            loop_enabled: false,
            engine_adapter: tracker_engine_adapter::TrackerEngineAdapter::new(),
        }
    }

    pub fn play(&mut self) {
        self.is_playing = true;
    }

    pub(crate) fn is_playing(&self) -> bool {
        self.is_playing
    }

    pub fn stop(&mut self) {
        self.is_playing = false;
        self.position = PlayerPosition::default(); // Reset position
        self.engine_adapter.stop_all_notes(); // Stop all notes when stopping playback
    }

    fn set_song(&mut self, song: Arc<Song>) {
        self.timing.set_bpm(song.initial_bpm as f64);
        self.timing.set_tpl(song.initial_speed as u32);
        self.timing.reset();
        self.position.reset();
        self.song = song;
    }

    fn load_song(&mut self, song: Arc<Song>) {
        debug!("Loading song: {}", song.name);
        self.stop();
        self.engine_adapter.clear_instruments();
        self.set_song(song);
    }

    pub fn handle_command(&mut self, command: Command) {
        match command {
            Command::Sequencer(SequencerCmd::LoadSong { song }) => {
                self.load_song(song);
            }
            Command::Sequencer(SequencerCmd::PlaySong { song }) => {
                debug!("Playing song: {}", song.name);
                self.set_song(song);
                self.play();
            }
            Command::Transport(TransportCmd::StopSong) => {
                self.stop();
            }
            Command::Transport(TransportCmd::SetLooping { enabled }) => {
                self.loop_enabled = enabled;
            }
            Command::Transport(TransportCmd::PlayLastSong) => self.play(),
            Command::Instrument(command) => {
                self.engine_adapter.handle_engine_command(command.into())
            }
            Command::Mixer(command) => self.engine_adapter.handle_engine_command(command.into()),
        }
    }

    /// This is the main function to be called from your audio callback.
    /// It processes a block of samples, advances the sequencer state,
    /// and forwards audio buffers to the engine adapter.
    pub fn process(
        &mut self,
        left: &mut [f32],
        right: &mut [f32],
        sample_rate: f32,
        buffer_len_samples: usize,
    ) {
        if !self.is_playing {
            return;
        }

        let ticks_to_process = self.timing.advance(buffer_len_samples);

        for _ in 0..ticks_to_process {
            self.advance_tick();
            if !self.is_playing {
                break;
            }
        }

        self.engine_adapter.process(left, right, sample_rate);
    }

    /// This is the heart of the sequencer. It processes a single tick,
    /// triggers notes if it's the start of a new row, and advances the playback position.
    fn advance_tick(&mut self) {
        // On the first tick of a row (tick 0), we read the pattern data and trigger notes.
        // On subsequent ticks, we would process effects like vibrato or volume slides.
        if self.position.tick_counter == 0 {
            self.trigger_notes_for_current_row();
        } else {
            // TODO: Process "in-between" tick effects here (e.g., vibrato, slides)
        }

        // --- Advance the playback position ---
        self.position.tick_counter += 1;

        if self.position.tick_counter >= self.timing.tpl() {
            // A row has finished, reset tick counter and advance to the next phrase step.
            self.position.tick_counter = 0;

            let mut song_step_needs_advancing = false;
            for track_pos in self.position.track_positions.iter_mut() {
                track_pos.phrase_step += 1;

                if track_pos.phrase_step >= DEFAULT_PHRASE_LENGTH as u8 {
                    track_pos.phrase_step = 0;
                    track_pos.chain_step += 1;

                    if track_pos.chain_step >= DEFAULT_CHAIN_LENGTH as u8 {
                        track_pos.chain_step = 0;
                        // This track has finished its chain. The song step should advance.
                        // We only want to do this once per row, so we use a flag.
                        song_step_needs_advancing = true;
                    }
                }
            }

            if song_step_needs_advancing {
                self.position.song_step += 1;
                if self.position.song_step >= self.song.arrangement.len() {
                    // End of song reached
                    if self.loop_enabled {
                        // self.engine_adapter.stop_all_notes();
                        self.position.reset();
                        self.timing.reset();
                        debug!("Looping back to start of song");
                    } else {
                        // Stop playback and reset state
                        self.stop();
                        debug!("Reached end of song, stopping playback");
                    }
                }
            }
        }
    }

    /// Reads the current row for all tracks and sends NoteOn commands.
    fn trigger_notes_for_current_row(&mut self) {
        // Get the current row from the song's master arrangement
        let Some(current_song_row) = self.song.arrangement.get(self.position.song_step) else {
            return; // Song position is out of bounds, do nothing.
        };

        // Iterate through each track
        for track_index in 0..MAX_TRACKS {
            let track_pos = &self.position.track_positions[track_index];
            let chain_index = current_song_row.chain_indices[track_index];

            debug!(
                "Processing track {}: chain_index={}, chain_step={}, phrase_step={}",
                track_index, chain_index, track_pos.chain_step, track_pos.phrase_step
            );

            if chain_index == sequencer::models::EMPTY_CHAIN_SLOT {
                continue;
            }

            // Look up the chain, then the phrase, then the event
            if let Some(chain) = self.song.chain_bank.get(chain_index) {
                let phrase_index = chain.phrase_indices[track_pos.chain_step as usize];
                if phrase_index == sequencer::models::EMPTY_PHRASE_SLOT {
                    continue;
                }

                if let Some(phrase) = self.song.phrase_bank.get(phrase_index) {
                    if let Some(event) = phrase.events.get(track_pos.phrase_step as usize) {
                        // Fetch instrument_id if there is a track specified one
                        let instrument_id = self.engine_adapter.cache_instrument_id_for_track(
                            track_index,
                            event.instrument_id as InstrumentId,
                        );

                        // For NoNote at some point, we should still process effects in the event.
                        if event.note != NoteSentinelValues::NoNote as u8
                            && event.note != NoteSentinelValues::NoteOff as u8
                        {
                            debug!(
                                "Playing note: {} on track: {} with velocity: {} and instrument_id: {}",
                                event.note, track_index, event.volume, instrument_id
                            );
                            // TODO: A real implementation would also need to know which instrument to use.
                            // This is often implicit (the last one used on the track) or specified in the event.
                            // For now, we'll assume instrument 1.
                            // let instrument_id = 1;
                            // Default missing volume (0 from UI meaning blank) to full velocity (255)
                            let velocity = if event.volume == 0 { 255 } else { event.volume };
                            self.engine_adapter
                                .note_on(instrument_id, event.note, velocity);
                        } else if event.note == NoteSentinelValues::NoteOff as u8 {
                            // Handle NoteOff events
                            self.engine_adapter.note_off(instrument_id);
                        }
                        // TODO: effects, etc.
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stops_processing_remaining_ticks_after_reaching_song_end() {
        let mut song = Song::new("single arrangement row");
        song.initial_bpm = 120;
        song.initial_speed = 1;
        let mut player = Player::new(Arc::new(song), 48_000.0);
        player.play();
        let mut left = [0.0];
        let mut right = [0.0];

        // At 120 BPM one tick is 1,000 samples. Three hundred ticks are
        // enough to finish the fixed 16x16 tracker chain in one process call,
        // leaving additional ticks that must not restart the reset position.
        player.process(&mut left, &mut right, 48_000.0, 300_000);

        assert!(!player.is_playing());
        assert_eq!(player.position.song_step, 0);
        assert_eq!(player.position.tick_counter, 0);
        assert!(player
            .position
            .track_positions
            .iter()
            .all(|position| position.chain_step == 0 && position.phrase_step == 0));
    }
}
