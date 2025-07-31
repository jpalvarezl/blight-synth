use std::fmt::{Debug, Display};

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
