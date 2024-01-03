use cpal::traits::{HostTrait, DeviceTrait};
use crate::Result;

pub fn get_default_output_device_name() -> Result<String> {
    let default_device = get_default_device()?;
    Ok(default_device.name()?)
}

pub fn get_default_device() -> Result<cpal::Device> {
    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .ok_or_else(|| anyhow::Error::msg("Default output device is not available"))?;
    println!("Output device : {}", device.name()?);

    let config = device.default_output_config()?;
    println!("Default output config : {:?}", config);

    Ok(device)
}

// pub fn setup_stream() -> Result<cpal::Stream> {
//     let (_host, device, config) = host_device_setup()?;

//     match config.sample_format() {
//         cpal::SampleFormat::I8 => make_stream::<i8>(&device, &config.into()),
//         cpal::SampleFormat::I16 => make_stream::<i16>(&device, &config.into()),
//         cpal::SampleFormat::I32 => make_stream::<i32>(&device, &config.into()),
//         cpal::SampleFormat::I64 => make_stream::<i64>(&device, &config.into()),
//         cpal::SampleFormat::U8 => make_stream::<u8>(&device, &config.into()),
//         cpal::SampleFormat::U16 => make_stream::<u16>(&device, &config.into()),
//         cpal::SampleFormat::U32 => make_stream::<u32>(&device, &config.into()),
//         cpal::SampleFormat::U64 => make_stream::<u64>(&device, &config.into()),
//         cpal::SampleFormat::F32 => make_stream::<f32>(&device, &config.into()),
//         cpal::SampleFormat::F64 => make_stream::<f64>(&device, &config.into()),
//         sample_format => Err(anyhow::Error::msg(format!(
//             "Unsupported sample format '{sample_format}'"
//         ))),
//     }
// }

// pub fn make_stream<T>(
//     device: &cpal::Device,
//     config: &cpal::StreamConfig,
// ) -> Result<cpal::Stream>
// where
//     T: SizedSample + FromSample<f32>,
// {
//     let num_channels = config.channels as usize;
//     let mut oscillator = Oscillator {
//         waveform: Waveform::Sine,
//         sample_rate: config.sample_rate.0 as f32,
//         current_sample_index: 0.0,
//         frequency_hz: 440.0,
//     };
//     let err_fn = |err| eprintln!("Error building output sound stream: {}", err);

//     let time_at_start = std::time::Instant::now();
//     println!("Time at start: {:?}", time_at_start);

//     let stream = device.build_output_stream(
//         config,
//         move |output: &mut [T], _: &cpal::OutputCallbackInfo| {
//             // for 0-1s play sine, 1-2s play square, 2-3s play saw, 3-4s play triangle_wave
//             let time_since_start = std::time::Instant::now()
//                 .duration_since(time_at_start)
//                 .as_secs_f32();
//             if time_since_start < 1.0 {
//                 oscillator.set_waveform(Waveform::Sine);
//             } else if time_since_start < 2.0 {
//                 oscillator.set_waveform(Waveform::Triangle);
//             } else if time_since_start < 3.0 {
//                 oscillator.set_waveform(Waveform::Square);
//             } else if time_since_start < 4.0 {
//                 oscillator.set_waveform(Waveform::Saw);
//             } else {
//                 oscillator.set_waveform(Waveform::Sine);
//             }
//             process_frame(output, &mut oscillator, num_channels)
//         },
//         err_fn,
//         None,
//     )?;

//     Ok(stream)
// }
