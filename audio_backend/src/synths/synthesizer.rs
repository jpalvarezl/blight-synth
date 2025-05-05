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
    sample_rate: f32, // Store sample rate for oscillator frequency updates
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

    // Helper for tests
    #[cfg(test)]
    fn get_envelope_state(&self) -> crate::envelope::EnvelopeState {
        self.envelope.get_state()
    }
}

// --- Synthesizer Struct (Refactored) ---

// Remove the old SynthControl struct if it exists

#[derive(Debug, Clone)]
pub struct Synthesizer {
    voice: Arc<Mutex<Voice>>, // Use Arc<Mutex> for thread safety
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
        Synthesizer {
            voice: Arc::new(Mutex::new(Voice::new(sample_rate))),
            sample_rate,
        }
    }

    // --- Methods to control the voice ---

    pub fn set_frequency(&self, freq: f32) {
        if let Ok(mut voice) = self.voice.lock() {
            voice.set_frequency(freq);
        } else {
            eprintln!("Error locking voice mutex in set_frequency");
        }
    }

    pub fn set_waveform(&self, waveform: ActiveWaveform) {
        if let Ok(mut voice) = self.voice.lock() {
            voice.set_waveform(waveform);
        } else {
            eprintln!("Error locking voice mutex in set_waveform");
        }
    }

    pub fn set_adsr(&self, attack: f32, decay: f32, sustain: f32, release: f32) {
        if let Ok(mut voice) = self.voice.lock() {
            voice.set_adsr(attack, decay, sustain, release);
        } else {
            eprintln!("Error locking voice mutex in set_adsr");
        }
    }

    pub fn note_on(&self, freq: f32) {
        if let Ok(mut voice) = self.voice.lock() {
            voice.note_on(freq);
        } else {
            eprintln!("Error locking voice mutex in note_on");
        }
    }

    pub fn note_off(&self) {
        if let Ok(mut voice) = self.voice.lock() {
            voice.note_off();
        } else {
            eprintln!("Error locking voice mutex in note_off");
        }
    }

    // Method needed by the audio callback to get access to the voice
    // Cloning Arc is cheap (increases reference count)
    pub fn get_voice_handle(&self) -> Arc<Mutex<Voice>> {
        self.voice.clone()
    }

    pub fn get_sample_rate(&self) -> f32 {
        self.sample_rate
    }

    // Helper for tests
    #[cfg(test)]
    fn get_voice_frequency(&self) -> f32 {
        self.voice.lock().unwrap().current_frequency
    }
    #[cfg(test)]
    fn get_voice_waveform(&self) -> ActiveWaveform {
        self.voice.lock().unwrap().active_waveform
    }
    #[cfg(test)]
    fn get_voice_envelope_state(&self) -> crate::envelope::EnvelopeState {
        self.voice.lock().unwrap().get_envelope_state()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::envelope::EnvelopeState; // Import EnvelopeState for comparison

    const TEST_SAMPLE_RATE: f32 = 44100.0;
    const TEST_FREQ: f32 = 440.0; // A4
    const TOLERANCE: f32 = 0.001;

    fn process_voice_for_samples(voice: &mut Voice, num_samples: usize) {
        for _ in 0..num_samples {
            voice.next_sample();
        }
    }

    fn process_voice_for_secs(voice: &mut Voice, seconds: f32) {
        let num_samples = (seconds * voice.sample_rate).ceil() as usize;
        process_voice_for_samples(voice, num_samples);
    }

    #[test]
    fn voice_creation() {
        let voice = Voice::new(TEST_SAMPLE_RATE);
        assert_eq!(voice.current_frequency, 440.0);
        assert_eq!(voice.active_waveform, ActiveWaveform::Sine);
        assert!(!voice.is_active()); // Starts inactive
        assert_eq!(voice.sample_rate, TEST_SAMPLE_RATE);
    }

    #[test]
    fn voice_note_on_off() {
        let mut voice = Voice::new(TEST_SAMPLE_RATE);
        assert!(!voice.is_active());
        assert_eq!(voice.get_envelope_state(), EnvelopeState::Idle);

        voice.note_on(TEST_FREQ);
        assert!(voice.is_active());
        assert_eq!(voice.current_frequency, TEST_FREQ);
        assert_eq!(voice.get_envelope_state(), EnvelopeState::Attack);

        // Process a bit (less than attack time)
        process_voice_for_samples(&mut voice, 100);
        assert!(voice.is_active());
        assert_ne!(voice.get_envelope_state(), EnvelopeState::Idle); // Should be Attack or Decay

        voice.note_off();
        assert!(voice.is_active()); // Still active during release
        assert_eq!(voice.get_envelope_state(), EnvelopeState::Release);

        // Process enough time for release to finish (default release is 0.2s)
        process_voice_for_secs(&mut voice, 0.3);
        assert!(!voice.is_active());
        assert_eq!(voice.get_envelope_state(), EnvelopeState::Idle);
    }

    #[test]
    fn voice_waveform_change() {
        let mut voice = Voice::new(TEST_SAMPLE_RATE);
        voice.note_on(TEST_FREQ);
        voice.set_waveform(ActiveWaveform::Square);
        assert_eq!(voice.active_waveform, ActiveWaveform::Square);

        // Process one sample to check if the correct oscillator is used (indirect check)
        let sample1 = voice.next_sample();
        // Change waveform and process again
        voice.set_waveform(ActiveWaveform::Saw);
        assert_eq!(voice.active_waveform, ActiveWaveform::Saw);
        let sample2 = voice.next_sample();

        // We expect different waveforms to produce different samples generally
        // This isn't a perfect test but checks the switch is happening
        // A more robust test would analyze the output over time
        assert_ne!(
            sample1, sample2,
            "Sample should differ after waveform change"
        );
    }

    #[test]
    fn voice_frequency_change() {
        let mut voice = Voice::new(TEST_SAMPLE_RATE);
        voice.note_on(TEST_FREQ);
        assert_eq!(voice.current_frequency, TEST_FREQ);

        let freq2 = 880.0;
        voice.set_frequency(freq2);
        assert_eq!(voice.current_frequency, freq2);

        // Process a sample - frequency update happens inside next_sample
        voice.next_sample();
        // Check internal oscillator frequency (requires making osc fields pub(crate) or adding getters)
        // For now, we assume set_frequency is called correctly within next_sample
    }

    #[test]
    fn voice_adsr_change() {
        let mut voice = Voice::new(TEST_SAMPLE_RATE);
        // Set very short ADSR
        voice.set_adsr(0.001, 0.001, 0.1, 0.001);
        voice.note_on(TEST_FREQ);

        // Should quickly reach sustain level (0.1)
        process_voice_for_secs(&mut voice, 0.01);
        assert_eq!(voice.get_envelope_state(), EnvelopeState::Sustain);
        assert!((voice.envelope.get_level() - 0.1).abs() < TOLERANCE);

        voice.note_off();
        // Should quickly become idle
        process_voice_for_secs(&mut voice, 0.01);
        assert_eq!(voice.get_envelope_state(), EnvelopeState::Idle);
    }

    #[test]
    fn synthesizer_controls_voice() {
        let synth = Synthesizer::new(TEST_SAMPLE_RATE);
        assert_eq!(synth.get_sample_rate(), TEST_SAMPLE_RATE);

        // Test note on/off via synthesizer
        assert_eq!(synth.get_voice_envelope_state(), EnvelopeState::Idle);
        synth.note_on(TEST_FREQ);
        assert_eq!(synth.get_voice_frequency(), TEST_FREQ);
        assert_eq!(synth.get_voice_envelope_state(), EnvelopeState::Attack);

        synth.note_off();
        // Need to process some samples for state to change in underlying envelope
        // This test only checks the immediate effect of the call on the state flag
        // A full test would involve running the audio callback simulation
        assert_eq!(synth.get_voice_envelope_state(), EnvelopeState::Release);

        // Test waveform change
        synth.set_waveform(ActiveWaveform::Triangle);
        assert_eq!(synth.get_voice_waveform(), ActiveWaveform::Triangle);

        // Test frequency change
        let freq2 = 660.0;
        synth.set_frequency(freq2);
        assert_eq!(synth.get_voice_frequency(), freq2);

        // Test ADSR change
        synth.set_adsr(0.1, 0.2, 0.7, 0.3);
        // We can't easily verify the internal rates without more access,
        // but we check the call doesn't panic.
    }
}
