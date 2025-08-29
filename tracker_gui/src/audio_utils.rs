use audio_backend::Waveform as BackendWaveform;
use sequencer::models::Waveform;

pub fn map_waveform_to_backend(w: Waveform) -> BackendWaveform {
    match w {
        Waveform::Sine => BackendWaveform::Sine,
        Waveform::Square => BackendWaveform::Square,
        Waveform::Sawtooth => BackendWaveform::Sawtooth,
        Waveform::Triangle => BackendWaveform::Triangle,
        Waveform::NesTriangle => BackendWaveform::NesTriangle,
    }
}
