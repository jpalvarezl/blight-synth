use devices::streams::run_audio_engine;
use std::sync::{Arc, Mutex};
use synths::new_synthesizer::Synthesizer;

pub mod devices;
pub mod envelope;
pub mod synths; // Add the new module

type Result<T> = anyhow::Result<T, anyhow::Error>;

pub fn start_audio_thread(synth: Arc<Mutex<Synthesizer>>) {
    println!("Starting audio thread");
    let _thread_handle = std::thread::spawn(move || {
        let _stream = run_audio_engine(synth).expect("Stream creation error");

        // block the thread
        loop {}
        // stream.pause().expect("Stream stop error");
    });
}
