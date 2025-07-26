// ⚙️ Optimization Ideas (Later)
//   - Use lookup tables (LUTs) for sine/trig functions
//   - SIMD waveform generation
//   - PolyBLEP for band-limited oscillators (avoids aliasing)

use std::f32::consts::{PI, TAU};

use crate::synths::Waveform;

#[derive(Debug, Clone)]
pub struct Oscillator {
    waveform: Waveform,
    frequency: f32,
    sample_rate: f32,
    phase: f32,
}

impl Oscillator {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            waveform: Waveform::Sine,
            frequency: 440.0,
            sample_rate,
            phase: 0.0,
        }
    }

    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.waveform = waveform;
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
    }

    pub fn reset(&mut self) {
        self.phase = 0.0;
    }

    pub fn next_sample(&mut self) -> f32 {
        let phase_inc = self.frequency * TAU / self.sample_rate;

        let sample = match self.waveform {
            Waveform::Sine => (self.phase).sin(),
            Waveform::Square => {
                if self.phase < PI - f32::EPSILON {
                    1.0
                } else {
                    -1.0
                }
            }
            Waveform::Sawtooth => 2.0 * (self.phase / TAU) - 1.0,
            Waveform::Triangle => {
                // Correct triangle wave: rises from -1 to 1 in first half, falls from 1 to -1 in second half
                let normalized_phase = self.phase / TAU; // 0 to 1
                if normalized_phase < 0.5 {
                    // Rising: -1 to 1
                    4.0 * normalized_phase - 1.0
                } else {
                    // Falling: 1 to -1
                    3.0 - 4.0 * normalized_phase
                }
            }
        };

        // Debug output to see what's happening - include more info
        // if self.phase < phase_inc * 2.0 { // Print at start of each cycle
        //     println!("🎵 Oscillator: waveform={:?}, freq={:.1}Hz, phase={:.6}, phase_inc={:.6}, sample={:.4}",
        //         self.waveform, self.frequency, self.phase, phase_inc, sample);
        // }

        // More precise phase accumulation to avoid floating point drift
        self.phase += phase_inc;
        if self.phase >= TAU {
            self.phase -= TAU;
        }

        sample
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use {
        crate::test::audio_backend_utils::{run_audio_stream_with_producer, SampleProducer},
        std::sync::{Arc, Mutex},
        std::thread,
        std::time::Duration,
    };

    impl SampleProducer for Oscillator {
        fn next_sample(&mut self) -> f32 {
            self.next_sample() * 0.1 // Reduce amplitude to prevent loud output
        }
    }

    #[test]
    fn test_oscillator_sine_wave() {
        let mut osc = Oscillator::new(44100.0);
        osc.set_waveform(Waveform::Sine);
        osc.set_frequency(441.0); // 100x slower than sample rate
        let sample1 = osc.next_sample();
        let sample2 = osc.next_sample();
        assert!(sample1.abs() < 1.0, "Sine sample should be in [-1, 1]");
        assert!(sample2.abs() < 1.0, "Sine sample should be in [-1, 1]");
        assert_ne!(sample1, sample2, "Samples should differ as phase advances");
    }

    #[test]
    fn test_oscillator_square_wave() {
        let mut osc = Oscillator::new(44100.0);
        osc.set_waveform(Waveform::Square);
        osc.set_frequency(44100.0 / 4.0); // 4 samples per cycle
        let samples: Vec<f32> = (0..4).map(|_| osc.next_sample()).collect();

        // First half of cycle should be 1.0, second half should be -1.0
        assert_eq!(samples[0], 1.0); // phase = 0
        assert_eq!(samples[1], 1.0); // phase = π/2 (still < π)
        assert_eq!(samples[2], -1.0); // phase = π (>= π)
        assert_eq!(samples[3], -1.0); // phase = 3π/2 (>= π)
    }

    #[test]
    fn test_oscillator_sawtooth_wave() {
        let mut osc = Oscillator::new(44100.0);
        osc.set_waveform(Waveform::Sawtooth);
        osc.set_frequency(44100.0 / 2.0); // 2 samples per cycle
        let s1 = osc.next_sample();
        let s2 = osc.next_sample();
        assert!(s1 >= -1.0 && s1 <= 1.0);
        assert!(s2 >= -1.0 && s2 <= 1.0);
        assert_ne!(s1, s2);
    }

    #[test]
    fn test_oscillator_triangle_wave() {
        let mut osc = Oscillator::new(44100.0);
        osc.set_waveform(Waveform::Triangle);
        osc.set_frequency(44100.0 / 4.0); // 4 samples per cycle
        let s1 = osc.next_sample();
        let s2 = osc.next_sample();
        let s3 = osc.next_sample();
        let s4 = osc.next_sample();
        assert!(s1 >= -1.0 && s1 <= 1.0);
        assert!(s2 >= -1.0 && s2 <= 1.0);
        assert!(s3 >= -1.0 && s3 <= 1.0);
        assert!(s4 >= -1.0 && s4 <= 1.0);
        // Should see a pattern: up, up, down, down (or similar)
        assert_ne!(s1, s2);
        assert_ne!(s2, s3);
        assert_ne!(s3, s4);
    }

    #[test]
    fn test_oscillator_reset() {
        let mut osc = Oscillator::new(44100.0);
        osc.set_waveform(Waveform::Sine);
        let s1 = osc.next_sample();
        osc.reset();
        let s2 = osc.next_sample();
        assert_eq!(s1, s2, "Reset should return phase to start");
    }

    #[test]
    fn test_waveform_switching() {
        let mut osc = Oscillator::new(44100.0);
        osc.set_waveform(Waveform::Sine);
        let s_sine = osc.next_sample();
        osc.set_waveform(Waveform::Square);
        let s_square = osc.next_sample();
        osc.set_waveform(Waveform::Sawtooth);
        let s_saw = osc.next_sample();
        osc.set_waveform(Waveform::Triangle);
        let s_tri = osc.next_sample();
        assert_ne!(s_sine, s_square);
        assert_ne!(s_square, s_saw);
        assert_ne!(s_saw, s_tri);
    }

    #[test]
    fn test_sine_oscillator_audio() {
        println!("🎵 Testing Sine Oscillator - 440 Hz for 1 second");

        let mut osc = Oscillator::new(44100.0);
        osc.set_waveform(Waveform::Sine);
        osc.set_frequency(440.0);

        let oscillator = Arc::new(Mutex::new(osc));

        match run_audio_stream_with_producer(oscillator) {
            Ok(stream) => {
                println!("🔊 Playing Sine wave at 440 Hz...");
                thread::sleep(Duration::from_secs(1));
                drop(stream);
                println!("✅ Sine wave test completed");
            }
            Err(e) => {
                eprintln!("❌ Failed to create audio stream: {}", e);
                panic!("Audio test failed");
            }
        }
    }

    #[test]
    fn test_square_oscillator_audio() {
        println!("🎵 Testing Square Oscillator - 440 Hz for 1 second");

        let mut osc = Oscillator::new(44100.0);
        osc.set_waveform(Waveform::Square);
        osc.set_frequency(440.0);

        let oscillator = Arc::new(Mutex::new(osc));

        match run_audio_stream_with_producer(oscillator) {
            Ok(stream) => {
                println!("🔊 Playing Square wave at 440 Hz...");
                thread::sleep(Duration::from_secs(1));
                drop(stream);
                println!("✅ Square wave test completed");
            }
            Err(e) => {
                eprintln!("❌ Failed to create audio stream: {}", e);
                panic!("Audio test failed");
            }
        }
    }

    #[test]
    fn test_triangle_oscillator_audio() {
        println!("🎵 Testing Triangle Oscillator - 440 Hz for 1 second");

        let mut osc = Oscillator::new(44100.0);
        osc.set_waveform(Waveform::Triangle);
        osc.set_frequency(440.0);

        let oscillator = Arc::new(Mutex::new(osc));

        match run_audio_stream_with_producer(oscillator) {
            Ok(stream) => {
                println!("🔊 Playing Triangle wave at 440 Hz...");
                thread::sleep(Duration::from_secs(1));
                drop(stream);
                println!("✅ Triangle wave test completed");
            }
            Err(e) => {
                eprintln!("❌ Failed to create audio stream: {}", e);
                panic!("Audio test failed");
            }
        }
    }

    #[test]
    fn test_sawtooth_oscillator_audio() {
        println!("🎵 Testing Sawtooth Oscillator - 440 Hz for 1 second");

        let mut osc = Oscillator::new(44100.0);
        osc.set_waveform(Waveform::Sawtooth);
        osc.set_frequency(440.0);

        let oscillator = Arc::new(Mutex::new(osc));

        match run_audio_stream_with_producer(oscillator) {
            Ok(stream) => {
                println!("🔊 Playing Sawtooth wave at 440 Hz...");
                thread::sleep(Duration::from_secs(1));
                drop(stream);
                println!("✅ Sawtooth wave test completed");
            }
            Err(e) => {
                eprintln!("❌ Failed to create audio stream: {}", e);
                panic!("Audio test failed");
            }
        }
    }
}
