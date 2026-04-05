use crate::state::AppState;
use anyhow::Result;

pub struct DspEngine {
    state: AppState,
    // TODO: add cpal Stream handle here
}

impl DspEngine {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    /// Initialise the audio host and open the default output stream.
    pub fn start(&self) -> Result<()> {
        // TODO: enumerate cpal hosts and devices
        // TODO: build output stream config (sample rate, buffer size, channels)
        // TODO: open stream with process_audio as callback
        Ok(())
    }

    pub fn stop(&self) -> Result<()> {
        // TODO: drop the cpal stream
        Ok(())
    }

    /// The real-time audio callback. Must never allocate or block.
    fn process_audio(output: &mut [f32], state: &AppState) {
        // TODO: pull parameters from state (lock-free reads)
        // TODO: run DSP graph / chain
        // TODO: write processed samples to output buffer
    }
}
