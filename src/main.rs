use cpal::traits::{HostTrait, DeviceTrait};

fn main() -> Result<(), anyhow::Error>{
    let available_hosts = cpal::available_hosts();
    for host_id in available_hosts {
        println!("Found host: {}", host_id.name());
        let host = cpal::host_from_id(host_id)?;

        for device in host.devices()? {
            println!("Found device: {}", device.name()?);
        }
    }
    Ok(())
}
