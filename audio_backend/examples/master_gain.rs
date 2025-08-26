#![cfg(not(feature = "tracker"))]
use std::thread;

use audio_backend::{BlightAudio, Command, InstrumentDefinition, MixerCmd, SynthCmd};

pub fn main() {
    match &mut BlightAudio::new() {
        Ok(audio) => {
            println!("BlightAudio initialized successfully!");
            audio.send_command(Command::Mixer(MixerCmd::AddMasterEffect {
                effect: audio.get_effect_factory().create_stereo_gain(0.1),
            }));

            audio.send_command(Command::Synth(SynthCmd::PlayNote {
                note: 60,
                voice: audio.get_voice_factory().create_voice(
                    0,
                    InstrumentDefinition::Oscillator,
                    0.0,
                ),
                velocity: 127,
            }));

            // Play a very short note - 200ms
            thread::sleep(std::time::Duration::from_millis(200));
            audio.send_command(Command::Synth(SynthCmd::StopNote { voice_id: 0 }));

            // Wait to hear the release decay
            thread::sleep(std::time::Duration::from_millis(3000));

            // Play a chord
            audio.send_command(Command::Synth(SynthCmd::PlayNote {
                note: 60, // C
                voice: audio.get_voice_factory().create_voice(
                    1,
                    InstrumentDefinition::Oscillator,
                    0.0,
                ),
                velocity: 100,
            }));

            audio.send_command(Command::Synth(SynthCmd::PlayNote {
                note: 64, // E
                voice: audio.get_voice_factory().create_voice(
                    2,
                    InstrumentDefinition::Oscillator,
                    0.0,
                ),
                velocity: 100,
            }));
            audio.send_command(Command::Synth(SynthCmd::PlayNote {
                note: 67, // G
                voice: audio.get_voice_factory().create_voice(
                    3,
                    InstrumentDefinition::Oscillator,
                    0.0,
                ),
                velocity: 100,
            }));

            thread::sleep(std::time::Duration::from_millis(500));

            // Stop all notes
            audio.send_command(Command::Synth(SynthCmd::StopNote { voice_id: 1 }));
            audio.send_command(Command::Synth(SynthCmd::StopNote { voice_id: 2 }));
            audio.send_command(Command::Synth(SynthCmd::StopNote { voice_id: 3 }));

            // Listen to the release tail
            thread::sleep(std::time::Duration::from_millis(5000));
        }
        Err(e) => eprintln!("Failed to initialize BlightAudio: {}", e),
    };
}
