use super::{BlightAudio, CommandSender, CommandSubmissionResult};
use crate::{
    AudioProcessor, Command, EffectFactory, InstrumentFactory, MeterState, ResourceManager,
    VoiceFactory,
};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use log::info;
use ringbuf::storage::Heap;
use ringbuf::traits::*;
use ringbuf::SharedRb;
use sequencer::models::Song;
use std::sync::Arc;

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

        let command_sender = CommandSender::new(command_tx);

        // Create the real-time processor and move it into the audio thread.
        let meter = Arc::new(MeterState::new());
        let mut audio_processor =
            AudioProcessor::new(command_rx, sample_rate as f32, channels, meter.clone());

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

        let command_sender = CommandSender::new(command_tx);

        // Create the real-time processor seeded with a Song.
        let meter = Arc::new(MeterState::new());
        let mut audio_processor = AudioProcessor::new_with_song(
            song,
            command_rx,
            sample_rate as f32,
            channels,
            meter.clone(),
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
        })
    }

    /// Attempts to submit one command without blocking.
    ///
    /// `Full` and `Disconnected` return the original command in the error.
    /// Callers that acknowledge state changes must do so only after `Ok(())`.
    pub fn try_send_command(&mut self, command: Command) -> CommandSubmissionResult {
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
        self.command_sender.send_until(command, cancelled)
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
}
