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
    InstrumentTrait, MonoEffect, SynthNode, Voice, VoiceEffects, VoiceTrait,
};

/// A Voice container used by instruments to handle envelope lifecycles and sample generation.
struct VoiceSlot<S: SynthNode> {
    /// The Voice used by the instrument, forwarding commands and handling the underlying SynthNode emitting the samples
    inner: Voice<S>,
    /// The Note ID currently assigned to this voice, ignored in monophonic instruments.
    note_id: Option<NoteId>,
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

    fn note_on(&mut self, note: u8, velocity: u8) {
        self.voice.inner.note_on(note, velocity);
    }

    fn note_off(&mut self) {
        self.voice.inner.note_off();
    }

    fn process(&mut self, left_buf: &mut [f32], right_buf: &mut [f32], sample_rate: f32) {
        self.voice.inner.process(left_buf, right_buf, sample_rate);
    }

    fn set_pan(&mut self, pan: f32) {
        self.voice.inner.set_pan(pan);
    }

    fn add_effect(&mut self, effect: Box<dyn MonoEffect>) {
        self.voice.inner.add_effect(effect);
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

/// Polyphonic instrument: multiple voices
pub struct PolyphonicInstrument<S: SynthNode> {
    instrument_id: crate::id::InstrumentId,
    voices: Vec<VoiceSlot<S>>,
}

impl<S: SynthNode> InstrumentTrait for PolyphonicInstrument<S> {
    fn id(&self) -> crate::id::InstrumentId {
        self.instrument_id
    }

    fn note_on(&mut self, note: u8, velocity: u8) {
        #[cfg(debug_assertions)]
        if crate::__rt_log_enabled(crate::__RtLogLevel::Debug) {
            crate::__emit_rt_log(
                crate::__RtLogLevel::Debug,
                format_args!("Looking for voice"),
            );
            for voice in &self.voices {
                crate::__emit_rt_log(
                    crate::__RtLogLevel::Debug,
                    format_args!(
                        "Checking voice with ID: {:#?}, state: {}",
                        voice.note_id,
                        voice.inner.is_active()
                    ),
                );
            }
        }
        // Find a free voice or a voice with the same note to retrigger envelope
        let free_voice = self
            .voices
            .iter_mut()
            .find(|slot| !slot.inner.is_active() || slot.note_id == Some(note));

        crate::rt_debug_log!(
            "Free voice found: {:?}, ID is: {:?}",
            free_voice.is_some(),
            free_voice.as_ref().map(|slot| slot.note_id)
        );

        if let Some(slot) = free_voice {
            // We found a free voice, so we can use it.
            slot.note_id = Some(note);
            slot.inner.note_on(note, velocity);
        } else {
            // No free voices. This is where you would implement voice stealing.
            // For now, we'll just ignore the new note.
        }
    }

    /// This mutes all voices.
    fn note_off(&mut self) {
        for voice in &mut self.voices {
            if voice.inner.is_active() {
                voice.inner.note_off();
            }
        }
    }

    fn process(&mut self, left_buf: &mut [f32], right_buf: &mut [f32], sample_rate: f32) {
        // process active voices
        for voice in self.voices.iter_mut() {
            voice.inner.process(left_buf, right_buf, sample_rate);
        }
    }

    fn set_pan(&mut self, pan: f32) {
        for voice in &mut self.voices {
            voice.inner.set_pan(pan);
        }
    }

    fn add_effect(&mut self, _effect: Box<dyn MonoEffect>) {
        // Polyphonic instruments require one effect instance per voice.
        // Use add_voice_effects with pre-constructed per-voice effects instead.
        crate::rt_warn_log!(
            "PolyphonicInstrument: add_effect is a no-op; use add_voice_effects instead"
        );
    }

    fn add_voice_effects(&mut self, effects: VoiceEffects) {
        for (slot, effect) in self.voices.iter_mut().zip(effects) {
            slot.inner.add_effect(effect);
        }
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
