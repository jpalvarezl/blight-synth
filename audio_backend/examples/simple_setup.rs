use std::thread;

use audio_backend::{BlightAudio, Command, InstrumentDefinition, SynthCmd};

fn main() {
    // This is a placeholder for the main function.
    // The actual implementation will depend on how you want to use the BlightAudio API.
    match &mut BlightAudio::new() {
        Ok(audio) => {
            println!("BlightAudio initialized successfully!");
            // You can now use `audio` to send commands, etc.
            audio.send_command(
                SynthCmd::PlayNote {
                    note: 60,
                    voice: audio.get_voice_factory().create_voice(
                        0,
                        InstrumentDefinition::Oscillator,
                        0.0,
                    ),
                    velocity: 127,
                }
                .into(),
            );
            thread::sleep(std::time::Duration::from_millis(1000));
            audio.send_command(Command::Synth(SynthCmd::StopNote { voice_id: 0 }));
            audio.send_command(
                SynthCmd::PlayNote {
                    note: 63,
                    voice: audio.get_voice_factory().create_voice(
                        1,
                        InstrumentDefinition::Oscillator,
                        0.0,
                    ),
                    velocity: 127,
                }
                .into(),
            );
            thread::sleep(std::time::Duration::from_millis(1000));
            audio.send_command(Command::Synth(SynthCmd::StopNote { voice_id: 1 }));
            audio.send_command(
                SynthCmd::PlayNote {
                    note: 66,
                    voice: audio.get_voice_factory().create_voice(
                        2,
                        InstrumentDefinition::Oscillator,
                        0.0,
                    ),
                    velocity: 127,
                }
                .into(),
            );
            thread::sleep(std::time::Duration::from_millis(1000));
            audio.send_command(Command::Synth(SynthCmd::StopNote { voice_id: 2 }));
        }
        Err(e) => eprintln!("Failed to initialize BlightAudio: {}", e),
    }
}
