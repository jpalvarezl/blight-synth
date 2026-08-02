mod commands;

/// Maximum frame count of one prepared render slice. Device callbacks larger
/// than this are chunked, and offline configuration rejects larger blocks.
pub(crate) const MAX_RENDER_SLICE_FRAMES: usize = 4_096;
#[cfg(feature = "device-host")]
pub mod device_host;
mod offline;
mod player;
mod resources;
mod result;
mod song_hydration;
#[cfg(feature = "standalone-process")]
pub mod standalone_process;

// Re-export reusable layers so existing audio_backend consumers can migrate incrementally.
pub use dsp::*;
pub use engine::*;

pub use commands::*;
#[cfg(feature = "device-host")]
pub use device_host::*;
pub use offline::*;
pub(crate) use player::Player;
pub use player::{EventLaneStatus, PlayerProcessStatus};
pub use resources::*;
pub use result::*;
pub use song_hydration::*;
#[cfg(feature = "standalone-process")]
pub use standalone_process::*;
