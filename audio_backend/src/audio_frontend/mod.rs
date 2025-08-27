mod blight_audio;

use crate::Command;
use ringbuf::HeapProd;

use crate::EffectFactory;
use crate::{ResourceManager, VoiceFactory};

/// The public-facing API for the audio backend. Lives in the NRT (not real-time) world.
pub struct BlightAudio {
    /// The producer end of the command queue.
    command_tx: HeapProd<Command>,
    /// Instrument factory for creating and managing instruments.
    instrument_factory: crate::factories::InstrumentFactory,
    /// Voice factory for creating and managing voices.
    voice_factory: VoiceFactory,
    /// Resource manager for audio samples and other resources.
    resource_manager: ResourceManager,
    /// Effect factory for creating audio effects.
    effect_factory: EffectFactory,
    /// The audio stream for real-time audio processing.
    _stream: cpal::Stream,
}
