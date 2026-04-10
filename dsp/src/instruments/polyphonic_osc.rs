use crate::id::InstrumentId;
use crate::{instruments::VoiceSlot, OscillatorNode};
use crate::{Envelope, MonoEffectChain, PolyphonicInstrument, Voice, VoiceTrait};

pub type PolyphonicOscillator = PolyphonicInstrument<OscillatorNode>;

impl PolyphonicOscillator {
    pub fn new(instrument_id: InstrumentId, pan: f32, sample_rate: f32, max_polyphony: u8) -> Self {
        let mut envelope = Envelope::new(sample_rate);
        envelope.set_parameters(0.1, 0.1, 1.0, 1.0); // Default ADSR values
        let voices: Vec<VoiceSlot<OscillatorNode>> = (0..max_polyphony)
            .map(|_| VoiceSlot {
                note_id: None,
                inner: Voice::new(
                    0,
                    OscillatorNode::new(),
                    envelope.clone(),
                    pan,
                    MonoEffectChain::new(10),
                ),
            })
            .collect();

        log::info!("Allocating PolyphonicOscillator with ID: {}", instrument_id);
        for voice in &voices {
            log::info!(
                "Created voice with ID: {}, state: {}",
                voice.inner.id(),
                voice.inner.is_active()
            );
        }
        log::info!("Allocation complete");

        PolyphonicOscillator {
            instrument_id,
            voices,
        }
    }
}
