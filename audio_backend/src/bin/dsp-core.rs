use std::io::{self, Write};

use anyhow::Result;
use audio_backend::{BlightAudio, OscServer};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    log::info!("starting standalone DSP core");

    // Keep the audio stream alive for the lifetime of this process.
    let audio = BlightAudio::new()?;
    let osc_server = OscServer::bind(audio.shared_state()).await?;

    // Contract with the future Bun host: stdout readiness detection waits for this line.
    // Print it only after audio is initialized and OSC is listening.
    println!("READY");
    io::stdout().flush()?;

    log::info!("standalone DSP core ready; waiting for shutdown signal");
    tokio::select! {
        result = osc_server.run() => result?,
        result = tokio::signal::ctrl_c() => {
            result?;
            log::info!("shutdown signal received; stopping standalone DSP core");
        }
    }

    Ok(())
}
