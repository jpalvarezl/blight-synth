use crate::Result;
use cpal::traits::{DeviceTrait, HostTrait};

pub mod audio_engine;
pub(crate) mod streams;

pub fn get_default_device() -> Result<cpal::Device> {
    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .ok_or_else(|| anyhow::Error::msg("Default output device is not available"))?;
    println!("Output device : {}", device.name()?);

    Ok(device)
}
