pub type VoiceId = u64;
pub type SampleId = u64;
pub type InstrumentId = u64;
pub type EffectChainId = u64;

/// Instrument IDs registry
pub enum Instrument {
    Oscillator = 1,
    SamplePlayer,
}

impl From<InstrumentId> for Instrument {
    fn from(value: InstrumentId) -> Self {
        match value {
            1 => Instrument::Oscillator,
            2 => Instrument::SamplePlayer,
            _ => panic!(
                "Not sure if a panic is the best thing to do when instrument ID is not found"
            ), // Default case
        }
    }
}
