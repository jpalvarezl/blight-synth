use std::io::{self, Write};

use anyhow::Result;
use audio_backend::{BlightAudio, MixerCmd, OscServer, MASTER_GAIN_EFFECT_ID};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    log::info!("starting standalone DSP core");

    // Keep the audio stream alive for the lifetime of this process.
    let mut audio = BlightAudio::new()?;

    // Install the existing master gain effect that OSC `/param/set gain <db>` controls.
    audio.send_command(
        MixerCmd::AddMasterEffect {
            effect: audio
                .get_effect_factory()
                .create_stereo_gain(MASTER_GAIN_EFFECT_ID, 1.0),
        }
        .into(),
    );

    let osc_server = OscServer::bind().await?;

    // Contract with the future Bun host: stdout readiness detection waits for this line.
    // Print it only after audio is initialized and OSC is listening.
    println!("READY");
    io::stdout().flush()?;

    log::info!("standalone DSP core ready; waiting for shutdown signal");
    tokio::select! {
        result = osc_server.run(&mut audio) => result?,
        result = tokio::signal::ctrl_c() => {
            result?;
            log::info!("shutdown signal received; stopping standalone DSP core");
        }
    }

    Ok(())
}
