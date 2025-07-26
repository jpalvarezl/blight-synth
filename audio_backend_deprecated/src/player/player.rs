use sequencer::models::Song;

use crate::devices::audio_engine::AudioEngine;

pub struct Player(pub(crate) PlayerInner);

pub(crate) struct PlayerInner {
    song: Song,
    audio_engine: AudioEngine,
}

impl Player {
    pub fn new(song: Song, audio_engine: AudioEngine) -> Self {
        Player(PlayerInner { song, audio_engine })
    }

    pub fn play(&self) -> Result<(), anyhow::Error> {
        // Logic to play the song using the audio engine
        // This is a placeholder for actual playback logic
        Ok(())
    }

    pub fn stop(&self) -> Result<(), anyhow::Error> {
        // Logic to stop playback
        Ok(())
    }
}
