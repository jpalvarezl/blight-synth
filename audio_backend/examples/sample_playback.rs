#![cfg(not(feature = "tracker"))]
use std::path::PathBuf;
use std::thread;

use audio_backend::BlightAudio;
use audio_backend::InstrumentDefinition;
use audio_backend::Result;
use audio_backend::{Command, SynthCmd};

fn main() -> Result<()> {
    match &mut BlightAudio::new() {
        Ok(audio) => {
            // Successfully created BlightAudio instance
            let resource_manager = audio.get_resource_manager();
            let sample_id = 1; // Example SampleId
            let path = PathBuf::from(
                "audio_backend/examples/assets/sample 2 chan - 24 bit - 44.1 khz.wav",
            );
            resource_manager.add_sample_from_file(sample_id, path)?;
            let instrument = InstrumentDefinition::SamplePlayer(
                resource_manager.get_sample_unsafe(sample_id).clone(),
            );

            audio.send_command(Command::Synth(SynthCmd::PlayNote {
                voice: audio
                    .get_voice_factory()
                    .create_voice_with_envelope(0, instrument, 0.0, 0.0, 0.0, 1.0, 0.0),
                note: 60,
                velocity: 127,
            }));

            thread::sleep(std::time::Duration::from_millis(2000));
        }
        Err(e) => {
            eprintln!("Error initializing audio: {}", e);
        }
    }
    Ok(())
}
