#[cfg(not(feature = "tracker"))]
mod no_sequencer;
#[cfg(not(feature = "tracker"))]
pub use no_sequencer::*;

#[cfg(feature = "tracker")]
mod tracker;
#[cfg(feature = "tracker")]
pub use tracker::*;

#[cfg(feature = "tracker")]
use crate::Player;

use crate::{Command, Synthesizer};
use ringbuf::{HeapCons};

/// The core audio processor. Lives exclusively in the RT world.
/// The behaviour of the AudioProcessor changes substrantially based on the "tracker" feature flag.
pub struct AudioProcessor {
    command_rx: HeapCons<Command>,
    synthesizer: Synthesizer,
    sample_rate: f32,
    channels: usize,
    // Pre-allocated, non-interleaved buffers for processing.
    left_buf: Vec<f32>,
    right_buf: Vec<f32>,
    // Player
    #[cfg(feature = "tracker")]
    player: Player,
}
