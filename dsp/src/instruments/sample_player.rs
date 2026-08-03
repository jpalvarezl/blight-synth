use std::sync::Arc;

use crate::{
    id::InstrumentId,
    instruments::{LoopRegion, MonophonicInstrument, SamplePlayerNode, VoiceSlot},
    Envelope, MonoEffectChain, SampleData, Voice,
};

pub type SamplePlayer = MonophonicInstrument<SamplePlayerNode>;

impl SamplePlayer {
    /// Create a new one-shot SamplePlayer instrument with the given sample data
    pub fn new_one_shot(
        instrument_id: InstrumentId,
        sample_data: Arc<SampleData>,
        sample_rate: f32,
        pan: f32,
    ) -> Self {
        let voice = Voice::new_no_envelope(
            crate::id::VoiceId::from_raw(0),
            SamplePlayerNode::new(sample_data.clone(), sample_rate, None),
            pan,
            MonoEffectChain::new(10),
        );

        MonophonicInstrument {
            instrument_id,
            voice: VoiceSlot::new(voice),
        }
    }

    pub fn new_with_loop(
        instrument_id: InstrumentId,
        sample_data: Arc<SampleData>,
        sample_rate: f32,
        pan: f32,
        loop_region: LoopRegion,
    ) -> Self {
        let envelope = Envelope::new_adsr(sample_rate, 1.0, 1.0, 1.0, 1.0);
        let voice = Voice::new(
            crate::id::VoiceId::from_raw(0),
            SamplePlayerNode::new(sample_data.clone(), sample_rate, Some(loop_region)),
            envelope,
            pan,
            MonoEffectChain::new(10),
        );

        MonophonicInstrument {
            instrument_id,
            voice: VoiceSlot::new(voice),
        }
    }
}
