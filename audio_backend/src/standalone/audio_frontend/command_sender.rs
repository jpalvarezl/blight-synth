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

type UnboxedSubmissionResult = Result<(), (CommandSubmissionErrorKind, Command)>;

/// Owns the producer side of the bounded callback command queue.
pub(crate) struct CommandSender {
    producer: HeapProd<Command>,
}

impl CommandSender {
    pub(crate) fn new(producer: HeapProd<Command>) -> Self {
        Self { producer }
    }

    pub(crate) fn try_send(&mut self, command: Command) -> CommandSubmissionResult {
        self.try_send_unboxed(command)
            .map_err(|(kind, command)| CommandSubmissionError::new(kind, command))
    }

    /// Reliably submits one command in FIFO order from a caller-owned NRT
    /// thread. A full queue applies producer backpressure: this call retains
    /// the command and cooperatively yields until RT frees a slot. It returns
    /// an error only when the callback-side consumer disconnects.
    pub(crate) fn send(&mut self, command: Command) -> CommandSubmissionResult {
        self.send_until(command, || false)
    }

    pub(crate) fn send_until(
        &mut self,
        mut command: Command,
        cancelled: impl Fn() -> bool,
    ) -> CommandSubmissionResult {
        loop {
            if cancelled() {
                return Err(CommandSubmissionError::new(
                    CommandSubmissionErrorKind::Full,
                    command,
                ));
            }
            match self.try_send_unboxed(command) {
                Ok(()) => return Ok(()),
                Err((CommandSubmissionErrorKind::Full, rejected)) => {
                    command = rejected;
                    std::thread::yield_now();
                }
                Err((CommandSubmissionErrorKind::Disconnected, rejected)) => {
                    return Err(CommandSubmissionError::new(
                        CommandSubmissionErrorKind::Disconnected,
                        rejected,
                    ));
                }
            }
        }
    }

    #[allow(
        clippy::result_large_err,
        reason = "the private unboxed path lets blocking submission retry the exact non-Clone command without allocating on each full-queue attempt"
    )]
    fn try_send_unboxed(&mut self, command: Command) -> UnboxedSubmissionResult {
        if !self.producer.read_is_held() {
            return Err((CommandSubmissionErrorKind::Disconnected, command));
        }

        match self.producer.try_push(command) {
            Ok(()) => Ok(()),
            Err(command) if !self.producer.read_is_held() => {
                Err((CommandSubmissionErrorKind::Disconnected, command))
            }
            Err(command) => Err((CommandSubmissionErrorKind::Full, command)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TransportCmd;
    use ringbuf::{storage::Heap, traits::Split, SharedRb};

    fn play_command() -> Command {
        TransportCmd::PlayLastSong.into()
    }

    fn stop_command() -> Command {
        TransportCmd::StopSong.into()
    }

    #[test]
    fn try_send_reports_saturation_returns_the_command_and_recovers() {
        let rb = SharedRb::<Heap<Command>>::new(2);
        let (command_tx, mut command_rx) = rb.split();
        let mut sender = CommandSender::new(command_tx);

        assert!(sender.try_send(play_command()).is_ok());
        assert!(sender.try_send(play_command()).is_ok());
        let rejected_command = match sender.try_send(stop_command()) {
            Err(error) => {
                assert_eq!(error.kind(), CommandSubmissionErrorKind::Full);
                error.into_command()
            }
            Ok(()) => panic!("expected a full queue rejection"),
        };
        assert!(matches!(
            &rejected_command,
            Command::Transport(TransportCmd::StopSong)
        ));

        assert!(command_rx.try_pop().is_some());
        assert!(sender.try_send(rejected_command).is_ok());
    }

    #[test]
    fn send_waits_through_saturation_without_reordering_commands() {
        let rb = SharedRb::<Heap<Command>>::new(1);
        let (command_tx, mut command_rx) = rb.split();
        let mut sender = CommandSender::new(command_tx);
        assert!(sender.try_send(play_command()).is_ok());
        let rejected_command = match sender.try_send(stop_command()) {
            Err(error) => {
                assert_eq!(error.kind(), CommandSubmissionErrorKind::Full);
                error.into_command()
            }
            Ok(()) => panic!("expected a full queue rejection"),
        };

        let consumer = std::thread::spawn(move || {
            let first = loop {
                if let Some(command) = command_rx.try_pop() {
                    break command;
                }
                std::thread::yield_now();
            };
            let second = loop {
                if let Some(command) = command_rx.try_pop() {
                    break command;
                }
                std::thread::yield_now();
            };
            (first, second)
        });

        assert!(sender.send(rejected_command).is_ok());
        let (first, second) = consumer.join().expect("consumer thread must finish");
        assert!(matches!(
            first,
            Command::Transport(TransportCmd::PlayLastSong)
        ));
        assert!(matches!(second, Command::Transport(TransportCmd::StopSong)));
    }

    #[test]
    fn try_send_and_send_report_disconnection_and_return_the_command() {
        let rb = SharedRb::<Heap<Command>>::new(1);
        let (command_tx, command_rx) = rb.split();
        let mut sender = CommandSender::new(command_tx);
        drop(command_rx);

        let rejected_command = match sender.try_send(play_command()) {
            Err(error) => {
                assert_eq!(error.kind(), CommandSubmissionErrorKind::Disconnected);
                error.into_command()
            }
            Ok(()) => panic!("expected a disconnected queue rejection"),
        };
        let rejected_command = match sender.send(rejected_command) {
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
