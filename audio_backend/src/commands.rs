pub use engine::{EngineCommand, InstrumentCmd, MixerCmd};
use sequencer::models::Song;
use std::sync::Arc;

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
