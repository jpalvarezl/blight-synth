use crate::id::InstrumentId;
use crate::{instruments::VoiceSlot, OscillatorNode};
use crate::{Envelope, InstrumentTrait, MonoEffectChain, Voice, VoiceTrait};

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
}

impl InstrumentTrait for MonophonicOscillator {
    fn id(&self) -> crate::id::InstrumentId {
        self.instrument_id
    }

    fn note_on(&mut self, note: u8, velocity: u8) {
        VoiceTrait::note_on(&mut self.voice.inner, note, velocity);
    }

    fn note_off(&mut self) {
        VoiceTrait::note_off(&mut self.voice.inner);
    }

    fn process(&mut self, left_buf: &mut [f32], right_buf: &mut [f32], sample_rate: f32) {
        VoiceTrait::process(&mut self.voice.inner, left_buf, right_buf, sample_rate);
    }

    fn set_pan(&mut self, pan: f32) {
        VoiceTrait::set_pan(&mut self.voice.inner, pan);
    }

    fn add_effect(&mut self, _effect: Box<dyn crate::StereoEffect>) {
        // TODO: reconcile per voice MonoEffect and InstrumentEffect
        // VoiceTrait::add_effect(&mut self.voice.inner, effect);
    }
}
