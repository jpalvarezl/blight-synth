#[cfg(feature = "tracker")]
mod tracker;
#[cfg(feature = "tracker")]
pub use tracker::*;

#[cfg(not(feature = "tracker"))]
mod blight_audio;
#[cfg(not(feature = "tracker"))]
pub use blight_audio::*;

#[cfg(not(feature = "tracker"))]
use ringbuf::{HeapProd};
#[cfg(not(feature = "tracker"))]
use crate::Command;


use crate::effect_factory::EffectFactory;
use crate::{ResourceManager, VoiceFactory};

/// The public-facing API for the audio backend. Lives in the NRT (not real-time) world.
pub struct BlightAudio {
    #[cfg(not(feature = "tracker"))]
    /// The producer end of the command queue.
    command_tx: HeapProd<Command>,
    /// Voice factory for creating and managing voices.
    voice_factory: VoiceFactory,
    /// Resource manager for audio samples and other resources.
    resource_manager: ResourceManager,
    /// Effect factory for creating audio effects.
    effect_factory: EffectFactory,
    /// The audio stream for real-time audio processing.
    _stream: cpal::Stream,
}