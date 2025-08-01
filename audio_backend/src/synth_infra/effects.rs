/// A trait for any real-time audio effect. Not to be confused with command transformer effects,
/// such as an Arpeggiator for example, which operates from the NRT world and would constitute a
/// sequencer effect.
///
/// An `Effect` processes audio in-place on a pair of stereo buffers. Normally used as master effects.
/// It must be `Send + Sync` so that it can be safely created in the NRT world
/// and sent to the RT audio thread.
pub trait StereoEffect: Send + Sync {
    /// Processes a block of stereo audio.
    ///
    /// This method is called on every audio block and must be real-time safe.
    /// It should not allocate memory, block, or perform any non-deterministic operations.
    ///
    /// # Arguments
    /// * `left_buf` - A mutable slice representing the left channel's audio buffer.
    /// * `right_buf` - A mutable slice representing the right channel's audio buffer.
    /// * `sample_rate` - The current sample rate of the audio engine.
    fn process(&mut self, left_buf: &mut [f32], right_buf: &mut [f32], sample_rate: f32);

    /// Sets a parameter on the effect.
    ///
    /// This is the hook for controlling the effect from the NRT world via commands.
    /// The `index` allows a single effect to have multiple controllable parameters.
    ///
    /// # Arguments
    /// * `index` - The zero-based index of the parameter to change.
    /// * `value` - The new value for the parameter.
    fn set_parameter(&mut self, index: u32, value: f32);
}

/// A chain of audio effects that are processed in sequence.
pub struct StereoEffectChain {
    effects: Vec<Box<dyn StereoEffect>>,
}

impl StereoEffectChain {
    /// Creates a new, empty effect chain with a pre-allocated capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            effects: Vec::with_capacity(capacity),
        }
    }

    /// Adds a new effect to the chain. This is called on the audio thread
    /// in response to a command from the NRT world.
    /// As long as capacity is not exceeded, this will not allocate.
    pub fn add_effect(&mut self, effect: Box<dyn StereoEffect>) {
        if self.effects.len() < self.effects.capacity() {
            self.effects.push(effect);
        } else {
            // In a real application, you might log a warning here.
            eprintln!("Effect chain is full, cannot add new effect.");
        }
    }

    /// Processes the audio through all effects in the chain, in order.
    pub fn process(&mut self, left_buf: &mut [f32], right_buf: &mut [f32], sample_rate: f32) {
        for effect in &mut self.effects {
            effect.process(left_buf, right_buf, sample_rate);
        }
    }
}

/// A trait for mono audio effects. These are typically used for per-voice effects
/// that operate on a single channel of audio.
pub trait MonoEffect: Send + Sync {
    /// Processes a block of mono audio.
    ///
    /// This method is called on every audio block and must be real-time safe.
    /// It should not allocate memory, block, or perform any non-deterministic operations.
    ///
    /// # Arguments
    /// * `buf` - A mutable slice representing the audio buffer.
    /// * `sample_rate` - The current sample rate of the audio engine.
    fn process(&mut self, buf: &mut [f32], sample_rate: f32);

    /// Sets a parameter on the effect.
    ///
    /// This is the hook for controlling the effect from the NRT world via commands.
    /// The `index` allows a single effect to have multiple controllable parameters.
    ///
    /// # Arguments
    /// * `index` - The zero-based index of the parameter to change.
    /// * `value` - The new value for the parameter.
    fn set_parameter(&mut self, index: u32, value: f32);
}

/// A chain of mono audio effects for Voices.
pub struct MonoEffectChain {
    effects: Vec<Box<dyn MonoEffect>>,
}

impl MonoEffectChain {
    /// Creates a new, empty mono effect chain with a pre-allocated capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            effects: Vec::with_capacity(capacity),
        }
    }

    /// Adds a new mono effect to the chain. This is called on the audio thread
    /// in response to a command from the NRT world.
    /// As long as capacity is not exceeded, this will not allocate.
    pub fn add_effect(&mut self, effect: Box<dyn MonoEffect>) {
        if self.effects.len() < self.effects.capacity() {
            self.effects.push(effect);
        } else {
            // In a real application, you might log a warning here.
            eprintln!("Mono effect chain is full, cannot add new effect.");
        }
    }

    /// Processes the audio through all mono effects in the chain, in order.
    pub fn process(&mut self, buf: &mut [f32], sample_rate: f32) {
        for effect in &mut self.effects {
            effect.process(buf, sample_rate);
        }
    }
}
