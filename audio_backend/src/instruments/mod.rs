mod hihat;
mod kick_drum;
mod monophonic_osc;
mod polyphonic_osc;
mod snare_drum;
mod synth_nodes;

pub use hihat::*;
pub use kick_drum::*;
pub use monophonic_osc::*;
pub use polyphonic_osc::*;
pub use snare_drum::*;
pub use synth_nodes::*;

use crate::{id::NoteId, InstrumentTrait, MonoEffect, SynthNode, Voice, VoiceTrait};

struct VoiceSlot<S: SynthNode> {
    inner: Voice<S>,
    note_id: Option<NoteId>, // This is the MIDI value of the note we use for identifying which voice is playing it.
}

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

    fn set_effect_parameter(&mut self, effect_index: usize, param_index: u32, value: f32) {
        self.voice
            .inner
            .set_effect_parameter(effect_index, param_index, value);
    }

    fn try_handle_command(&mut self, cmd: &crate::SynthCmd) -> bool {
        self.voice.inner.try_handle_command(cmd)
    }
}