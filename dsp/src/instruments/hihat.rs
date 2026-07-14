use crate::{
    id::InstrumentId,
    instruments::{MonophonicInstrument, NoiseGenerator, VoiceSlot},
    Envelope, MonoEffectChain, Voice,
};

/// short noise burst. Use short decays.
pub type HiHat = MonophonicInstrument<NoiseGenerator>;

impl HiHat {
    pub fn new(instrument_id: InstrumentId, pan: f32, sample_rate: f32) -> Self {
        let mut envelope = Envelope::new(sample_rate);
        envelope.set_parameters(0.01, 0.05, 0.0, 0.1);

        // Note ID is unused in a monophonic instrument.
        let voice = VoiceSlot {
            inner: Voice::new(
                0,
                NoiseGenerator::default(),
                envelope,
                pan,
                MonoEffectChain::new(10),
            ),
            note_id: None,
        };
        HiHat {
            instrument_id,
            voice,
        }
    }
}
