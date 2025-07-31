use crate::{MonoEffect, StereoEffect};

/// A simple effect that adjusts the volume of the audio signal. Units are in decibels (dB).
pub struct Gain {
    // We store gain as a linear amplitude factor for fast multiplication.
    gain_factor: f32,
}

impl Gain {
    pub fn new(gain_factor: f32) -> Self {
        Self { gain_factor } // 1.0 Default to no change in volume.
    }
}

impl StereoEffect for Gain {
    fn process(&mut self, left_buf: &mut [f32], right_buf: &mut [f32], _sample_rate: f32) {
        // Use zip to iterate over both channels safely and efficiently.
        for (left_sample, right_sample) in left_buf.iter_mut().zip(right_buf.iter_mut()) {
            *left_sample *= self.gain_factor;
            *right_sample *= self.gain_factor;
        }
    }

    fn set_parameter(&mut self, index: u32, value: f32) {
        // This effect only has one parameter (index 0).
        if index == 0 {
            // It's common to control volume in decibels (dB) from the UI.
            // We convert the dB value to a linear amplitude factor for processing.
            // A value of 0.0 dB results in a factor of 1.0 (no change).
            self.gain_factor = 10.0_f32.powf(value / 20.0);
        }
    }
}

impl MonoEffect for Gain {
    fn process(&mut self, buffer: &mut [f32], _sample_rate: f32) {
        // For mono processing, we can use the same logic as stereo but only on one channel.
        for sample in buffer.iter_mut() {
            *sample *= self.gain_factor;
        }
    }

    fn set_parameter(&mut self, index: u32, value: f32) {
        // Mono effect also has the same single parameter.
        if index == 0 {
            self.gain_factor = 10.0_f32.powf(value / 20.0);
        }
    }
}