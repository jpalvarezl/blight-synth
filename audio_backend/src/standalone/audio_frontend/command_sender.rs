use crate::Command;
use ringbuf::{traits::*, HeapProd};

/// Reason an audio command could not be submitted to the callback queue.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommandSubmissionError {
    /// The callback is connected, but the bounded queue has no free slot.
    Full,
    /// The callback-side queue consumer no longer exists.
    Disconnected,
}

/// Result of submitting one owned command to the callback queue.
///
/// Rejections box and return the original command on the non-real-time thread.
/// The box keeps the `Result` small without adding callback-side allocation.
pub type CommandSubmissionResult = Result<(), (CommandSubmissionError, Box<Command>)>;

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
            return Err((CommandSubmissionError::Disconnected, Box::new(command)));
        }

        match self.producer.try_push(command) {
            Ok(()) => Ok(()),
            Err(command) if !self.producer.read_is_held() => {
                Err((CommandSubmissionError::Disconnected, Box::new(command)))
            }
            Err(command) => Err((CommandSubmissionError::Full, Box::new(command))),
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
            Err((CommandSubmissionError::Full, command)) => *command,
            _ => panic!("expected a full queue rejection"),
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
            Err((CommandSubmissionError::Disconnected, command)) => *command,
            _ => panic!("expected a disconnected queue rejection"),
        };
        assert!(matches!(
            rejected_command,
            Command::Transport(TransportCmd::PlayLastSong)
        ));
    }
}
