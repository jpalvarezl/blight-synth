use crate::{MonophonicInstrument, MoogNode};

pub type MoogDFAM = MonophonicInstrument<MoogNode>;

impl MoogDFAM {
    pub fn new(instrument_id: crate::id::InstrumentId, pan: f32, sample_rate: f32) -> Self {
        let envelope = crate::Envelope::new(sample_rate);
        let voice = crate::Voice::new(
            0,
            MoogNode::new(
                crate::OscillatorNode::new_with_waveform(crate::Waveform::Square),
                crate::OscillatorNode::new_with_waveform(crate::Waveform::Square),
                crate::NoiseGenerator::default(),
            ),
            envelope,
            pan,
            crate::MonoEffectChain::new(10),
        );
        // Note ID is unused in a monophonic instrument.
        let voice = crate::instruments::VoiceSlot {
            inner: voice,
            note_id: None,
        };
        MoogDFAM {
            instrument_id,
            voice,
        }
    }
}