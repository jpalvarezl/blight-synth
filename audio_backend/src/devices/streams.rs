use std::sync::{Arc, Mutex};

use cpal::traits::{HostTrait, StreamTrait};
use cpal::Stream;
use cpal::{traits::DeviceTrait, FromSample, Sample};

use crate::synths::synthesizer::{Synthesizer, Voice}; // Import Voice

// This function sets up and runs the CPAL audio stream.
// It takes the synthesizer as input.
// It returns the Stream object, which must be kept alive for audio to play.
pub fn run_audio_engine(synth: Arc<Synthesizer>) -> anyhow::Result<Stream> {
    // --- CPAL Setup ---
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| anyhow::anyhow!("No output device available"))?;
    let mut supported_configs_range = device.supported_output_configs()?;
    let supported_config = supported_configs_range
        .next()
        .ok_or_else(|| anyhow::anyhow!("No supported config found"))?
        .with_max_sample_rate();

    let err_fn = |err| eprintln!("An error occurred on the output audio stream: {}", err);
    let sample_format = supported_config.sample_format();
    let config: cpal::StreamConfig = supported_config.into();
    // let sample_rate = config.sample_rate.0 as f32; // Sample rate is now managed by Synthesizer/Voice
    let channels = config.channels as usize;

    println!("Using device: {}", device.name()?);
    println!("Using config: {:?}", config);

    // Get the thread-safe handle to the voice
    let voice_handle = synth.get_voice_handle(); // Use the new method

    // --- Build and Play Stream ---
    println!("Attempting to build stream...");
    let stream = match sample_format {
        cpal::SampleFormat::F32 => device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                // Pass only the voice handle
                write_data(data, channels, &voice_handle);
            },
            err_fn,
            None,
        )?,
        cpal::SampleFormat::I16 => device.build_output_stream(
            &config,
            move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                write_data(data, channels, &voice_handle);
            },
            err_fn,
            None,
        )?,
        cpal::SampleFormat::U16 => device.build_output_stream(
            &config,
            move |data: &mut [u16], _: &cpal::OutputCallbackInfo| {
                write_data(data, channels, &voice_handle);
            },
            err_fn,
            None,
        )?,
        _ => {
            return Err(anyhow::anyhow!(
                "Unsupported sample format {}",
                sample_format
            ))
        }
    };
    println!("Stream built successfully!");
    stream.play()?; // Start playing audio
    Ok(stream) // Return the stream
}

// --- Generic Audio Callback Function (Refactored) ---
fn write_data<T>(
    output: &mut [T],
    channels: usize,
    voice_handle: &Arc<Mutex<Voice>>, // Use the Voice handle
) where
    T: Sample + FromSample<f32>,
{
    // Lock the mutex *once* before processing the buffer.
    let mut voice_guard = match voice_handle.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            // Mutex is poisoned - fill the entire buffer with silence and return.
            eprintln!("Audio thread: Voice mutex poisoned, outputting silence.");
            let sample: T = T::from_sample(0.0f32);
            for frame in output.chunks_mut(channels) {
                for dest_sample in frame.iter_mut() {
                    *dest_sample = sample;
                }
            }
            // If the mutex is poisoned, we might want to return the poisoned guard
            // to potentially recover, but for now, just returning is simpler.
            // return poisoned.into_inner(); // Or handle recovery
            return;
        }
    };

    // Now process the buffer with the lock held.
    for frame in output.chunks_mut(channels) {
        // Get the next sample value directly from the voice using the existing guard.
        let value = voice_guard.next_sample();

        // Convert to the target sample format T and write to output channels
        let sample: T = T::from_sample(value);
        for dest_sample in frame.iter_mut() {
            *dest_sample = sample;
        }
    }

    // The lock (voice_guard) is automatically released here when it goes out of scope.
}
