use std::sync::Arc;

use crate::{
    instruments::VoiceSlot, Envelope, MonoEffectChain, MonophonicInstrument, SampleData,
    SamplePlayerNode, Voice,
};

pub type SamplePlayer = MonophonicInstrument<SamplePlayerNode>;

impl SamplePlayer {
    /// Create a new SamplePlayer instrument with the given sample data
    pub fn new(
        instrument_id: crate::id::InstrumentId,
        sample_data: Arc<SampleData>,
        sample_rate: f32,
        pan: f32,
    ) -> Self {
        let envelope = Envelope::new_adsr(sample_rate, 3.0, 2.0, 1.0, 2.0);
        let voice = Voice::new(
            0,
            SamplePlayerNode::new(sample_data.clone(), sample_rate),
            envelope,
            pan,
            MonoEffectChain::new(10),
        );

        MonophonicInstrument {
            instrument_id,
            voice: VoiceSlot {
                note_id: None,
                inner: voice,
            },
        }
    }
}
