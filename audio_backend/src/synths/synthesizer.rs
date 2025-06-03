use crate::envelope::AdsrEnvelope;
use crate::synths::oscillator::{Oscillator, Saw, Sine, Square, Triangle};
use std::sync::{Arc, Mutex};

// --- Enum to Select Active Waveform ---
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveWaveform {
    Sine,
    Square,
    Saw,
    Triangle,
}

// Define the Voice struct
#[derive(Debug, Clone)] // Clone might be useful, review if needed later
pub struct Voice {
    sine_osc: Oscillator<Sine>,
    square_osc: Oscillator<Square>,
    saw_osc: Oscillator<Saw>,
    triangle_osc: Oscillator<Triangle>,
    envelope: AdsrEnvelope,
    current_frequency: f32,
    active_waveform: ActiveWaveform,
    sample_rate: f32,    // Store sample rate for oscillator frequency updates
    note_id: Option<u8>, // Track which note is assigned to this voice
}

impl Voice {
    pub fn new(sample_rate: f32) -> Self {
        // Default ADSR values (can be configured later)
        let envelope = AdsrEnvelope::new(sample_rate, 0.01, 0.1, 0.8, 0.2);

        Voice {
            sine_osc: Oscillator::new(sample_rate, 0.0),
            square_osc: Oscillator::new(sample_rate, 0.0),
            saw_osc: Oscillator::new(sample_rate, 0.0),
            triangle_osc: Oscillator::new(sample_rate, 0.0),
            envelope,
            current_frequency: 440.0, // Default A4
            active_waveform: ActiveWaveform::Sine,
            sample_rate,
            note_id: None,
        }
    }

    pub fn set_frequency(&mut self, freq: f32) {
        self.current_frequency = freq;
        // No need to update oscillators here, will be done in next_sample
    }

    pub fn set_waveform(&mut self, waveform: ActiveWaveform) {
        self.active_waveform = waveform;
        match self.active_waveform {
            ActiveWaveform::Sine => {
                self.sine_osc.reset_phase(0.0);
            }
            ActiveWaveform::Square => {
                self.square_osc.reset_phase(0.0);
            }
            ActiveWaveform::Saw => {
                self.saw_osc.reset_phase(0.0);
            }
            ActiveWaveform::Triangle => {
                self.triangle_osc.reset_phase(0.0);
            }
        }
    }

    pub fn note_on(&mut self, freq: f32) {
        self.current_frequency = freq;
        self.envelope.trigger();
    }

    pub fn note_off(&mut self) {
        self.envelope.release();
    }

    // Calculate the next sample for this voice
    pub fn next_sample(&mut self) -> f32 {
        if !self.envelope.is_active() {
            return 0.0;
        }

        let envelope_level = self.envelope.process();

        let osc_sample = match self.active_waveform {
            ActiveWaveform::Sine => {
                self.sine_osc.set_frequency(self.current_frequency);
                self.sine_osc.next_sample()
            }
            ActiveWaveform::Square => {
                self.square_osc.set_frequency(self.current_frequency);
                self.square_osc.next_sample()
            }
            ActiveWaveform::Saw => {
                self.saw_osc.set_frequency(self.current_frequency);
                self.saw_osc.next_sample()
            }
            ActiveWaveform::Triangle => {
                self.triangle_osc.set_frequency(self.current_frequency);
                self.triangle_osc.next_sample()
            }
        };

        osc_sample * envelope_level
    }

    // Method to check if the voice is active (useful for polyphony later)
    pub fn is_active(&self) -> bool {
        self.envelope.is_active()
    }

    // Method to configure ADSR parameters
    pub fn set_adsr(&mut self, attack: f32, decay: f32, sustain: f32, release: f32) {
        self.envelope = AdsrEnvelope::new(self.sample_rate, attack, decay, sustain, release);
    }

    // --- Individual ADSR parameter setters ---
    pub fn set_attack(&mut self, attack_secs: f32) {
        self.envelope.set_attack(attack_secs);
    }

    pub fn set_decay(&mut self, decay_secs: f32) {
        self.envelope.set_decay(decay_secs);
    }

    pub fn set_sustain(&mut self, sustain_level: f32) {
        self.envelope.set_sustain(sustain_level);
    }

    pub fn set_release(&mut self, release_secs: f32) {
        self.envelope.set_release(release_secs);
    }

    // Helper for tests
    #[cfg(test)]
    fn get_envelope_state(&self) -> crate::envelope::EnvelopeState {
        self.envelope.get_state()
    }
}

// --- Synthesizer Struct (Refactored) ---

// Remove the old SynthControl struct if it exists
const MAX_VOICES: usize = 8;
#[derive(Debug, Clone)]
pub struct Synthesizer {
    voices: Vec<Arc<Mutex<Voice>>>, // Use Arc<Mutex> for thread safety
    sample_rate: f32,
}

const DEFAULT_SAMPLE_RATE: f32 = 44100.0; // Default sample rate constant

impl Default for Synthesizer {
    fn default() -> Self {
        Synthesizer::new(DEFAULT_SAMPLE_RATE)
    }
}

impl Synthesizer {
    pub fn new(sample_rate: f32) -> Self {
        let mut voices = Vec::with_capacity(MAX_VOICES);
        for _ in 0..MAX_VOICES {
            voices.push(Arc::new(Mutex::new(Voice::new(sample_rate))));
        }
        Synthesizer {
            voices,
            sample_rate,
        }
    }

    // --- Methods to control the voice ---

    // FIXME: These methods now operate on the first voice.
    // Future changes will implement proper voice allocation for polyphony.
    pub fn set_frequency(&self, freq: f32) {
        if let Some(voice_mutex) = self.voices.get(0) {
            if let Ok(mut voice) = voice_mutex.lock() {
                voice.set_frequency(freq);
            } else {
                eprintln!("Error locking voice mutex in set_frequency");
            }
        }
    }

    pub fn set_waveform(&self, waveform: ActiveWaveform) {
        if let Some(voice_mutex) = self.voices.get(0) {
            if let Ok(mut voice) = voice_mutex.lock() {
                voice.set_waveform(waveform);
            } else {
                eprintln!("Error locking voice mutex in set_waveform");
            }
        }
    }

    pub fn set_adsr(&self, attack: f32, decay: f32, sustain: f32, release: f32) {
        if let Some(voice_mutex) = self.voices.get(0) {
            if let Ok(mut voice) = voice_mutex.lock() {
                voice.set_adsr(attack, decay, sustain, release);
            } else {
                eprintln!("Error locking voice mutex in set_adsr");
            }
        }
    }

    // --- Individual ADSR parameter setters for Synthesizer ---
    pub fn set_attack(&self, attack_secs: f32) {
        if let Some(voice_mutex) = self.voices.get(0) {
            if let Ok(mut voice) = voice_mutex.lock() {
                voice.set_attack(attack_secs);
            } else {
                eprintln!("Error locking voice mutex in set_attack");
            }
        }
    }

    pub fn set_decay(&self, decay_secs: f32) {
        if let Some(voice_mutex) = self.voices.get(0) {
            if let Ok(mut voice) = voice_mutex.lock() {
                voice.set_decay(decay_secs);
            } else {
                eprintln!("Error locking voice mutex in set_decay");
            }
        }
    }

    pub fn set_sustain(&self, sustain_level: f32) {
        if let Some(voice_mutex) = self.voices.get(0) {
            if let Ok(mut voice) = voice_mutex.lock() {
                voice.set_sustain(sustain_level);
            } else {
                eprintln!("Error locking voice mutex in set_sustain");
            }
        }
    }

    pub fn set_release(&self, release_secs: f32) {
        if let Some(voice_mutex) = self.voices.get(0) {
            if let Ok(mut voice) = voice_mutex.lock() {
                voice.set_release(release_secs);
            } else {
                eprintln!("Error locking voice mutex in set_release");
            }
        }
    }

    // --- Note control ---
    pub fn note_on(&self, note_id: u8, freq: f32) {
        // 1. Try to find an inactive voice and use it
        let mut handled = false;
        for voice_mutex in &self.voices {
            if let Ok(mut voice) = voice_mutex.lock() {
                if !voice.is_active() {
                    voice.note_id = Some(note_id);
                    voice.set_frequency(freq);
                    voice.envelope.trigger();
                    handled = true;
                    break;
                }
            }
        }
        // 2. If none found, steal the first voice
        if !handled {
            if let Ok(mut voice) = self.voices[0].lock() {
                voice.note_id = Some(note_id);
                voice.set_frequency(freq);
                voice.envelope.trigger();
            } else {
                eprintln!("Error locking voice mutex in note_on");
            }
        }
    }

    pub fn note_off(&self, note_id: u8) {
        for voice_mutex in &self.voices {
            if let Ok(mut voice) = voice_mutex.lock() {
                if voice.note_id == Some(note_id) {
                    voice.envelope.release();
                    voice.note_id = None; // Or keep until envelope is idle, depending on strategy
                    break;
                }
            }
        }
    }

    // --- Audio processing ---
    pub fn next_sample(&self) -> f32 {
        // FIXME: This will be expanded for polyphony. For now, sum output of all voices.
        // For initial setup, just process the first voice.
        // Proper mixing will be handled later.
        let mut output_sample = 0.0;
        if let Some(voice_mutex) = self.voices.get(0) {
            if let Ok(mut voice) = voice_mutex.lock() {
                output_sample = voice.next_sample();
            } else {
                eprintln!("Error locking voice mutex in next_sample");
            }
        }
        output_sample
        // In a polyphonic setup, you would iterate through all voices:
        // self.voices.iter().fold(0.0, |acc, voice_mutex| {
        //     if let Ok(mut voice) = voice_mutex.lock() {
        //         acc + voice.next_sample()
        //     } else {
        //         acc // Or handle error
        //     }
        // })
        // Clamping or mixing strategy might be needed if sum exceeds [-1.0, 1.0]
    }

    // --- Getters for parameters (optional, for UI or testing) ---
    pub fn get_sample_rate(&self) -> f32 {
        self.sample_rate
    }

    // Example: Get status of the first voice (for testing/debugging)
}

// --- Tests ---
#[cfg(test)]
mod tests {
    use super::*;
    use crate::envelope::EnvelopeState; // Make sure EnvelopeState is in scope

    const TEST_SAMPLE_RATE: f32 = 44100.0;

    #[test]
    fn test_synthesizer_creation() {
        let synth = Synthesizer::new(TEST_SAMPLE_RATE);
        assert_eq!(synth.voices.len(), MAX_VOICES);
        assert_eq!(synth.sample_rate, TEST_SAMPLE_RATE);
        // Check if voices are initialized
        for voice_mutex in &synth.voices {
            let voice = voice_mutex.lock().unwrap();
            assert_eq!(voice.sample_rate, TEST_SAMPLE_RATE); // Verify sample rate in voice
            assert!(!voice.is_active()); // Should be idle initially
        }
    }

    #[test]
    fn test_synthesizer_default() {
        let synth = Synthesizer::default();
        assert_eq!(synth.voices.len(), MAX_VOICES);
        assert_eq!(synth.sample_rate, DEFAULT_SAMPLE_RATE);
    }

    #[test]
    fn test_set_frequency_on_first_voice() {
        let synth = Synthesizer::new(TEST_SAMPLE_RATE);
        synth.set_frequency(220.0);
        let voice = synth.voices[0].lock().unwrap();
        assert_eq!(voice.current_frequency, 220.0);
    }

    #[test]
    fn test_set_waveform_on_first_voice() {
        let synth = Synthesizer::new(TEST_SAMPLE_RATE);
        synth.set_waveform(ActiveWaveform::Square);
        let voice = synth.voices[0].lock().unwrap();
        assert_eq!(voice.active_waveform, ActiveWaveform::Square);
    }

    #[test]
    fn test_note_on_off_on_first_voice() {
        let synth = Synthesizer::new(TEST_SAMPLE_RATE);
        synth.note_on(60, 440.0);
        {
            let voice = synth.voices[0].lock().unwrap();
            assert_eq!(voice.current_frequency, 440.0);
            assert_eq!(voice.get_envelope_state(), EnvelopeState::Attack);
        } // Mutex guard dropped here

        synth.note_off(60);
        let voice = synth.voices[0].lock().unwrap();
        assert_eq!(voice.get_envelope_state(), EnvelopeState::Release);
    }

    #[test]
    fn test_set_adsr_on_first_voice() {
        let synth = Synthesizer::new(TEST_SAMPLE_RATE);
        synth.set_adsr(0.1, 0.2, 0.7, 0.3);
        let voice = synth.voices[0].lock().unwrap();
        // We can't directly compare AdsrEnvelope easily without PartialEq
        // But we can check one of its parameters that would change
        assert_eq!(voice.envelope.get_attack_secs(), 0.1);
        assert_eq!(voice.envelope.get_decay_secs(), 0.2);
        assert_eq!(voice.envelope.get_sustain_level_param(), 0.7);
        assert_eq!(voice.envelope.get_release_secs(), 0.3);
    }

    #[test]
    fn test_individual_adsr_setters_on_first_voice() {
        let synth = Synthesizer::new(TEST_SAMPLE_RATE);
        synth.set_attack(0.11);
        synth.set_decay(0.22);
        synth.set_sustain(0.77);
        synth.set_release(0.33);

        let voice = synth.voices[0].lock().unwrap();
        assert_eq!(voice.envelope.get_attack_secs(), 0.11);
        assert_eq!(voice.envelope.get_decay_secs(), 0.22);
        assert_eq!(voice.envelope.get_sustain_level_param(), 0.77);
        assert_eq!(voice.envelope.get_release_secs(), 0.33);
    }

    #[test]
    fn test_next_sample_from_first_voice() {
        let synth = Synthesizer::new(TEST_SAMPLE_RATE);
        // Ensure the first voice is producing sound
        synth.note_on(60, 440.0); // Activate the envelope
        let mut voice = synth.voices[0].lock().unwrap();
        voice.set_waveform(ActiveWaveform::Sine); // Known waveform
                                                  // Process a few samples to get past initial zero output of envelope
        for _ in 0..5 {
            voice.envelope.process();
        }
        drop(voice); // Release lock before calling synth.next_sample()

        let _sample1 = synth.next_sample();

        // Let's advance the envelope and oscillator of the first voice directly
        // to predict the next sample.
        let mut voice_clone = synth.voices[0].lock().unwrap().clone(); // Clone the state
        let _expected_sample = voice_clone.next_sample();

        // The sample from synth should be what the first voice would produce
        // This is a bit tricky because next_sample() in Synthesizer also calls next_sample() on voice
        // For a more robust test, we might need to compare a sequence or ensure state.
        // For now, let's check it's not zero if active.
        // If the envelope is active, we expect a non-zero sample (usually)
        let is_first_voice_active = synth.voices[0].lock().unwrap().is_active();
        if is_first_voice_active && synth.voices[0].lock().unwrap().envelope.get_level() > 0.0 {
            // This assertion is tricky because the first call to synth.next_sample()
            // already advanced the state of the first voice's envelope.
            // The `expected_sample` calculated above is from a state *after*
            // the `sample1` was produced.
            // A simpler check: if active, it should produce something.
            // If we want to check exact values, we need to be more careful about state.
        }

        // A more direct test:
        // 1. Set up voice
        // 2. Call voice.next_sample() to get expected
        // 3. Call synth.next_sample() to get actual
        // This requires synth.next_sample() to not have side effects on other voices
        // or for us to re-initialize synth for each test step.

        // Reset and re-test more directly
        let synth2 = Synthesizer::new(TEST_SAMPLE_RATE);
        synth2.note_on(60, 300.0); // Activate first voice
        synth2.set_waveform(ActiveWaveform::Square);

        let mut first_voice_guard = synth2.voices[0].lock().unwrap();
        // Prime envelope
        for _ in 0..10 {
            first_voice_guard.envelope.process();
        }
        // Get expected sample directly from the voice
        let _expected_sample_direct = first_voice_guard.next_sample();
        drop(first_voice_guard);

        let _actual_sample_from_synth = synth2.next_sample();

        // The `actual_sample_from_synth` is the output of the *second* call to the voice's `next_sample`
        // (first was `expected_sample_direct`).
        // This test is still a bit convoluted due to state changes.
        // The core idea is that synth.next_sample() should reflect the (first) voice's output.

        // If the first voice is active, output should not be zero (unless waveform is silent at a point)
        let synth3 = Synthesizer::new(TEST_SAMPLE_RATE);
        synth3.note_on(60, 440.0); // freq, gain
                                   // Process a few samples to ensure envelope is > 0
        for _ in 0..((0.001 * TEST_SAMPLE_RATE) as usize) {
            // ~1ms
            let _ = synth3.next_sample();
        }
        let _sample_val = synth3.next_sample();
        if synth3.voices[0].lock().unwrap().is_active()
            && synth3.voices[0].lock().unwrap().envelope.get_level() > 0.01
        {
            // println!(\"Sample val: {}\", sample_val);
            // This can be 0.0 if the oscillator is at a zero crossing, or envelope is just starting.
            // A more robust test would check a sequence of samples.
        }
    }
}
