#![cfg_attr(not(feature = "tracker"), allow(dead_code))]

use ringbuf::HeapCons;
use ringbuf::traits::*;

use crate::Command;

#[cfg(feature = "tracker")]
use crate::Player;
#[cfg(not(feature = "tracker"))]
use crate::Synthesizer;

#[cfg(feature = "tracker")]
use std::sync::Arc;
#[cfg(feature = "tracker")]
use sequencer::models::Song;

const MAX_BUFFER_SIZE: usize = 4096;

pub struct AudioProcessor {
    pub(crate) command_rx: HeapCons<Command>,
    #[cfg(feature = "tracker")]
    pub(crate) player: Player,
    #[cfg(not(feature = "tracker"))]
    pub(crate) synthesizer: Synthesizer,
    pub(crate) sample_rate: f32,
    pub(crate) channels: usize,
    // Pre-allocated, non-interleaved buffers for processing.
    pub(crate) left_buf: Vec<f32>,
    pub(crate) right_buf: Vec<f32>,
}

impl AudioProcessor {
    #[cfg(feature = "tracker")]
    pub fn new(
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

    #[cfg(not(feature = "tracker"))]
    pub fn new(command_rx: HeapCons<Command>, sample_rate: f32, channels: usize) -> Self {
        Self {
            command_rx,
            synthesizer: Synthesizer::new(),
            sample_rate,
            channels,
            left_buf: vec![0.0; MAX_BUFFER_SIZE],
            right_buf: vec![0.0; MAX_BUFFER_SIZE],
        }
    }

    pub fn process(&mut self, output_buffer: &mut [f32]) {
        let frame_count = output_buffer.len() / self.channels;

        #[cfg(feature = "tracker")]
        {
            // 1. Drain commands and hand over to Player
            while let Some(command) = self.command_rx.try_pop() {
                self.player.handle_command(command);
            }
            self.player.process(frame_count);

            // Calculate the number of frames in this block.
            let (left, right) = (
                &mut self.left_buf[..frame_count],
                &mut self.right_buf[..frame_count],
            );

            // Clear internal buffers.
            left.fill(0.0);
            right.fill(0.0);

            // 2. Process audio into our non-interleaved buffers.
            self.player
                .synthesizer
                .process(left, right, self.sample_rate);

            // 3. Re-interleave the processed audio back into the output buffer.
            for (i, frame) in output_buffer.chunks_mut(self.channels).enumerate() {
                frame[0] = left[i];
                if self.channels > 1 {
                    frame[1] = right[i];
                }
            }
        }

        #[cfg(not(feature = "tracker"))]
        {
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
}

