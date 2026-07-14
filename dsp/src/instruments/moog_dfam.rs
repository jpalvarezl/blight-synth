use crate::id::InstrumentId;
use crate::instruments::{
    MonophonicInstrument, MoogNode, NoiseGenerator, OscillatorNode, VoiceSlot, Waveform,
};
use crate::{Envelope, MonoEffectChain, Voice};

pub type MoogDFAM = MonophonicInstrument<MoogNode>;

impl MoogDFAM {
    pub fn new(instrument_id: InstrumentId, pan: f32, sample_rate: f32) -> Self {
        let envelope = Envelope::new(sample_rate);
        let voice = Voice::new(
            0,
            MoogNode::new(
                OscillatorNode::new_with_waveform(Waveform::Square),
                OscillatorNode::new_with_waveform(Waveform::Square),
                NoiseGenerator::default(),
            ),
            envelope,
            pan,
            MonoEffectChain::new(10),
        );
        // Note ID is unused in a monophonic instrument.
        let voice = VoiceSlot {
            inner: voice,
            note_id: None,
        };
        MoogDFAM {
            instrument_id,
            voice,
        }
    }
}
