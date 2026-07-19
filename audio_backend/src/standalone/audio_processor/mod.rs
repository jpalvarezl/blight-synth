use ringbuf::traits::*;
use ringbuf::HeapCons;

use crate::Command;
use crate::MeterState;
use crate::Player;
use sequencer::models::Song;
use std::sync::Arc;

const MAX_BUFFER_SIZE: usize = 4096;

pub struct AudioProcessor {
    pub(crate) command_rx: HeapCons<Command>,
    pub(crate) player: Player,
    pub(crate) sample_rate: f32,
    pub(crate) channels: usize,
    // Pre-allocated, non-interleaved buffers for processing.
    pub(crate) left_buf: Vec<f32>,
    pub(crate) right_buf: Vec<f32>,
    // Shared metering written once per block (read by the OSC server).
    pub(crate) meter: Arc<MeterState>,
}

impl AudioProcessor {
    pub fn new_with_song(
        song: Arc<Song>,
        command_rx: HeapCons<Command>,
        sample_rate: f32,
        channels: usize,
        meter: Arc<MeterState>,
    ) -> Self {
        Self {
            command_rx,
            sample_rate,
            channels,
            left_buf: vec![0.0; MAX_BUFFER_SIZE],
            right_buf: vec![0.0; MAX_BUFFER_SIZE],
            meter,
            player: Player::new(song, sample_rate as f64),
        }
    }

    pub fn new(
        command_rx: HeapCons<Command>,
        sample_rate: f32,
        channels: usize,
        meter: Arc<MeterState>,
    ) -> Self {
        let default_song = Arc::new(sequencer::models::Song::new("Untitled"));
        Self {
            command_rx,
            sample_rate,
            channels,
            left_buf: vec![0.0; MAX_BUFFER_SIZE],
            right_buf: vec![0.0; MAX_BUFFER_SIZE],
            meter,
            player: Player::new(default_song, sample_rate as f64),
        }
    }

    /// The main processing function called by the audio driver.
    pub fn process(&mut self, output_buffer: &mut [f32]) {
        // 1. Drain the command queue to update state. This is non-blocking.
        while let Some(command) = self.command_rx.try_pop() {
            // For now route all to player; Engine/Mixer handled inside player.synthesizer
            self.player.handle_command(command);

            // Here we need a way to select a self.synthesizer, from synth_infra/synthesizer.rs
            // for when we want to operate as an instrument and handle voice allocs through commands
        }

        if self.channels == 0 {
            // A CPAL stream always has at least one channel, but keep this method
            // total for tests and future non-CPAL hosts.
            output_buffer.fill(0.0);
            return;
        }

        // Scratch buffers in AudioProcessor and Voice are preallocated to this
        // bound. A host callback may still provide more frames, so process it in
        // bounded chunks instead of slicing past the buffers and panicking.
        let samples_per_chunk = MAX_BUFFER_SIZE * self.channels;
        let complete_sample_count = (output_buffer.len() / self.channels) * self.channels;
        let (complete_frames, trailing_samples) = output_buffer.split_at_mut(complete_sample_count);

        for output_chunk in complete_frames.chunks_mut(samples_per_chunk) {
            self.process_chunk(output_chunk);
        }

        // Host buffers should contain complete frames. Silence any malformed
        // trailing samples rather than leaving stale output behind.
        trailing_samples.fill(0.0);
    }

    fn process_chunk(&mut self, output_buffer: &mut [f32]) {
        let frame_count = output_buffer.len() / self.channels;
        let (left, right) = (
            &mut self.left_buf[..frame_count],
            &mut self.right_buf[..frame_count],
        );

        left.fill(0.0);
        right.fill(0.0);

        // Move the play-head by the frames in this bounded processing chunk.
        self.player
            .process(left, right, self.sample_rate, frame_count);

        // Record the post-master stereo chunk for meter streaming.
        self.meter.record_block(left, right);

        // Re-interleave stereo output. Additional host channels are explicitly
        // silenced until the engine has a channel-layout/routing contract.
        for (i, frame) in output_buffer.chunks_mut(self.channels).enumerate() {
            frame.fill(0.0);
            frame[0] = left[i];
            if self.channels > 1 {
                frame[1] = right[i];
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ringbuf::{storage::Heap, traits::Split, SharedRb};

    fn processor(channels: usize) -> AudioProcessor {
        let rb = SharedRb::<Heap<Command>>::new(8);
        let (_command_tx, command_rx) = rb.split();
        AudioProcessor::new(command_rx, 44_100.0, channels, Arc::new(MeterState::new()))
    }

    #[test]
    fn process_chunks_host_buffers_larger_than_internal_scratch_space() {
        let channels = 2;
        let frame_count = MAX_BUFFER_SIZE * 2 + 17;
        let mut output = vec![1.0; frame_count * channels];
        let mut processor = processor(channels);

        processor.process(&mut output);

        // The default player is stopped, so every chunk must be silent. Most
        // importantly, processing the oversized callback must not panic.
        assert!(output.iter().all(|sample| *sample == 0.0));
    }

    #[test]
    fn process_silences_extra_channels_and_incomplete_frames() {
        let channels = 3;
        let mut output = vec![1.0; channels * 4 + 1];
        let mut processor = processor(channels);

        processor.process(&mut output);

        assert!(output.iter().all(|sample| *sample == 0.0));
    }
}
