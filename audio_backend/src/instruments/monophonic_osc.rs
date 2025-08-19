use crate::id::InstrumentId;
use crate::{instruments::VoiceSlot, OscillatorNode};
use crate::{Envelope, InstrumentTrait, MonoEffectChain, SynthNode, Voice, VoiceTrait};

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
        let voice = VoiceSlot { synth_node: voice };
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
        VoiceTrait::note_on(&mut self.voice.synth_node, note, velocity);
    }

    fn note_off(&mut self) {
        VoiceTrait::note_off(&mut self.voice.synth_node);
    }

    fn process(&mut self, left_buf: &mut [f32], right_buf: &mut [f32], sample_rate: f32) {
        VoiceTrait::process(&mut self.voice.synth_node, left_buf, right_buf, sample_rate);
    }

    fn set_pan(&mut self, pan: f32) {
        todo!()
    }

    fn add_effect(&mut self, effect: Box<dyn crate::StereoEffect>) {
        todo!()
        // self.voice.synth_node.node.add_effect(effect);
    }
}
