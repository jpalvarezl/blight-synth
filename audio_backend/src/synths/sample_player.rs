use std::sync::Arc;

use crate::{SampleData, SynthNode};

pub struct SamplePlayerNode {
    sample: Arc<SampleData>,
    output_sample_rate: f32,

    /// The current playback position in the sample buffer.
    /// This is a float to allow for fractional positions, which is necessary for resampling.
    position: f64,

    is_playing: bool,

    /// The rate at which we advance the `position` for each output sample.
    /// A rate of 1.0 plays at the original pitch, corrected for the output sample rate.
    /// A rate > 1.0 is pitched up, < 1.0 is pitched down.
    playback_rate: f64,

    /// The MIDI note that corresponds to the original pitch of the sample.
    /// For example, if the sample is a C4 note, this would be 60.
    base_note: u8,
}

impl SamplePlayerNode {
    pub fn new(sample: Arc<SampleData>, output_sample_rate: f32) -> Self {
        Self {
            sample: sample.clone(),
            position: 0.0,
            is_playing: false,
            playback_rate: 1.0,
            output_sample_rate,
            base_note: 60, // Default to middle C, can be set later.
        }
    }

    /// A helper function to safely get a mono sample from the buffer at a given integer index.
    /// It handles both mono and stereo source files.
    fn get_mono_sample_at(&self, index: usize) -> f32 {
        if self.sample.channels > 1 {
            // For stereo, we need to multiply the frame index by 2 to get the sample index.
            let frame_index = index * 2;
            // Safely get left and right samples, defaulting to 0.0 if out of bounds.
            let left = self.sample.data.get(frame_index).cloned().unwrap_or(0.0);
            let right = self
                .sample
                .data
                .get(frame_index + 1)
                .cloned()
                .unwrap_or(0.0);
            // Mix down to mono.
            (left + right) * 0.5
        } else {
            self.sample.data.get(index).cloned().unwrap_or(0.0)
        }
    }
}

impl SynthNode for SamplePlayerNode {
    fn process(&mut self, output_buffer: &mut [f32], _sample_rate: f32) {
        for sample in output_buffer.iter_mut() {
            if !self.is_active() {
                *sample = 0.0;
                continue;
            }

            // Get the integer and fractional parts of the position.
            let index_floor = self.position.floor();
            let index0 = index_floor as usize;
            let fraction = self.position - index_floor;

            // Get the two samples to interpolate between.
            let sample0 = self.get_mono_sample_at(index0);
            let sample1 = self.get_mono_sample_at(index0 + 1);

            // Perform linear interpolation. This is the simplest form of high-quality resampling.
            let interpolated_sample = sample0 as f64 * (1.0 - fraction) + sample1 as f64 * fraction;
            *sample = interpolated_sample as f32;

            // Advance the playback position by our calculated rate.
            self.position += self.playback_rate;
        }
    }

    fn note_on(&mut self, note: u8, _velocity: u8) {
        // 1. Calculate the sample rate correction factor. This is the core of fixing the pitch distortion.
        // If the file is 44.1k and output is 48k, this will be < 1.0, so we advance slower.
        let sr_correction = self.sample.sample_rate as f64 / self.output_sample_rate as f64;

        // 2. Calculate the pitch shift factor from the MIDI note.
        // The formula for pitch shifting by N semitones is 2^(N/12).
        let semitones_diff = note as f64 - self.base_note as f64;
        let pitch_shift = 2.0_f64.powf(semitones_diff / 12.0);

        // 3. Combine them to get the final playback rate.
        self.playback_rate = sr_correction * pitch_shift;

        self.position = 0.0;
        self.is_playing = true;
    }

    fn note_off(&mut self) {
        // For a one-shot sample, note_off might do nothing.
        // If it were a looped sample, this is where you'd stop the loop.
        // For simplicity, we'll allow the sample to play out fully.
    }

    fn is_active(&self) -> bool {
        // The number of frames in the sample buffer.
        let buffer_len_frames = if self.sample.channels > 1 {
            self.sample.data.len() / 2
        } else {
            self.sample.data.len()
        };
        // The voice is active if it's playing and the position is still within the buffer's bounds.
        self.is_playing && (self.position < buffer_len_frames as f64)
    }
}
