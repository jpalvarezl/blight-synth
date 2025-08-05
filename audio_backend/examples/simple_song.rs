#![cfg(feature = "tracker")]

use std::sync::Arc;

use audio_backend::BlightAudio;
use sequencer::models::Song;

pub fn main() {
    match &mut BlightAudio::new(Arc::new(load_song())) {
        Ok(audio) => {
            
        }
        Err(e) => {
            eprintln!("Failed to initialize BlightAudio: {}", e);
        }
    }
}

pub fn load_song() -> Song {
    // Load your song here, this is just a placeholder.
    Song::new("Empty song")
}