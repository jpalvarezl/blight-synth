use crate::instrument_manager::backend::hydrate_instrument;
use audio_backend::{BlightAudio, Command, CommandSubmissionErrorKind, SequencerCmd, TransportCmd};
use sequencer::models::Song;
use std::sync::Arc;

// Tracker GUI reuses a single effect id until proper routing is needed.
pub const TRACKER_EFFECT_ID: audio_backend::id::EffectId = 1;

/// Coordinates every interaction between the egui frontend and the realtime
/// `audio_backend`. It owns the single `BlightAudio` instance, keeps transport
/// state in sync with the UI, and exposes helpers to (re)hydrate instruments
/// from the authored `Song` model.
#[derive(Default)]
pub struct AudioManager {
    pub audio: Option<BlightAudio>,
    pub is_playing: bool,
    pub loop_enabled: bool,
}

impl AudioManager {
    pub fn init_audio(&mut self, song: &Song) {
        if self.audio.is_none() {
            match BlightAudio::with_song(Arc::new(song.clone())) {
                Ok(mut audio) => {
                    self.hydrate_from_song(&mut audio, song);
                    // Ensure backend loop state matches UI preference
                    submit_command(
                        &mut audio,
                        TransportCmd::SetLooping {
                            enabled: self.loop_enabled,
                        }
                        .into(),
                    );
                    self.audio = Some(audio);
                    log::info!("Audio system initialized successfully");
                }
                Err(e) => {
                    log::error!("Failed to initialize audio system: {}", e);
                }
            }
        }
    }

    pub fn reset_with_song(&mut self, song: &Song) {
        match BlightAudio::with_song(Arc::new(song.clone())) {
            Ok(mut audio) => {
                self.hydrate_from_song(&mut audio, song);
                // Keep loop state in sync after reset
                submit_command(
                    &mut audio,
                    TransportCmd::SetLooping {
                        enabled: self.loop_enabled,
                    }
                    .into(),
                );
                self.audio = Some(audio);
                self.is_playing = false;
                log::info!("Audio system reset for loaded song");
            }
            Err(e) => {
                log::error!("Failed to reset audio system: {}", e);
            }
        }
    }

    pub fn play_song(&mut self, song: &Song) {
        self.init_audio(song);

        if let Some(audio) = &mut self.audio {
            let accepted = submit_command(
                audio,
                SequencerCmd::PlaySong {
                    song: Arc::new(song.clone()),
                }
                .into(),
            );
            if accepted {
                self.is_playing = true;
                log::info!("Playing song: {}", song.name);
            }
        }
    }

    pub fn stop_song(&mut self) {
        if let Some(audio) = &mut self.audio
            && submit_command(audio, TransportCmd::StopSong.into())
        {
            self.is_playing = false;
            log::info!("Stopped song");
        }
    }

    pub fn toggle_playback(&mut self, song: &Song) {
        if self.is_playing {
            self.stop_song();
        } else {
            self.play_song(song);
        }
    }

    pub fn set_looping(&mut self, enabled: bool) {
        if let Some(audio) = &mut self.audio {
            if submit_command(audio, TransportCmd::SetLooping { enabled }.into()) {
                self.loop_enabled = enabled;
            }
        } else {
            self.loop_enabled = enabled;
        }
    }

    pub fn toggle_looping(&mut self) {
        let enabled = !self.loop_enabled;
        self.set_looping(enabled);
    }

    /// Sends a command to the audio thread via `BlightAudio::send_command`.
    /// UI systems should call this instead of touching the backend directly so
    /// every update flows through the same queue.
    pub fn dispatch(&mut self, cmd: impl Into<audio_backend::Command>) {
        if let Some(audio) = &mut self.audio {
            submit_command(audio, cmd.into());
        }
    }

    /// Rebuilds the backend instruments/effects from the current `Song` data.
    /// Used when the app starts, when a song is loaded, or whenever we need to
    /// guarantee the mixer mirrors the editor state.
    pub fn hydrate_from_song(&self, audio: &mut BlightAudio, song: &Song) {
        for inst in &song.instrument_bank {
            hydrate_instrument(audio, inst.id as u8, &inst.data);
        }
    }
}

pub(crate) fn submit_command(audio: &mut BlightAudio, command: Command) -> bool {
    match audio.send_command(command) {
        Ok(()) => true,
        Err(error) => match error.kind() {
            CommandSubmissionErrorKind::Full => {
                log::debug!("audio command rejected: command queue is full");
                false
            }
            CommandSubmissionErrorKind::Disconnected => {
                log::error!("audio command rejected: audio callback is disconnected");
                false
            }
        },
    }
}
