use crate::{id::InstrumentId, instruments::VoiceSlot, KickDrumVoice, MonophonicInstrument, Voice};

/// Kick:
/// - Amp envelope → short decay (so the kick fades out).
/// - Pitch envelope → even shorter decay (for the downward sweep).
/// - Oscillator → sine or triangle wave.
pub type KickDrum = MonophonicInstrument<KickDrumVoice>;

impl KickDrum {
    pub fn new(instrument_id: InstrumentId, pan: f32, sample_rate: f32) -> Self {
        let voice = Voice::new_no_envelope(
            0,
            KickDrumVoice::new(sample_rate),
            pan,
            crate::MonoEffectChain::new(10),
        );
        // Note ID is unused in a monophonic instrument.
        let voice = VoiceSlot {
            inner: voice,
            note_id: None,
        };
        KickDrum {
            instrument_id,
            voice,
        }
    }
}
