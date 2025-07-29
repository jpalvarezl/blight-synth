use std::fmt::{Debug, Display};

mod audio_processor;
mod blight_audio;
mod commands;
pub mod effects;
pub mod id;
mod resources;
mod synth_infra;
mod synths;
mod voice_factory;

pub(crate) use audio_processor::*;
pub use blight_audio::*;
pub use commands::*;
pub use resources::*;
pub use synth_infra::SampleData;
pub(crate) use synth_infra::*;
pub use synths::*;
pub use voice_factory::*;

pub type Result<T> = std::result::Result<T, AudioBackendError>;

pub struct AudioBackendError(pub String);

impl Display for AudioBackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AudioBackendError: {}", self.0)
    }
}

impl std::error::Error for AudioBackendError {}

impl Debug for AudioBackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AudioBackendError: {}", self.0)
    }
}

impl From<hound::Error> for AudioBackendError {
    fn from(err: hound::Error) -> Self {
        AudioBackendError(format!("Hound error: {}", err))
    }
}
