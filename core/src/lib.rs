use cpal::traits::{HostTrait, DeviceTrait};

pub fn print_hosts() -> Result<(), anyhow::Error>{
    let available_hosts = cpal::available_hosts();
    for host_id in available_hosts {
        println!("Found host: {}", host_id.name());
        let host = cpal::host_from_id(host_id)?;

        println!("Default output device: {:#?}", host.default_output_device().expect("output device fail").name());
        
        for device in host.devices()? {
            println!("Found device: {}", device.name()?);
        }
    }
    Ok(())
}
