use std::{thread, time::Duration};

use audio_backend_reworked::Command;

fn main() {
    match &mut audio_backend_reworked::BlightAudio::new() {
        Ok(audio) => {
            let voice_id = 0;
            let instrument_id = 0;
            let voice = audio.get_voice_factory().create_voice_with_envelope(
                voice_id,
                instrument_id,
                0.0,
                0.4,
                0.1,
                0.8,
                0.5,
            );
            audio.send_command(Command::PlayNote {
                voice,
                note: 60,
                velocity: 127,
            });
            thread::sleep(Duration::from_millis(600));
            audio.send_command(Command::StopNote { voice_id });
            thread::sleep(Duration::from_millis(800));

            // slow attack and release
            let voice_id = 0;
            let instrument_id = 0;
            let voice = audio.get_voice_factory().create_voice_with_envelope(
                voice_id,
                instrument_id,
                0.0,
                2.0,
                0.1,
                0.8,
                2.0,
            );
            audio.send_command(Command::PlayNote {
                voice,
                note: 60,
                velocity: 127,
            });
            thread::sleep(Duration::from_millis(2200));
            audio.send_command(Command::StopNote { voice_id });
            thread::sleep(Duration::from_millis(4000)); // wait for release to finish so the voice gets evicted from the voice manager

            // quick attack and release
            let voice_id = 0;
            let instrument_id = 0;
            let voice = audio.get_voice_factory().create_voice_with_envelope(
                voice_id,
                instrument_id,
                0.0,
                0.01,
                0.1,
                0.8,
                0.1,
            );
            audio.send_command(Command::PlayNote {
                voice,
                note: 60,
                velocity: 127,
            });
            thread::sleep(Duration::from_millis(200));
            audio.send_command(Command::StopNote { voice_id });
            thread::sleep(Duration::from_millis(1000));
        }
        Err(e) => {
            eprintln!("Error initializing audio: {}", e);
        }
    }
}
