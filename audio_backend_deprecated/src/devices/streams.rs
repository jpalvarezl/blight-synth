use cpal::{
    traits::{DeviceTrait, HostTrait},
    FromSample, Sample,
};
use cpal::{SizedSample, Stream};
use crossbeam::queue::SegQueue;
use log::{error, info};
use std::sync::Arc;

use super::audio_engine::AudioCommand;
use crate::synths::synthesizer::Synthesizer;

// New function that creates a stream with command processing
pub fn create_stream_with_commands(
    max_polyphony: usize,
    command_queue: Arc<SegQueue<AudioCommand>>,
) -> anyhow::Result<Stream> {
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

    let sample_rate = supported_config.sample_rate().0 as f32;
    let sample_format = supported_config.sample_format();
    let config: cpal::StreamConfig = supported_config.into();

    info!("Using device: {}", device.name()?);
    info!("Using config: {:?}", config);

    let synth = Synthesizer::new(sample_rate, max_polyphony);

    // --- Build Stream with Command Processing ---
    info!("Attempting to build stream with command processing...");
    let stream = match sample_format {
        cpal::SampleFormat::F32 => {
            setup_command_stream_for::<f32>(&device, &config, synth, command_queue)
        }
        cpal::SampleFormat::I16 => {
            setup_command_stream_for::<i16>(&device, &config, synth, command_queue)
        }
        cpal::SampleFormat::U16 => {
            setup_command_stream_for::<u16>(&device, &config, synth, command_queue)
        }
        cpal::SampleFormat::I8 => {
            setup_command_stream_for::<i8>(&device, &config, synth, command_queue)
        }
        cpal::SampleFormat::I32 => {
            setup_command_stream_for::<i32>(&device, &config, synth, command_queue)
        }
        cpal::SampleFormat::I64 => {
            setup_command_stream_for::<i64>(&device, &config, synth, command_queue)
        }
        cpal::SampleFormat::U8 => {
            setup_command_stream_for::<u8>(&device, &config, synth, command_queue)
        }
        cpal::SampleFormat::U32 => {
            setup_command_stream_for::<u32>(&device, &config, synth, command_queue)
        }
        cpal::SampleFormat::U64 => {
            setup_command_stream_for::<u64>(&device, &config, synth, command_queue)
        }
        cpal::SampleFormat::F64 => {
            setup_command_stream_for::<f64>(&device, &config, synth, command_queue)
        }
        _ => {
            return Err(anyhow::anyhow!(
                "Unsupported sample format: {:?}",
                sample_format
            ))
        }
    }?;
    info!("Stream built successfully!");
    Ok(stream)
}

// Stream setup with command processing
fn setup_command_stream_for<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    mut synth: Synthesizer,
    command_queue: Arc<SegQueue<AudioCommand>>,
) -> Result<cpal::Stream, anyhow::Error>
where
    T: SizedSample + FromSample<f32>,
{
    let err_fn = |err| error!("An error occurred on the output audio stream: {}", err);
    let channels = config.channels as usize;

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            // Process commands (lock-free, real-time safe)
            process_commands(&mut synth, &command_queue);

            // Generate audio
            write_data(data, channels, &mut synth);
        },
        err_fn,
        None,
    )?;
    Ok(stream)
}

// Process all pending commands in the audio callback
fn process_commands(synth: &mut Synthesizer, command_queue: &SegQueue<AudioCommand>) {
    // Process up to a reasonable number of commands per audio callback
    // to avoid spending too much time in command processing
    const MAX_COMMANDS_PER_CALLBACK: usize = 32;

    for _ in 0..MAX_COMMANDS_PER_CALLBACK {
        match command_queue.pop() {
            Some(cmd) => {
                match cmd {
                    AudioCommand::NoteOn { note, velocity } => {
                        synth.note_on(note, velocity);
                    }
                    AudioCommand::NoteOnWithWaveform {
                        note,
                        velocity,
                        waveform,
                    } => {
                        synth.note_on_with_waveform(note, velocity, waveform);
                    }
                    AudioCommand::NoteOff { note } => {
                        synth.note_off(note);
                    }
                    AudioCommand::SetAdsr {
                        attack,
                        decay,
                        sustain,
                        release,
                    } => {
                        synth.set_adsr(attack, decay, sustain, release);
                    }
                    AudioCommand::SetWaveform(waveform) => {
                        synth.set_waveform(waveform);
                    } // AudioCommand::Stop => {
                      //     // Handle stop command - might want to fade out or similar
                      //     break;
                      // }
                }
            }
            None => break, // No more commands
        }
    }
}

// --- Generic Audio Callback Function (Refactored) ---
fn write_data<T>(output: &mut [T], channels: usize, synth: &mut Synthesizer)
where
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
    // No locking needed since we own the synthesizer in the closure.
}
