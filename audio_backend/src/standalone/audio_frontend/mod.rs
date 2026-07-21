mod blight_audio;
mod command_sender;

pub use command_sender::{CommandSubmissionError, CommandSubmissionResult};

use crate::MeterState;
use std::sync::Arc;

use crate::EffectFactory;
use crate::{InstrumentFactory, ResourceManager, VoiceFactory};
use command_sender::CommandSender;

/// The public-facing API for the audio backend. Lives in the NRT (not real-time) world.
pub struct BlightAudio {
    /// The producer end of the command queue.
    command_sender: CommandSender,
    /// Instrument factory for creating and managing instruments.
    instrument_factory: InstrumentFactory,
    /// Voice factory for creating and managing voices.
    voice_factory: VoiceFactory,
    /// Resource manager for audio samples and other resources.
    resource_manager: ResourceManager,
    /// Effect factory for creating audio effects.
    effect_factory: EffectFactory,
    /// Lock-free metering state written by the audio callback.
    meter: Arc<MeterState>,
    /// The audio stream for real-time audio processing.
    _stream: cpal::Stream,
}
