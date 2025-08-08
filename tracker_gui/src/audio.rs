use audio_backend::{BlightAudio, InstrumentDefinition, TrackerCommand};
use sequencer::models::{MAX_TRACKS, Song};
use std::sync::Arc;

pub struct AudioManager {
    pub audio: Option<BlightAudio>,
    pub is_playing: bool,
}

impl Default for AudioManager {
    fn default() -> Self {
        Self {
            audio: None,
            is_playing: false,
        }
    }
}

impl AudioManager {
    pub fn init_audio(&mut self, song: &Song) {
        if self.audio.is_none() {
            match BlightAudio::new(Arc::new(song.clone())) {
                Ok(mut audio) => {
                    // Add some default instruments to each track
                    for track_id in 0..MAX_TRACKS {
                        audio.send_command(TrackerCommand::AddTrackInstrument {
                            track_id,
                            instrument: audio.get_voice_factory().create_voice(
                                track_id as u64,
                                InstrumentDefinition::Oscillator,
                                0.0,
                            ),
                        });
                    }
                    self.audio = Some(audio);
                    log::info!("Audio system initialized successfully");
                }
                Err(e) => {
                    eprintln!("Failed to initialize audio system: {}", e);
                }
            }
        }
    }

    pub fn play_song(&mut self, song: &Song) {
        self.init_audio(song);

        if let Some(audio) = &mut self.audio {
            audio.send_command(TrackerCommand::PlaySong {
                song: Arc::new(song.clone()),
            });
            self.is_playing = true;
            log::info!("Playing song: {}", song.name);
        }
    }

    pub fn stop_song(&mut self) {
        if let Some(audio) = &mut self.audio {
            audio.send_command(TrackerCommand::StopSong);
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

    pub fn set_track_instrument(
        &mut self,
        track_id: usize,
        instrument_def: InstrumentDefinition,
    ) -> Result<(), String> {
        if let Some(audio) = &mut self.audio {
            let instrument = audio.get_voice_factory().create_voice(
                track_id as u64,
                instrument_def,
                0.0, // Center pan
            );

            audio.send_command(TrackerCommand::AddTrackInstrument {
                track_id,
                instrument,
            });

            log::info!("Updated track {} instrument", track_id);
            Ok(())
        } else {
            Err("Audio system not initialized. Please initialize audio first using the Playback menu.".to_string())
        }
    }
}
