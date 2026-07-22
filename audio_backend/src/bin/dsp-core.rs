use std::io::{self, Write};

use anyhow::Result;
use audio_backend::{OscServer, StandaloneControlWorker};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    env_logger::init();

    log::info!("starting standalone DSP core");

    let (mut control, meter) = StandaloneControlWorker::spawn()?;
    let osc_server = OscServer::bind().await?;

    // Contract with the future Bun host: stdout readiness detection waits for this line.
    // Print it only after audio is initialized and OSC is listening.
    println!("READY");
    io::stdout().flush()?;

    log::info!("standalone DSP core ready; waiting for shutdown signal");
    tokio::select! {
        result = osc_server.run_with_meter(&mut control, &meter) => result?,
        result = tokio::signal::ctrl_c() => {
            result?;
            log::info!("shutdown signal received; stopping standalone DSP core");
        }
    }

    Ok(())
}
