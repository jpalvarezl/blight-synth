use crate::synths::voice::Voice;
use super::waveform::Waveform;

#[derive(Debug, Clone)]
pub struct VoiceManager {
    voices: Vec<Voice>,
    // sample_rate: f32,
    max_polyphony: usize,

    // Global ADSR settings
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,

    // Global Waveform settings
    waveform: Waveform, // If you want to set a default waveform for all voices
}

impl VoiceManager {
    pub fn new(sample_rate: f32, max_polyphony: usize) -> Self {
        let voices = (0..max_polyphony)
            .map(|_| Voice::new(sample_rate))
            .collect();

        Self {
            voices,
            max_polyphony,
            // Default ADSR settings
            attack: 0.01,
            decay: 0.1,
            sustain: 0.8,
            release: 0.2,
            // Default waveform
            waveform: Waveform::Sine,
        }
    }
  
    pub fn note_on(&mut self, note: u8, velocity: f32) {
        self.note_on_with_velocity_and_waveform(note, velocity, self.waveform);
    }

    pub fn note_on_with_velocity_and_waveform(&mut self, note: u8, velocity: f32, waveform: Waveform) {
        let (attack, decay, sustain, release) = self.current_adsr_params();
        if let Some(voice) = self.find_free_voice() {
            voice.set_adsr(attack, decay, sustain, release);
            voice.note_on(note, velocity, waveform);
        } else {
            // Voice stealing: replace the oldest active voice
            if let Some(voice) = self.find_voice_to_steal() {
                voice.set_adsr(attack, decay, sustain, release);
                voice.note_on(note, velocity, waveform);
            }
        }
    }

    pub fn note_off(&mut self, note: u8) {
        for voice in self.voices.iter_mut() {
            if voice.is_playing(note) {
                voice.note_off();
            }
        }
    }

    pub fn set_adsr(&mut self, attack: f32, decay: f32, sustain: f32, release: f32) {
        self.attack = attack;
        self.decay = decay;
        self.sustain = sustain;
        self.release = release;

        // Should I update all voices with the new ADSR settings?
        // This is optional, depending on whether you want to change ADSR dynamically.
        // for voice in &mut self.voices {
        //     voice.set_adsr(attack, decay, sustain, release);
        // }
    }

    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.waveform = waveform;

        // To dynamically change the waveform for all voices, uncomment the following:
        // for voice in &mut self.voices {
        //     voice.oscillator.set_waveform(waveform);
        // }
    }

    pub fn next_sample(&mut self) -> f32 {
        if self.voices.is_empty() {
            return 0.0; // No voices to process
        }

        // Sum the samples from all active voices and normalize
        (self.voices
            .iter_mut()
            .map(|voice| voice.next_sample())
            .sum::<f32>()
            / self.max_polyphony as f32).clamp(-1.0, 1.0) // normalize
    }

    fn find_free_voice(&mut self) -> Option<&mut Voice> {
        self.voices.iter_mut().find(|v| !v.is_active())
    }

    fn find_voice_to_steal(&mut self) -> Option<&mut Voice> {
        self.voices.iter_mut().min_by_key(|v| v.start_time())
    }

    pub fn current_adsr_params(&self) -> (f32, f32, f32, f32) {
        (self.attack, self.decay, self.sustain, self.release)
    }

    pub fn current_waveform(&self) -> Waveform {
        self.waveform
    }
}
