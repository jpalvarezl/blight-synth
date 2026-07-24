use crate::{
    id::{EffectId, InstrumentId},
    MonoEffect, VoiceEffects,
};

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
    /// Add a mono effect to this instrument's effect chain.
    ///
    /// On rejection, returns the exact boxed effect so the RT caller can transfer
    /// it to NRT retirement instead of dropping/deallocating it in the callback.
    /// Polyphonic instruments reject this single-effect form because each voice
    /// requires its own prepared effect instance.
    fn add_effect(&mut self, effect: Box<dyn MonoEffect>) -> Result<(), Box<dyn MonoEffect>>;

    /// Add a batch of pre-constructed per-voice effects. Default implementation uses the first
    /// element for mono instruments and returns every effect it could not install.
    fn add_voice_effects(&mut self, mut effects: VoiceEffects) -> VoiceEffects {
        let mut rejected = VoiceEffects::new();
        if !effects.is_empty() {
            let first = effects.remove(0);
            if let Err(effect) = self.add_effect(first) {
                rejected.push(effect);
            }
        }
        rejected.extend(effects);
        rejected
    }

    /// Set a parameter on one of the instrument's effects.
    fn set_effect_parameter(&mut self, effect_id: EffectId, param_index: u32, value: f32);

    fn try_handle_command(&mut self, cmd: &crate::SynthCmd) -> bool;
}
