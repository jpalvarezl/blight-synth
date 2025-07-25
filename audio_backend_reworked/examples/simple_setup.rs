use std::thread;

fn main() {
    // This is a placeholder for the main function.
    // The actual implementation will depend on how you want to use the BlightAudio API.
    match &mut audio_backend_reworked::BlightAudio::new() {
        Ok(audio) => {
            println!("BlightAudio initialized successfully!");
            // You can now use `audio` to send commands, etc.
            audio.send_command(audio_backend_reworked::Command::PlayNote {
                note: 60,
                velocity: 127.0,
            });
            thread::sleep(std::time::Duration::from_secs(1));
            audio.send_command(audio_backend_reworked::Command::PlayNote {
                note: 63,
                velocity: 127.0,
            });
            thread::sleep(std::time::Duration::from_secs(1));
            audio.send_command(audio_backend_reworked::Command::PlayNote {
                note: 66,
                velocity: 127.0,
            });
            thread::sleep(std::time::Duration::from_secs(1));
        }
        Err(e) => eprintln!("Failed to initialize BlightAudio: {}", e),
    };
}
