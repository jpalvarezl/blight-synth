use crate::effects::{Delay, Distortion, DistortionType, Filter, FilterType, Reverb, StereoReverb};
use crate::{MonoEffect, StereoEffect};

pub struct EffectFactory {
    sample_rate: f32,
}

impl EffectFactory {
    pub fn new(sample_rate: f32) -> Self {
        Self { sample_rate }
    }

    /// Create a mono reverb effect
    pub fn create_mono_reverb(&self, room_size: f32, damping: f32) -> Box<dyn MonoEffect> {
        Box::new(Reverb::new(self.sample_rate, room_size, damping))
    }

    /// Create a stereo reverb effect
    pub fn create_stereo_reverb(&self, room_size: f32, damping: f32) -> Box<dyn StereoEffect> {
        Box::new(StereoReverb::new(self.sample_rate, room_size, damping))
    }

    /// Create a mono delay effect
    pub fn create_mono_delay(&self, delay_ms: f32, feedback: f32, mix: f32) -> Box<dyn MonoEffect> {
        Box::new(Delay::new(self.sample_rate, delay_ms, feedback, mix))
    }

    /// Create a distortion effect
    pub fn create_distortion(
        &self,
        distortion_type: DistortionType,
        drive: f32,
        level: f32,
        mix: f32,
    ) -> Box<dyn MonoEffect> {
        Box::new(Distortion::new(distortion_type, drive, level, mix))
    }

    /// Create a filter effect
    pub fn create_filter(
        &self,
        filter_type: FilterType,
        cutoff: f32,
        resonance: f32,
    ) -> Box<dyn MonoEffect> {
        Box::new(Filter::new(
            filter_type,
            cutoff,
            resonance,
            self.sample_rate,
        ))
    }
}
