use crate::id::InstrumentId;
use crate::instruments::{MonophonicInstrument, OscillatorNode, VoiceSlot, Waveform};
use crate::{Envelope, MonoEffectChain, Voice};

pub type MonophonicOscillator = MonophonicInstrument<OscillatorNode>;

impl MonophonicOscillator {
    pub fn new(instrument_id: InstrumentId, pan: f32, sample_rate: f32) -> Self {
        let envelope = Envelope::new(sample_rate);
        let voice = Voice::new(
            crate::id::VoiceId::from_raw(0),
            OscillatorNode::new(),
            envelope,
            pan,
            MonoEffectChain::new(10),
        );
        // Note ID is unused in a monophonic instrument.
        let voice = VoiceSlot::new(voice);
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
            crate::id::VoiceId::from_raw(0),
            OscillatorNode::new_with_waveform(waveform),
            envelope,
            pan,
            MonoEffectChain::new(10),
        );
        // Note ID is unused in a monophonic instrument.
        let voice = VoiceSlot::new(voice);
        MonophonicOscillator {
            instrument_id,
            voice,
        }
    }
}
