use crate::MonoEffect;

/// Freeverb-inspired reverb implementation
/// 
/// This uses the Freeverb algorithm by Jezar at Dreampoint, which sounds
/// much more natural than the basic Schroeder reverb.
/// 
/// Architecture:
/// - 8 parallel comb filters (instead of 4)
/// - 4 series allpass filters (instead of 2)
/// - Better tuned delay times
/// - Pre-delay for clarity
pub struct Reverb {
    // Pre-delay for initial gap before reverb
    pre_delay_buffer: Vec<f32>,
    pre_delay_index: usize,
    pre_delay_time: f32,
    
    // 8 comb filters for richer sound
    comb_filters: Vec<CombFilter>,
    
    // 4 allpass filters for better diffusion
    allpass_filters: Vec<AllpassFilter>,
    
    // Parameters
    room_size: f32,
    damping: f32,
    width: f32,
    wet_level: f32,
    dry_level: f32,
    
    // Early reflections for more realistic space
    early_tap1: usize,
    early_tap2: usize,
    early_gain: f32,
}

impl Reverb {
    pub fn new(sample_rate: f32, room_size: f32, damping: f32) -> Self {
        // Pre-delay: 10-50ms helps separate dry from wet
        let pre_delay_ms = 20.0;
        let pre_delay_samples = ((pre_delay_ms / 1000.0) * sample_rate) as usize;
        
        // Freeverb-style comb filter delays (in samples at 44.1kHz)
        // Scaled for the actual sample rate
        let scale = sample_rate / 44100.0;
        let comb_tunings = [
            1116, 1188, 1277, 1356, 1422, 1491, 1557, 1617
        ];
        
        let mut comb_filters = Vec::new();
        for &tuning in &comb_tunings {
            let delay_samples = (tuning as f32 * scale) as usize;
            comb_filters.push(CombFilter::new(delay_samples));
        }
        
        // Allpass filter delays
        let allpass_tunings = [556, 441, 341, 225];
        let mut allpass_filters = Vec::new();
        for &tuning in &allpass_tunings {
            let delay_samples = (tuning as f32 * scale) as usize;
            allpass_filters.push(AllpassFilter::new(delay_samples));
        }
        
        // Early reflection taps
        let early_tap1 = (0.007 * sample_rate) as usize;
        let early_tap2 = (0.013 * sample_rate) as usize;
        
        Self {
            pre_delay_buffer: vec![0.0; pre_delay_samples],
            pre_delay_index: 0,
            pre_delay_time: pre_delay_ms,
            comb_filters,
            allpass_filters,
            room_size: room_size.clamp(0.0, 0.98),
            damping: damping.clamp(0.0, 1.0),
            width: 1.0,
            wet_level: 0.3,
            dry_level: 0.7,
            early_tap1,
            early_tap2,
            early_gain: 0.2,
        }
    }
    
    pub fn set_mix(&mut self, mix: f32) {
        let mix = mix.clamp(0.0, 1.0);
        self.wet_level = mix * 0.3; // Scale down wet level
        self.dry_level = 1.0 - mix;
    }
}

impl MonoEffect for Reverb {

    fn process(&mut self, buffer: &mut [f32], _sample_rate: f32) {
        for sample in buffer.iter_mut() {
            let input = *sample;
            
            // Pre-delay
            let pre_delayed = self.pre_delay_buffer[self.pre_delay_index];
            self.pre_delay_buffer[self.pre_delay_index] = input;
            self.pre_delay_index = (self.pre_delay_index + 1) % self.pre_delay_buffer.len();
            
            // Input diffusion - spread the input across frequency spectrum
            let diffused_input = pre_delayed * 0.015; // Much lower input gain
            
            // Process through parallel comb filters
            let mut comb_sum = 0.0;
            for (i, comb) in self.comb_filters.iter_mut().enumerate() {
                // Vary feedback slightly per comb for richer sound
                let feedback = self.room_size * (0.98 - i as f32 * 0.01);
                comb_sum += comb.process(diffused_input, feedback, self.damping);
            }
            
            // Scale comb output
            let mut output = comb_sum / self.comb_filters.len() as f32;
            
            // Process through series allpass filters
            for allpass in &mut self.allpass_filters {
                output = allpass.process(output);
            }
            
            // Mix dry and wet signals
            *sample = input * self.dry_level + output * self.wet_level;
            
            // Soft clip to prevent harsh distortion
            *sample = sample.tanh();
        }
    }
    
    fn set_parameter(&mut self, index: u32, value: f32) {
        todo!()
    }
}

/// Improved Comb Filter with better damping
struct CombFilter {
    buffer: Vec<f32>,
    index: usize,
    filter_state: f32,
}

impl CombFilter {
    fn new(delay_samples: usize) -> Self {
        Self {
            buffer: vec![0.0; delay_samples],
            index: 0,
            filter_state: 0.0,
        }
    }
    
    fn process(&mut self, input: f32, feedback: f32, damping: f32) -> f32 {
        let delayed = self.buffer[self.index];
        
        // Improved damping filter (one-pole lowpass)
        let damp1 = damping;
        let damp2 = 1.0 - damping;
        self.filter_state = delayed * damp2 + self.filter_state * damp1;
        
        // Write to buffer with feedback
        self.buffer[self.index] = input + self.filter_state * feedback;
        self.index = (self.index + 1) % self.buffer.len();
        
        delayed
    }
}

/// Improved Allpass Filter
struct AllpassFilter {
    buffer: Vec<f32>,
    index: usize,
}

impl AllpassFilter {
    fn new(delay_samples: usize) -> Self {
        Self {
            buffer: vec![0.0; delay_samples],
            index: 0,
        }
    }
    
    fn process(&mut self, input: f32) -> f32 {
        let delayed = self.buffer[self.index];
        let feedback = 0.5;
        
        // Classic allpass structure
        let output = delayed - input;
        self.buffer[self.index] = input + delayed * feedback;
        self.index = (self.index + 1) % self.buffer.len();
        
        output
    }
}

use crate::{StereoEffect};

// ...existing Reverb struct and implementation...

/// Stereo Reverb Effect
/// 
/// Creates a spacious stereo reverb by using two slightly different
/// mono reverbs with cross-coupling between channels.
/// 
/// ## Parameters (inherited from mono Reverb):
/// 
/// - `room_size`: Controls the decay time
///   - Range: 0.0 to 0.98
///   - 0.0 = no reverb
///   - 0.5 = medium room
///   - 0.9 = large hall
///   - 0.98 = cathedral
/// 
/// - `damping`: High-frequency damping
///   - Range: 0.0 to 1.0
///   - 0.0 = bright/metallic
///   - 0.5 = natural
///   - 1.0 = very dark
pub struct StereoReverb {
    /// Left channel reverb processor
    left_reverb: Reverb,
    /// Right channel reverb processor  
    right_reverb: Reverb,
    /// Cross-feed amount between channels (0.0-1.0)
    /// Higher values create wider stereo image
    stereo_width: f32,
    /// Wet/dry mix (0.0 = fully dry, 1.0 = fully wet)
    wet_mix: f32,
}

impl StereoReverb {
    pub fn new(sample_rate: f32, room_size: f32, damping: f32) -> Self {
        // Create slightly different reverbs for each channel
        let left_reverb = Reverb::new(sample_rate, room_size, damping);
        let right_reverb = Reverb::new(
            sample_rate, 
            room_size * 0.98,  // Slightly different size
            damping            // Same damping
        );
        
        Self {
            left_reverb,
            right_reverb,
            stereo_width: 0.5,
            wet_mix: 0.3,
        }
    }
    
    /// Set the stereo width
    /// 
    /// # Arguments
    /// - `width`: 0.0 = mono (no stereo), 1.0 = maximum width
    pub fn set_stereo_width(&mut self, width: f32) {
        self.stereo_width = width.clamp(0.0, 1.0);
    }
    
    /// Set the wet/dry mix
    /// 
    /// # Arguments
    /// - `mix`: 0.0 = fully dry, 1.0 = fully wet
    pub fn set_mix(&mut self, mix: f32) {
        self.wet_mix = mix.clamp(0.0, 1.0);
        self.left_reverb.set_mix(mix);
        self.right_reverb.set_mix(mix);
    }
}

impl StereoEffect for StereoReverb {
    fn process(&mut self, left: &mut [f32], right: &mut [f32], _sample_rate: f32) {
        // Store original for stereo processing
        let len = left.len().min(right.len());
        
        // Process each channel with subtle cross-feed
        for i in 0..len {
            let left_in = left[i];
            let right_in = right[i];
            
            // Subtle stereo cross-feed
            left[i] = left_in + right_in * 0.1;
            right[i] = right_in + left_in * 0.1;
        }
        
        // Process each channel through its reverb
        self.left_reverb.process(left, _sample_rate);
        self.right_reverb.process(right, _sample_rate);
    }
    
    fn set_parameter(&mut self, index: u32, value: f32) {
        match index {
            0 => { // Room size
                self.left_reverb.room_size = value.clamp(0.0, 0.98);
                self.right_reverb.room_size = (value * 0.98).clamp(0.0, 0.98);
            }
            1 => { // Damping
                self.left_reverb.damping = value.clamp(0.0, 1.0);
                self.right_reverb.damping = value.clamp(0.0, 1.0);
            }
            2 => self.set_stereo_width(value), // Stereo width
            3 => self.set_mix(value),           // Wet/dry mix
            _ => {}
        }
    }
}