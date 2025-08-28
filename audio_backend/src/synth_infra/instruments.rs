use crate::{id::InstrumentId, MonoEffect, VoiceEffects};

/// A trait for a complete instrument, which is responsible for managing
/// its own voices and polyphony according to its specific behavior.
pub trait InstrumentTrait: Send + Sync {
    /// Returns the unique identifier for this instrument.
    fn id(&self) -> InstrumentId;

    /// Handles a note-on event for this instrument.
    /// It will decide whether to create a new voice, re-trigger an existing one, etc.
    fn note_on(&mut self, note: u8, velocity: u8);

    /// Handles a note-off event for a specific voice ID.
    fn note_off(&mut self);

    /// Processes all active voices for this instrument, adding their
    /// output to the main stereo buffers.
    fn process(&mut self, left_buf: &mut [f32], right_buf: &mut [f32], sample_rate: f32);

    /// Sets the stereo pan for this instrument.
    fn set_pan(&mut self, pan: f32);

    // /// Try to handle a synth-specific command
    // fn handle_command(&mut self, command: &PlayerCommand);

    // TODO: reconsider if the we should only handle planar data
    /// Add a mono effect to this voice's effect chain. Instruments process planar audio
    fn add_effect(&mut self, effect: Box<dyn MonoEffect>);

    /// Add a batch of pre-constructed per-voice effects. Default implementation uses the first
    /// element for mono instruments.
    fn add_voice_effects(&mut self, mut effects: VoiceEffects) {
        if let Some(first) = effects.pop() {
            self.add_effect(first);
        }
        // Remaining elements (if any) will be dropped here. Because VoiceEffects is an
        // ArrayVec with fixed capacity, dropping it does not deallocate heap memory.
    }

    /// Report instrument polyphony (number of simultaneously managed voices).
    /// Default is 1 (monophonic).
    fn polyphony(&self) -> usize {
        1
    }
}
