mod blight_audio;

use crate::Command;
use crate::MeterState;
use ringbuf::HeapProd;
use std::sync::Arc;

use crate::EffectFactory;
use crate::{InstrumentFactory, ResourceManager, VoiceFactory};

/// Result of a non-blocking command submission to the audio callback.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommandSubmissionStatus {
    /// The command was placed in the bounded queue.
    Accepted,
    /// The callback is connected, but the bounded queue has no free slot.
    Full,
    /// The callback-side queue consumer no longer exists.
    Disconnected,
}

impl CommandSubmissionStatus {
    pub fn is_accepted(self) -> bool {
        self == Self::Accepted
    }
}

/// The public-facing API for the audio backend. Lives in the NRT (not real-time) world.
pub struct BlightAudio {
    /// The producer end of the command queue.
    command_tx: HeapProd<Command>,
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
