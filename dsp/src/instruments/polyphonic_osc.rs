use crate::id::InstrumentId;
use crate::instruments::{OscillatorNode, PolyphonicInstrument, VoiceSlot};
use crate::{Envelope, MonoEffectChain, Voice};

pub type PolyphonicOscillator = PolyphonicInstrument<OscillatorNode>;

impl PolyphonicOscillator {
    pub fn new(instrument_id: InstrumentId, pan: f32, sample_rate: f32, max_polyphony: u8) -> Self {
        let mut envelope = Envelope::new(sample_rate);
        envelope.set_parameters(0.1, 0.1, 1.0, 1.0); // Default ADSR values
        // The voice pool is fixed and preallocated here (off the audio thread);
        // it never grows, so note allocation and stealing stay heap-free on RT.
        let voices: Vec<VoiceSlot<OscillatorNode>> = (0..max_polyphony)
            .map(|_| {
                VoiceSlot::new(Voice::new(
                    0,
                    OscillatorNode::new(),
                    envelope.clone(),
                    pan,
                    MonoEffectChain::new(10),
                ))
            })
            .collect();

        PolyphonicOscillator {
            instrument_id,
            voices,
            next_age: 0,
        }
    }
}
