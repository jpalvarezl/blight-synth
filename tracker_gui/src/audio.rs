use crate::instrument_manager::backend::{
    hydrate_instrument_on_worker, send_amp_envelope_on_worker,
};
use audio_backend::{BlightAudio, Command, CommandSubmissionErrorKind, SequencerCmd, TransportCmd};
use sequencer::models::{AmpEnvelopeParams, InstrumentData, Song};
#[cfg(test)]
use std::time::Duration;
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender},
    },
    thread::{self, JoinHandle},
};

// Tracker GUI reuses a single effect id until proper routing is needed.
pub const TRACKER_EFFECT_ID: audio_backend::id::EffectId = 1;
#[cfg(test)]
const WORKER_POLL_INTERVAL: Duration = Duration::from_millis(1);

enum AudioRequest {
    Initialize(Arc<Song>),
    Reset(Arc<Song>),
    Play(Arc<Song>),
    Stop,
    SetLooping(bool),
    HydrateInstrument {
        id: u8,
        data: InstrumentData,
    },
    SetAmpEnvelope {
        instrument_id: u8,
        envelope: AmpEnvelopeParams,
    },
    Command(Box<Command>),
    #[cfg(test)]
    Probe {
        label: &'static str,
        observed: Sender<&'static str>,
    },
    #[cfg(test)]
    Block {
        release: Arc<AtomicBool>,
        started: Sender<()>,
    },
}

enum AudioEvent {
    Initialized,
    Reset,
    Playing(bool),
    Looping(bool),
    Error(String),
}

/// UI-side handle for the dedicated tracker NRT audio-control worker.
pub struct AudioManager {
    request_tx: Option<Sender<AudioRequest>>,
    event_rx: Receiver<AudioEvent>,
    shutdown: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
    initialized: bool,
    pub is_playing: bool,
    pub loop_enabled: bool,
}

impl Default for AudioManager {
    fn default() -> Self {
        let (request_tx, request_rx) = mpsc::channel();
        let (event_tx, event_rx) = mpsc::channel();
        let shutdown = Arc::new(AtomicBool::new(false));
        let worker_shutdown = shutdown.clone();
        let worker = thread::Builder::new()
            .name("blight-tracker-audio-control".to_string())
            .spawn(move || run_audio_worker(request_rx, event_tx, worker_shutdown))
            .expect("failed to spawn tracker audio-control worker");

        Self {
            request_tx: Some(request_tx),
            event_rx,
            shutdown,
            worker: Some(worker),
            initialized: false,
            is_playing: false,
            loop_enabled: false,
        }
    }
}

impl AudioManager {
    pub fn poll(&mut self) {
        for event in self.event_rx.try_iter() {
            match event {
                AudioEvent::Initialized => {
                    self.initialized = true;
                    log::info!("Audio system initialized successfully");
                }
                AudioEvent::Reset => {
                    self.initialized = true;
                    self.is_playing = false;
                    log::info!("Audio system reset for loaded song");
                }
                AudioEvent::Playing(playing) => self.is_playing = playing,
                AudioEvent::Looping(enabled) => self.loop_enabled = enabled,
                AudioEvent::Error(error) => {
                    self.initialized = false;
                    self.is_playing = false;
                    log::error!("{error}");
                }
            }
        }
    }

    pub fn init_audio(&mut self, song: &Song) {
        if !self.initialized {
            self.send_request(AudioRequest::Initialize(Arc::new(song.clone())));
        }
    }

    pub fn reset_with_song(&mut self, song: &Song) {
        self.send_request(AudioRequest::Reset(Arc::new(song.clone())));
    }

    pub fn play_song(&mut self, song: &Song) {
        self.init_audio(song);
        self.send_request(AudioRequest::Play(Arc::new(song.clone())));
    }

    pub fn stop_song(&mut self) {
        self.send_request(AudioRequest::Stop);
    }

    pub fn toggle_playback(&mut self, song: &Song) {
        if self.is_playing {
            self.stop_song();
        } else {
            self.play_song(song);
        }
    }

    pub fn set_looping(&mut self, enabled: bool) {
        self.send_request(AudioRequest::SetLooping(enabled));
    }

    pub fn toggle_looping(&mut self) {
        self.set_looping(!self.loop_enabled);
    }

    pub fn dispatch(&mut self, cmd: impl Into<Command>) {
        self.send_request(AudioRequest::Command(Box::new(cmd.into())));
    }

    pub(crate) fn hydrate_instrument(&mut self, id: u8, data: InstrumentData) {
        self.send_request(AudioRequest::HydrateInstrument { id, data });
    }

    pub(crate) fn set_amp_envelope(&mut self, instrument_id: u8, envelope: AmpEnvelopeParams) {
        self.send_request(AudioRequest::SetAmpEnvelope {
            instrument_id,
            envelope,
        });
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    fn send_request(&mut self, request: AudioRequest) {
        let send_failed = self
            .request_tx
            .as_ref()
            .is_none_or(|request_tx| request_tx.send(request).is_err());
        if send_failed {
            self.request_tx.take();
            self.initialized = false;
            self.is_playing = false;
            log::error!("tracker audio-control worker is disconnected");
        }
    }
}

impl Drop for AudioManager {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Release);
        self.request_tx.take();
        if let Some(worker) = self.worker.take()
            && worker.join().is_err()
        {
            log::error!("tracker audio-control worker panicked during shutdown");
        }
    }
}

fn run_audio_worker(
    request_rx: Receiver<AudioRequest>,
    event_tx: Sender<AudioEvent>,
    shutdown: Arc<AtomicBool>,
) {
    let mut audio = None;
    let mut loop_enabled = false;

    while !shutdown.load(Ordering::Acquire) {
        let request = match request_rx.recv() {
            Ok(request) => request,
            Err(_) => break,
        };

        match request {
            AudioRequest::Initialize(song) => {
                if audio.is_none() {
                    match create_audio(&song, &shutdown, loop_enabled) {
                        Ok(created) => {
                            audio = Some(created);
                            let _ = event_tx.send(AudioEvent::Initialized);
                        }
                        Err(error) => {
                            let _ = event_tx.send(AudioEvent::Error(error));
                        }
                    }
                }
            }
            AudioRequest::Reset(song) => match create_audio(&song, &shutdown, loop_enabled) {
                Ok(replacement) => {
                    audio = Some(replacement);
                    let _ = event_tx.send(AudioEvent::Reset);
                }
                Err(error) => {
                    audio = None;
                    let _ = event_tx.send(AudioEvent::Error(format!(
                        "Audio reset failed; playback disabled until re-initialization: {error}"
                    )));
                }
            },
            AudioRequest::Play(song) => {
                let accepted = audio.as_mut().is_some_and(|audio| {
                    submit_command(audio, SequencerCmd::PlaySong { song }.into(), &shutdown)
                });
                if accepted {
                    let _ = event_tx.send(AudioEvent::Playing(true));
                } else if audio.is_some() && !shutdown.load(Ordering::Acquire) {
                    audio = None;
                    let _ = event_tx.send(AudioEvent::Error(
                        "Audio callback disconnected while starting playback".to_string(),
                    ));
                }
            }
            AudioRequest::Stop => {
                let accepted = audio.as_mut().is_some_and(|audio| {
                    submit_command(audio, TransportCmd::StopSong.into(), &shutdown)
                });
                if accepted {
                    let _ = event_tx.send(AudioEvent::Playing(false));
                } else if audio.is_some() && !shutdown.load(Ordering::Acquire) {
                    audio = None;
                    let _ = event_tx.send(AudioEvent::Error(
                        "Audio callback disconnected while stopping playback".to_string(),
                    ));
                }
            }
            AudioRequest::SetLooping(enabled) => {
                loop_enabled = enabled;
                if audio.is_none() {
                    let _ = event_tx.send(AudioEvent::Looping(enabled));
                    continue;
                }
                let accepted = audio.as_mut().is_some_and(|audio| {
                    submit_command(
                        audio,
                        TransportCmd::SetLooping { enabled }.into(),
                        &shutdown,
                    )
                });
                if accepted {
                    let _ = event_tx.send(AudioEvent::Looping(enabled));
                } else if !shutdown.load(Ordering::Acquire) {
                    audio = None;
                    let _ = event_tx.send(AudioEvent::Error(
                        "Audio callback disconnected while setting loop state".to_string(),
                    ));
                }
            }
            AudioRequest::HydrateInstrument { id, data } => {
                let failed = audio.as_mut().is_some_and(|audio| {
                    !hydrate_instrument_on_worker(audio, id, &data, &shutdown)
                });
                if failed && !shutdown.load(Ordering::Acquire) {
                    audio = None;
                    let _ = event_tx.send(AudioEvent::Error(
                        "Audio callback disconnected while hydrating an instrument".to_string(),
                    ));
                }
            }
            AudioRequest::SetAmpEnvelope {
                instrument_id,
                envelope,
            } => {
                let failed = audio.as_mut().is_some_and(|audio| {
                    !send_amp_envelope_on_worker(audio, instrument_id, &envelope, &shutdown)
                });
                if failed && !shutdown.load(Ordering::Acquire) {
                    audio = None;
                    let _ = event_tx.send(AudioEvent::Error(
                        "Audio callback disconnected while updating an envelope".to_string(),
                    ));
                }
            }
            AudioRequest::Command(command) => {
                let failed = audio
                    .as_mut()
                    .is_some_and(|audio| !submit_command(audio, *command, &shutdown));
                if failed && !shutdown.load(Ordering::Acquire) {
                    audio = None;
                    let _ = event_tx.send(AudioEvent::Error(
                        "Audio callback disconnected while submitting a command".to_string(),
                    ));
                }
            }
            #[cfg(test)]
            AudioRequest::Probe { label, observed } => {
                let _ = observed.send(label);
            }
            #[cfg(test)]
            AudioRequest::Block { release, started } => {
                let _ = started.send(());
                while !release.load(Ordering::Acquire) && !shutdown.load(Ordering::Acquire) {
                    thread::park_timeout(WORKER_POLL_INTERVAL);
                }
            }
        }
    }
}

fn create_audio(
    song: &Arc<Song>,
    shutdown: &AtomicBool,
    loop_enabled: bool,
) -> Result<BlightAudio, String> {
    match BlightAudio::with_song(song.clone()) {
        Ok(mut audio) => {
            for instrument in &song.instrument_bank {
                if !hydrate_instrument_on_worker(
                    &mut audio,
                    instrument.id as u8,
                    &instrument.data,
                    shutdown,
                ) {
                    return Err(if shutdown.load(Ordering::Acquire) {
                        "audio worker shutdown interrupted song hydration".to_string()
                    } else {
                        "audio callback disconnected while hydrating the song".to_string()
                    });
                }
            }
            if !submit_command(
                &mut audio,
                TransportCmd::SetLooping {
                    enabled: loop_enabled,
                }
                .into(),
                shutdown,
            ) {
                return Err(if shutdown.load(Ordering::Acquire) {
                    "audio worker shutdown interrupted loop-state setup".to_string()
                } else {
                    "audio callback disconnected while setting loop state".to_string()
                });
            }
            Ok(audio)
        }
        Err(error) => Err(format!("Failed to initialize audio system: {error}")),
    }
}

pub(crate) fn submit_command(
    audio: &mut BlightAudio,
    command: Command,
    shutdown: &AtomicBool,
) -> bool {
    match audio.send_command_until(command, || shutdown.load(Ordering::Acquire)) {
        Ok(()) => true,
        Err(error) => match error.kind() {
            CommandSubmissionErrorKind::Full => false,
            CommandSubmissionErrorKind::Disconnected => {
                log::error!("audio callback is disconnected");
                false
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worker_processes_requests_in_fifo_order() {
        let mut manager = AudioManager::default();
        let (observed_tx, observed_rx) = mpsc::channel();
        for label in ["first", "second", "third"] {
            manager.send_request(AudioRequest::Probe {
                label,
                observed: observed_tx.clone(),
            });
        }

        assert_eq!(
            [
                observed_rx.recv_timeout(Duration::from_secs(1)).unwrap(),
                observed_rx.recv_timeout(Duration::from_secs(1)).unwrap(),
                observed_rx.recv_timeout(Duration::from_secs(1)).unwrap(),
            ],
            ["first", "second", "third"]
        );
    }

    #[test]
    fn later_requests_cannot_overtake_blocked_work_and_shutdown_is_cancellable() {
        let mut manager = AudioManager::default();
        let release = Arc::new(AtomicBool::new(false));
        let (started_tx, started_rx) = mpsc::channel();
        manager.send_request(AudioRequest::Block {
            release: release.clone(),
            started: started_tx,
        });
        started_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("worker must start blocked request");

        let (observed_tx, observed_rx) = mpsc::channel();
        manager.send_request(AudioRequest::Probe {
            label: "after",
            observed: observed_tx,
        });
        assert!(observed_rx.try_recv().is_err());

        release.store(true, Ordering::Release);
        assert_eq!(
            observed_rx.recv_timeout(Duration::from_secs(1)).unwrap(),
            "after"
        );

        drop(manager);
    }

    #[test]
    fn shutdown_interrupts_blocked_work() {
        let mut manager = AudioManager::default();
        let release = Arc::new(AtomicBool::new(false));
        let (started_tx, started_rx) = mpsc::channel();
        manager.send_request(AudioRequest::Block {
            release,
            started: started_tx,
        });
        started_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("worker must start blocked request");

        drop(manager);
    }
}
