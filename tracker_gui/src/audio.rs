use audio_backend::{BlightAudio, InstrumentDefinition, TrackerCommand, id::InstrumentId};
use sequencer::models::Song;
use std::sync::Arc;

use crate::ui_components::AvailableInstrument;

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
                    self.add_all_default_instruments(&mut audio);
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

    // This method shouldn't be used. We should initialize instruments from the stored "instrument_bank" JSON key.
    pub fn add_all_default_instruments(&self, audio: &mut BlightAudio) {
        // Add default instruments to the synthesizer
        AvailableInstrument::all().iter().for_each(|instrument| {
            if let Some(instrument_def) = instrument.to_instrument_definition() {
                audio.send_command(TrackerCommand::AddTrackInstrument {
                    instrument_id: instrument.get_instrument_id(),
                    instrument: audio.get_voice_factory().create_voice(
                        instrument.get_instrument_id(),
                        instrument_def,
                        0.0, // Center pan
                    ),
                });
            }
        });
    }

    pub fn set_track_instrument(
        &mut self,
        instrument_id: InstrumentId,
        instrument_def: InstrumentDefinition,
    ) -> Result<(), String> {
        if let Some(audio) = &mut self.audio {
            let instrument = audio.get_voice_factory().create_voice(
                instrument_id,
                instrument_def,
                0.0, // Center pan
            );
            // Adding reverb just for test
            audio.send_command(TrackerCommand::AddTrackInstrument {
                instrument_id,
                instrument,
            });

            log::info!("Updated track {} instrument", instrument_id);
            Ok(())
        } else {
            Err("Audio system not initialized. Please initialize audio first using the Playback menu.".to_string())
        }
    }
}
