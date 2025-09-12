use crate::{
    id::InstrumentId, instruments::VoiceSlot, Envelope, InstrumentTrait, KickDrumVoice,
    PitchEnvelope, Voice, VoiceTrait,
};

/// Kick:
/// - Amp envelope → short decay (so the kick fades out).
/// - Pitch envelope → even shorter decay (for the downward sweep).
/// - Oscillator → sine or triangle wave.
pub struct KickDrum {
    instrument_id: InstrumentId,
    voice: VoiceSlot<KickDrumVoice<Envelope, PitchEnvelope>>,
}

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

impl InstrumentTrait for KickDrum {
    fn id(&self) -> InstrumentId {
        self.instrument_id
    }

    fn note_on(&mut self, note: u8, velocity: u8) {
        self.voice.inner.note_on(note, velocity);
    }

    fn note_off(&mut self) {
        self.voice.inner.note_off();
    }

    fn process(&mut self, left_buf: &mut [f32], right_buf: &mut [f32], sample_rate: f32) {
        self.voice.inner.process(left_buf, right_buf, sample_rate);
    }

    fn set_pan(&mut self, pan: f32) {
        self.voice.inner.set_pan(pan);
    }

    fn add_effect(&mut self, effect: Box<dyn crate::MonoEffect>) {
        self.voice.inner.add_effect(effect);
    }

    fn set_effect_parameter(&mut self, effect_index: usize, param_index: u32, value: f32) {
        self.voice
            .inner
            .set_effect_parameter(effect_index, param_index, value);
    }
}
