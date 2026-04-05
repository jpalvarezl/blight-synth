mod audio;
mod osc;
mod engine;
mod state;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let state = state::AppState::new();
    let engine = engine::DspEngine::new(state.clone());
    let osc_server = osc::OscServer::new(state.clone());

    // Start audio engine
    engine.start()?;

    // Start OSC server — blocks until shutdown
    osc_server.run().await?;

    Ok(())
}
