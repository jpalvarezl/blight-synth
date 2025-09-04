use crate::id::InstrumentId;
use crate::{instruments::VoiceSlot, OscillatorNode};
use crate::{Envelope, InstrumentTrait, MonoEffect, MonoEffectChain, Voice, VoiceTrait, Waveform};

pub struct MonophonicOscillator {
    instrument_id: InstrumentId,
    voice: VoiceSlot<OscillatorNode>,
}

impl MonophonicOscillator {
    pub fn new(instrument_id: InstrumentId, pan: f32, sample_rate: f32) -> Self {
        let envelope = Envelope::new(sample_rate);
        let voice = Voice::new(
            0,
            OscillatorNode::new(),
            envelope,
            pan,
            MonoEffectChain::new(10),
        );
        // Note ID is unused in a monophonic instrument.
        let voice = VoiceSlot {
            inner: voice,
            note_id: None,
        };
        MonophonicOscillator {
            instrument_id,
            voice,
        }
    }

    pub fn new_with_waveform(
        instrument_id: InstrumentId,
        pan: f32,
        sample_rate: f32,
        waveform: Waveform,
    ) -> Self {
        let envelope = Envelope::new(sample_rate);
        let voice = Voice::new(
            0,
            OscillatorNode::new_with_waveform(waveform),
            envelope,
            pan,
            MonoEffectChain::new(10),
        );
        // Note ID is unused in a monophonic instrument.
        let voice = VoiceSlot {
            inner: voice,
            note_id: None,
        };
        MonophonicOscillator {
            instrument_id,
            voice,
        }
    }
}

impl InstrumentTrait for MonophonicOscillator {
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
        self.voice.inner.set_effect_parameter(effect_index, param_index, value);
    }
}
