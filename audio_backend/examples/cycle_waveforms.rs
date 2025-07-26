use std::thread;

use audio_backend::{oscillator_node::Waveform, Command, BlightAudio};

fn main() {
    // This is a placeholder for the main function.
    // The actual implementation will depend on how you want to use the BlightAudio API.
    match &mut BlightAudio::new() {
        Ok(audio) => {
            println!("BlightAudio initialized successfully!");
            let voice_id = 0;
            // You can now use `audio` to send commands, etc.
            audio.send_command(Command::PlayNote {
                note: 60,
                voice: audio.get_voice_factory().create_voice(voice_id, 0, 0.0),
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
        }
        Err(e) => eprintln!("Failed to initialize BlightAudio: {}", e),
    };
}
