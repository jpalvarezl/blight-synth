use crate::{
    id::{EffectId, InstrumentId, NoteEvent, NoteId},
    MonoEffect, VoiceEffects,
};

/// Why a prepared mono effect could not be installed on an instrument.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectInstallErrorKind {
    /// A polyphonic instrument requires one independently prepared effect per voice.
    UnsupportedForPolyphonicInstrument,
    /// The target effect chain has no remaining prepared capacity.
    ChainFull,
}

/// Typed effect-install rejection that preserves ownership for NRT retirement.
pub struct EffectInstallError {
    kind: EffectInstallErrorKind,
    effect: Box<dyn MonoEffect>,
}

impl EffectInstallError {
    pub fn new(kind: EffectInstallErrorKind, effect: Box<dyn MonoEffect>) -> Self {
        Self { kind, effect }
    }

    pub fn kind(&self) -> EffectInstallErrorKind {
        self.kind
    }

    pub fn into_effect(self) -> Box<dyn MonoEffect> {
        self.effect
    }
}

/// A trait for a complete instrument, which is responsible for managing
/// its own voices and polyphony according to its specific behavior.
pub trait InstrumentTrait: Send + Sync {
    /// Returns the unique identifier for this instrument.
    fn id(&self) -> InstrumentId;

    /// Handles a note-on event for this instrument.
    ///
    /// The [`NoteEvent`] bundles the stable identity ([`NoteId`]), the MIDI
    /// pitch to render, and the velocity so they cannot be mismatched. The
    /// identity is distinct from the pitch: the instrument decides whether to
    /// allocate a free voice, retrigger the voice already holding `event.id`,
    /// or steal an active voice when the fixed polyphony pool is exhausted.
    fn note_on(&mut self, event: NoteEvent);

    /// Releases only the voice that currently owns `note_id`, leaving every
    /// other sounding voice untouched. Unknown identities are a no-op.
    fn note_off(&mut self, note_id: NoteId);

    /// Releases every currently sounding voice (used for host-level
    /// all-notes-off, panic, and structural teardown).
    fn all_notes_off(&mut self);

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
    fn add_effect(&mut self, effect: Box<dyn MonoEffect>) -> Result<(), EffectInstallError>;

    /// Add a batch of pre-constructed per-voice effects. Default implementation uses the first
    /// element for mono instruments and returns every effect it could not install.
    fn add_voice_effects(&mut self, mut effects: VoiceEffects) -> VoiceEffects {
        let mut rejected = VoiceEffects::new();
        if !effects.is_empty() {
            let first = effects.remove(0);
            if let Err(error) = self.add_effect(first) {
                rejected.push(error.into_effect());
            }
        }
        rejected.extend(effects);
        rejected
    }

    /// Set a parameter on one of the instrument's effects.
    fn set_effect_parameter(&mut self, effect_id: EffectId, param_index: u32, value: f32);

    /// Resolve a concrete effect and invoke its existing infallible scalar setter.
    /// Generic/custom instruments support coalesced confirmation by overriding
    /// this method; absence is the conservative default.
    fn try_set_effect_parameter(
        &mut self,
        _effect_id: EffectId,
        _param_index: u32,
        _value: f32,
    ) -> bool {
        false
    }

    fn try_handle_command(&mut self, cmd: &crate::SynthCmd) -> bool;
}
