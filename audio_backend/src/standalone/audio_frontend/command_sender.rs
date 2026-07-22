use crate::Command;
use ringbuf::{traits::*, HeapProd};
use std::fmt;

/// Reason an audio command could not be submitted to the callback queue.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommandSubmissionErrorKind {
    /// The callback is connected, but the bounded queue has no free slot.
    Full,
    /// The callback-side queue consumer no longer exists.
    Disconnected,
}

/// A rejected command and the reason it was not accepted.
pub struct CommandSubmissionError(Box<CommandSubmissionErrorInner>);

struct CommandSubmissionErrorInner {
    kind: CommandSubmissionErrorKind,
    command: Command,
}

impl CommandSubmissionError {
    pub(crate) fn new(kind: CommandSubmissionErrorKind, command: Command) -> Self {
        Self(Box::new(CommandSubmissionErrorInner { kind, command }))
    }

    pub fn kind(&self) -> CommandSubmissionErrorKind {
        self.0.kind
    }

    pub fn into_command(self) -> Command {
        self.0.command
    }
}

impl fmt::Debug for CommandSubmissionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CommandSubmissionError")
            .field("kind", &self.kind())
            .finish_non_exhaustive()
    }
}

impl fmt::Display for CommandSubmissionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind() {
            CommandSubmissionErrorKind::Full => formatter.write_str("command queue is full"),
            CommandSubmissionErrorKind::Disconnected => {
                formatter.write_str("audio callback is disconnected")
            }
        }
    }
}

impl std::error::Error for CommandSubmissionError {}

/// Result of submitting one owned command to the callback queue.
///
/// Rejections box the error and original command together on the non-real-time
/// thread. This keeps the `Result` small without adding callback-side work.
pub type CommandSubmissionResult = Result<(), CommandSubmissionError>;

/// Owns the producer side of the bounded callback command queue.
pub(crate) struct CommandSender {
    producer: HeapProd<Command>,
}

impl CommandSender {
    pub(crate) fn new(producer: HeapProd<Command>) -> Self {
        Self { producer }
    }

    pub(crate) fn send(&mut self, command: Command) -> CommandSubmissionResult {
        if !self.producer.read_is_held() {
            return Err(CommandSubmissionError::new(
                CommandSubmissionErrorKind::Disconnected,
                command,
            ));
        }

        match self.producer.try_push(command) {
            Ok(()) => Ok(()),
            Err(command) if !self.producer.read_is_held() => Err(CommandSubmissionError::new(
                CommandSubmissionErrorKind::Disconnected,
                command,
            )),
            Err(command) => Err(CommandSubmissionError::new(
                CommandSubmissionErrorKind::Full,
                command,
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TransportCmd;
    use ringbuf::{storage::Heap, traits::Split, SharedRb};

    fn command() -> Command {
        TransportCmd::PlayLastSong.into()
    }

    #[test]
    fn reports_saturation_returns_the_command_and_recovers_after_space_is_freed() {
        let rb = SharedRb::<Heap<Command>>::new(2);
        let (command_tx, mut command_rx) = rb.split();
        let mut sender = CommandSender::new(command_tx);

        assert!(sender.send(command()).is_ok());
        assert!(sender.send(command()).is_ok());
        let rejected_command = match sender.send(command()) {
            Err(error) => {
                assert_eq!(error.kind(), CommandSubmissionErrorKind::Full);
                error.into_command()
            }
            Ok(()) => panic!("expected a full queue rejection"),
        };
        assert!(matches!(
            &rejected_command,
            Command::Transport(TransportCmd::PlayLastSong)
        ));

        assert!(command_rx.try_pop().is_some());
        assert!(sender.send(rejected_command).is_ok());
    }

    #[test]
    fn reports_disconnection_and_returns_the_command() {
        let rb = SharedRb::<Heap<Command>>::new(1);
        let (command_tx, command_rx) = rb.split();
        let mut sender = CommandSender::new(command_tx);
        drop(command_rx);

        let rejected_command = match sender.send(command()) {
            Err(error) => {
                assert_eq!(error.kind(), CommandSubmissionErrorKind::Disconnected);
                error.into_command()
            }
            Ok(()) => panic!("expected a disconnected queue rejection"),
        };
        assert!(matches!(
            rejected_command,
            Command::Transport(TransportCmd::PlayLastSong)
        ));
    }
}
