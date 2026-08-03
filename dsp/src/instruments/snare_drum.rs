use crate::{
    id::InstrumentId,
    instruments::{MonophonicInstrument, SnareDrumVoice, VoiceSlot},
    MonoEffectChain, Voice,
};

pub type SnareDrum = MonophonicInstrument<SnareDrumVoice>;

impl SnareDrum {
    pub fn new(instrument_id: InstrumentId, pan: f32, sample_rate: f32) -> Self {
        let voice = Voice::new_no_envelope(
            crate::id::VoiceId::from_raw(0),
            SnareDrumVoice::new(sample_rate),
            pan,
            MonoEffectChain::new(10),
        );
        // Note ID is unused in a monophonic instrument.
        let voice = VoiceSlot::new(voice);
        SnareDrum {
            instrument_id,
            voice,
        }
    }
}
