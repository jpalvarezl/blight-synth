//! Audio Backend Utilities
//!
//! This module provides generalized utility functions for setting up CPAL audio streams
//! with any sample producer callback or struct. This is a more flexible alternative to
//! the specific `run_audio_engine` function that only works with `Synthesizer`.
//!
//! # Usage Examples
//!
//! ## Using a closure as sample producer:
//! ```ignore
//! let sine_gen = create_sine_wave_generator(440.0, 44100.0);
//! let stream = run_audio_stream_with_callback(sine_gen)?;
//! // Keep stream alive...
//! ```
//!
//! ## Using a struct that implements SampleProducer:
//! ```ignore
//! let sine_wave = Arc::new(Mutex::new(SimpleSineWave::new(440.0, 44100.0, 0.1)));
//! let stream = run_audio_stream_with_producer(sine_wave)?;
//! // Keep stream alive...
//! ```
//!
//! ## Implementing SampleProducer for your own types:
//! ```ignore
//! struct MyCustomSynth {
//!     // your fields here
//! }
//!
//! impl SampleProducer for MyCustomSynth {
//!     fn next_sample(&mut self) -> f32 {
//!         // your sample generation logic
//!         0.0
//!     }
//! }
//!
//! let my_synth = Arc::new(Mutex::new(MyCustomSynth { /* ... */ }));
//! let stream = run_audio_stream_with_producer(my_synth)?;
//! ```

use std::sync::{Arc, Mutex};

use cpal::traits::{HostTrait, StreamTrait};
use cpal::{traits::DeviceTrait, FromSample, Sample};
use cpal::{SizedSample, Stream};

/// A trait for any type that can produce audio samples
pub trait SampleProducer {
    fn next_sample(&mut self) -> f32;
}

/// Generic function that sets up and runs a CPAL audio stream with any sample producer callback.
/// Takes a callback function that produces samples.
/// Returns the Stream object, which must be kept alive for audio to play.
pub fn run_audio_stream_with_callback<F>(sample_producer: F) -> anyhow::Result<Stream>
where
    F: FnMut() -> f32 + Send + 'static,
{
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

    // Wrap the callback in Arc<Mutex<>> for thread safety
    let sample_producer = Arc::new(Mutex::new(sample_producer));

    // --- Build and Play Stream ---
    println!("Attempting to build stream...");
    let stream = match sample_format {
        cpal::SampleFormat::F32 => {
            setup_generic_stream_for::<f32>(&device, &config, sample_producer)
        }
        cpal::SampleFormat::I16 => {
            setup_generic_stream_for::<i16>(&device, &config, sample_producer)
        }
        cpal::SampleFormat::U16 => {
            setup_generic_stream_for::<u16>(&device, &config, sample_producer)
        }
        cpal::SampleFormat::I8 => setup_generic_stream_for::<i8>(&device, &config, sample_producer),
        cpal::SampleFormat::I32 => {
            setup_generic_stream_for::<i32>(&device, &config, sample_producer)
        }
        cpal::SampleFormat::I64 => {
            setup_generic_stream_for::<i64>(&device, &config, sample_producer)
        }
        cpal::SampleFormat::U8 => setup_generic_stream_for::<u8>(&device, &config, sample_producer),
        cpal::SampleFormat::U32 => {
            setup_generic_stream_for::<u32>(&device, &config, sample_producer)
        }
        cpal::SampleFormat::U64 => {
            setup_generic_stream_for::<u64>(&device, &config, sample_producer)
        }
        cpal::SampleFormat::F64 => {
            setup_generic_stream_for::<f64>(&device, &config, sample_producer)
        }
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

/// Alternative version that works with any type implementing SampleProducer trait
pub fn run_audio_stream_with_producer<P>(producer: Arc<Mutex<P>>) -> anyhow::Result<Stream>
where
    P: SampleProducer + Send + 'static,
{
    let callback = {
        let producer = producer.clone();
        move || {
            let mut producer = producer.lock().expect("Failed to lock sample producer");
            producer.next_sample()
        }
    };

    run_audio_stream_with_callback(callback)
}

/// Generic setup function for different sample formats that works with any sample producer callback
fn setup_generic_stream_for<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    sample_producer: Arc<Mutex<dyn FnMut() -> f32 + Send>>,
) -> Result<cpal::Stream, anyhow::Error>
where
    T: SizedSample + FromSample<f32>,
{
    let err_fn = |err| eprintln!("An error occurred on the output audio stream: {}", err);
    let channels = config.channels as usize;

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            write_generic_data(data, channels, &sample_producer);
        },
        err_fn,
        None,
    )?;
    Ok(stream)
}

/// Generic audio callback function that works with any sample producer
fn write_generic_data<T>(
    output: &mut [T],
    channels: usize,
    sample_producer: &Arc<Mutex<dyn FnMut() -> f32 + Send>>,
) where
    T: Sample + FromSample<f32>,
{
    for frame in output.chunks_mut(channels) {
        // Get the next sample value from the sample producer
        let next_sample = {
            let mut producer = sample_producer
                .lock()
                .expect("Failed to lock sample producer");
            producer()
        };

        // Convert to the target sample format T and write to output channels
        let sample: T = T::from_sample(next_sample);
        for dest_sample in frame.iter_mut() {
            *dest_sample = sample;
        }
    }
}

/// Convenience function to create a simple sine wave generator for testing
pub fn create_sine_wave_generator(frequency: f32, sample_rate: f32) -> impl FnMut() -> f32 {
    let mut phase: f32 = 0.0;
    let phase_increment = frequency * 2.0 * std::f32::consts::PI / sample_rate;

    move || {
        let sample = phase.sin() * 0.1; // Low amplitude to prevent loud output
        phase += phase_increment;
        if phase >= 2.0 * std::f32::consts::PI {
            phase -= 2.0 * std::f32::consts::PI;
        }
        sample
    }
}

/// Example implementation of SampleProducer for testing
pub struct SimpleSineWave {
    phase: f32,
    frequency: f32,
    sample_rate: f32,
    amplitude: f32,
}

impl SimpleSineWave {
    pub fn new(frequency: f32, sample_rate: f32, amplitude: f32) -> Self {
        Self {
            phase: 0.0,
            frequency,
            sample_rate,
            amplitude,
        }
    }
}

impl SampleProducer for SimpleSineWave {
    fn next_sample(&mut self) -> f32 {
        let sample = self.phase.sin() * self.amplitude;
        self.phase += self.frequency * 2.0 * std::f32::consts::PI / self.sample_rate;
        if self.phase >= 2.0 * std::f32::consts::PI {
            self.phase -= 2.0 * std::f32::consts::PI;
        }
        sample
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_simple_sine_wave_playback() {
        // Create a 440 Hz sine wave at 44.1 kHz sample rate with low amplitude
        let sine_wave = Arc::new(Mutex::new(SimpleSineWave::new(440.0, 44100.0, 0.1)));

        // Start the audio stream
        let stream = run_audio_stream_with_producer(sine_wave.clone())
            .expect("Failed to create audio stream");

        println!("Playing 440 Hz sine wave for 1 second...");

        // Play for 1 second
        std::thread::sleep(Duration::from_secs(1));

        // Stop the stream by dropping it
        drop(stream);

        println!("Playback completed.");

        // Verify that the sine wave actually generated some samples
        // by checking that the phase has advanced
        let sine_wave_guard = sine_wave.lock().unwrap();
        assert!(
            sine_wave_guard.phase > 0.0,
            "Phase should have advanced during playback"
        );
    }

    #[test]
    fn test_sine_wave_sample_generation() {
        let mut sine_wave = SimpleSineWave::new(440.0, 44100.0, 0.5);

        // Generate a few samples and verify they're in expected range
        let mut samples = Vec::new();
        for _ in 0..100 {
            samples.push(sine_wave.next_sample());
        }

        // Check that all samples are within the expected amplitude range
        for sample in &samples {
            assert!(
                *sample >= -0.5 && *sample <= 0.5,
                "Sample {} is outside expected amplitude range [-0.5, 0.5]",
                sample
            );
        }

        // Check that we have some variation (not all zeros)
        let max_sample = samples.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        let min_sample = samples.iter().fold(f32::INFINITY, |a, &b| a.min(b));

        assert!(
            max_sample > 0.1,
            "Maximum sample should be reasonably positive"
        );
        assert!(
            min_sample < -0.1,
            "Minimum sample should be reasonably negative"
        );

        println!(
            "Generated {} samples with range [{:.3}, {:.3}]",
            samples.len(),
            min_sample,
            max_sample
        );
    }

    #[test]
    fn test_callback_based_sine_wave() {
        // Test the callback-based approach
        let sine_gen = create_sine_wave_generator(440.0, 44100.0);

        // This test just verifies the stream can be created successfully
        // We don't actually play it to avoid lengthy test execution
        let stream = run_audio_stream_with_callback(sine_gen);

        assert!(
            stream.is_ok(),
            "Failed to create audio stream with callback"
        );

        // Immediately drop the stream
        drop(stream);

        println!("Callback-based sine wave stream created successfully");
    }
}
