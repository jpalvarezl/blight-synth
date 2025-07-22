use cpal::traits::StreamTrait;
use devices::audio_engine::AudioEngine;
use log::info;

use crate::devices::streams::create_stream_with_commands;
// use devices::AudioEngineCommand;

pub mod devices;
mod player;
pub mod synths; // Add the new module

pub use player::*;

#[cfg(test)]
mod test;

type Result<T> = anyhow::Result<T, anyhow::Error>;

pub fn start_lockfree_audio_engine(max_polyphony: usize) -> Result<AudioEngine> {
    env_logger::init();
    info!("Starting lock-free audio engine");
    let audio_engine = AudioEngine::new()?;
    let command_queue = audio_engine.command_queue();

    // Launch audio thread
    std::thread::spawn(move || {
        let stream = create_stream_with_commands(max_polyphony, command_queue)
            .expect("Failed to create audio stream");
        stream.play().expect("Failed to play audio stream");
        loop {}
    });

    Ok(audio_engine)
}
