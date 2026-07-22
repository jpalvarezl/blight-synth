use std::thread;

use audio_backend::{BlightAudio, InstrumentCmd, TransportCmd};

fn main() {
    // This is a placeholder for the main function.
    // The actual implementation will depend on how you want to use the BlightAudio API.
    match &mut BlightAudio::new() {
        Ok(audio) => {
            let instrument_id = 0;
            let _ = audio.send_command(
                InstrumentCmd::AddInstrument {
                    instrument: audio
                        .get_instrument_factory()
                        .create_dfam(instrument_id, 0.0),
                }
                .into(),
            );
            let _ = audio.send_command(TransportCmd::PlayLastSong.into());
            let _ = audio.send_command(
                InstrumentCmd::NoteOn {
                    instrument_id,
                    note: 60,
                    velocity: 127,
                }
                .into(),
            );
            thread::sleep(std::time::Duration::from_millis(1000));
            let _ = audio.send_command(InstrumentCmd::NoteOff { instrument_id }.into());
            let _ = audio.send_command(
                InstrumentCmd::NoteOn {
                    instrument_id,
                    note: 63,
                    velocity: 127,
                }
                .into(),
            );
            thread::sleep(std::time::Duration::from_millis(1000));
            let _ = audio.send_command(InstrumentCmd::NoteOff { instrument_id }.into());
            let _ = audio.send_command(
                InstrumentCmd::NoteOn {
                    instrument_id,
                    note: 63,
                    velocity: 127,
                }
                .into(),
            );
            thread::sleep(std::time::Duration::from_millis(1000));
            let _ = audio.send_command(InstrumentCmd::NoteOff { instrument_id }.into());
            let _ = audio.send_command(
                InstrumentCmd::NoteOn {
                    instrument_id,
                    note: 66,
                    velocity: 127,
                }
                .into(),
            );
            thread::sleep(std::time::Duration::from_millis(1000));
            let _ = audio.send_command(InstrumentCmd::NoteOff { instrument_id }.into());
            thread::sleep(std::time::Duration::from_millis(1000));
        }
        Err(e) => eprintln!("Failed to initialize BlightAudio: {}", e),
    }
}
