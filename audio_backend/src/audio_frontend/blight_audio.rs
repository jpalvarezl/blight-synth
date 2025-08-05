#![cfg(not(feature = "tracker"))]

use crate::{
    effect_factory::EffectFactory, AudioProcessor, Command, ResourceManager, VoiceFactory,
};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::storage::Heap;
use ringbuf::{HeapProd, SharedRb};
use ringbuf::{traits::*, HeapRb};
use super::BlightAudio;

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
    

        // Create the real-time processor and move it into the audio thread.
        let mut audio_processor = AudioProcessor::new(command_rx, sample_rate as f32, channels);

        let stream = device.build_output_stream(
            &config.into(),
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

        stream.play()?;

        Ok(BlightAudio {
            command_tx,
            voice_factory,
            resource_manager,
            effect_factory,
            _stream: stream,
        })
    }

    /// Public method to send a command to the audio thread.
    pub fn send_command(&mut self, command: Command) {
        if self.command_tx.try_push(command).is_err() {
            // In a real app, handle this more gracefully (e.g., log, drop command).
            eprintln!("Command queue is full. Command dropped.");
        }
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
}
