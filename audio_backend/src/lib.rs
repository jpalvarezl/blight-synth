use devices::streams::run_audio_engine;
use std::sync::Arc;
use synths::synthesizer::Synthesizer;

pub mod devices;
pub mod synths;

type Result<T> = anyhow::Result<T, anyhow::Error>;

pub fn start_audio_thread(synth: Arc<Synthesizer>) {
    println!("Starting audio thread");
    let _thread_handle = std::thread::spawn(move || {
        let _stream = run_audio_engine(synth.clone()).expect("Stream creation error");

        // block the thread
        loop {}
        // stream.pause().expect("Stream stop error");
    });
}
