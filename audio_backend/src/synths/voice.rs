use std::time::Instant;

use crate::synths::{adsr::ADSR, new_oscillator::Oscillator, waveform::Waveform};

#[cfg(test)]
use {
    crate::test::audio_backend_utils::{SampleProducer, run_audio_stream_with_producer},
    std::sync::{Arc, Mutex},
    std::thread,
    std::time::Duration,
};

#[derive(Debug, Clone)]
pub struct Voice {
    oscillator: Oscillator,
    envelope: ADSR,
    note: Option<u8>,
    velocity: f32,
    start_time: u64,
}

impl Voice {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            oscillator: Oscillator::new(sample_rate),
            envelope: ADSR::new(sample_rate),
            note: None,
            velocity: 0.0,
            start_time: 0,
        }
    }

    pub fn note_on(&mut self, note: u8, velocity: f32, waveform: Waveform) {
        let freq = midi_note_to_freq(note);

        self.note = Some(note);
        self.velocity = velocity;
        self.start_time = get_current_time(); // you provide this

        // Should we reset the oscillator phase? Probably not, as it can create clicks.
        // self.oscillator.reset();
        self.oscillator.set_waveform(waveform);
        self.oscillator.set_frequency(freq);


        self.envelope.reset();
        self.envelope.note_on_with_velocity(velocity);
    }

    pub fn note_off(&mut self) {
        // println!("[Voice] note_off called");
        self.envelope.note_off();
        // Don't clear note here - let it clear when envelope finishes
    }

    pub fn is_active(&self) -> bool {
        let active = !self.envelope.is_finished();
        // println!("[Voice] is_active check: note={:?}, envelope_finished={}, returning={}", 
        //          self.note, self.envelope.is_finished(), active);
        active
    }

    pub fn is_playing(&self, note: u8) -> bool {
        self.note == Some(note)
    }

    pub fn set_adsr(&mut self, attack: f32, decay: f32, sustain: f32, release: f32) {
        self.envelope.set_params(attack, decay, sustain, release);
    }

    pub fn start_time(&self) -> u64 {
        self.start_time
    }

    pub fn next_sample(&mut self) -> f32 {
        let env = self.envelope.next_sample();
        let sample = self.oscillator.next_sample();

        // println!("[Voice] next_sample: note={:?}, env={}, sample={}, velocity={}, is_finished={}", 
        //          self.note, env, sample, self.velocity, self.envelope.is_finished());

        if self.envelope.is_finished() {
            if self.note.is_some() {
                // println!("[Voice] Clearing note - envelope finished");
            }
            self.note = None;
            0.0
        } else {
            sample * env * self.velocity
        }
    }
}

fn midi_note_to_freq(note: u8) -> f32 {
    // Standard MIDI note to frequency conversion
    // A4 (note 69) = 440 Hz
    // Each semitone is a factor of 2^(1/12)
    const A4_FREQ: f32 = 440.0;
    const A4_NOTE: f32 = 69.0;
    const SEMITONES_PER_OCTAVE: f32 = 12.0;
    
    let freq = A4_FREQ * (2.0_f32).powf((note as f32 - A4_NOTE) / SEMITONES_PER_OCTAVE);
    
    // Debug output to verify frequency calculation
    // println!("🎵 MIDI Note {} -> {:.2} Hz", note, freq);
    
    freq
}

// You provide this depending on your context
fn get_current_time() -> u64 {
    // Can be a global audio frame counter, or std::time::Instant duration in frames
    Instant::now().elapsed().as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::audio_backend_utils::run_audio_stream_with_producer;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_midi_note_to_freq() {
        // Test known MIDI note frequencies
        assert_eq!(midi_note_to_freq(69), 440.0); // A4
        assert!((midi_note_to_freq(60) - 261.63).abs() < 0.1); // C4 (Middle C)
        assert!((midi_note_to_freq(72) - 523.25).abs() < 0.1); // C5
        assert!((midi_note_to_freq(57) - 220.0).abs() < 0.1);  // A3 (note 57 is A3, not G3)
        
        // Test edge cases
        assert!(midi_note_to_freq(0) > 0.0);    // Lowest MIDI note should be positive
        assert!(midi_note_to_freq(127) > 0.0);  // Highest MIDI note should be positive
        
        // Test frequency increases with note number
        assert!(midi_note_to_freq(61) > midi_note_to_freq(60));
        assert!(midi_note_to_freq(70) > midi_note_to_freq(69));
    }

    #[test]
    fn test_octave_relationships() {
        // Each octave should double the frequency
        let c4 = midi_note_to_freq(60);  // C4
        let c5 = midi_note_to_freq(72);  // C5 (one octave higher)
        assert!((c5 / c4 - 2.0).abs() < 0.001); // Should be exactly 2x
        
        let a4 = midi_note_to_freq(69);  // A4
        let a5 = midi_note_to_freq(81);  // A5 (one octave higher)
        assert!((a5 / a4 - 2.0).abs() < 0.001); // Should be exactly 2x
    }

    #[test]
    fn test_voice_sine_audio() {
        println!("🎵 Testing Voice with Sine Wave - 440 Hz for 1 second");
        
        let mut voice = Voice::new(44100.0);
        voice.note_on(69, 0.5, Waveform::Sine); // A4 (440 Hz) with 50% velocity
        
        let voice_arc = Arc::new(Mutex::new(voice));
        
        match run_audio_stream_with_producer(voice_arc) {
            Ok(stream) => {
                println!("🔊 Playing Voice sine wave at 440 Hz...");
                thread::sleep(Duration::from_secs(1));
                drop(stream);
                println!("✅ Voice sine wave test completed");
            },
            Err(e) => {
                eprintln!("❌ Failed to create audio stream: {}", e);
                panic!("Audio test failed");
            }
        }
    }

    #[test]
    fn test_voice_adsr_envelope_audio() {
        println!("🎵 Testing Voice with ADSR Envelope - 440 Hz with attack/decay/sustain/release");
        
        let mut voice = Voice::new(44100.0);
        
        // Set ADSR parameters: 0.3s attack, 0.4s decay, 0.6 sustain level, 0.5s release
        voice.set_adsr(0.3, 0.4, 0.6, 0.5);
        
        // Start the note with A4 (440 Hz) at 80% velocity
        voice.note_on(69, 0.8, Waveform::Sine);
        
        let voice_arc = Arc::new(Mutex::new(voice));
        
        match run_audio_stream_with_producer(voice_arc.clone()) {
            Ok(stream) => {
                // println!("🔊 Playing Voice with ADSR envelope...");
                
                // Let it play through attack and decay phases (0.7s total)
                // println!("📈 Attack and Decay phases (0.7s)...");
                // thread::sleep(Duration::from_millis(700));
                
                // Now in sustain phase - play for a bit
                // println!("🔄 Sustain phase (0.5s)...");
                thread::sleep(Duration::from_millis(500));
                
                // Trigger note off to start release phase
                println!("📉 Starting release phase...");
                {
                    let mut voice = voice_arc.lock().unwrap();
                    voice.note_off();
                }
                
                // Let release phase complete (0.5s)
                println!("📉 Release phase (0.5s)...");
                thread::sleep(Duration::from_millis(2000));
                
                drop(stream);
                println!("✅ Voice ADSR envelope test completed");
            },
            Err(e) => {
                eprintln!("❌ Failed to create audio stream: {}", e);
                panic!("Audio test failed");
            }
        }
    }
}

#[cfg(test)]
impl crate::test::audio_backend_utils::SampleProducer for Voice {
    fn next_sample(&mut self) -> f32 {
        self.next_sample()
    }
}
