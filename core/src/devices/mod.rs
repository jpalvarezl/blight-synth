use std::os;

use cpal::{traits::{HostTrait, DeviceTrait, StreamTrait}, SizedSample, FromSample,Sample, StreamConfig};
use crate::Result;

use crate::synths::oscilator::{Oscillator, Waveform};

pub fn get_default_device() -> Result<cpal::Device> {
    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .ok_or_else(|| anyhow::Error::msg("Default output device is not available"))?;
    println!("Output device : {}", device.name()?);

    Ok(device)
}

pub fn setup_stream() -> Result<cpal::Stream> {
    let device = get_default_device()?;
    let config = device.default_output_config()?;
    println!("Default output config : {:?}", &config);
    match config.sample_format() {
        cpal::SampleFormat::I8 => make_stream::<i8>(&device, &config.into()),
        cpal::SampleFormat::I16 => make_stream::<i16>(&device, &config.into()),
        cpal::SampleFormat::I32 => make_stream::<i32>(&device, &config.into()),
        cpal::SampleFormat::I64 => make_stream::<i64>(&device, &config.into()),
        cpal::SampleFormat::U8 => make_stream::<u8>(&device, &config.into()),
        cpal::SampleFormat::U16 => make_stream::<u16>(&device, &config.into()),
        cpal::SampleFormat::U32 => make_stream::<u32>(&device, &config.into()),
        cpal::SampleFormat::U64 => make_stream::<u64>(&device, &config.into()),
        cpal::SampleFormat::F32 => make_stream::<f32>(&device, &config.into()),
        cpal::SampleFormat::F64 => make_stream::<f64>(&device, &config.into()),
        sample_format => Err(anyhow::Error::msg(format!(
            "Unsupported sample format '{sample_format}'"
        ))),
    }
}

fn get_oscillator(config: &StreamConfig) -> Oscillator {
    Oscillator {
        waveform: Waveform::Sine,
        sample_rate: config.sample_rate.0,
        current_sample_index: 0,
        frequency_hz: 440.0,
    }
}

pub fn make_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
) -> Result<cpal::Stream>
where
    T: SizedSample + FromSample<f32>,
{
    let num_channels = config.channels as usize; 
    let mut sample_index = 0.0;
    let sample_rate = config.sample_rate.0 as f32;
    let oscillator = get_oscillator(&config.clone().into());

    let stream = device.build_output_stream(
        config,
        move |output: &mut [T], _info: &cpal::OutputCallbackInfo| 
            process_frame(output, &oscillator, num_channels,&mut sample_index, sample_rate),
        |err| eprintln!("Error building output sound stream: {}", err),
        None,
    )?;

    Ok(stream)
}

fn process_frame<SampleType>(
    output: &mut [SampleType],
    oscillator: &Oscillator,
    num_channels: usize,
    sample_index: &mut f32,
    sample_rate: f32,
) where
    SampleType: Sample + FromSample<f32>,
{
    for frame in output.chunks_mut(num_channels) {

        // still unsure if I should calculate the sample "time" here or push it to the oscillator
        let value: SampleType = SampleType::from_sample(oscillator.tick(*sample_index / sample_rate));

        // copy the same value to all channels
        for sample in frame.iter_mut() {
            *sample = value;
        }
        *sample_index = (*sample_index + 1.0) % sample_rate;
    }
}
