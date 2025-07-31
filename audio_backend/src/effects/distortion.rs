use crate::MonoEffect;

/// Various distortion algorithms
#[derive(Clone, Copy)]
pub enum DistortionType {
    /// Soft clipping using tanh
    Soft,
    /// Hard clipping
    Hard,
    /// Tube-like asymmetric distortion
    Tube,
    /// Foldback distortion
    Foldback,
}

/// Distortion effect with pre/post filtering
///
/// Signal chain:
/// input -> pre-gain -> distortion -> post-gain -> output
///
/// Common distortion algorithms:
/// - Soft clipping: tanh(x) - smooth compression
/// - Hard clipping: clamp(x) - harsh digital clipping
/// - Tube: asymmetric soft clipping
/// - Foldback: wraps signal back when it exceeds threshold
pub struct Distortion {
    distortion_type: DistortionType,
    pre_gain: f32,
    post_gain: f32,
    mix: f32,
}

impl Distortion {
    pub fn new(distortion_type: DistortionType, drive: f32, level: f32, mix: f32) -> Self {
        Self {
            distortion_type,
            pre_gain: drive,
            post_gain: level,
            mix: mix.clamp(0.0, 1.0),
        }
    }
}

impl MonoEffect for Distortion {
    fn process(&mut self, buffer: &mut [f32], _sample_rate: f32) {
        for sample in buffer.iter_mut() {
            let input = *sample;

            // Apply pre-gain
            let driven = input * self.pre_gain;

            // Apply distortion
            let distorted = match self.distortion_type {
                DistortionType::Soft => {
                    // Soft clipping with tanh
                    driven.tanh()
                }
                DistortionType::Hard => {
                    // Hard clipping
                    driven.clamp(-1.0, 1.0)
                }
                DistortionType::Tube => {
                    // Asymmetric tube-like distortion
                    if driven > 0.0 {
                        1.0 - (-driven).exp()
                    } else {
                        -1.0 + (driven).exp()
                    }
                }
                DistortionType::Foldback => {
                    // Foldback distortion
                    let mut x = driven;
                    while x > 1.0 || x < -1.0 {
                        if x > 1.0 {
                            x = 2.0 - x;
                        } else if x < -1.0 {
                            x = -2.0 - x;
                        }
                    }
                    x
                }
            };

            // Apply post-gain and mix
            let output = distorted * self.post_gain;
            *sample = input * (1.0 - self.mix) + output * self.mix;
        }
    }

    fn set_parameter(&mut self, _index: u32, _value: f32) {
        // not implemented yet
    }
}
