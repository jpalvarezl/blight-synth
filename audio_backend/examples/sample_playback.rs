use std::path::PathBuf;

use audio_backend::BlightAudio;
use audio_backend::Result;

fn main() -> Result<()> {
    match &mut BlightAudio::new() {
        Ok(audio) => {
            // Successfully created BlightAudio instance
            let resource_manager = audio.get_resource_manager();
            let sample_id = 1; // Example SampleId
            let path =
                PathBuf::from("audio_backend/examples/sample 2 chan - 24 bit - 44.1 khz.wav");
            resource_manager.add_sample_from_file(sample_id, path)?;
        }
        Err(e) => {
            eprintln!("Error initializing audio: {}", e);
        }
    }
    Ok(())
}
