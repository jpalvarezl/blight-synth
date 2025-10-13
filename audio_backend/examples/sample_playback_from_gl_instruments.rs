use std::thread;

use audio_backend::BlightAudio;
use audio_backend::Result;

fn main() -> Result<()> {
    match &mut BlightAudio::new() {
        Ok(audio) => {
            // Successfully created BlightAudio instance
            let resource_manager = audio.get_resource_manager();
            let samples_loaded = resource_manager.load_macos_dls_samples()?;

            println!("Loaded {} samples from macOS DLS file", samples_loaded);

            let sample_data = resource_manager.get_sample_unsafe(444); // TOM__60

            let instrument_id = 0;
            audio.send_command(
                audio_backend::SequencerCmd::AddTrackInstrument {
                    instrument: audio
                        .get_instrument_factory()
                        .create_sample_player(
                            instrument_id,
                            0.0,
                            sample_data.clone()
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

            thread::sleep(std::time::Duration::from_millis(2000));
        }
        Err(e) => {
            eprintln!("Error initializing audio: {}", e);
        }
    }
    Ok(())
}
