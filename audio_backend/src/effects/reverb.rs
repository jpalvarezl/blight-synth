use crate::MonoEffect;
use crate::StereoEffect;

/// Simple Schroeder reverb implementation
///
/// This uses the classic Schroeder reverb architecture:
/// - 4 parallel comb filters (for echo density)
/// - 2 series allpass filters (for echo diffusion)
///
/// Reference: "Natural Sounding Artificial Reverberation" by M.R. Schroeder (1962)
pub struct Reverb {
    comb_filters: [CombFilter; 4],
    allpass_filters: [AllpassFilter; 2],
    room_size: f32,
    damping: f32,
    wet_gain: f32,
}

impl Reverb {
    pub fn new(sample_rate: f32, room_size: f32, damping: f32) -> Self {
        // Comb filter delay times (in ms) - tuned to avoid metallic sound
        let comb_delays = [29.7, 37.1, 41.1, 43.7];
        let comb_filters = [
            CombFilter::new(sample_rate, comb_delays[0], 0.83),
            CombFilter::new(sample_rate, comb_delays[1], 0.81),
            CombFilter::new(sample_rate, comb_delays[2], 0.79),
            CombFilter::new(sample_rate, comb_delays[3], 0.77),
        ];

        // Allpass filter delay times (in ms)
        let allpass_delays = [5.0, 1.7];
        let allpass_filters = [
            AllpassFilter::new(sample_rate, allpass_delays[0]),
            AllpassFilter::new(sample_rate, allpass_delays[1]),
        ];

        Self {
            comb_filters,
            allpass_filters,
            room_size,
            damping,
            wet_gain: 0.3,
        }
    }
}

impl MonoEffect for Reverb {
    fn process(&mut self, buffer: &mut [f32], _sample_rate: f32) {
        for sample in buffer.iter_mut() {
            let input = *sample;

            // Sum the outputs of parallel comb filters
            let mut comb_sum = 0.0;
            for comb in &mut self.comb_filters {
                comb_sum += comb.process(input, self.room_size, self.damping);
            }
            comb_sum *= 0.25; // Average the 4 comb outputs

            // Process through series allpass filters
            let mut allpass_out = comb_sum;
            for allpass in &mut self.allpass_filters {
                allpass_out = allpass.process(allpass_out);
            }

            // Mix wet and dry signals
            *sample = input + allpass_out * self.wet_gain;
        }
    }

    fn set_parameter(&mut self, _index: u32, _value: f32) {
        // not implemented yet
    }
}

/// Feedback Comb Filter (building block for reverb)
struct CombFilter {
    buffer: Vec<f32>,
    index: usize,
    feedback: f32,
}

impl CombFilter {
    fn new(sample_rate: f32, delay_ms: f32, feedback: f32) -> Self {
        let delay_samples = ((delay_ms / 1000.0) * sample_rate) as usize;
        Self {
            buffer: vec![0.0; delay_samples],
            index: 0,
            feedback,
        }
    }

    fn process(&mut self, input: f32, room_size: f32, damping: f32) -> f32 {
        let delayed = self.buffer[self.index];
        let feedback_sample = delayed * self.feedback * room_size;

        // Apply damping (simple lowpass filter)
        let damped = feedback_sample * (1.0 - damping) + delayed * damping;

        self.buffer[self.index] = input + damped;
        self.index = (self.index + 1) % self.buffer.len();

        delayed
    }
}

/// Allpass Filter (for diffusion in reverb)
struct AllpassFilter {
    buffer: Vec<f32>,
    index: usize,
}

impl AllpassFilter {
    fn new(sample_rate: f32, delay_ms: f32) -> Self {
        let delay_samples = ((delay_ms / 1000.0) * sample_rate) as usize;
        Self {
            buffer: vec![0.0; delay_samples],
            index: 0,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let delayed = self.buffer[self.index];
        let output = -input + delayed;
        self.buffer[self.index] = input + delayed * 0.5;
        self.index = (self.index + 1) % self.buffer.len();
        output
    }
}

/// Stereo reverb with cross-coupling for spaciousness
///
/// Uses two separate reverb engines with cross-feedback
/// to create a wide stereo image
pub struct StereoReverb {
    left_reverb: Reverb,
    right_reverb: Reverb,
    stereo_width: f32,
}

impl StereoReverb {
    pub fn new(sample_rate: f32, room_size: f32, damping: f32) -> Self {
        // Slightly different parameters for each channel
        let mut left_reverb = Reverb::new(sample_rate, room_size, damping);
        let mut right_reverb = Reverb::new(sample_rate, room_size * 1.05, damping * 0.95);

        Self {
            left_reverb,
            right_reverb,
            stereo_width: 0.5,
        }
    }
}

impl StereoEffect for StereoReverb {
    fn process(&mut self, left: &mut [f32], right: &mut [f32], sample_rate: f32) {
        // Process with cross-coupling for stereo width
        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            let left_in = *l;
            let right_in = *r;

            // Cross-feed some signal to opposite channel
            let left_feed = left_in + right_in * self.stereo_width;
            let right_feed = right_in + left_in * self.stereo_width;

            // Process through separate reverbs
            let mut left_out = left_feed;
            let mut right_out = right_feed;

            self.left_reverb
                .process(std::slice::from_mut(&mut left_out), sample_rate);
            self.right_reverb
                .process(std::slice::from_mut(&mut right_out), sample_rate);

            *l = left_in * 0.7 + left_out * 0.3;
            *r = right_in * 0.7 + right_out * 0.3;
        }
    }

    fn set_parameter(&mut self, _index: u32, _value: f32) {
        // TODO: implement parameter setting
    }
}
