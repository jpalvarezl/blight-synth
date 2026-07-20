mod blight_audio;

use crate::Command;
use crate::MeterState;
use ringbuf::HeapProd;
use std::{fmt, sync::Arc};

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

/// Outcome of submitting an owned command to the bounded callback queue.
///
/// Rejected variants return the original command so the non-real-time caller
/// can retry, defer, or drop prepared state deliberately.
pub enum CommandSubmission {
    Accepted,
    Full(Command),
    Disconnected(Command),
}

impl CommandSubmission {
    pub fn status(&self) -> CommandSubmissionStatus {
        match self {
            Self::Accepted => CommandSubmissionStatus::Accepted,
            Self::Full(_) => CommandSubmissionStatus::Full,
            Self::Disconnected(_) => CommandSubmissionStatus::Disconnected,
        }
    }

    pub fn is_accepted(&self) -> bool {
        self.status().is_accepted()
    }

    pub fn into_rejected_command(self) -> Option<Command> {
        match self {
            Self::Accepted => None,
            Self::Full(command) | Self::Disconnected(command) => Some(command),
        }
    }
}

impl fmt::Debug for CommandSubmission {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Accepted => "Accepted",
            Self::Full(_) => "Full(..)",
            Self::Disconnected(_) => "Disconnected(..)",
        })
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
