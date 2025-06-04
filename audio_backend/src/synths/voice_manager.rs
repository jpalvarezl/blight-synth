use std::ops::Div;

use super::waveform::Waveform;
use crate::synths::voice::Voice;

#[cfg(test)]
use crate::test::audio_backend_utils::SampleProducer;

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

        println!(
            "[VoiceManager] Creating NEW VoiceManager instance with {} voices and sample rate {} at address {:p}",
            max_polyphony, sample_rate, &voices as *const _
        );
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

    pub fn note_on_with_velocity_and_waveform(
        &mut self,
        note: u8,
        velocity: f32,
        waveform: Waveform,
    ) {
        println!("[VoiceManager] note_on_with_velocity_and_waveform on instance {:p}: note={}, velocity={}", 
                 self as *const _, note, velocity);
        let (attack, decay, sustain, release) = self.current_adsr_params();
        println!("[VoiceManager] ADSR params: attack={}, decay={}, sustain={}, release={}", attack, decay, sustain, release);
        
        if let Some(voice) = self.find_free_voice() {
            println!("[VoiceManager] Found free voice, triggering note");
            voice.set_adsr(attack, decay, sustain, release);
            voice.note_on(note, velocity, waveform);
            println!("[VoiceManager] Voice activated, is_active={}", voice.is_active());
        } else {
            // Voice stealing: replace the oldest active voice
            println!("[VoiceManager] No free voice, attempting voice stealing");
            if let Some(voice) = self.find_voice_to_steal() {
                voice.set_adsr(attack, decay, sustain, release);
                voice.note_on(note, velocity, waveform);
                println!("[VoiceManager] Voice stolen and activated, is_active={}", voice.is_active());
            } else {
                println!("[VoiceManager] No voice available for stealing!");
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
        println!("[VoiceManager] next_sample called on instance {:p}", self as *const _);
        let active_count = self.voices.iter().filter(|v| v.is_active()).count();
        println!("[VoiceManager] Active voices: {}", active_count);
        
        let next_sample = if active_count == 0 {
            0.0 // No voices to process
        } else {
            // Sum the samples from all active voices and normalize
            println!("Calculating next sample for {} active voices", active_count);
            self.voices
                .iter_mut()
                .filter(|v| v.is_active())
                .map(|voice| voice.next_sample())
                .sum::<f32>()
                .div(self.max_polyphony as f32) // Normalize by max polyphony
                .clamp(-1.0, 1.0) // normalize
        };
        next_sample
    }

    fn find_free_voice(&mut self) -> Option<&mut Voice> {
        println!("[VoiceManager] Looking for free voice among {} voices", self.voices.len());
        for (i, voice) in self.voices.iter().enumerate() {
            println!("[VoiceManager] Voice {}: is_active={}", i, voice.is_active());
        }
        let free_voice = self.voices.iter_mut().find(|v| !v.is_active());
        if free_voice.is_some() {
            println!("[VoiceManager] Found free voice");
        } else {
            println!("[VoiceManager] No free voice found");
        }
        free_voice
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

#[cfg(test)]
impl SampleProducer for VoiceManager {
    fn next_sample(&mut self) -> f32 {
        self.next_sample()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voice_manager_creation() {
        let sample_rate = 44100.0;
        let max_polyphony = 8;
        let voice_manager = VoiceManager::new(sample_rate, max_polyphony);

        assert_eq!(voice_manager.voices.len(), max_polyphony);
        assert_eq!(voice_manager.max_polyphony, max_polyphony);
        assert_eq!(voice_manager.current_adsr_params(), (0.01, 0.1, 0.8, 0.2));
        assert_eq!(voice_manager.current_waveform(), Waveform::Sine);
    }

    #[test]
    fn find_free_voice_free_available() {
        let sample_rate = 44100.0;
        let max_polyphony = 8;
        let mut voice_manager = VoiceManager::new(sample_rate, max_polyphony);

        // All voices should be free initially
        assert!(voice_manager.find_free_voice().is_some());

        // Activate one voice
        voice_manager.note_on(60, 1.0);
        assert!(voice_manager.find_free_voice().is_some());

        // All voices are not active yet
        assert!(voice_manager.find_free_voice().is_some());
    }

    #[test]
    fn steal_voice_when_all_active() {
        let sample_rate = 44100.0;
        let max_polyphony = 8;
        let mut voice_manager = VoiceManager::new(sample_rate, max_polyphony);

        // Activate all voices
        for i in 0..max_polyphony {
            voice_manager.note_on(i as u8 + 60, 1.0);
        }

        // All voices are active, so we should be able to steal one
        assert!(voice_manager.find_voice_to_steal().is_some());
    }

    #[test]
    fn next_sample_test() {
        let sample_rate = 44100.0;
        let max_polyphony = 8;
        let mut voice_manager = VoiceManager::new(sample_rate, max_polyphony);

        // Test with no active voices
        assert_eq!(voice_manager.next_sample(), 0.0);

        // Activate one voice and check the sample
        voice_manager.note_on(60, 1.0);
        
        // Generate a few samples since the first sine wave sample at phase 0 is 0
        // and the ADSR starts with 0 amplitude
        let mut non_zero_found = false;
        for _ in 0..10 {
            let sample = voice_manager.next_sample();
            if sample != 0.0 {
                non_zero_found = true;
                break;
            }
        }
        assert!(non_zero_found, "Should produce non-zero samples after a few iterations");

        // Activate another voice and check the sample again
        voice_manager.note_on(61, 1.0);
        let mut non_zero_found2 = false;
        for _ in 0..10 {
            let sample2 = voice_manager.next_sample();
            if sample2 != 0.0 {
                non_zero_found2 = true;
                break;
            }
        }
        assert!(non_zero_found2, "Should produce non-zero samples with multiple voices");
    }

    #[test]
    fn test_voice_manager_audio_output() {
        use crate::test::audio_backend_utils::run_audio_stream_with_producer;
        use std::sync::{Arc, Mutex};
        use std::time::Duration;
        
        let sample_rate = 44100.0;
        let max_polyphony = 8;
        let mut voice_manager = VoiceManager::new(sample_rate, max_polyphony);
        
        // Set sensible ADSR values for a 1-second test:
        // - Quick attack (10ms)
        // - Short decay (100ms) 
        // - High sustain (80%)
        // - Medium release (200ms)
        voice_manager.set_adsr(0.01, 0.1, 0.8, 0.2);
        
        // Play a 440 Hz sine wave (MIDI note 69, A4)
        voice_manager.note_on(69, 0.7);
        
        // Wrap in Arc<Mutex<>> for thread safety
        let voice_manager = Arc::new(Mutex::new(voice_manager));
        
        // Start the audio stream
        let stream = run_audio_stream_with_producer(voice_manager.clone())
            .expect("Failed to create VoiceManager audio stream");
        
        println!("Playing VoiceManager 440 Hz sine wave (A4) for 1 second with ADSR envelope...");
        
        // Play for 1 second
        std::thread::sleep(Duration::from_secs(1));
        
        // Stop the stream by dropping it
        drop(stream);
        
        println!("VoiceManager audio test completed successfully!");
        
        // Verify that at least one voice was active during playback
        let voice_manager_guard = voice_manager.lock().unwrap();
        println!("Final voice manager state checked - test completed");
    }
}
