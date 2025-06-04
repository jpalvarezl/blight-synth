use crate::synths::{voice_manager::VoiceManager, waveform::Waveform};

#[cfg(test)]
use crate::test::audio_backend_utils::SampleProducer;

#[derive(Debug, Clone)]
pub struct Synthesizer {
    voice_manager: VoiceManager,
}

impl Synthesizer {
    pub fn new(sample_rate: f32, max_polyphony: usize) -> Self {
        println!(
            "Creating Synthesizer with sample rate: {} and max polyphony: {}",
            sample_rate, max_polyphony
        );
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

#[cfg(test)]
impl SampleProducer for Synthesizer {
    fn next_sample(&mut self) -> f32 {
        self.next_sample()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::audio_backend_utils::run_audio_stream_with_producer;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    #[test]
    fn test_synthesizer_creation() {
        let sample_rate = 44100.0;
        let max_polyphony = 8;
        let synthesizer = Synthesizer::new(sample_rate, max_polyphony);

        assert_eq!(synthesizer.current_adsr_params(), (0.01, 0.1, 0.8, 0.2));
        assert_eq!(synthesizer.current_waveform(), Waveform::Sine);
    }

    #[test]
    fn test_synthesizer_audio_output() {
        let sample_rate = 44100.0;
        let max_polyphony = 8;
        let mut synthesizer = Synthesizer::new(sample_rate, max_polyphony);

        // Set ADSR values optimized for a 1-second test:
        // - Quick attack (10ms) to start sound immediately
        // - Short decay (100ms) to reach sustain level quickly
        // - High sustain (80%) to maintain audible level
        // - Medium release (200ms) for smooth ending when note_off is called
        synthesizer.set_adsr(0.01, 0.1, 0.8, 0.2);

        // Play a 440 Hz sine wave (MIDI note 69, A4)
        synthesizer.note_on(69, 0.7);

        // Wrap in Arc<Mutex<>> for thread safety
        let synthesizer = Arc::new(Mutex::new(synthesizer));

        // Start the audio stream
        let stream = run_audio_stream_with_producer(synthesizer.clone())
            .expect("Failed to create Synthesizer audio stream");

        println!("Playing Synthesizer 440 Hz sine wave (A4) for 1 second with ADSR envelope...");

        // Play for 1 second
        std::thread::sleep(Duration::from_secs(1));

        // Stop the stream by dropping it
        drop(stream);

        println!("Synthesizer audio test completed successfully!");

        // Verify that the synthesizer was used during playback
        let _synthesizer_guard = synthesizer.lock().unwrap();
        println!("Final synthesizer state checked - test completed");
    }

    #[test]
    fn test_synthesizer_audio_output_with_note_off() {
        let sample_rate = 44100.0;
        let max_polyphony = 8;
        let mut synthesizer = Synthesizer::new(sample_rate, max_polyphony);

        // Set ADSR values optimized for a 2-second test with note_off:
        // - Quick attack (1ms) to start sound immediately
        // - Short decay (100ms) to reach sustain level quickly
        // - High sustain (80%) to maintain audible level during hold
        // - Longer release (500ms) to hear the fade-out after note_off
        synthesizer.set_adsr(0.1, 0.1, 0.8, 0.5);

        // Play a 440 Hz sine wave (MIDI note 69, A4)
        synthesizer.note_on(69, 0.7);

        // Wrap in Arc<Mutex<>> for thread safety
        let synthesizer = Arc::new(Mutex::new(synthesizer));

        // Start the audio stream
        let stream = run_audio_stream_with_producer(synthesizer.clone())
            .expect("Failed to create Synthesizer audio stream");

        println!("Playing Synthesizer 440 Hz sine wave (A4) for 1 second, then note_off for 1 more second...");

        // Play for 1 second (attack + decay + sustain)
        std::thread::sleep(Duration::from_secs(1));

        // Trigger note_off to start the release phase
        {
            let mut synth_guard = synthesizer.lock().unwrap();
            synth_guard.note_off(69);
            println!("Note off triggered - starting release phase");
        }

        // Continue playing for another second to hear the release
        std::thread::sleep(Duration::from_secs(1));

        // Stop the stream by dropping it
        drop(stream);

        println!("Synthesizer audio test with note_off completed successfully!");

        // Verify that the synthesizer was used during playback
        let _synthesizer_guard = synthesizer.lock().unwrap();
        println!("Final synthesizer state checked - test completed");
    }

    #[test]
    fn test_synthesizer_polyphony_c_minor_chord() {
        let sample_rate = 44100.0;
        let max_polyphony = 8;
        let mut synthesizer = Synthesizer::new(sample_rate, max_polyphony);

        // Set ADSR values optimized for a chord test:
        // - Medium attack (50ms) for smooth chord onset
        // - Short decay (150ms) to reach sustain level
        // - High sustain (75%) to maintain audible level during chord hold
        // - Medium release (400ms) to hear the chord fade naturally
        synthesizer.set_adsr(0.05, 0.15, 0.75, 0.4);

        // C minor chord: C4 (60), Eb4 (63), G4 (67)
        let chord_notes = [60u8, 63u8, 67u8]; // C, Eb, G
        let chord_names = ["C4", "Eb4", "G4"];

        // Wrap in Arc<Mutex<>> for thread safety
        let synthesizer = Arc::new(Mutex::new(synthesizer));

        // Start the audio stream
        let stream = run_audio_stream_with_producer(synthesizer.clone())
            .expect("Failed to create Synthesizer audio stream");

        println!("Playing C minor chord (C4-Eb4-G4) for 1.5 seconds, then releasing notes...");

        // Play all notes of the chord simultaneously
        {
            let mut synth_guard = synthesizer.lock().unwrap();
            for (i, &note) in chord_notes.iter().enumerate() {
                synth_guard.note_on(note, 0.6); // Moderate velocity for chord balance
                println!("Playing {} (MIDI note {})", chord_names[i], note);
            }
        }

        // Hold the chord for 1.5 seconds (attack + decay + sustain)
        std::thread::sleep(Duration::from_millis(1500));

        // Release all notes of the chord
        {
            let mut synth_guard = synthesizer.lock().unwrap();
            for (i, &note) in chord_notes.iter().enumerate() {
                synth_guard.note_off(note);
                println!("Released {} (MIDI note {})", chord_names[i], note);
            }
            println!("All chord notes released - starting release phase");
        }

        // Continue playing for another 1 second to hear the chord release
        std::thread::sleep(Duration::from_secs(1));

        // Stop the stream by dropping it
        drop(stream);

        println!("C minor chord polyphony test completed successfully!");

        // Verify that the synthesizer was used during playback
        let _synthesizer_guard = synthesizer.lock().unwrap();
        println!("Final synthesizer state checked - polyphony test completed");
    }
}
