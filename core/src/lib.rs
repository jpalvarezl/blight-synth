use cpal::traits::{DeviceTrait, StreamTrait};
use std::sync::{Arc, RwLock};
use synths::oscillator::Oscillator;

pub mod devices;
pub mod synths;

type Result<T> = anyhow::Result<T, anyhow::Error>;

pub fn start_audio_thread(oscillator_initial_state: Arc<RwLock<Oscillator>>) {
    println!("Starting audio thread");
    let _thread_handle = std::thread::spawn(move || {
        let stream =
            devices::streams::setup_stream(oscillator_initial_state).expect("Stream setup error");
        stream.play().expect("Stream start error");

        // block the thread
        loop {}
        // stream.pause().expect("Stream stop error");
    });
}

pub fn get_default_output_device_name() -> Result<String> {
    let default_device = devices::get_default_device()?;
    Ok(default_device.name()?)
}
