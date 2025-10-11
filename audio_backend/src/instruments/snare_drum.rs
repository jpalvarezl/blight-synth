use crate::{
    id::InstrumentId, instruments::VoiceSlot, MonophonicInstrument, SnareDrumVoice, Voice,
};

pub type SnareDrum = MonophonicInstrument<SnareDrumVoice>;

impl SnareDrum {
    pub fn new(instrument_id: InstrumentId, pan: f32, sample_rate: f32) -> Self {
        let voice = Voice::new_no_envelope(
            0,
            SnareDrumVoice::new(sample_rate),
            pan,
            crate::MonoEffectChain::new(10),
        );
        // Note ID is unused in a monophonic instrument.
        let voice = VoiceSlot {
            inner: voice,
            note_id: None,
        };
        SnareDrum {
            instrument_id,
            voice,
        }
    }
}
