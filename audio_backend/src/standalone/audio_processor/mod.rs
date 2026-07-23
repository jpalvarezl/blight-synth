use engine::{RetireSink, RetiredState};
use ringbuf::traits::*;
use ringbuf::{HeapCons, HeapProd};

use crate::Command;
use crate::MeterState;
use crate::Player;
use sequencer::models::Song;
use std::sync::Arc;

const MAX_BUFFER_SIZE: usize = 4096;
/// Maximum compatibility control commands applied before rendering one host
/// callback block. A backlog remains FIFO-queued for later blocks so control
/// bursts cannot postpone rendering indefinitely.
pub(crate) const MAX_COMMANDS_PER_PROCESS_BLOCK: usize = 64;

struct CallbackRetireSink<'a> {
    retirement_tx: &'a mut HeapProd<RetiredState>,
    pending_retired: &'a mut Vec<RetiredState>,
}

impl RetireSink for CallbackRetireSink<'_> {
    fn retire(&mut self, retired: RetiredState) {
        if !self.retirement_tx.read_is_held() {
            self.pending_retired.push(retired);
            return;
        }
        if let Err(retired) = self.retirement_tx.try_push(retired) {
            self.pending_retired.push(retired);
        }
    }
}

pub struct AudioProcessor {
    pub(crate) command_rx: HeapCons<Command>,
    retirement_tx: HeapProd<RetiredState>,
    pending_retired: Vec<RetiredState>,
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
        retirement_tx: HeapProd<RetiredState>,
        sample_rate: f32,
        channels: usize,
        meter: Arc<MeterState>,
    ) -> Self {
        Self {
            command_rx,
            retirement_tx,
            pending_retired: Vec::with_capacity(MAX_COMMANDS_PER_PROCESS_BLOCK),
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
        retirement_tx: HeapProd<RetiredState>,
        sample_rate: f32,
        channels: usize,
        meter: Arc<MeterState>,
    ) -> Self {
        let default_song = Arc::new(sequencer::models::Song::new("Untitled"));
        Self {
            command_rx,
            retirement_tx,
            pending_retired: Vec::with_capacity(MAX_COMMANDS_PER_PROCESS_BLOCK),
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
        self.flush_retired();

        // If a previous block retained retirement ownership, pause this block's
        // command consumption until it reaches NRT. The current bounded command
        // loop may add at most 64 pending owners before this gate takes effect.
        if self.pending_retired.is_empty() {
            for _ in 0..MAX_COMMANDS_PER_PROCESS_BLOCK {
                let Some(command) = self.command_rx.try_pop() else {
                    break;
                };
                let mut retired = CallbackRetireSink {
                    retirement_tx: &mut self.retirement_tx,
                    pending_retired: &mut self.pending_retired,
                };
                self.player.handle_command(command, &mut retired);
            }
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

    fn flush_retired(&mut self) {
        while let Some(retired) = self.pending_retired.pop() {
            if !self.retirement_tx.read_is_held() {
                self.pending_retired.push(retired);
                break;
            }
            if let Err(retired) = self.retirement_tx.try_push(retired) {
                self.pending_retired.push(retired);
                break;
            }
        }
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
    use crate::{
        id::{EffectId, InstrumentId},
        InstrumentCmd, InstrumentTrait, MonoEffect, SynthCmd, TransportCmd, VoiceEffects,
    };
    use ringbuf::{storage::Heap, traits::Split, HeapCons, HeapProd, SharedRb};
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct RenderCounterInstrument {
        renders: Arc<AtomicUsize>,
    }

    struct DropProbeInstrument {
        id: InstrumentId,
        drops: Arc<AtomicUsize>,
    }

    impl Drop for DropProbeInstrument {
        fn drop(&mut self) {
            self.drops.fetch_add(1, Ordering::Relaxed);
        }
    }

    impl InstrumentTrait for DropProbeInstrument {
        fn id(&self) -> InstrumentId {
            self.id
        }
        fn note_on(&mut self, _note: u8, _velocity: u8) {}
        fn note_off(&mut self) {}
        fn process(&mut self, _left: &mut [f32], _right: &mut [f32], _sample_rate: f32) {}
        fn set_pan(&mut self, _pan: f32) {}
        fn add_effect(&mut self, _effect: Box<dyn MonoEffect>) {}
        fn add_voice_effects(&mut self, _effects: VoiceEffects) {}
        fn set_effect_parameter(&mut self, _effect_id: EffectId, _param_index: u32, _value: f32) {}
        fn try_handle_command(&mut self, _command: &SynthCmd) -> bool {
            false
        }
    }

    impl InstrumentTrait for RenderCounterInstrument {
        fn id(&self) -> InstrumentId {
            1
        }

        fn note_on(&mut self, _note: u8, _velocity: u8) {}

        fn note_off(&mut self) {}

        fn process(&mut self, left: &mut [f32], right: &mut [f32], _sample_rate: f32) {
            self.renders.fetch_add(1, Ordering::Relaxed);
            for (left, right) in left.iter_mut().zip(right) {
                *left += 0.25;
                *right += 0.25;
            }
        }

        fn set_pan(&mut self, _pan: f32) {}

        fn add_effect(&mut self, _effect: Box<dyn MonoEffect>) {}

        fn add_voice_effects(&mut self, _effects: VoiceEffects) {}

        fn set_effect_parameter(&mut self, _effect_id: EffectId, _param_index: u32, _value: f32) {}

        fn try_handle_command(&mut self, _command: &SynthCmd) -> bool {
            false
        }
    }

    fn processor(channels: usize) -> AudioProcessor {
        processor_with_capacity(channels, 8).1
    }

    fn processor_with_capacity(
        channels: usize,
        capacity: usize,
    ) -> (HeapProd<Command>, AudioProcessor) {
        let (command_tx, _retirement_rx, processor) =
            processor_with_retirement(channels, capacity, capacity.max(1));
        (command_tx, processor)
    }

    fn processor_with_retirement(
        channels: usize,
        command_capacity: usize,
        retirement_capacity: usize,
    ) -> (HeapProd<Command>, HeapCons<RetiredState>, AudioProcessor) {
        let rb = SharedRb::<Heap<Command>>::new(command_capacity);
        let (command_tx, command_rx) = rb.split();
        let retirement_rb = SharedRb::<Heap<RetiredState>>::new(retirement_capacity);
        let (retirement_tx, retirement_rx) = retirement_rb.split();
        (
            command_tx,
            retirement_rx,
            AudioProcessor::new(
                command_rx,
                retirement_tx,
                44_100.0,
                channels,
                Arc::new(MeterState::new()),
            ),
        )
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

    #[test]
    fn command_budget_is_per_host_callback_not_per_internal_render_chunk() {
        let queued = MAX_COMMANDS_PER_PROCESS_BLOCK + 1;
        let (mut command_tx, mut processor) = processor_with_capacity(2, queued);
        for index in 0..queued {
            assert!(command_tx
                .try_push(
                    TransportCmd::SetLooping {
                        enabled: index % 2 == 0,
                    }
                    .into(),
                )
                .is_ok());
        }
        let frame_count = MAX_BUFFER_SIZE * 2 + 17;
        let mut output = vec![0.0; frame_count * 2];

        processor.process(&mut output);

        assert_eq!(processor.command_rx.occupied_len(), 1);
    }

    #[test]
    fn replaced_instrument_crosses_retirement_ring_before_nrt_drop() {
        let (mut command_tx, mut retirement_rx, mut processor) = processor_with_retirement(2, 8, 8);
        let drops = Arc::new(AtomicUsize::new(0));
        assert!(command_tx
            .try_push(
                InstrumentCmd::AddInstrument {
                    instrument: Box::new(DropProbeInstrument {
                        id: 7,
                        drops: drops.clone(),
                    }),
                }
                .into(),
            )
            .is_ok());
        let mut output = [0.0; 16];
        processor.process(&mut output);

        assert!(command_tx
            .try_push(
                InstrumentCmd::AddInstrument {
                    instrument: Box::new(DropProbeInstrument {
                        id: 7,
                        drops: Arc::new(AtomicUsize::new(0)),
                    }),
                }
                .into(),
            )
            .is_ok());
        processor.process(&mut output);

        assert_eq!(drops.load(Ordering::Relaxed), 0);
        let retired = retirement_rx.try_pop().expect("retired owner reaches NRT");
        drop(retired);
        assert_eq!(drops.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn full_retirement_ring_pauses_later_commands_until_nrt_drains() {
        let (mut command_tx, mut retirement_rx, mut processor) =
            processor_with_retirement(2, 16, 1);
        let first_drops = Arc::new(AtomicUsize::new(0));
        let second_drops = Arc::new(AtomicUsize::new(0));
        for (id, drops) in [(1, first_drops.clone()), (2, second_drops.clone())] {
            assert!(command_tx
                .try_push(
                    InstrumentCmd::AddInstrument {
                        instrument: Box::new(DropProbeInstrument { id, drops }),
                    }
                    .into(),
                )
                .is_ok());
        }
        let mut output = [0.0; 16];
        processor.process(&mut output);

        for id in [1, 2] {
            assert!(command_tx
                .try_push(
                    InstrumentCmd::AddInstrument {
                        instrument: Box::new(DropProbeInstrument {
                            id,
                            drops: Arc::new(AtomicUsize::new(0)),
                        }),
                    }
                    .into(),
                )
                .is_ok());
        }
        processor.process(&mut output);
        assert_eq!(processor.pending_retired.len(), 1);

        assert!(command_tx
            .try_push(TransportCmd::PlayLastSong.into())
            .is_ok());
        processor.process(&mut output);
        assert!(!processor.player.is_playing());
        assert_eq!(processor.command_rx.occupied_len(), 1);

        drop(retirement_rx.try_pop().expect("first owner reaches NRT"));
        processor.process(&mut output);
        assert!(processor.player.is_playing());
        assert!(processor.pending_retired.is_empty());
        drop(retirement_rx.try_pop().expect("pending owner reaches NRT"));
        assert_eq!(
            first_drops.load(Ordering::Relaxed) + second_drops.load(Ordering::Relaxed),
            2
        );
    }

    #[test]
    fn command_burst_is_fifo_bounded_and_rendering_progresses_between_slices() {
        let burst_len = MAX_COMMANDS_PER_PROCESS_BLOCK * 3 + 1;
        let (mut command_tx, mut processor) = processor_with_capacity(2, burst_len);
        let renders = Arc::new(AtomicUsize::new(0));

        assert!(
            command_tx
                .try_push(
                    InstrumentCmd::AddInstrument {
                        instrument: Box::new(RenderCounterInstrument {
                            renders: renders.clone(),
                        }),
                    }
                    .into(),
                )
                .is_ok(),
            "setup command fits"
        );
        assert!(
            command_tx
                .try_push(TransportCmd::PlayLastSong.into())
                .is_ok(),
            "play command fits"
        );
        for index in 2..burst_len - 1 {
            assert!(
                command_tx
                    .try_push(
                        TransportCmd::SetLooping {
                            enabled: index % 2 == 0,
                        }
                        .into(),
                    )
                    .is_ok(),
                "control burst fits"
            );
        }
        assert!(
            command_tx.try_push(TransportCmd::StopSong.into()).is_ok(),
            "recovery command fits"
        );

        let mut output = [0.0; 32];
        for block in 1..=3 {
            processor.process(&mut output);

            assert_eq!(renders.load(Ordering::Relaxed), block);
            assert!(output.iter().any(|sample| *sample != 0.0));
            assert_eq!(
                processor.command_rx.occupied_len(),
                burst_len - block * MAX_COMMANDS_PER_PROCESS_BLOCK
            );
            output.fill(0.0);
        }

        // FIFO fairness leaves the final stop/recovery command queued until the
        // next block; it is then applied before rendering that block.
        assert_eq!(processor.command_rx.occupied_len(), 1);
        processor.process(&mut output);
        assert_eq!(processor.command_rx.occupied_len(), 0);
        assert_eq!(renders.load(Ordering::Relaxed), 3);
        assert!(output.iter().all(|sample| *sample == 0.0));
    }
}
