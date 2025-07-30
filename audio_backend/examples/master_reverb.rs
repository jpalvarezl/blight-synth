use std::thread;

use audio_backend::{BlightAudio, Command, InstrumentDefinition, Waveform};

pub fn main() {
    // This is a placeholder for the main function.
    // The actual implementation will depend on how you want to use the BlightAudio API.
    match &mut BlightAudio::new() {
        Ok(audio) => {
            println!("BlightAudio initialized successfully!");
            let voice_id = 0;

            // Add a stereo reverb to the master effect chain
            // Parameters:
            // - room_size: 0.7 (large hall, ~2-3 second decay)
            // - damping: 0.5 (natural room damping, balanced brightness)
            // 
            // Safe parameter ranges:
            // - Small room: room_size=0.3, damping=0.7
            // - Medium hall: room_size=0.5, damping=0.5
            // - Large hall: room_size=0.7, damping=0.5
            // - Cathedral: room_size=0.9, damping=0.3
            // 
            // WARNING: room_size >= 1.0 will cause runaway feedback!
            audio.send_command(Command::AddMasterEffect { effect: audio.get_effect_factory().create_stereo_reverb(0.8, 0.5) });
            audio.send_command(Command::AddMasterEffect { effect: audio.get_effect_factory().create_gain(0.5) });
            // You can now use `audio` to send commands, etc.
            audio.send_command(Command::PlayNote {
                note: 60,
                voice: audio.get_voice_factory().create_voice(
                    voice_id,
                    InstrumentDefinition::Oscillator,
                    0.0,
                ),
                velocity: 127,
            });
            thread::sleep(std::time::Duration::from_millis(1000));
            audio.send_command(Command::ChangeWaveform {
                voice_id,
                waveform: Waveform::Sawtooth,
            });
            thread::sleep(std::time::Duration::from_millis(1000));
            audio.send_command(Command::ChangeWaveform {
                voice_id,
                waveform: Waveform::Square,
            });
            thread::sleep(std::time::Duration::from_millis(1000));
            audio.send_command(Command::ChangeWaveform {
                voice_id,
                waveform: Waveform::Triangle,
            });
            thread::sleep(std::time::Duration::from_millis(1000));
            audio.send_command(Command::StopNote { voice_id: voice_id });
            thread::sleep(std::time::Duration::from_millis(5000));
        }
        Err(e) => eprintln!("Failed to initialize BlightAudio: {}", e),
    };
}