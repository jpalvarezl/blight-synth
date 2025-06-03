use crate::synths::{voice_manager::VoiceManager, waveform::Waveform};

pub struct Synthesizer {
    sample_rate: f32,
    voice_manager: VoiceManager,
}

impl Synthesizer {
    pub fn new(sample_rate: f32, max_polyphony: usize) -> Self {
        Self {
            sample_rate,
            voice_manager: VoiceManager::new(sample_rate, max_polyphony),
        }
    }

    pub fn note_on(&mut self, note: u8, velocity: f32, waveform: Waveform) {
        self.voice_manager.note_on(note, velocity, waveform);
    }

    pub fn note_off(&mut self, note: u8) {
        self.voice_manager.note_off(note);
    }

    pub fn next_sample(&mut self) -> f32 {
        self.voice_manager.next_sample()
    }
}
