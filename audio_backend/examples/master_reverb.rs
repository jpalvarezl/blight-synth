use std::thread;

use audio_backend::{BlightAudio, Command, InstrumentDefinition};

pub fn main() {
    match &mut BlightAudio::new() {
        Ok(audio) => {
            println!("BlightAudio initialized successfully!");
            let voice_id = 0;

            // Try more aggressive reverb settings to make it obvious
            // Using a smaller room with less damping for more audible effect
            audio.send_command(Command::AddMasterEffect { 
                effect: audio.get_effect_factory().create_gain(0.1) 
            });
            
            // Remove the gain effect for now - it might be interfering
            // audio.send_command(Command::AddMasterEffect { effect: audio.get_effect_factory().create_gain(0.5) });
            
            // Play a short staccato note to hear the reverb tail
            println!("Playing C4 (60) with sine wave");
            audio.send_command(Command::PlayNote {
                note: 60,
                voice: audio.get_voice_factory().create_voice(
                    voice_id,
                    InstrumentDefinition::Oscillator,
                    0.0,
                ),
                velocity: 127,
            });
            
            // Play a very short note - 200ms
            thread::sleep(std::time::Duration::from_millis(200));
            
            // Stop the note - you should hear the reverb tail continue
            println!("Stopping note - listen for reverb tail");
            audio.send_command(Command::StopNote { voice_id });

            // Wait to hear the release decay
            thread::sleep(std::time::Duration::from_millis(3000));
            
            // Play a chord
            audio.send_command(Command::PlayNote {
                note: 60, // C
                voice: audio.get_voice_factory().create_voice(1, InstrumentDefinition::Oscillator, 0.0),
                velocity: 100,
            });
            audio.send_command(Command::PlayNote {
                note: 64, // E
                voice: audio.get_voice_factory().create_voice(2, InstrumentDefinition::Oscillator, 0.0),
                velocity: 100,
            });
            audio.send_command(Command::PlayNote {
                note: 67, // G
                voice: audio.get_voice_factory().create_voice(3, InstrumentDefinition::Oscillator, 0.0),
                velocity: 100,
            });
            
            thread::sleep(std::time::Duration::from_millis(500));
            
            // Stop all notes
            audio.send_command(Command::StopNote { voice_id: 1 });
            audio.send_command(Command::StopNote { voice_id: 2 });
            audio.send_command(Command::StopNote { voice_id: 3 });
            
            // Listen to the release tail
            thread::sleep(std::time::Duration::from_millis(5000));
        }
        Err(e) => eprintln!("Failed to initialize BlightAudio: {}", e),
    };
}