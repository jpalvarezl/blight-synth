use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, RecvTimeoutError, SyncSender, TrySendError},
        Arc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use rosc::OscPacket;
use sequencer::models::Song;

use crate::{
    prepare_song_file_for_audio, AudioBackendError, BlightAudio, Command,
    CommandSubmissionErrorKind, CommandSubmissionResult, MeterState, Result as BackendResult,
};

use super::osc::{song_load_error, song_loaded};

const CONTROL_REQUEST_CAPACITY: usize = 1024;
const WORKER_POLL_INTERVAL: Duration = Duration::from_millis(1);

/// An internal engine command translated from OSC plus the protocol response
/// that may be emitted only after RT-ring acceptance.
pub(crate) struct OscCommandRequest {
    pub(crate) command: Command,
    pub(crate) accepted_response: Option<OscPacket>,
}

enum ControlRequest {
    Commands(Vec<OscCommandRequest>),
    LoadSong(PathBuf),
}

/// Narrow worker backend seam used to test FIFO/retry/shutdown behavior without
/// constructing a hardware CPAL stream. `BlightAudio` is the only production
/// implementation.
trait ControlTarget {
    fn try_submit(&mut self, command: Command) -> CommandSubmissionResult;
    fn prepare_song(&self, path: &Path) -> BackendResult<(Song, Vec<Command>)>;
}

impl ControlTarget for BlightAudio {
    fn try_submit(&mut self, command: Command) -> CommandSubmissionResult {
        self.try_send_command(command)
    }

    fn prepare_song(&self, path: &Path) -> BackendResult<(Song, Vec<Command>)> {
        prepare_song_file_for_audio(self, path)
            .map_err(|error| AudioBackendError(error.to_string()))
    }
}

/// Dedicated NRT owner for standalone command preparation and reliable FIFO
/// submission. The Tokio/OSC thread only enqueues bounded requests and polls
/// completed protocol responses.
struct RunningGuard(Arc<AtomicBool>);

impl Drop for RunningGuard {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

pub struct StandaloneControlWorker {
    request_tx: Option<SyncSender<ControlRequest>>,
    response_rx: Receiver<OscPacket>,
    shutdown: Arc<AtomicBool>,
    running: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl StandaloneControlWorker {
    /// Creates `BlightAudio` on the dedicated worker because CPAL streams are
    /// intentionally not `Send` across platform boundaries.
    pub fn spawn() -> BackendResult<(Self, Arc<MeterState>)> {
        let (request_tx, request_rx) = mpsc::sync_channel(CONTROL_REQUEST_CAPACITY);
        let (response_tx, response_rx) = mpsc::channel();
        let (startup_tx, startup_rx) = mpsc::sync_channel(1);
        let shutdown = Arc::new(AtomicBool::new(false));
        let worker_shutdown = shutdown.clone();
        let running = Arc::new(AtomicBool::new(true));
        let worker_running = running.clone();
        let thread = thread::Builder::new()
            .name("blight-standalone-control".to_string())
            .spawn(move || {
                let _running_guard = RunningGuard(worker_running);
                let startup = (|| -> BackendResult<(BlightAudio, Arc<MeterState>)> {
                    let audio =
                        BlightAudio::new().map_err(|error| AudioBackendError(error.to_string()))?;
                    // The device-host constructor already installed the reserved
                    // master gain and its initial coalesced generation on NRT.
                    let meter = audio.meter_state();
                    Ok((audio, meter))
                })();

                match startup {
                    Ok((audio, meter)) => {
                        if startup_tx.send(Ok(meter)).is_ok() {
                            run_worker(audio, request_rx, response_tx, worker_shutdown);
                        }
                    }
                    Err(error) => {
                        let _ = startup_tx.send(Err(error.to_string()));
                    }
                }
            })
            .map_err(|error| AudioBackendError(error.to_string()))?;

        let meter = startup_rx
            .recv()
            .map_err(|_| {
                AudioBackendError("standalone control worker exited during startup".to_string())
            })?
            .map_err(AudioBackendError)?;
        Ok((
            Self {
                request_tx: Some(request_tx),
                response_rx,
                shutdown,
                running,
                thread: Some(thread),
            },
            meter,
        ))
    }

    pub(crate) fn try_submit_commands(
        &self,
        submissions: Vec<OscCommandRequest>,
    ) -> std::result::Result<(), TrySendError<Vec<OscCommandRequest>>> {
        let Some(request_tx) = &self.request_tx else {
            return Err(TrySendError::Disconnected(submissions));
        };
        match request_tx.try_send(ControlRequest::Commands(submissions)) {
            Ok(()) => Ok(()),
            Err(TrySendError::Full(ControlRequest::Commands(submissions))) => {
                Err(TrySendError::Full(submissions))
            }
            Err(TrySendError::Disconnected(ControlRequest::Commands(submissions))) => {
                Err(TrySendError::Disconnected(submissions))
            }
            Err(TrySendError::Full(ControlRequest::LoadSong(_)))
            | Err(TrySendError::Disconnected(ControlRequest::LoadSong(_))) => {
                unreachable!("submitted a command request")
            }
        }
    }

    pub(crate) fn try_load_song(
        &self,
        path: PathBuf,
    ) -> std::result::Result<(), TrySendError<PathBuf>> {
        let Some(request_tx) = &self.request_tx else {
            return Err(TrySendError::Disconnected(path));
        };
        match request_tx.try_send(ControlRequest::LoadSong(path)) {
            Ok(()) => Ok(()),
            Err(TrySendError::Full(ControlRequest::LoadSong(path))) => {
                Err(TrySendError::Full(path))
            }
            Err(TrySendError::Disconnected(ControlRequest::LoadSong(path))) => {
                Err(TrySendError::Disconnected(path))
            }
            Err(TrySendError::Full(ControlRequest::Commands(_)))
            | Err(TrySendError::Disconnected(ControlRequest::Commands(_))) => {
                unreachable!("submitted a song-load request")
            }
        }
    }

    pub(crate) fn drain_responses(&mut self) -> Vec<OscPacket> {
        self.response_rx.try_iter().collect()
    }

    pub(crate) fn is_running(&self) -> bool {
        self.running.load(Ordering::Acquire)
    }

    pub fn shutdown(mut self) {
        self.stop_and_join();
    }

    fn stop_and_join(&mut self) {
        self.shutdown.store(true, Ordering::Release);
        self.request_tx.take();
        if let Some(thread) = self.thread.take() {
            if thread.join().is_err() {
                log::error!("standalone control worker panicked during shutdown");
            }
        }
    }
}

impl Drop for StandaloneControlWorker {
    fn drop(&mut self) {
        self.stop_and_join();
    }
}

fn run_worker<T>(
    mut target: T,
    request_rx: Receiver<ControlRequest>,
    response_tx: mpsc::Sender<OscPacket>,
    shutdown: Arc<AtomicBool>,
) where
    T: ControlTarget,
{
    let mut pending = VecDeque::new();
    let mut batch_response = None;

    while !shutdown.load(Ordering::Acquire) {
        if pending.is_empty() {
            if let Some(response) = batch_response.take() {
                if response_tx.send(response).is_err() {
                    break;
                }
            }

            match request_rx.recv_timeout(WORKER_POLL_INTERVAL) {
                Ok(ControlRequest::Commands(submissions)) => pending.extend(submissions),
                Ok(ControlRequest::LoadSong(path)) => match target.prepare_song(&path) {
                    Ok((song, commands)) => {
                        pending.extend(commands.into_iter().map(|command| OscCommandRequest {
                            command,
                            accepted_response: None,
                        }));
                        batch_response = Some(song_loaded(&path, &song.name));
                    }
                    Err(error) => {
                        let response = song_load_error(&path, &error.to_string());
                        if response_tx.send(response).is_err() {
                            break;
                        }
                    }
                },
                Err(RecvTimeoutError::Timeout) => continue,
                Err(RecvTimeoutError::Disconnected) => break,
            }
            continue;
        }

        let submission = pending.pop_front().expect("pending queue is not empty");
        match target.try_submit(submission.command) {
            Ok(()) => {
                if let Some(response) = submission.accepted_response {
                    if response_tx.send(response).is_err() {
                        break;
                    }
                }
            }
            Err(error) => match error.kind() {
                CommandSubmissionErrorKind::Full => {
                    pending.push_front(OscCommandRequest {
                        command: error.into_command(),
                        accepted_response: submission.accepted_response,
                    });
                    thread::park_timeout(WORKER_POLL_INTERVAL);
                }
                CommandSubmissionErrorKind::Disconnected => {
                    log::error!("standalone control worker lost the audio callback");
                    break;
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Mutex,
    };

    use super::*;
    use crate::{CommandSubmissionError, TransportCmd};
    use rosc::{OscMessage, OscType};

    fn spawn_test_worker<T>(target: T) -> StandaloneControlWorker
    where
        T: ControlTarget + Send + 'static,
    {
        let (request_tx, request_rx) = mpsc::sync_channel(CONTROL_REQUEST_CAPACITY);
        let (response_tx, response_rx) = mpsc::channel();
        let shutdown = Arc::new(AtomicBool::new(false));
        let worker_shutdown = shutdown.clone();
        let running = Arc::new(AtomicBool::new(true));
        let worker_running = running.clone();
        let thread = thread::Builder::new()
            .name("blight-standalone-control-test".to_string())
            .spawn(move || {
                let _running_guard = RunningGuard(worker_running);
                run_worker(target, request_rx, response_tx, worker_shutdown);
            })
            .expect("failed to spawn standalone control test worker");

        StandaloneControlWorker {
            request_tx: Some(request_tx),
            response_rx,
            shutdown,
            running,
            thread: Some(thread),
        }
    }

    struct FakeTarget {
        accepting: Arc<AtomicBool>,
        attempts: Arc<AtomicUsize>,
        accepted: Arc<Mutex<Vec<&'static str>>>,
    }

    impl ControlTarget for FakeTarget {
        fn try_submit(&mut self, command: Command) -> CommandSubmissionResult {
            self.attempts.fetch_add(1, Ordering::Relaxed);
            if !self.accepting.load(Ordering::Acquire) {
                return Err(CommandSubmissionError::new(
                    CommandSubmissionErrorKind::Full,
                    command,
                ));
            }
            let label = match command {
                Command::Transport(TransportCmd::PlayLastSong) => "play",
                Command::Transport(TransportCmd::StopSong) => "stop",
                _ => "other",
            };
            self.accepted.lock().unwrap().push(label);
            Ok(())
        }

        fn prepare_song(&self, _path: &Path) -> BackendResult<(Song, Vec<Command>)> {
            Err(AudioBackendError("not used by this test".to_string()))
        }
    }

    struct DisconnectedTarget;

    impl ControlTarget for DisconnectedTarget {
        fn try_submit(&mut self, command: Command) -> CommandSubmissionResult {
            Err(CommandSubmissionError::new(
                CommandSubmissionErrorKind::Disconnected,
                command,
            ))
        }

        fn prepare_song(&self, _path: &Path) -> BackendResult<(Song, Vec<Command>)> {
            Err(AudioBackendError("not used by this test".to_string()))
        }
    }

    fn response(value: &str) -> OscPacket {
        OscPacket::Message(OscMessage {
            addr: "/accepted".to_string(),
            args: vec![OscType::String(value.to_string())],
        })
    }

    #[test]
    fn saturation_retains_fifo_order_and_emits_responses_only_after_acceptance() {
        let accepting = Arc::new(AtomicBool::new(false));
        let attempts = Arc::new(AtomicUsize::new(0));
        let accepted = Arc::new(Mutex::new(Vec::new()));
        let mut worker = spawn_test_worker(FakeTarget {
            accepting: accepting.clone(),
            attempts,
            accepted: accepted.clone(),
        });
        assert!(worker
            .try_submit_commands(vec![
                OscCommandRequest {
                    command: TransportCmd::PlayLastSong.into(),
                    accepted_response: Some(response("play")),
                },
                OscCommandRequest {
                    command: TransportCmd::StopSong.into(),
                    accepted_response: Some(response("stop")),
                },
            ])
            .is_ok());

        thread::sleep(Duration::from_millis(5));
        assert!(accepted.lock().unwrap().is_empty());
        assert!(worker.drain_responses().is_empty());

        accepting.store(true, Ordering::Release);
        let deadline = std::time::Instant::now() + Duration::from_secs(1);
        while accepted.lock().unwrap().len() < 2 && std::time::Instant::now() < deadline {
            thread::sleep(Duration::from_millis(1));
        }

        assert_eq!(*accepted.lock().unwrap(), ["play", "stop"]);
        assert_eq!(worker.drain_responses().len(), 2);
    }

    #[test]
    fn bounded_request_queue_reports_full_without_reordering_accepted_work() {
        let accepting = Arc::new(AtomicBool::new(false));
        let attempts = Arc::new(AtomicUsize::new(0));
        let accepted = Arc::new(Mutex::new(Vec::new()));
        let worker = spawn_test_worker(FakeTarget {
            accepting,
            attempts: attempts.clone(),
            accepted,
        });
        assert!(worker
            .try_submit_commands(vec![OscCommandRequest {
                command: TransportCmd::PlayLastSong.into(),
                accepted_response: None,
            }])
            .is_ok());
        let deadline = std::time::Instant::now() + Duration::from_secs(1);
        while attempts.load(Ordering::Relaxed) == 0 && std::time::Instant::now() < deadline {
            thread::sleep(Duration::from_millis(1));
        }
        assert!(attempts.load(Ordering::Relaxed) > 0);

        for _ in 0..CONTROL_REQUEST_CAPACITY {
            assert!(worker
                .try_submit_commands(vec![OscCommandRequest {
                    command: TransportCmd::StopSong.into(),
                    accepted_response: None,
                }])
                .is_ok());
        }
        assert!(matches!(
            worker.try_submit_commands(vec![OscCommandRequest {
                command: TransportCmd::StopSong.into(),
                accepted_response: None,
            }]),
            Err(TrySendError::Full(_))
        ));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn saturated_worker_does_not_block_current_thread_executor() {
        let accepting = Arc::new(AtomicBool::new(false));
        let attempts = Arc::new(AtomicUsize::new(0));
        let accepted = Arc::new(Mutex::new(Vec::new()));
        let worker = spawn_test_worker(FakeTarget {
            accepting,
            attempts,
            accepted,
        });
        assert!(worker
            .try_submit_commands(vec![OscCommandRequest {
                command: TransportCmd::PlayLastSong.into(),
                accepted_response: None,
            }])
            .is_ok());

        tokio::time::timeout(Duration::from_millis(50), async {
            tokio::time::sleep(Duration::from_millis(1)).await;
        })
        .await
        .expect("Tokio timer must remain responsive while worker is saturated");
    }

    #[test]
    fn shutdown_completes_while_target_remains_saturated() {
        let accepting = Arc::new(AtomicBool::new(false));
        let attempts = Arc::new(AtomicUsize::new(0));
        let accepted = Arc::new(Mutex::new(Vec::new()));
        let worker = spawn_test_worker(FakeTarget {
            accepting,
            attempts: attempts.clone(),
            accepted,
        });
        assert!(worker
            .try_submit_commands(vec![OscCommandRequest {
                command: TransportCmd::PlayLastSong.into(),
                accepted_response: None,
            }])
            .is_ok());
        let deadline = std::time::Instant::now() + Duration::from_secs(1);
        while attempts.load(Ordering::Relaxed) == 0 && std::time::Instant::now() < deadline {
            thread::sleep(Duration::from_millis(1));
        }
        assert!(attempts.load(Ordering::Relaxed) > 0);

        worker.shutdown();
    }

    #[test]
    fn disconnection_stops_worker_and_rejects_later_requests() {
        let worker = spawn_test_worker(DisconnectedTarget);
        assert!(worker
            .try_submit_commands(vec![OscCommandRequest {
                command: TransportCmd::PlayLastSong.into(),
                accepted_response: None,
            }])
            .is_ok());

        let deadline = std::time::Instant::now() + Duration::from_secs(1);
        while worker.is_running() && std::time::Instant::now() < deadline {
            thread::sleep(Duration::from_millis(1));
        }
        assert!(!worker.is_running());
        assert!(matches!(
            worker.try_submit_commands(Vec::new()),
            Err(TrySendError::Disconnected(_))
        ));
    }
}
