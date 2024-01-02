use cpal::traits::{HostTrait, DeviceTrait};

pub mod harmony;

pub fn get_default_output_device_name() -> Result<String , anyhow::Error> {
    let available_hosts = cpal::available_hosts();
    for host_id in available_hosts {
        println!("Found host: {}", host_id.name());
        let host = cpal::host_from_id(host_id)?;

        if let Some(default_output_device) = host.default_output_device() {
            // for device in host.devices()? {
            //     println!("Found device: {}", device.name()?);
            // }
            return Ok(default_output_device.name()?);
        } else {
            return Err(anyhow::anyhow!("No default device found"));
        };
    }
    return Err(anyhow::anyhow!("No host found"));
}
