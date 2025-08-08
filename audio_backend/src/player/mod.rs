#![cfg(feature = "tracker")]

pub(super) mod commands;
mod synthesizer;

use std::sync::Arc;

use log::debug;
use sequencer::{
    models::{NoteSentinelValues, Song, DEFAULT_CHAIN_LENGTH, DEFAULT_PHRASE_LENGTH, MAX_TRACKS},
    timing::TimingState,
};

use crate::TrackerCommand;

/// Holds the playback position for a single track.
#[derive(Debug, Clone, Copy)]
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

impl Default for PlayerPosition {
    fn default() -> Self {
        Self {
            song_step: 0,
            tick_counter: 0,
            track_positions: [TrackPosition::default(); MAX_TRACKS],
        }
    }
}

impl Default for TrackPosition {
    fn default() -> Self {
        Self {
            chain_step: 0,
            phrase_step: 0,
        }
    }
}

/// The Player is the main "conductor" of the song. It reads the song data,
/// keeps track of time, and sends commands to the AudioEngine.
pub struct Player {
    song: Arc<Song>,
    timing: TimingState,
    position: PlayerPosition,
    is_playing: bool,
    pub synthesizer: synthesizer::Synthesizer,
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
            synthesizer: synthesizer::Synthesizer::new(),
        }
    }

    pub fn play(&mut self) {
        self.is_playing = true;
    }

    pub fn stop(&mut self) {
        self.is_playing = false;
        self.position = PlayerPosition::default(); // Reset position
        self.synthesizer.stop_all_notes(); // Stop all notes when stopping playback
    }

    pub fn handle_command(&mut self, command: TrackerCommand) {
        match command {
            TrackerCommand::PlaySong { song } => {
                self.song = song;
                self.position = PlayerPosition::default();
                self.play();
            }
            TrackerCommand::StopSong => {
                self.stop();
            }
            TrackerCommand::AddTrackInstrument {
                track_id,
                instrument,
            } => {
                self.synthesizer.add_track_instrument(track_id, instrument);
            }
            TrackerCommand::AddEffectToTrack { track_id, effect } => {
                self.synthesizer.add_effect_to_track(track_id, effect);
            }
        }
    }

    /// This is the main function to be called from your audio callback.
    /// It processes a block of samples, advances the sequencer state,
    /// and sends commands to the audio engine.
    pub fn process(&mut self, buffer_len_samples: usize) {
        if !self.is_playing {
            return;
        }

        let ticks_to_process = self.timing.advance(buffer_len_samples);

        for _ in 0..ticks_to_process {
            self.advance_tick();
        }
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
                // Loop the song if it reaches the end of the arrangement
                if self.position.song_step >= self.song.arrangement.len() {
                    self.position.song_step = 0; // Or a specific restart position
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
                        // We have an event! Process it.
                        // For now, we only care about NoteOn events.
                        if event.note != NoteSentinelValues::NoNote as u8
                            && event.note != NoteSentinelValues::NoteOff as u8
                        {
                            debug!(
                                "Playing note: {} on track: {} with velocity: {}",
                                event.note, track_index, event.volume
                            );
                            // TODO: A real implementation would also need to know which instrument to use.
                            // This is often implicit (the last one used on the track) or specified in the event.
                            // For now, we'll assume instrument 1.
                            // let instrument_id = 1;
                            self.synthesizer
                                .handle_command(commands::PlayerCommand::PlayNote {
                                    track_id: track_index,
                                    note: event.note,
                                    velocity: event.volume,
                                });
                        } else if event.note == NoteSentinelValues::NoteOff as u8 {
                            // Handle NoteOff events
                            self.synthesizer
                                .handle_command(commands::PlayerCommand::StopNote {
                                    track_id: track_index,
                                });
                        }
                        // TODO: Handle NoteOff, effects, etc.
                    }
                }
            }
        }
    }
}
