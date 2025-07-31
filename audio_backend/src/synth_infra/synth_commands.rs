use crate::Waveform;

#[derive(Debug, Clone, Copy)]
pub enum SynthCommand {
    SetWaveform(Waveform),
    SetFilterCutoff(f32),
    SetFilterResonance(f32),
    SetPitchBend(f32),
}
