use std::thread;

use audio_backend::{BlightAudio, InstrumentCmd, SequencerCmd, TransportCmd};

fn main() {
    // This is a placeholder for the main function.
    // The actual implementation will depend on how you want to use the BlightAudio API.
    match &mut BlightAudio::new() {
        Ok(audio) => {
            let instrument_id = 0;
            audio.send_command(
                SequencerCmd::AddTrackInstrument {
                    instrument: audio
                        .get_instrument_factory()
                        .create_dfam(instrument_id, 0.0),
                }
                .into(),
            );
            audio.send_command(TransportCmd::PlayLastSong.into());
            audio.send_command(
                InstrumentCmd::NoteOn {
                    instrument_id,
                    note: 60,
                    velocity: 127,
                }
                .into(),
            );
            thread::sleep(std::time::Duration::from_millis(1000));
            audio.send_command(InstrumentCmd::NoteOff { instrument_id }.into());
            audio.send_command(
                InstrumentCmd::NoteOn {
                    instrument_id,
                    note: 63,
                    velocity: 127,
                }
                .into(),
            );
            thread::sleep(std::time::Duration::from_millis(1000));
            audio.send_command(InstrumentCmd::NoteOff { instrument_id }.into());
            audio.send_command(
                InstrumentCmd::NoteOn {
                    instrument_id,
                    note: 63,
                    velocity: 127,
                }
                .into(),
            );
            thread::sleep(std::time::Duration::from_millis(1000));
            audio.send_command(InstrumentCmd::NoteOff { instrument_id }.into());
            audio.send_command(
                InstrumentCmd::NoteOn {
                    instrument_id,
                    note: 66,
                    velocity: 127,
                }
                .into(),
            );
            thread::sleep(std::time::Duration::from_millis(1000));
            audio.send_command(InstrumentCmd::NoteOff { instrument_id }.into());
            thread::sleep(std::time::Duration::from_millis(1000));
        }
        Err(e) => eprintln!("Failed to initialize BlightAudio: {}", e),
    }
}
