use audio_backend::{BlightAudio, InstrumentDefinition, TrackerCommand};
use sequencer::models::{InstrumentData, Song};
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
                    self.hydrate_from_song(&mut audio, song);
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

    fn map_instrument_definition(data: &InstrumentData) -> Option<InstrumentDefinition> {
        match data {
            InstrumentData::SimpleOscillator(_) => Some(InstrumentDefinition::Oscillator),
            // Extend mapping as new instrument types become supported in the backend
            _ => None,
        }
    }

    pub fn hydrate_from_song(&self, audio: &mut BlightAudio, song: &Song) {
        for inst in &song.instrument_bank {
            if let Some(def) = Self::map_instrument_definition(&inst.data) {
                let iid = audio_backend::id::InstrumentId::from(inst.id as u32);
                let instrument = audio
                    .get_instrument_factory()
                    .create_simple_oscillator(iid, def, 0.0);
                audio.send_command(TrackerCommand::AddTrackInstrument {
                    instrument_id: iid,
                    instrument,
                });
            } else {
                log::warn!(
                    "Skipping unsupported instrument id={} when hydrating audio",
                    inst.id
                );
            }
        }
    }
}
