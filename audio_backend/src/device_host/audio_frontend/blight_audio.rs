use super::{BlightAudio, CommandSender, CommandSubmissionResult};
use crate::{
    prepare_initial_parameter_generation, AudioProcessor, Command, DeviceHostParameterFacade,
    EffectFactory, InstrumentFactory, MeterState, ResourceManager, RetiredState, VoiceFactory,
};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use log::{info, warn};
use ringbuf::storage::Heap;
use ringbuf::traits::*;
use ringbuf::SharedRb;
use sequencer::models::Song;
use std::sync::Arc;

const RETIREMENT_QUEUE_CAPACITY: usize = 128;

impl BlightAudio {
    pub fn new() -> Result<Self, anyhow::Error> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("no output device available");
        let config = device.default_output_config()?.config();
        let sample_rate = config.sample_rate.0;
        let channels = config.channels as usize;

        // Create the SPSC ring buffer for commands using a heap-allocated buffer.
        // let rb = HeapRb::<Command>::new(1024); // Capacity for 1024 commands
        let rb = SharedRb::<Heap<Command>>::new(1024);
        let (command_tx, command_rx) = rb.split();
        let retirement_rb = SharedRb::<Heap<RetiredState>>::new(RETIREMENT_QUEUE_CAPACITY);
        let (retirement_tx, retirement_rx) = retirement_rb.split();

        let command_sender = CommandSender::new(command_tx);

        // Prepare the complete initial parameter generation before the
        // real-time processor can enter the audio callback.
        let (parameter_state, parameters) = prepare_initial_parameter_generation()?.into_parts();
        let meter = Arc::new(MeterState::new());
        let mut audio_processor = AudioProcessor::new(
            command_rx,
            retirement_tx,
            sample_rate as f32,
            channels,
            meter.clone(),
            parameter_state,
        );

        let stream = device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                // This closure is the audio callback.
                audio_processor.process(data);
            },
            |err| eprintln!("an error occurred on stream: {}", err),
            None,
        )?;

        let resource_manager = ResourceManager::new();
        let voice_factory = VoiceFactory::new(sample_rate as f32);
        let effect_factory = EffectFactory::new(sample_rate as f32);
        let instrument_factory = InstrumentFactory::new(sample_rate as f32);

        stream.play()?;

        Ok(BlightAudio {
            command_sender,
            instrument_factory,
            voice_factory,
            resource_manager,
            effect_factory,
            meter,
            _stream: stream,
            retirement_rx,
            parameters,
        })
    }

    pub fn with_song(song: Arc<Song>) -> Result<Self, anyhow::Error> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("no output device available");
        let config = device.default_output_config()?.config();
        let sample_rate = config.sample_rate.0;
        let channels = config.channels as usize;

        info!("Audio output device: {}", device.name()?);
        info!("Default output config: {:?}", config);
        info!("Sample rate: {}", sample_rate);
        info!("Channels: {}", channels);

        // Create the SPSC ring buffer for commands using a heap-allocated buffer.
        let rb = SharedRb::<Heap<Command>>::new(1024);
        let (command_tx, command_rx) = rb.split();
        let retirement_rb = SharedRb::<Heap<RetiredState>>::new(RETIREMENT_QUEUE_CAPACITY);
        let (retirement_tx, retirement_rx) = retirement_rb.split();

        let command_sender = CommandSender::new(command_tx);

        // Prepare the complete initial parameter generation before the
        // real-time processor seeded with a Song can enter the callback.
        let (parameter_state, parameters) = prepare_initial_parameter_generation()?.into_parts();
        let meter = Arc::new(MeterState::new());
        let mut audio_processor = AudioProcessor::new_with_song(
            song,
            command_rx,
            retirement_tx,
            sample_rate as f32,
            channels,
            meter.clone(),
            parameter_state,
        );

        let stream = device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                audio_processor.process(data);
            },
            |err| eprintln!("an error occurred on stream: {}", err),
            None,
        )?;

        let resource_manager = ResourceManager::new();
        let voice_factory = VoiceFactory::new(sample_rate as f32);
        let effect_factory = EffectFactory::new(sample_rate as f32);
        let instrument_factory = InstrumentFactory::new(sample_rate as f32);

        stream.play()?;

        Ok(BlightAudio {
            command_sender,
            instrument_factory,
            voice_factory,
            resource_manager,
            effect_factory,
            meter,
            _stream: stream,
            retirement_rx,
            parameters,
        })
    }

    /// Attempts to submit one command without blocking.
    ///
    /// `Full` and `Disconnected` return the original command in the error.
    /// Callers that acknowledge state changes must do so only after `Ok(())`.
    pub fn try_send_command(&mut self, command: Command) -> CommandSubmissionResult {
        self.reclaim_retired();
        self.command_sender.try_send(command)
    }

    /// Reliably submits one command from a caller-owned non-real-time thread.
    ///
    /// A full queue applies producer backpressure: this method retains the
    /// command and parks briefly until the callback frees a slot, so a later
    /// command cannot overtake it. It returns an error only when the
    /// callback-side consumer disconnects. Callers must not invoke this method
    /// from a real-time, UI, or async-executor thread.
    pub fn send_command(&mut self, command: Command) -> CommandSubmissionResult {
        self.reclaim_retired();
        self.command_sender.send(command)
    }

    /// Reliably submits one command while allowing an NRT owner to cancel a
    /// saturation wait during shutdown. Cancellation returns `Full` with the
    /// original command; repeated retries stay unboxed internally.
    pub fn send_command_until(
        &mut self,
        command: Command,
        cancelled: impl Fn() -> bool,
    ) -> CommandSubmissionResult {
        self.reclaim_retired();
        self.command_sender.send_until(command, cancelled)
    }

    /// Stops the real-time callback and reclaims every retired owner still in
    /// flight, on this NRT caller.
    ///
    /// This is the shutdown sequencing that makes the exactly-once reclamation
    /// guarantee independent of struct field declaration order: pausing the
    /// stream first ensures the callback can no longer push to (or hold
    /// ownership feeding) the retirement ring, so the subsequent drain — and
    /// the later field-order drop of the callback-owned `AudioProcessor` and
    /// `retirement_rx` — each destroy their owners exactly once, off the RT
    /// path. See the [`Drop`] impl below.
    fn stop_and_reclaim(&mut self) -> usize {
        // Pausing may be unsupported on some hosts; the exactly-once guarantee
        // still holds because each owner lives in exactly one place (in-ring,
        // in `pending_retired`, or live in the player) and is dropped once when
        // that place drops on this NRT thread. Pausing only tightens RT-safety
        // by preventing a concurrent callback from racing the drain.
        if let Err(err) = self._stream.pause() {
            warn!("failed to pause audio stream during shutdown: {err}");
        }
        self.reclaim_retired()
    }

    /// Drops all currently retired owners on this NRT caller.
    pub fn reclaim_retired(&mut self) -> usize {
        let mut reclaimed = 0;
        while let Some(retired) = self.retirement_rx.try_pop() {
            drop(retired);
            reclaimed += 1;
        }
        reclaimed
    }

    pub fn get_voice_factory(&self) -> &VoiceFactory {
        &self.voice_factory
    }

    pub fn get_resource_manager(&mut self) -> &mut ResourceManager {
        &mut self.resource_manager
    }

    pub fn get_effect_factory(&self) -> &EffectFactory {
        &self.effect_factory
    }

    pub fn get_instrument_factory(&self) -> &InstrumentFactory {
        &self.instrument_factory
    }

    /// Returns a handle to the shared metering state. Cloning is cheap (an
    /// `Arc` bump); callers read levels via [`MeterState::take_levels`].
    pub fn meter_state(&self) -> Arc<MeterState> {
        self.meter.clone()
    }

    /// Clone the NRT stable-ID facade for this static parameter generation.
    /// The clone and its final drop must remain off the audio callback.
    pub fn parameter_facade(&self) -> DeviceHostParameterFacade {
        self.parameters.clone()
    }
}

impl Drop for BlightAudio {
    fn drop(&mut self) {
        // Explicit shutdown sequencing so the exactly-once reclamation guarantee
        // no longer depends on struct field declaration order: stop the RT
        // callback, then drain the retirement ring on this NRT thread. After
        // this returns, the remaining fields drop in declaration order — the
        // callback-owned `AudioProcessor` (with its `pending_retired` buffer and
        // live player song/instruments) and then `retirement_rx` — each
        // destroying their owners exactly once. A future field reorder cannot
        // reintroduce an RT-thread drop or a double-drop.
        self.stop_and_reclaim();
        self.parameters.disconnect();
    }
}
