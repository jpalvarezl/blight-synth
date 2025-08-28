use crate::MonoEffect;
use std::f32::consts::PI;

/// Filter types
#[derive(Clone, Copy)]
pub enum FilterType {
    /// Attenuates high frequencies
    LowPass,
    /// Attenuates low frequencies
    HighPass,
    /// Attenuates frequencies outside a band
    BandPass,
    /// Attenuates frequencies within a band
    Notch,
}

/// Biquad filter implementation
///
/// Digital implementation of various filter types using the biquad structure.
///
/// Transfer function: H(z) = (b0 + b1*z^-1 + b2*z^-2) / (1 + a1*z^-1 + a2*z^-2)
///
/// Reference: "Cookbook formulae for audio EQ biquad filter coefficients"
/// by Robert Bristow-Johnson
pub struct Filter {
    filter_type: FilterType,
    cutoff: f32,
    resonance: f32,
    // Biquad coefficients
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    // State variables
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl Filter {
    pub fn new(filter_type: FilterType, cutoff: f32, resonance: f32, sample_rate: f32) -> Self {
        let mut filter = Self {
            filter_type,
            cutoff,
            resonance,
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        };
        filter.update_coefficients(sample_rate);
        filter
    }

    fn update_coefficients(&mut self, sample_rate: f32) {
        let omega = 2.0 * PI * self.cutoff / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let q = 1.0 / (2.0 * self.resonance.max(0.5));
        let alpha = sin_omega / (2.0 * q);

        match self.filter_type {
            FilterType::LowPass => {
                let b0 = (1.0 - cos_omega) / 2.0;
                let b1 = 1.0 - cos_omega;
                let b2 = (1.0 - cos_omega) / 2.0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_omega;
                let a2 = 1.0 - alpha;

                // Normalize coefficients
                self.b0 = b0 / a0;
                self.b1 = b1 / a0;
                self.b2 = b2 / a0;
                self.a1 = a1 / a0;
                self.a2 = a2 / a0;
            }
            FilterType::HighPass => {
                let b0 = (1.0 + cos_omega) / 2.0;
                let b1 = -(1.0 + cos_omega);
                let b2 = (1.0 + cos_omega) / 2.0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_omega;
                let a2 = 1.0 - alpha;

                self.b0 = b0 / a0;
                self.b1 = b1 / a0;
                self.b2 = b2 / a0;
                self.a1 = a1 / a0;
                self.a2 = a2 / a0;
            }
            FilterType::BandPass => {
                let b0 = alpha;
                let b1 = 0.0;
                let b2 = -alpha;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_omega;
                let a2 = 1.0 - alpha;

                self.b0 = b0 / a0;
                self.b1 = b1 / a0;
                self.b2 = b2 / a0;
                self.a1 = a1 / a0;
                self.a2 = a2 / a0;
            }
            FilterType::Notch => {
                let b0 = 1.0;
                let b1 = -2.0 * cos_omega;
                let b2 = 1.0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_omega;
                let a2 = 1.0 - alpha;

                self.b0 = b0 / a0;
                self.b1 = b1 / a0;
                self.b2 = b2 / a0;
                self.a1 = a1 / a0;
                self.a2 = a2 / a0;
            }
        }
    }

    pub fn set_cutoff(&mut self, cutoff: f32, sample_rate: f32) {
        self.cutoff = cutoff;
        self.update_coefficients(sample_rate);
    }

    pub fn set_resonance(&mut self, resonance: f32, sample_rate: f32) {
        self.resonance = resonance;
        self.update_coefficients(sample_rate);
    }
}

impl MonoEffect for Filter {
    fn process(&mut self, buffer: &mut [f32], _sample_rate: f32) {
        for sample in buffer.iter_mut() {
            let input = *sample;

            // Biquad difference equation
            let output = self.b0 * input + self.b1 * self.x1 + self.b2 * self.x2
                - self.a1 * self.y1
                - self.a2 * self.y2;

            // Update state variables
            self.x2 = self.x1;
            self.x1 = input;
            self.y2 = self.y1;
            self.y1 = output;

            *sample = output;
        }
    }

    fn set_parameter(&mut self, _index: u32, _value: f32) {
        // TODO: figure out how to get sample_rate here
        // match index {
        //     0 => self.set_cutoff(value, sample_rate),
        //     1 => self.set_resonance(value, sample_rate),
        //     _ => (),
        // }
    }

    fn reset(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }
}
