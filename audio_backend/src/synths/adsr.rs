#[derive(Debug, Clone, Copy, PartialEq)]
enum ADSRState {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Debug, Clone)]
pub struct ADSR {
    state: ADSRState,
    sample_rate: f32,

    attack_time: f32,
    decay_time: f32,
    sustain_level: f32,
    release_time: f32,

    current_amplitude: f32,
    step: f32,
    peak_amplitude: f32,
}

impl ADSR {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            state: ADSRState::Idle,
            sample_rate,

            attack_time: 0.01,
            decay_time: 0.1,
            sustain_level: 0.8,
            release_time: 0.05, // Much shorter release time - was 0.2

            current_amplitude: 0.0,
            step: 0.0,
            peak_amplitude: 1.0,
        }
    }

    pub fn set_params(&mut self, attack: f32, decay: f32, sustain: f32, release: f32) {
        self.attack_time = attack;
        self.decay_time = decay;
        self.sustain_level = sustain;
        self.release_time = release;
    }

    pub fn reset(&mut self) {
        self.state = ADSRState::Idle;
        self.current_amplitude = 0.0;
        self.step = 0.0;
        self.peak_amplitude = 1.0;
    }

    pub fn note_on(&mut self) {
        self.peak_amplitude = 1.0;
        self.state = ADSRState::Attack;
        self.step = self.peak_amplitude / (self.attack_time * self.sample_rate);
    }

    pub fn note_on_with_velocity(&mut self, velocity: f32) {
        self.peak_amplitude = velocity.clamp(0.0, 1.0);
        self.current_amplitude = 0.0; // Reset amplitude to start attack from zero
        self.state = ADSRState::Attack;
        self.step = self.peak_amplitude / (self.attack_time * self.sample_rate);
        println!("[ADSR] note_on_with_velocity: velocity={}, peak_amplitude={}, step={}, attack_time={}, sample_rate={}", 
                 velocity, self.peak_amplitude, self.step, self.attack_time, self.sample_rate);
    }

    pub fn note_off(&mut self) {
        if self.state != ADSRState::Idle {
            println!("[ADSR] note_off called: current_state={:?}, current_amplitude={}", self.state, self.current_amplitude);
            self.state = ADSRState::Release;
            // Calculate step based on current amplitude, not sustain target
            // This ensures proper release regardless of what state we were in
            self.step = self.current_amplitude / (self.release_time * self.sample_rate);
            println!("[ADSR] note_off: current_amplitude={}, release_time={}, sample_rate={}, calculated_step={}", 
                     self.current_amplitude, self.release_time, self.sample_rate, self.step);
            println!("[ADSR] note_off: denominator = release_time * sample_rate = {} * {} = {}", 
                     self.release_time, self.sample_rate, self.release_time * self.sample_rate);
        }
    }

    pub fn next_sample(&mut self) -> f32 {
        println!("[ADSR] next_sample called: state={:?}, current_amplitude={}, step={}", 
                 self.state, self.current_amplitude, self.step);

        let old_state = self.state;
        match self.state {
            ADSRState::Idle => {
                self.current_amplitude = 0.0;
            }
            ADSRState::Attack => {
                self.current_amplitude += self.step;
                if self.current_amplitude >= self.peak_amplitude {
                    self.current_amplitude = self.peak_amplitude;
                    self.state = ADSRState::Decay;
                    let delta = self.peak_amplitude - (self.peak_amplitude * self.sustain_level);
                    self.step = delta / (self.decay_time * self.sample_rate);
                    println!("[ADSR] Attack->Decay: amplitude={}, new_step={}", self.current_amplitude, self.step);
                }
            }
            ADSRState::Decay => {
                self.current_amplitude -= self.step;
                let sustain_target = self.peak_amplitude * self.sustain_level;
                if self.current_amplitude <= sustain_target {
                    self.current_amplitude = sustain_target;
                    self.state = ADSRState::Sustain;
                    println!("[ADSR] Decay->Sustain: amplitude={}", self.current_amplitude);
                }
            }
            ADSRState::Sustain => {
                // Hold steady
            }
            ADSRState::Release => {
                self.current_amplitude -= self.step;
                // Use a small threshold to avoid extremely long release times
                const MIN_AMPLITUDE: f32 = 0.001;
                if self.current_amplitude <= MIN_AMPLITUDE {
                    self.current_amplitude = 0.0;
                    self.state = ADSRState::Idle;
                    println!("[ADSR] Release->Idle: amplitude={} (hit minimum threshold)", self.current_amplitude);
                }
            }
        }

        if old_state != self.state {
            println!("[ADSR] State change: {:?} -> {:?}, amplitude={}", old_state, self.state, self.current_amplitude);
        }

        self.current_amplitude
    }

    pub fn is_finished(&self) -> bool {
        self.state == ADSRState::Idle
    }
}

#[cfg(test)]
mod tests {
use super::*;

    const SAMPLE_RATE: f32 = 1000.0; // 1kHz for easy calculations
    const EPS: f32 = 0.01; // Tolerance for floating point comparisons

    #[test]
    fn test_adsr_initialization() {
        let adsr = ADSR::new(44100.0);
        assert_eq!(adsr.state, ADSRState::Idle);
        assert_eq!(adsr.current_amplitude, 0.0);
        assert!(adsr.is_finished());
    }

    #[test]
    fn test_adsr_attack_phase() {
        let mut adsr = ADSR::new(SAMPLE_RATE); // 1kHz for easy math
        adsr.set_params(0.1, 0.1, 0.7, 0.1); // 100ms attack = 100 samples at 1kHz
        adsr.note_on_with_velocity(1.0);
        
        assert_eq!(adsr.state, ADSRState::Attack);
        assert!(!adsr.is_finished());
        
        // First sample should start increasing from 0
        let first_sample = adsr.next_sample();
        println!("First attack sample: {}", first_sample);
        assert!(first_sample > 0.0);
        assert!(first_sample < 1.0);
        
        // After ~100 samples, should reach peak and move to decay
        for i in 0..99 {
            let sample = adsr.next_sample();
            if i < 5 {
                println!("Attack sample {}: {}", i+2, sample);
            }
        }
        
        let final_attack_sample = adsr.next_sample();
        println!("Final attack sample: {}, state: {:?}", final_attack_sample, adsr.state);
        assert_eq!(adsr.state, ADSRState::Decay);
        assert!((final_attack_sample - 1.0).abs() < EPS);
    }

    #[test]
    fn test_adsr_decay_phase() {
        let mut adsr = ADSR::new(SAMPLE_RATE);
        adsr.set_params(0.01, 0.1, 0.5, 0.1); // Fast attack, 100ms decay
        adsr.note_on_with_velocity(1.0);
        
        // Skip through attack quickly
        for _ in 0..20 {
            adsr.next_sample();
            if adsr.state == ADSRState::Decay { break; }
        }
        
        assert_eq!(adsr.state, ADSRState::Decay);
        let start_amplitude = adsr.current_amplitude;
        println!("Decay start amplitude: {}", start_amplitude);
        
        // Run through decay phase
        for i in 0..120 {
            let sample = adsr.next_sample();
            if i < 5 || i % 20 == 0 {
                println!("Decay sample {}: {}", i, sample);
            }
            if adsr.state == ADSRState::Sustain { break; }
        }
        
        assert_eq!(adsr.state, ADSRState::Sustain);
        assert!((adsr.current_amplitude - 0.5).abs() < EPS, 
                "Should reach sustain level of 0.5, got {}", adsr.current_amplitude);
    }

    #[test]
    fn test_adsr_sustain_phase() {
        let mut adsr = ADSR::new(SAMPLE_RATE);
        adsr.set_params(0.01, 0.01, 0.7, 0.1);
        adsr.note_on_with_velocity(1.0);
        
        // Get to sustain quickly
        for _ in 0..50 {
            adsr.next_sample();
            if adsr.state == ADSRState::Sustain { break; }
        }
        
        assert_eq!(adsr.state, ADSRState::Sustain);
        let sustain_level = adsr.current_amplitude;
        println!("Sustain level: {}", sustain_level);
        
        // Verify sustain holds steady for many samples
        for i in 0..100 {
            let sample = adsr.next_sample();
            assert!((sample - sustain_level).abs() < EPS, 
                    "Sample {} should hold sustain level {}, got {}", i, sustain_level, sample);
        }
    }

    #[test]
    fn test_adsr_release_step_calculation() {
        let mut adsr = ADSR::new(44100.0); // Real sample rate
        adsr.set_params(0.1, 0.1, 0.8, 0.2); // 200ms release
        adsr.note_on_with_velocity(1.0);
        
        // Set to a known amplitude in sustain
        adsr.current_amplitude = 0.8;
        adsr.state = ADSRState::Sustain;
        
        // Trigger release
        adsr.note_off();
        
        // Expected step: 0.8 / (0.2 * 44100) = 0.8 / 8820 ≈ 0.0000907
        let expected_step = 0.8 / (0.2 * 44100.0);
        println!("Release step calculation: amplitude={}, release_time={}, sample_rate={}, step={}, expected={}",
                 0.8, 0.2, 44100.0, adsr.step, expected_step);
        assert!((adsr.step - expected_step).abs() < 0.0000001);
    }

    #[test]
    fn test_adsr_release_timing() {
        let mut adsr = ADSR::new(SAMPLE_RATE);
        adsr.set_params(0.01, 0.01, 0.6, 0.1); // 100ms release
        adsr.note_on_with_velocity(1.0);
        
        // Skip to sustain quickly
        for _ in 0..30 {
            adsr.next_sample();
        }
        
        // Verify we're in sustain
        assert_eq!(adsr.state, ADSRState::Sustain);
        let sustain_amplitude = adsr.current_amplitude;
        println!("Sustain amplitude: {}", sustain_amplitude);
        
        // Trigger release
        adsr.note_off();
        assert_eq!(adsr.state, ADSRState::Release);
        println!("Release step: {}", adsr.step);
        
        // Count samples until finished
        let mut samples_in_release = 0;
        while !adsr.is_finished() && samples_in_release < 200 {
            let sample = adsr.next_sample();
            samples_in_release += 1;
            
            if samples_in_release <= 5 || samples_in_release % 20 == 0 {
                println!("Release sample {}: {}", samples_in_release, sample);
            }
            
            assert!(sample >= 0.0);
            assert!(sample <= sustain_amplitude);
        }
        
        println!("Release took {} samples, expected ~100", samples_in_release);
        assert!(adsr.is_finished());
        assert_eq!(adsr.current_amplitude, 0.0);
        assert!(samples_in_release >= 90);
        assert!(samples_in_release <= 110);
    }

    #[test]
    fn test_complete_adsr_cycle() {
        let mut adsr = ADSR::new(SAMPLE_RATE);
        adsr.set_params(0.05, 0.05, 0.6, 0.05); // 50ms each phase
        adsr.note_on_with_velocity(0.8);
        
        println!("=== Complete ADSR Cycle Test ===");
        
        // Attack phase - should reach 0.8 peak
        let mut samples_in_attack = 0;
        while adsr.state == ADSRState::Attack && samples_in_attack < 100 {
            let sample = adsr.next_sample();
            samples_in_attack += 1;
            if samples_in_attack <= 5 {
                println!("Attack sample {}: {}", samples_in_attack, sample);
            }
        }
        println!("Attack took {} samples", samples_in_attack);
        assert_eq!(adsr.state, ADSRState::Decay);
        assert!((adsr.current_amplitude - 0.8).abs() < EPS);
        
        // Decay phase - should reach 0.6 * 0.8 = 0.48
        let mut samples_in_decay = 0;
        while adsr.state == ADSRState::Decay && samples_in_decay < 100 {
            let sample = adsr.next_sample();
            samples_in_decay += 1;
            if samples_in_decay <= 5 {
                println!("Decay sample {}: {}", samples_in_decay, sample);
            }
        }
        println!("Decay took {} samples", samples_in_decay);
        assert_eq!(adsr.state, ADSRState::Sustain);
        assert!((adsr.current_amplitude - 0.48).abs() < EPS);
        
        // Sustain phase - should hold at 0.48
        for i in 0..20 {
            let sample = adsr.next_sample();
            if i < 3 {
                println!("Sustain sample {}: {}", i, sample);
            }
            assert!((sample - 0.48).abs() < EPS);
        }
        assert_eq!(adsr.state, ADSRState::Sustain);
        
        // Release phase - should go to 0
        adsr.note_off();
        assert_eq!(adsr.state, ADSRState::Release);
        let mut samples_in_release = 0;
        while !adsr.is_finished() && samples_in_release < 100 {
            let sample = adsr.next_sample();
            samples_in_release += 1;
            if samples_in_release <= 5 {
                println!("Release sample {}: {}", samples_in_release, sample);
            }
        }
        println!("Release took {} samples", samples_in_release);
        assert!(adsr.is_finished());
        assert_eq!(adsr.current_amplitude, 0.0);
    }

    #[test]
    fn test_velocity_scaling() {
        let mut adsr = ADSR::new(SAMPLE_RATE);
        adsr.set_params(0.001, 0.1, 0.5, 0.1);
        adsr.note_on_with_velocity(0.5);
        let mut last = 0.0;
        for _ in 0..1000 {
            last = adsr.next_sample();
            if adsr.state == ADSRState::Decay { break; }
        }
        assert!((last - 0.5).abs() < EPS, "Attack should reach velocity peak");
    }
}