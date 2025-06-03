use crate::synths::{voice_manager::VoiceManager, waveform::Waveform};

#[derive(Debug, Clone)]
pub struct Synthesizer {
    voice_manager: VoiceManager,
}

impl Synthesizer {
    pub fn new(sample_rate: f32, max_polyphony: usize) -> Self {
        Self {
            voice_manager: VoiceManager::new(sample_rate, max_polyphony),
        }
    }

    pub fn note_on(&mut self, note: u8, velocity: f32) {
        self.voice_manager.note_on(note, velocity);
    }

    pub fn note_off(&mut self, note: u8) {
        self.voice_manager.note_off(note);
    }

    pub fn next_sample(&mut self) -> f32 {
        self.voice_manager.next_sample()
    }

    pub fn set_adsr(&mut self, attack: f32, decay: f32, sustain: f32, release: f32) {
        self.voice_manager.set_adsr(attack, decay, sustain, release);
    }

    pub fn current_adsr_params(&self) -> (f32, f32, f32, f32) {
        self.voice_manager.current_adsr_params()
    }

    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.voice_manager.set_waveform(waveform);
    }

    pub fn current_waveform(&self) -> Waveform {
        self.voice_manager.current_waveform()
    }
}
