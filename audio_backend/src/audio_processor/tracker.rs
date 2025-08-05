#![cfg(feature = "tracker")]

use std::sync::Arc;

use ringbuf::{HeapCons, HeapProd, traits::*};
use sequencer::models::Song;

use crate::{AudioProcessor, Command, Player, Synthesizer};

impl AudioProcessor {
    pub fn new(
        song: Arc<Song>,
        command_rx: HeapCons<Command>,
        command_tx: HeapProd<Command>,
        sample_rate: f32,
        channels: usize,
    ) -> Self {
        // A reasonable max buffer size. cpal can change this, but this is a safe upper bound.
        const MAX_BUFFER_SIZE: usize = 4096;
        Self {
            command_rx,
            synthesizer: Synthesizer::new(),
            sample_rate,
            channels,
            left_buf: vec![0.0; MAX_BUFFER_SIZE],
            right_buf: vec![0.0; MAX_BUFFER_SIZE],
            player: Player::new(song, command_tx, sample_rate as f64),
        }
    }

    // The main processing function called by the audio driver.
    pub fn process(&mut self, output_buffer: &mut [f32]) {
        // frames in this block.
        let frame_count = output_buffer.len() / self.channels;

        self.player.process(frame_count);

        // 1. Drain the command queue to update state. This is non-blocking.
        while let Some(command) = self.command_rx.try_pop() {
            self.synthesizer.handle_command(command);
        }

        // Calculate the number of frames in this block.
        let (left, right) = (
            &mut self.left_buf[..frame_count],
            &mut self.right_buf[..frame_count],
        );

        // Clear internal buffers.
        left.fill(0.0);
        right.fill(0.0);

        // 2. Process audio into our non-interleaved buffers.
        self.synthesizer.process(left, right, self.sample_rate);

        // 3. Re-interleave the processed audio back into the output buffer.
        for (i, frame) in output_buffer.chunks_mut(self.channels).enumerate() {
            frame[0] = left[i];
            if self.channels > 1 {
                frame[1] = right[i];
            }
        }
    }
}
