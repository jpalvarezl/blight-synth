use crate::MonoEffect;
use crate::StereoEffect;

/// Schroeder Reverb Implementation
/// 
/// This implements the classic Schroeder reverb architecture from his 1962 paper
/// "Natural Sounding Artificial Reverberation". The design uses:
/// 
/// - 4 parallel comb filters (create the initial echo density)
/// - 2 series allpass filters (add diffusion without coloration)
/// 
/// ## Algorithm Overview:
/// ```
/// Input -> [Comb1] ----+
///       -> [Comb2] ----+
///       -> [Comb3] ----+--> Sum -> [Allpass1] -> [Allpass2] -> Output
///       -> [Comb4] ----+
/// ```
/// 
/// ## Parameters:
/// 
/// - `room_size`: Controls the decay time (RT60)
///   - Range: 0.0 to 0.99
///   - 0.0 = no reverb (completely dry)
///   - 0.5 = medium room (~1 second decay)
///   - 0.9 = large hall (~3-4 seconds decay)
///   - 0.99 = cathedral (very long decay, >5 seconds)
///   - Values >= 1.0 will cause unstable feedback!
/// 
/// - `damping`: High-frequency damping (absorption)
///   - Range: 0.0 to 1.0
///   - 0.0 = no damping (bright, metallic sound)
///   - 0.5 = natural room damping
///   - 1.0 = maximum damping (very dark, muffled)
///   - Simulates how real rooms absorb high frequencies more than low
/// 
/// ## Technical Details:
/// The comb filter delays are prime numbers to minimize harmonic buildup:
/// - 29.7ms, 37.1ms, 41.1ms, 43.7ms
/// 
/// The allpass delays are kept short for dense diffusion:
/// - 5.0ms, 1.7ms
pub struct Reverb {
    comb_filters: [CombFilter; 4],
    allpass_filters: [AllpassFilter; 2],
    room_size: f32,
    damping: f32,
    /// Wet signal level (0.0 to 1.0)
    wet_gain: f32,
    /// Dry signal level (0.0 to 1.0)
    dry_gain: f32,
}

impl Reverb {
    /// Create a new reverb effect
    /// 
    /// # Arguments
    /// - `sample_rate`: The audio sample rate in Hz (typically 44100 or 48000)
    /// - `room_size`: Reverb decay time (0.0-0.99, see struct docs)
    /// - `damping`: High-frequency damping (0.0-1.0, see struct docs)
    pub fn new(sample_rate: f32, room_size: f32, damping: f32) -> Self {
        // Clamp parameters to safe ranges
        let room_size = room_size.clamp(0.0, 0.99);
        let damping = damping.clamp(0.0, 1.0);
        
        // Comb filter delay times (in milliseconds)
        // These are tuned to avoid harmonic relationships
        let comb_delays = [29.7, 37.1, 41.1, 43.7];
        let comb_filters = [
            CombFilter::new(sample_rate, comb_delays[0]),
            CombFilter::new(sample_rate, comb_delays[1]),
            CombFilter::new(sample_rate, comb_delays[2]),
            CombFilter::new(sample_rate, comb_delays[3]),
        ];

        // Allpass filter delay times (in milliseconds)
        // Short delays for quick diffusion
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
            wet_gain: 0.3,  // 30% wet signal
            dry_gain: 0.7,  // 70% dry signal
        }
    }
    
    /// Set the wet/dry mix
    /// 
    /// # Arguments
    /// - `wet_mix`: 0.0 = fully dry, 1.0 = fully wet
    pub fn set_mix(&mut self, wet_mix: f32) {
        let wet_mix = wet_mix.clamp(0.0, 1.0);
        self.wet_gain = wet_mix;
        self.dry_gain = 1.0 - wet_mix;
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
            // Average the 4 comb outputs to maintain consistent level
            comb_sum *= 0.25;
            
            // Process through series allpass filters for diffusion
            let mut allpass_out = comb_sum;
            for allpass in &mut self.allpass_filters {
                allpass_out = allpass.process(allpass_out);
            }
            
            // Mix wet and dry signals with limiting
            let wet = allpass_out * self.wet_gain;
            let dry = input * self.dry_gain;
            // Hard limit to prevent speaker damage
            *sample = (dry + wet).clamp(-1.0, 1.0);
        }
    }
    
    fn set_parameter(&mut self, _index: u32, _value: f32) {
        
        // TODO: Implement parameter control
        // match index {
        //     0 => self.room_size = value.clamp(0.0, 0.99),
        //     1 => self.damping = value.clamp(0.0, 1.0),
        //     2 => self.wet_gain = value.clamp(0.0, 1.0),
        //     3 => self.dry_gain = value.clamp(0.0, 1.0),
        //     _ => eprintln!("Invalid parameter index"),
        // }
    }
}

/// Feedback Comb Filter
/// 
/// Creates discrete echoes with exponential decay.
/// Essential building block for reverb algorithms.
/// 
/// Signal flow:
/// ```
/// Input ---+---> Output
///          |
///          v
///     [Delay Line]
///          |
///          v
///     [Damping Filter]
///          |
///          +--[Feedback]--+
/// ```
struct CombFilter {
    /// Circular buffer for delay line
    buffer: Vec<f32>,
    /// Current write position in buffer
    index: usize,
    /// State variable for one-pole lowpass filter (damping)
    filtered: f32,
}

impl CombFilter {
    /// Create a new comb filter
    /// 
    /// # Arguments
    /// - `sample_rate`: Audio sample rate in Hz
    /// - `delay_ms`: Delay time in milliseconds
    fn new(sample_rate: f32, delay_ms: f32) -> Self {
        let delay_samples = ((delay_ms / 1000.0) * sample_rate) as usize;
        Self {
            buffer: vec![0.0; delay_samples],
            index: 0,
            filtered: 0.0,
        }
    }

    /// Process one sample through the comb filter
    /// 
    /// # Arguments
    /// - `input`: Input sample
    /// - `room_size`: Feedback amount (0.0-0.99)
    /// - `damping`: Lowpass filter coefficient (0.0-1.0)
    fn process(&mut self, input: f32, room_size: f32, damping: f32) -> f32 {
        // Read delayed sample
        let delayed = self.buffer[self.index];
        
        // Apply one-pole lowpass filter for damping
        // This simulates high-frequency absorption in real rooms
        self.filtered = delayed * (1.0 - damping) + self.filtered * damping;
        
        // Write new sample with scaled feedback
        self.buffer[self.index] = input + self.filtered * room_size;
        
        // Advance circular buffer index
        self.index = (self.index + 1) % self.buffer.len();
        
        // Return the delayed sample (not the filtered one)
        delayed
    }
}

/// Allpass Filter
/// 
/// Adds diffusion without changing the frequency response.
/// Makes reverb sound smooth rather than "grainy".
/// 
/// Transfer function: H(z) = (-g + z^-D) / (1 - g*z^-D)
/// where g = 0.5 (fixed feedback coefficient)
/// 
/// Signal flow:
/// ```
/// Input --[+]---> [Delay] ---> [+]---> Output
///          ^                    |
///          |                    v
///          +----[*-g]<----[*g]--+
/// ```
struct AllpassFilter {
    /// Circular buffer for delay line
    buffer: Vec<f32>,
    /// Current position in buffer
    index: usize,
}

impl AllpassFilter {
    /// Create a new allpass filter
    /// 
    /// # Arguments
    /// - `sample_rate`: Audio sample rate in Hz
    /// - `delay_ms`: Delay time in milliseconds (typically 1-10ms)
    fn new(sample_rate: f32, delay_ms: f32) -> Self {
        let delay_samples = ((delay_ms / 1000.0) * sample_rate) as usize;
        Self {
            buffer: vec![0.0; delay_samples],
            index: 0,
        }
    }

    /// Process one sample through the allpass filter
    /// 
    /// Uses a fixed feedback coefficient of 0.5 for stability
    fn process(&mut self, input: f32) -> f32 {
        let delayed = self.buffer[self.index];
        let output = -input + delayed;
        
        // Fixed feedback of 0.5 for good diffusion
        self.buffer[self.index] = input + delayed * 0.5;
        
        // Advance circular buffer
        self.index = (self.index + 1) % self.buffer.len();
        
        output
    }
}

/// Stereo Reverb Effect
/// 
/// Creates a spacious stereo reverb by using two slightly different
/// mono reverbs with cross-coupling between channels.
/// 
/// ## Stereo Processing:
/// ```
/// Left Input  ---> [Left Reverb]  ---> Left Output
///     |                                     ^
///     +----[Cross-feed]----+               |
///                          v               |
/// Right Input --> [Right Reverb] ---> Right Output
/// ```
/// 
/// ## Parameters (inherited from mono Reverb):
/// 
/// - `room_size`: Controls the decay time
///   - Range: 0.0 to 0.99
///   - 0.0 = no reverb
///   - 0.5 = medium room
///   - 0.9 = large hall
///   - 0.99 = cathedral
/// 
/// - `damping`: High-frequency damping
///   - Range: 0.0 to 1.0
///   - 0.0 = bright/metallic
///   - 0.5 = natural
///   - 1.0 = very dark
/// 
/// ## Stereo Imaging:
/// The left and right reverbs have slightly different parameters
/// to create stereo width without phase cancellation:
/// - Right room_size = Left room_size × 0.97
/// - Right damping = Left damping × 1.03
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
    /// Create a new stereo reverb
    /// 
    /// # Arguments
    /// - `sample_rate`: Audio sample rate in Hz (e.g., 44100)
    /// - `room_size`: Reverb decay (0.0-0.99)
    /// - `damping`: High-frequency absorption (0.0-1.0)
    /// 
    /// # Example
    /// ```
    /// // Medium hall with natural damping
    /// let reverb = StereoReverb::new(44100.0, 0.7, 0.5);
    /// ```
    pub fn new(sample_rate: f32, room_size: f32, damping: f32) -> Self {
        // Create slightly different reverbs for each channel
        let left_reverb = Reverb::new(sample_rate, room_size, damping);
        let right_reverb = Reverb::new(
            sample_rate, 
            room_size * 0.97,  // Slightly smaller room
            damping * 1.03     // Slightly more damping
        );
        
        let mut reverb = Self {
            left_reverb,
            right_reverb,
            stereo_width: 0.3,  // 30% cross-feed
            wet_mix: 0.3,       // 30% wet signal
        };
        
        // Set a reasonable mix for the reverbs
        reverb.set_mix(0.3); // 30% wet is a good starting point
        reverb
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
    fn process(&mut self, left: &mut [f32], right: &mut [f32], sample_rate: f32) {
        // IMPORTANT: Process in-place without allocating vectors!
        for i in 0..left.len().min(right.len()) {
            let left_in = left[i];
            let right_in = right[i];
            
            // Create cross-feed for stereo width
            // Scale properly to avoid clipping when summing
            let left_feed = left_in * (1.0 - self.stereo_width * 0.5) 
                          + right_in * (self.stereo_width * 0.5);
            let right_feed = right_in * (1.0 - self.stereo_width * 0.5) 
                           + left_in * (self.stereo_width * 0.5);
            
            // Process single samples through the mono reverbs
            // The mono reverbs already handle wet/dry mixing internally
            left[i] = left_feed;
            right[i] = right_feed;
        }
        
        // Process the entire buffers through the reverbs
        // This modifies the buffers in-place
        self.left_reverb.process(left, sample_rate);
        self.right_reverb.process(right, sample_rate);
    }
    
    fn set_parameter(&mut self, index: u32, value: f32) {
        match index {
            0 => { // Room size
                self.left_reverb.room_size = value.clamp(0.0, 0.99);
                self.right_reverb.room_size = (value * 0.97).clamp(0.0, 0.99);
            }
            1 => { // Damping
                self.left_reverb.damping = value.clamp(0.0, 1.0);
                self.right_reverb.damping = (value * 1.03).clamp(0.0, 1.0);
            }
            2 => self.set_stereo_width(value), // Stereo width
            3 => self.set_mix(value),           // Wet/dry mix
            _ => eprintln!("Invalid parameter index for StereoReverb"),
        }
    }
}