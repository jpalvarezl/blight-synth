#[cfg(not(feature = "tracker"))]
mod no_sequencer;

#[cfg(feature = "tracker")]
mod tracker;

#[cfg(feature = "tracker")]
use crate::{Player, Command};

#[cfg(not(feature = "tracker"))]
use crate::{Command, Synthesizer};
use ringbuf::HeapCons;

/// The core audio processor. Lives exclusively in the RT world.
/// The behaviour of the AudioProcessor changes substrantially based on the "tracker" feature flag.
pub struct AudioProcessor {
    #[cfg(feature = "tracker")]
    command_rx: HeapCons<Command>,
    #[cfg(feature = "tracker")]
    player: Player,
    #[cfg(not(feature = "tracker"))]
    command_rx: HeapCons<Command>,
    #[cfg(not(feature = "tracker"))]
    synthesizer: Synthesizer,
    sample_rate: f32,
    channels: usize,
    // Pre-allocated, non-interleaved buffers for processing.
    left_buf: Vec<f32>,
    right_buf: Vec<f32>,
}
