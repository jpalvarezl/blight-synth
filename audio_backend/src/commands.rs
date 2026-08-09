pub use engine::{EngineCommand, InstrumentCmd, MixerCmd};
use sequencer::models::Song;
use std::sync::Arc;

#[cfg(feature = "device-host")]
use engine::PreparedCoalescedParameterState;

/// Opaque lifecycle-validated parameter replacement payload.
#[cfg(feature = "device-host")]
#[doc(hidden)]
pub struct ParameterGenerationCommand(Arc<PreparedCoalescedParameterState>);

#[cfg(feature = "device-host")]
impl ParameterGenerationCommand {
    pub(crate) fn new(state: Arc<PreparedCoalescedParameterState>) -> Self {
        Self(state)
    }

    pub(crate) fn into_state(self) -> Arc<PreparedCoalescedParameterState> {
        self.0
    }
}

pub enum TransportCmd {
    PlayLastSong,
    StopSong,
    SetLooping { enabled: bool },
}

pub enum SequencerCmd {
    /// Replace the current song without starting playback.
    LoadSong {
        song: Arc<Song>,
    },
    PlaySong {
        song: Arc<Song>,
    },
}

/// Standalone/tracker command envelope.
///
/// Instrument and mixer payload types are owned by `engine` and re-exported
/// here for compatibility. Transport and song commands remain adapter-owned.
#[allow(
    clippy::large_enum_variant,
    reason = "Command contains the intentionally inline engine InstrumentCmd payload"
)]
pub enum Command {
    Transport(TransportCmd),
    Sequencer(SequencerCmd),
    Mixer(MixerCmd),
    Instrument(InstrumentCmd),
    /// NRT-prepared whole generation installed at the next callback boundary.
    #[cfg(feature = "device-host")]
    ReplaceParameterGeneration(ParameterGenerationCommand),
}

impl From<TransportCmd> for Command {
    fn from(value: TransportCmd) -> Self {
        Self::Transport(value)
    }
}

impl From<SequencerCmd> for Command {
    fn from(value: SequencerCmd) -> Self {
        Self::Sequencer(value)
    }
}

impl From<MixerCmd> for Command {
    fn from(value: MixerCmd) -> Self {
        Self::Mixer(value)
    }
}

impl From<InstrumentCmd> for Command {
    fn from(value: InstrumentCmd) -> Self {
        Self::Instrument(value)
    }
}

impl From<EngineCommand> for Command {
    fn from(value: EngineCommand) -> Self {
        match value {
            EngineCommand::Instrument(command) => Self::Instrument(command),
            EngineCommand::Mixer(command) => Self::Mixer(command),
        }
    }
}
