use std::sync::{Arc, Mutex};

use cpal::Stream;
use cpal::{traits::DeviceTrait, FromSample, Sample};
use cpal::traits::{HostTrait, StreamTrait};


use crate::synths::oscillator::{Oscillator, Saw, Sine, Square, Triangle};
use crate::synths::synthesizer::{ActiveWaveform, SynthControl, Synthesizer};

// pub fn setup_stream(oscillator: Arc<RwLock<Oscillator>>) -> Result<cpal::Stream> {
//     let device = super::get_default_device()?;
//     let config = device.default_output_config()?;
//     println!("Default output config : {:?}", &config);
//     match config.sample_format() {
//         cpal::SampleFormat::I8 => make_stream::<i8>(&device, &config.into(), oscillator),
//         cpal::SampleFormat::I16 => make_stream::<i16>(&device, &config.into(), oscillator),
//         cpal::SampleFormat::I32 => make_stream::<i32>(&device, &config.into(), oscillator),
//         cpal::SampleFormat::I64 => make_stream::<i64>(&device, &config.into(), oscillator),
//         cpal::SampleFormat::U8 => make_stream::<u8>(&device, &config.into(), oscillator),
//         cpal::SampleFormat::U16 => make_stream::<u16>(&device, &config.into(), oscillator),
//         cpal::SampleFormat::U32 => make_stream::<u32>(&device, &config.into(), oscillator),
//         cpal::SampleFormat::U64 => make_stream::<u64>(&device, &config.into(), oscillator),
//         cpal::SampleFormat::F32 => make_stream::<f32>(&device, &config.into(), oscillator),
//         cpal::SampleFormat::F64 => make_stream::<f64>(&device, &config.into(), oscillator),
//         sample_format => Err(anyhow::Error::msg(format!(
//             "Unsupported sample format '{sample_format}'"
//         ))),
//     }
// }

// fn make_stream<T>(
//     device: &cpal::Device,
//     config: &cpal::StreamConfig,
//     oscillator: Arc<RwLock<Oscillator>>,
// ) -> Result<cpal::Stream>
// where
//     T: SizedSample + FromSample<f32>,
// {
//     let num_channels = config.channels as usize;
//     let mut sample_index = 0.0;
//     let sample_rate = config.sample_rate.0 as f32;

//     let stream = device.build_output_stream(
//         config,
//         move |output: &mut [T], _info: &cpal::OutputCallbackInfo| {
//             process_frame(
//                 output,
//                 oscillator.clone(),
//                 num_channels,
//                 &mut sample_index,
//                 sample_rate,
//             )
//         },
//         |err| eprintln!("Error building output sound stream: {}", err),
//         None,
//     )?;

//     Ok(stream)
// }

// This function sets up and runs the CPAL audio stream.
// It takes the synthesizer control state as input.
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
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;

    println!("Using device: {}", device.name()?);
    println!("Using config: {:?}", config);

    // --- Oscillator Initialization (Local to the audio thread closure) ---
    let mut sine_osc = Oscillator::<Sine>::new(sample_rate, 0.0);
    let mut square_osc = Oscillator::<Square>::new(sample_rate, 0.0);
    let mut saw_osc = Oscillator::<Saw>::new(sample_rate, 0.0);
    let mut triangle_osc = Oscillator::<Triangle>::new(sample_rate, 0.0);

    // Clone the control Arc for the audio thread
    let control_clone = synth.get_control_clone();

    // --- Generic Audio Callback Function ---
    fn write_data<T>(
        output: &mut [T],
        channels: usize,
        control: &Arc<Mutex<SynthControl>>,
        sine_osc: &mut Oscillator<Sine>,
        square_osc: &mut Oscillator<Square>,
        saw_osc: &mut Oscillator<Saw>,
        triangle_osc: &mut Oscillator<Triangle>,
    ) where
        T: Sample + FromSample<f32>,
    {
        if let Ok(locked_control) = control.lock() { // Lock only once per buffer
            let freq = locked_control.frequency;
            let wave = locked_control.waveform;
            let amp = locked_control.amplitude;

            // Update frequency for the *active* oscillator before generating samples
            // This ensures the frequency change is applied smoothly within the buffer
            match wave {
                ActiveWaveform::Sine => sine_osc.set_frequency(freq),
                ActiveWaveform::Square => square_osc.set_frequency(freq),
                ActiveWaveform::Saw => saw_osc.set_frequency(freq),
                ActiveWaveform::Triangle => triangle_osc.set_frequency(freq),
            }

            // Drop the lock *after* setting frequency but *before* the sample loop
            // This allows the main thread to update controls while samples are generated
            drop(locked_control);

            // Generate samples for the buffer
            for frame in output.chunks_mut(channels) {
                // Call next_sample directly on the selected oscillator
                let value = match wave {
                    ActiveWaveform::Sine => sine_osc.next_sample(),
                    ActiveWaveform::Square => square_osc.next_sample(),
                    ActiveWaveform::Saw => saw_osc.next_sample(),
                    ActiveWaveform::Triangle => triangle_osc.next_sample(),
                } * amp; // Apply amplitude

                // Convert to the target sample format T
                let sample: T = T::from_sample(value);
                for dest_sample in frame.iter_mut() {
                    *dest_sample = sample;
                }
            }
        } else {
            // Mutex is poisoned, fill buffer with silence
             eprintln!("Audio thread: Mutex poisoned, outputting silence.");
             for frame in output.chunks_mut(channels) {
                 let sample: T = T::from_sample(0.0f32);
                 for dest_sample in frame.iter_mut() {
                    *dest_sample = sample;
                 }
             }
        }
    }

    // --- Build and Play Stream ---
    println!("Attempting to build stream...");
    let stream = match sample_format {
         cpal::SampleFormat::F32 => device.build_output_stream(
             &config,
             move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                 write_data(data, channels, &control_clone, &mut sine_osc, &mut square_osc, &mut saw_osc, &mut triangle_osc);
             },
             err_fn, None)?,
         cpal::SampleFormat::I16 => device.build_output_stream(
             &config,
             move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                 write_data(data, channels, &control_clone, &mut sine_osc, &mut square_osc, &mut saw_osc, &mut triangle_osc);
             },
             err_fn, None)?,
         cpal::SampleFormat::U16 => device.build_output_stream(
             &config,
             move |data: &mut [u16], _: &cpal::OutputCallbackInfo| {
                 write_data(data, channels, &control_clone, &mut sine_osc, &mut square_osc, &mut saw_osc, &mut triangle_osc);
             },
             err_fn, None)?,
         _ => return Err(anyhow::anyhow!("Unsupported sample format {}", sample_format)),
    };
    println!("Stream built successfully!");

    stream.play()?; // Start playing audio
    Ok(stream) // Return the stream
}