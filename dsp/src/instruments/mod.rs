mod hihat;
mod kick_drum;
mod monophonic_osc;
mod moog_dfam;
mod polyphonic_osc;
mod sample_player;
mod snare_drum;
mod synth_nodes;

pub use hihat::*;
pub use kick_drum::*;
pub use monophonic_osc::*;
pub use moog_dfam::*;
pub use polyphonic_osc::*;
pub use sample_player::*;
pub use snare_drum::*;
pub use synth_nodes::*;

use crate::{
    id::{EffectId, NoteId},
    EffectInstallError, EffectInstallErrorKind, InstrumentTrait, MonoEffect, SynthNode, Voice,
    VoiceEffects, VoiceTrait,
};

/// A Voice container used by instruments to handle envelope lifecycles and sample generation.
struct VoiceSlot<S: SynthNode> {
    /// The Voice used by the instrument, forwarding commands and handling the underlying SynthNode emitting the samples
    inner: Voice<S>,
    /// The stable identity of the note currently assigned to this voice, if any.
    /// Ignored by monophonic instruments; used by polyphonic instruments to
    /// target note-off and to retrigger a repeated note.
    note_id: Option<NoteId>,
    /// Monotonic sequence number stamped on each note-on. Lower values are
    /// older, giving polyphonic instruments a deterministic oldest-first
    /// voice-stealing order without wall-clock time or allocation.
    age: u64,
}

impl<S: SynthNode> VoiceSlot<S> {
    /// Wraps a prepared voice in an idle, unassigned slot.
    fn new(inner: Voice<S>) -> Self {
        Self {
            inner,
            note_id: None,
            age: 0,
        }
    }
}

/// Monophonic instrument: only one voice, no polyphony.
pub struct MonophonicInstrument<S: SynthNode> {
    instrument_id: crate::id::InstrumentId,
    voice: VoiceSlot<S>,
}

impl<S: SynthNode> InstrumentTrait for MonophonicInstrument<S> {
    fn id(&self) -> crate::id::InstrumentId {
        self.instrument_id
    }

    fn note_on(&mut self, note_id: NoteId, note: u8, velocity: u8) {
        // A monophonic instrument always reuses its single voice; the identity
        // is recorded only so a targeted note-off can match it.
        self.voice.note_id = Some(note_id);
        self.voice.inner.note_on(note, velocity);
    }

    fn note_off(&mut self, note_id: NoteId) {
        // Release only when the identity matches and the voice is still sounding,
        // so a duplicate/late note-off cannot re-gate an already-idle envelope.
        if self.voice.note_id == Some(note_id) && self.voice.inner.is_active() {
            self.voice.inner.note_off();
        }
    }

    fn all_notes_off(&mut self) {
        self.voice.inner.note_off();
    }

    fn process(&mut self, left_buf: &mut [f32], right_buf: &mut [f32], sample_rate: f32) {
        self.voice.inner.process(left_buf, right_buf, sample_rate);
    }

    fn set_pan(&mut self, pan: f32) {
        self.voice.inner.set_pan(pan);
    }

    fn add_effect(&mut self, effect: Box<dyn MonoEffect>) -> Result<(), EffectInstallError> {
        self.voice
            .inner
            .add_effect(effect)
            .map_err(|effect| EffectInstallError::new(EffectInstallErrorKind::ChainFull, effect))
    }

    fn set_effect_parameter(&mut self, effect_id: EffectId, param_index: u32, value: f32) {
        self.voice
            .inner
            .set_effect_parameter(effect_id, param_index, value);
    }

    fn try_handle_command(&mut self, cmd: &crate::SynthCmd) -> bool {
        self.voice.inner.try_handle_command(cmd)
    }
}

/// Polyphonic instrument: a fixed, preallocated pool of voices.
///
/// The voice pool is sized once (off the audio thread) and never grows, so
/// note-on, note-off, and voice stealing all run in bounded time without heap
/// activity on the callback.
pub struct PolyphonicInstrument<S: SynthNode> {
    instrument_id: crate::id::InstrumentId,
    voices: Vec<VoiceSlot<S>>,
    /// Monotonic counter used to stamp voice ages for deterministic stealing.
    next_age: u64,
}

impl<S: SynthNode> PolyphonicInstrument<S> {
    /// Picks the slot to (re)use for a note-on, following a deterministic,
    /// allocation-free policy:
    ///
    /// 1. Retrigger the voice already holding `note_id` (repeated/legato note).
    /// 2. Otherwise use the lowest-index idle voice.
    /// 3. Otherwise steal the oldest active voice (smallest age); ties break
    ///    toward the lowest index, so the choice is fully deterministic.
    fn allocate_slot(&mut self, note_id: NoteId) -> usize {
        let mut free: Option<usize> = None;
        let mut oldest = 0usize;
        for (index, slot) in self.voices.iter().enumerate() {
            if slot.note_id == Some(note_id) && slot.inner.is_active() {
                return index;
            }
            if free.is_none() && !slot.inner.is_active() {
                free = Some(index);
            }
            if slot.age < self.voices[oldest].age {
                oldest = index;
            }
        }
        free.unwrap_or(oldest)
    }
}

impl<S: SynthNode> InstrumentTrait for PolyphonicInstrument<S> {
    fn id(&self) -> crate::id::InstrumentId {
        self.instrument_id
    }

    fn note_on(&mut self, note_id: NoteId, note: u8, velocity: u8) {
        if self.voices.is_empty() {
            return;
        }
        let index = self.allocate_slot(note_id);
        let age = self.next_age;
        self.next_age = self.next_age.wrapping_add(1);
        let slot = &mut self.voices[index];
        slot.note_id = Some(note_id);
        slot.age = age;
        slot.inner.note_on(note, velocity);
    }

    /// Releases only the voice that owns `note_id`, leaving the rest sounding.
    fn note_off(&mut self, note_id: NoteId) {
        for slot in &mut self.voices {
            // Guard on `is_active`: a stale `note_id` can linger on a slot whose
            // voice has already finished releasing. Gating an idle envelope off
            // would push it back into its release phase and make `is_active`
            // report true again, stealing a slot from `allocate_slot`.
            if slot.note_id == Some(note_id) && slot.inner.is_active() {
                slot.inner.note_off();
            }
        }
    }

    fn all_notes_off(&mut self) {
        for slot in &mut self.voices {
            if slot.inner.is_active() {
                slot.inner.note_off();
            }
        }
    }

    fn process(&mut self, left_buf: &mut [f32], right_buf: &mut [f32], sample_rate: f32) {
        // Skip idle voices: an inactive voice contributes only silence, so
        // stepping it would waste callback budget without changing the output.
        for slot in self.voices.iter_mut() {
            if slot.inner.is_active() {
                slot.inner.process(left_buf, right_buf, sample_rate);
            }
        }
    }

    fn set_pan(&mut self, pan: f32) {
        for voice in &mut self.voices {
            voice.inner.set_pan(pan);
        }
    }

    fn add_effect(&mut self, effect: Box<dyn MonoEffect>) -> Result<(), EffectInstallError> {
        // Reject without dropping: polyphonic instruments require one prepared
        // effect instance per voice. The caller retires this returned allocation
        // and can surface the unsupported operation from NRT.
        Err(EffectInstallError::new(
            EffectInstallErrorKind::UnsupportedForPolyphonicInstrument,
            effect,
        ))
    }

    fn add_voice_effects(&mut self, effects: VoiceEffects) -> VoiceEffects {
        let mut rejected = VoiceEffects::new();
        let mut voices = self.voices.iter_mut();
        for effect in effects {
            if let Some(slot) = voices.next() {
                if let Err(effect) = slot.inner.add_effect(effect) {
                    rejected.push(effect);
                }
            } else {
                rejected.push(effect);
            }
        }
        rejected
    }

    fn set_effect_parameter(&mut self, effect_id: EffectId, param_index: u32, value: f32) {
        for voice in &mut self.voices {
            voice
                .inner
                .set_effect_parameter(effect_id, param_index, value);
        }
    }

    // TODO this is very dodgy, we are only stating the command was handled if at least one voice handled it
    fn try_handle_command(&mut self, cmd: &crate::SynthCmd) -> bool {
        let mut handled = false;
        for voice in &mut self.voices {
            if voice.inner.try_handle_command(cmd) {
                handled = true;
            }
        }
        handled
    }
}

#[cfg(test)]
mod polyphony_tests {
    use super::*;
    use crate::id::NoteId;
    use crate::{MonoEffectChain, SynthNode, Voice};

    /// A deterministic, allocation-free voice source whose activity is driven
    /// directly by note-on/note-off, so tests can observe voice allocation,
    /// targeting, and stealing without depending on envelope timing.
    #[derive(Default)]
    struct TestNode {
        active: bool,
        note_ons: u32,
        note_offs: u32,
    }

    impl SynthNode for TestNode {
        fn process(&mut self, output_buffer: &mut [f32], _sample_rate: f32) {
            output_buffer.fill(0.0);
        }
        fn note_on(&mut self, _note: u8, _velocity: u8) {
            self.active = true;
            self.note_ons += 1;
        }
        fn note_off(&mut self) {
            self.active = false;
            self.note_offs += 1;
        }
        fn is_active(&self) -> bool {
            self.active
        }
    }

    fn poly(max_polyphony: usize) -> PolyphonicInstrument<TestNode> {
        let voices = (0..max_polyphony)
            .map(|_| {
                VoiceSlot::new(Voice::new_no_envelope(
                    0,
                    TestNode::default(),
                    0.0,
                    MonoEffectChain::new(1),
                ))
            })
            .collect();
        PolyphonicInstrument {
            instrument_id: 1,
            voices,
            next_age: 0,
        }
    }

    fn active_voices(instrument: &PolyphonicInstrument<TestNode>) -> usize {
        instrument
            .voices
            .iter()
            .filter(|slot| slot.inner.is_active())
            .count()
    }

    #[test]
    fn repeated_note_with_same_identity_reuses_one_voice() {
        let mut instrument = poly(4);
        let id = NoteId::from_pitch(60);

        instrument.note_on(id, 60, 100);
        instrument.note_on(id, 60, 100);

        // A repeated note under the same identity retriggers the same slot
        // instead of consuming a second voice.
        assert_eq!(active_voices(&instrument), 1);
        assert_eq!(instrument.voices[0].note_id, Some(id));
        assert_eq!(instrument.voices[0].inner.node.note_ons, 2);
    }

    #[test]
    fn overlapping_notes_at_same_pitch_use_distinct_voices() {
        let mut instrument = poly(4);
        let first = NoteId(1);
        let second = NoteId(2);

        // Same MIDI pitch, distinct identities: two independently addressable
        // sounding voices.
        instrument.note_on(first, 60, 100);
        instrument.note_on(second, 60, 100);

        assert_eq!(active_voices(&instrument), 2);
        assert_eq!(instrument.voices[0].note_id, Some(first));
        assert_eq!(instrument.voices[1].note_id, Some(second));
    }

    #[test]
    fn targeted_note_off_releases_only_the_matching_voice() {
        let mut instrument = poly(4);
        let keep = NoteId(1);
        let release = NoteId(2);
        instrument.note_on(keep, 60, 100);
        instrument.note_on(release, 64, 100);

        instrument.note_off(release);

        assert!(instrument.voices[0].inner.is_active(), "kept note released");
        assert!(
            !instrument.voices[1].inner.is_active(),
            "targeted note not released"
        );
        assert_eq!(active_voices(&instrument), 1);
    }

    #[test]
    fn all_notes_off_releases_every_sounding_voice() {
        let mut instrument = poly(4);
        instrument.note_on(NoteId(1), 60, 100);
        instrument.note_on(NoteId(2), 64, 100);
        instrument.note_on(NoteId(3), 67, 100);

        instrument.all_notes_off();

        assert_eq!(active_voices(&instrument), 0);
    }

    #[test]
    fn voice_exhaustion_steals_the_oldest_voice_deterministically() {
        let mut instrument = poly(2);
        let oldest = NoteId(1);
        let newer = NoteId(2);
        let stealer = NoteId(3);

        instrument.note_on(oldest, 60, 100); // age 0 -> slot 0
        instrument.note_on(newer, 64, 100); // age 1 -> slot 1
        // Pool exhausted: the third note steals the oldest (slot 0), not slot 1.
        instrument.note_on(stealer, 67, 100);

        assert_eq!(instrument.voices[0].note_id, Some(stealer));
        assert_eq!(instrument.voices[1].note_id, Some(newer));
        assert_eq!(active_voices(&instrument), 2);

        // The stolen identity is gone; the survivor still responds to targeting.
        instrument.note_off(oldest);
        assert_eq!(active_voices(&instrument), 2);
        instrument.note_off(newer);
        assert_eq!(active_voices(&instrument), 1);
    }

    #[test]
    fn freed_voices_are_reused_before_stealing() {
        let mut instrument = poly(2);
        instrument.note_on(NoteId(1), 60, 100);
        instrument.note_on(NoteId(2), 64, 100);
        instrument.note_off(NoteId(1)); // slot 0 now idle

        // A new note should take the idle slot 0 rather than steal slot 1.
        instrument.note_on(NoteId(3), 67, 100);
        assert_eq!(instrument.voices[0].note_id, Some(NoteId(3)));
        assert!(instrument.voices[0].inner.is_active());
        assert_eq!(instrument.voices[1].note_id, Some(NoteId(2)));
        assert!(instrument.voices[1].inner.is_active());
    }

    #[test]
    fn duplicate_note_off_does_not_re_gate_an_idle_voice() {
        // Regression: envelope-backed voices report `is_active` through their
        // release phase, so a stale `note_id` must not be re-gated once idle.
        // A duplicate/late note-off must be a no-op on the finished voice.
        let mut instrument = poly(2);
        instrument.note_on(NoteId(1), 60, 100);
        instrument.note_on(NoteId(2), 64, 100);

        instrument.note_off(NoteId(1));
        assert!(!instrument.voices[0].inner.is_active());
        assert_eq!(instrument.voices[0].inner.node.note_offs, 1);

        // Duplicate targeted note-off on the now-idle slot: guarded out.
        instrument.note_off(NoteId(1));
        assert!(!instrument.voices[0].inner.is_active());
        assert_eq!(
            instrument.voices[0].inner.node.note_offs, 1,
            "idle voice must not be re-gated by a stale note-off"
        );

        // A fresh note reuses the idle slot instead of stealing the survivor.
        instrument.note_on(NoteId(3), 67, 100);
        assert_eq!(instrument.voices[0].note_id, Some(NoteId(3)));
        assert!(instrument.voices[1].inner.is_active());
        assert_eq!(instrument.voices[1].note_id, Some(NoteId(2)));
    }

    #[test]
    fn empty_voice_pool_ignores_notes_without_panicking() {
        let mut instrument = poly(0);
        instrument.note_on(NoteId(1), 60, 100);
        instrument.note_off(NoteId(1));
        instrument.all_notes_off();
        assert_eq!(active_voices(&instrument), 0);
    }
}
