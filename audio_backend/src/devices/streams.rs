use std::sync::Arc; // Removed Mutex

use cpal::traits::{HostTrait, StreamTrait};
use cpal::{traits::DeviceTrait, FromSample, Sample};
use cpal::{SizedSample, Stream};

use crate::synths::synthesizer::Synthesizer; // Import Synthesizer, removed Voice

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

    let sample_format = supported_config.sample_format();
    let config: cpal::StreamConfig = supported_config.into();

    println!("Using device: {}", device.name()?);
    println!("Using config: {:?}", config);

    // The Synthesizer instance itself is already an Arc, pass it directly.
    // No need for get_voice_handle as the synth itself will provide samples.

    // --- Build and Play Stream ---
    println!("Attempting to build stream...");
    let stream = match sample_format {
        cpal::SampleFormat::F32 => setup_stream_for::<f32>(&device, &config, synth.clone()),
        cpal::SampleFormat::I16 => setup_stream_for::<i16>(&device, &config, synth.clone()),
        cpal::SampleFormat::U16 => setup_stream_for::<u16>(&device, &config, synth.clone()),
        cpal::SampleFormat::I8 => setup_stream_for::<i8>(&device, &config, synth.clone()),
        cpal::SampleFormat::I32 => setup_stream_for::<i32>(&device, &config, synth.clone()),
        cpal::SampleFormat::I64 => setup_stream_for::<i64>(&device, &config, synth.clone()),
        cpal::SampleFormat::U8 => setup_stream_for::<u8>(&device, &config, synth.clone()),
        cpal::SampleFormat::U32 => setup_stream_for::<u32>(&device, &config, synth.clone()),
        cpal::SampleFormat::U64 => setup_stream_for::<u64>(&device, &config, synth.clone()),
        cpal::SampleFormat::F64 => setup_stream_for::<f64>(&device, &config, synth.clone()),
        _ => {
            return Err(anyhow::anyhow!(
                "Unsupported sample format: {:?}",
                sample_format
            ))
        }
    }?;
    println!("Stream built successfully!");
    stream.play()?; // Start playing audio
    Ok(stream) // Return the stream
}

fn setup_stream_for<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    synth: Arc<Synthesizer>, // Changed to Arc<Synthesizer>
) -> Result<cpal::Stream, anyhow::Error>
where
    T: SizedSample + FromSample<f32>,
{
    let err_fn = |err| eprintln!("An error occurred on the output audio stream: {}", err);
    let channels = config.channels as usize;

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            write_data(data, channels, &synth); // Pass the synthesizer Arc
        },
        err_fn,
        None,
    )?;
    Ok(stream)
}

// --- Generic Audio Callback Function (Refactored) ---
fn write_data<T>(
    output: &mut [T],
    channels: usize,
    synth: &Arc<Synthesizer>, // Changed to &Arc<Synthesizer>
) where
    T: Sample + FromSample<f32>,
{
    // The Synthesizer's next_sample() method now handles getting samples.
    // It currently sums the output of its voices (or just the first one).
    for frame in output.chunks_mut(channels) {
        // Get the next sample value from the synthesizer.
        let value = synth.next_sample();

        // Convert to the target sample format T and write to output channels
        let sample: T = T::from_sample(value);
        for dest_sample in frame.iter_mut() {
            *dest_sample = sample;
        }
    }
    // Locking is handled within synth.next_sample() for its voice pool.
}
