use std::sync::Arc;

use crate::{SampleData, SynthNode};

/// Cached loop region boundaries in frames (as f64 for interpolation precision)
#[derive(Debug, Clone, Copy)]
struct LoopRegion {
    start_frame: f64,
    end_frame: f64,
}

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

    /// Active loop region (calculated once from sample data in new())
    loop_region: Option<LoopRegion>,
}

impl SamplePlayerNode {
    pub fn new(sample: Arc<SampleData>, output_sample_rate: f32) -> Self {
        // Calculate loop region once from sample data
        let loop_region = if let (Some(start), Some(end)) = (sample.loop_start, sample.loop_end) {
            Some(LoopRegion {
                start_frame: start as f64,
                end_frame: end as f64,
            })
        } else {
            None
        };

        Self {
            sample: sample.clone(),
            position: 0.0,
            is_playing: false,
            playback_rate: 1.0,
            output_sample_rate,
            base_note: 60, // Default to middle C, can be set later.
            loop_region,
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

            // Handle loop wraparound BEFORE reading samples
            // This prevents reading past buffer bounds when looping
            if let Some(ref loop_region) = self.loop_region {
                if self.position >= loop_region.end_frame {
                    // Calculate how far we've gone past the loop end (overshoot)
                    // This is crucial for pitch accuracy when playback_rate > 1.0
                    let loop_length = loop_region.end_frame - loop_region.start_frame;
                    let overshoot = self.position - loop_region.end_frame;

                    // Wrap back to loop start, preserving the overshoot amount
                    // Modulo handles cases where we skip multiple loop lengths
                    self.position = loop_region.start_frame + (overshoot % loop_length);
                }
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
        // Do nothing - let the envelope handle the release
        // Looped samples will keep looping while envelope fades out
    }

    fn is_active(&self) -> bool {
        // Sample node is active as long as it's playing
        // For looped samples, this stays true indefinitely
        // For one-shot samples, this becomes false when position reaches buffer end
        if !self.is_playing {
            return false;
        }

        // If we have a loop, we're always active (envelope controls the actual voice lifetime)
        if self.loop_region.is_some() {
            return true;
        }

        // One-shot samples are active until position reaches buffer end
        let buffer_len_frames = if self.sample.channels > 1 {
            self.sample.data.len() / 2
        } else {
            self.sample.data.len()
        };
        self.position < buffer_len_frames as f64
    }
}
