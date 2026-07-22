#[cfg(target_os = "macos")]
use std::thread;

#[cfg(target_os = "macos")]
use audio_backend::{BlightAudio, Result};

#[cfg(target_os = "macos")]
fn main() -> Result<()> {
    match &mut BlightAudio::new() {
        Ok(audio) => {
            // Successfully created BlightAudio instance
            let resource_manager = audio.get_resource_manager();
            let samples_loaded = resource_manager.load_macos_dls_samples()?;

            println!("Loaded {} samples from macOS DLS file", samples_loaded);

            let sample_data = resource_manager.get_sample_unsafe(0); // TOM__60

            let instrument_id = 0;
            let _ = audio.send_command(
                audio_backend::InstrumentCmd::AddInstrument {
                    instrument: audio.get_instrument_factory().create_loop_sample_player(
                        instrument_id,
                        0.0,
                        sample_data.clone(),
                    ),
                }
                .into(),
            );
            let _ = audio.send_command(
                audio_backend::InstrumentCmd::NoteOn {
                    instrument_id,
                    note: 67,
                    velocity: 127,
                }
                .into(),
            );
            let _ = audio.send_command(audio_backend::TransportCmd::PlayLastSong.into());
            thread::sleep(std::time::Duration::from_millis(2000));
            let _ =
                audio.send_command(audio_backend::InstrumentCmd::NoteOff { instrument_id }.into());
            thread::sleep(std::time::Duration::from_millis(2000));
        }
        Err(e) => {
            eprintln!("Error initializing audio: {}", e);
        }
    }
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("sample_playback_from_gl_instruments is only available on macOS");
}
