use crate::effects::{
    Delay, Distortion, DistortionType, Filter, FilterType, Gain, MoogLadder, Reverb, StereoReverb,
};
use crate::id::EffectId;
use crate::{MonoEffect, StereoEffect};

pub struct EffectFactory {
    sample_rate: f32,
}

impl EffectFactory {
    pub fn new(sample_rate: f32) -> Self {
        Self { sample_rate }
    }

    /// Create a mono reverb effect
    pub fn create_mono_reverb(&self, id: EffectId) -> Box<dyn MonoEffect> {
        Box::new(Reverb::new(id, self.sample_rate))
    }

    /// Create a stereo reverb effect
    pub fn create_stereo_reverb(&self, id: EffectId) -> Box<dyn StereoEffect> {
        Box::new(StereoReverb::new(id, self.sample_rate))
    }

    /// Create a mono delay effect
    pub fn create_mono_delay(
        &self,
        id: EffectId,
        delay_seconds: f32,
        num_taps: usize,
        feedback: f32,
        mix: f32,
    ) -> Box<dyn MonoEffect> {
        Box::new(Delay::new(
            id,
            self.sample_rate,
            delay_seconds,
            num_taps,
            feedback,
            mix,
        ))
    }

    /// Create a distortion effect
    #[deprecated(note = "Doesn't work correctly")]
    pub fn create_distortion(
        &self,
        id: EffectId,
        distortion_type: DistortionType,
        drive: f32,
        level: f32,
        mix: f32,
    ) -> Box<dyn MonoEffect> {
        Box::new(Distortion::new(id, distortion_type, drive, level, mix))
    }

    /// Create a filter effect
    #[deprecated(note = "Doesn't work correctly")]
    pub fn create_filter(
        &self,
        id: EffectId,
        filter_type: FilterType,
        cutoff: f32,
        resonance: f32,
    ) -> Box<dyn MonoEffect> {
        Box::new(Filter::new(
            id,
            filter_type,
            cutoff,
            resonance,
            self.sample_rate,
        ))
    }

    pub fn create_stereo_gain(&self, id: EffectId, gain: f32) -> Box<dyn StereoEffect> {
        Box::new(Gain::new(id, gain))
    }

    pub fn create_mono_gain(&self, id: EffectId, gain: f32) -> Box<dyn MonoEffect> {
        Box::new(Gain::new(id, gain))
    }

    /// Create a Moog Ladder filter effect (mono) with provided cutoff & resonance.
    pub fn create_moog_ladder(
        &self,
        id: EffectId,
        cutoff: f32,
        resonance: f32,
    ) -> Box<dyn MonoEffect> {
        Box::new(MoogLadder::new(id, self.sample_rate, cutoff, resonance))
    }
}
