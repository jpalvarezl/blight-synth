use std::path::PathBuf;
use std::thread;

use audio_backend::BlightAudio;
use audio_backend::Result;

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
            let sample_data = resource_manager.get_sample_unsafe(sample_id);

            let instrument_id = 0;
            audio.send_command(
                audio_backend::SequencerCmd::AddTrackInstrument {
                    instrument: audio.get_instrument_factory().create_sample_player(
                        instrument_id,
                        0.0,
                        sample_data.clone(),
                    ),
                }
                .into(),
            );
            audio.send_command(
                audio_backend::InstrumentCmd::NoteOn {
                    instrument_id,
                    note: 60,
                    velocity: 127,
                }
                .into(),
            );
            audio.send_command(audio_backend::TransportCmd::PlayLastSong.into());

            thread::sleep(std::time::Duration::from_millis(10000));
        }
        Err(e) => {
            eprintln!("Error initializing audio: {}", e);
        }
    }
    Ok(())
}
