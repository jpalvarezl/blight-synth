use audio_backend::{BlightAudio, TrackerCommand, InstrumentDefinition};
use sequencer::models::{Song, MAX_TRACKS};
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
                    println!("Audio system initialized successfully");
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
            println!("Playing song: {}", song.name);
        }
    }
    
    pub fn stop_song(&mut self) {
        if let Some(audio) = &mut self.audio {
            audio.send_command(TrackerCommand::StopSong);
            self.is_playing = false;
            println!("Stopped song");
        }
    }
    
    pub fn toggle_playback(&mut self, song: &Song) {
        if self.is_playing {
            self.stop_song();
        } else {
            self.play_song(song);
        }
    }
}
