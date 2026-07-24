mod commands;
mod offline;
mod player;
mod resources;
mod result;
mod song_hydration;
#[cfg(feature = "device-host")]
pub mod device_host;
#[cfg(feature = "standalone-process")]
pub mod standalone_process;

// Re-export reusable layers so existing audio_backend consumers can migrate incrementally.
pub use dsp::*;
pub use engine::*;

pub use commands::*;
pub use offline::*;
pub(crate) use player::*;
pub use resources::*;
pub use result::*;
pub use song_hydration::*;
#[cfg(feature = "device-host")]
pub use device_host::*;
#[cfg(feature = "standalone-process")]
pub use standalone_process::*;
