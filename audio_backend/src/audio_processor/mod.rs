use ringbuf::traits::*;
use ringbuf::HeapCons;

use crate::Command;
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
}

impl AudioProcessor {
    pub fn new_with_song(
        song: Arc<Song>,
        command_rx: HeapCons<Command>,
        sample_rate: f32,
        channels: usize,
    ) -> Self {
        Self {
            command_rx,
            sample_rate,
            channels,
            left_buf: vec![0.0; MAX_BUFFER_SIZE],
            right_buf: vec![0.0; MAX_BUFFER_SIZE],
            player: Player::new(song, sample_rate as f64),
        }
    }

    pub fn new(command_rx: HeapCons<Command>, sample_rate: f32, channels: usize) -> Self {
        let default_song = Arc::new(sequencer::models::Song::new("Untitled"));
        Self {
            command_rx,
            sample_rate,
            channels,
            left_buf: vec![0.0; MAX_BUFFER_SIZE],
            right_buf: vec![0.0; MAX_BUFFER_SIZE],
            player: Player::new(default_song, sample_rate as f64),
        }
    }

    /// The main processing function called by the audio driver.
    pub fn process(&mut self, output_buffer: &mut [f32]) {
        // frames in this block.
        let frame_count = output_buffer.len() / self.channels;

        // 1. Drain the command queue to update state. This is non-blocking.
        while let Some(command) = self.command_rx.try_pop() {
            // For now route all to player; Engine/Mixer handled inside player.synthesizer
            self.player.handle_command(command);

            // Here we need a way to select a self.synthesizer, from synth_infra/synthesizer.rs
            // for when we want to operate as an instrument and handle voice allocs through commands
        }

        // 2. We move the play-head by as many ticks, depending on the frames.
        // Namely, we progress our control rate, as oppose to our audio rate.
        self.player.process(frame_count);

        // We allocate enough space and clear the left and right buffers.
        let (left, right) = (
            &mut self.left_buf[..frame_count],
            &mut self.right_buf[..frame_count],
        );

        left.fill(0.0);
        right.fill(0.0);

        // 3. Process the synthesizer with the prepared buffers.
        self.player
            .synthesizer
            .process(left, right, self.sample_rate);

        // 4. Re-interleave the processed buffers into the output buffer.
        for (i, frame) in output_buffer.chunks_mut(self.channels).enumerate() {
            frame[0] = left[i];
            if self.channels > 1 {
                frame[1] = right[i];
            }
        }
    }
}
