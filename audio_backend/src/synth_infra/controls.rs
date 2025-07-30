use crate::Waveform;

/// Trait for synth nodes that support waveform changes
pub trait WaveformControl: Send + Sync {
    fn set_waveform(&mut self, waveform: Waveform);
}

/// Trait for synth nodes that support filter control
pub trait FilterControl: Send + Sync {
    fn set_cutoff(&mut self, cutoff: f32);
    fn set_resonance(&mut self, resonance: f32);
}

/// Trait for synth nodes that support pitch modulation
pub trait PitchControl: Send + Sync {
    fn set_pitch_bend(&mut self, bend: f32);
    fn set_vibrato(&mut self, rate: f32, depth: f32);
}