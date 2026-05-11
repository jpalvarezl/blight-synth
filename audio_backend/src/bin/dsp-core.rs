use std::io::{self, Write};

use anyhow::Result;
use audio_backend::BlightAudio;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    log::info!("starting standalone DSP core");

    // Keep the audio stream alive for the lifetime of this process.
    let _audio = BlightAudio::new()?;

    // Contract with the future Bun host: stdout readiness detection waits for this line.
    println!("READY");
    io::stdout().flush()?;

    log::info!("standalone DSP core ready; waiting for shutdown signal");
    tokio::signal::ctrl_c().await?;
    log::info!("shutdown signal received; stopping standalone DSP core");

    Ok(())
}
