#![cfg(not(feature = "tracker"))]

use std::thread;

use audio_backend::{BlightAudio, InstrumentDefinition, SynthCmd, Waveform};

fn main() {
    // This is a placeholder for the main function.
    // The actual implementation will depend on how you want to use the BlightAudio API.
    match &mut BlightAudio::new() {
        Ok(audio) => {
            println!("BlightAudio initialized successfully!");
            let voice_id = 0;
            // You can now use `audio` to send commands, etc.
            audio.send_command(
                SynthCmd::PlayNote {
                    note: 60,
                    voice: audio.get_voice_factory().create_voice(
                        voice_id,
                        InstrumentDefinition::Oscillator,
                        0.0,
                    ),
                    velocity: 127,
                }
                .into(),
            );
            thread::sleep(std::time::Duration::from_millis(1000));
            audio.send_command(
                SynthCmd::ChangeWaveform {
                    voice_id,
                    waveform: Waveform::Sawtooth,
                }
                .into(),
            );
            thread::sleep(std::time::Duration::from_millis(1000));
            audio.send_command(
                SynthCmd::ChangeWaveform {
                    voice_id,
                    waveform: Waveform::Square,
                }
                .into(),
            );
            thread::sleep(std::time::Duration::from_millis(1000));
            audio.send_command(
                SynthCmd::ChangeWaveform {
                    voice_id,
                    waveform: Waveform::Triangle,
                }
                .into(),
            );
            thread::sleep(std::time::Duration::from_millis(1000));
        }
        Err(e) => eprintln!("Failed to initialize BlightAudio: {}", e),
    };
}
