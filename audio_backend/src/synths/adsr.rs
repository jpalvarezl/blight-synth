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
            release_time: 0.2,

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
        self.state = ADSRState::Attack;
        self.step = self.peak_amplitude / (self.attack_time * self.sample_rate);
    }

    pub fn note_off(&mut self) {
        if self.state != ADSRState::Idle {
            self.state = ADSRState::Release;
            let sustain_target = self.peak_amplitude * self.sustain_level;
            self.step = sustain_target / (self.release_time * self.sample_rate);
        }
    }

    pub fn next_sample(&mut self) -> f32 {
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
                }
            }
            ADSRState::Decay => {
                self.current_amplitude -= self.step;
                let sustain_target = self.peak_amplitude * self.sustain_level;
                if self.current_amplitude <= sustain_target {
                    self.current_amplitude = sustain_target;
                    self.state = ADSRState::Sustain;
                }
            }
            ADSRState::Sustain => {
                // Hold steady
            }
            ADSRState::Release => {
                self.current_amplitude -= self.step;
                if self.current_amplitude <= 0.0 {
                    self.current_amplitude = 0.0;
                    self.state = ADSRState::Idle;
                }
            }
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

    const SAMPLE_RATE: f32 = 44100.0;
    const EPS: f32 = 1e-4;

    #[test]
    fn test_initial_state() {
        let adsr = ADSR::new(SAMPLE_RATE);
        assert_eq!(adsr.state, ADSRState::Idle);
        assert!((adsr.current_amplitude - 0.0).abs() < EPS);
    }

    #[test]
    fn test_attack_phase_reaches_peak() {
        let mut adsr = ADSR::new(SAMPLE_RATE);
        adsr.set_params(0.001, 0.1, 0.5, 0.1); // Fast attack
        adsr.note_on();
        let mut last = 0.0;
        for _ in 0..(SAMPLE_RATE as usize) {
            last = adsr.next_sample();
            if adsr.state == ADSRState::Decay {
                break;
            }
        }
        assert!((last - 1.0).abs() < EPS, "Attack should reach peak amplitude");
        assert_eq!(adsr.state, ADSRState::Decay);
    }

    #[test]
    fn test_decay_phase_reaches_sustain() {
        let mut adsr = ADSR::new(SAMPLE_RATE);
        adsr.set_params(0.001, 0.001, 0.3, 0.1); // Fast attack/decay
        adsr.note_on();
        // Go through attack
        for _ in 0..100 { adsr.next_sample(); if adsr.state == ADSRState::Decay { break; } }
        // Go through decay
        let mut last = 0.0;
        for _ in 0..1000 {
            last = adsr.next_sample();
            if adsr.state == ADSRState::Sustain {
                break;
            }
        }
        assert!((last - 0.3).abs() < EPS, "Decay should reach sustain level");
        assert_eq!(adsr.state, ADSRState::Sustain);
    }

    #[test]
    fn test_sustain_holds() {
        let mut adsr = ADSR::new(SAMPLE_RATE);
        adsr.set_params(0.001, 0.001, 0.7, 0.1);
        adsr.note_on();
        // Go to sustain
        for _ in 0..1000 { adsr.next_sample(); if adsr.state == ADSRState::Sustain { break; } }
        let sustain_level = adsr.current_amplitude;
        for _ in 0..100 {
            let val = adsr.next_sample();
            assert!((val - sustain_level).abs() < EPS, "Sustain should hold");
        }
    }

    #[test]
    fn test_release_to_idle() {
        let mut adsr = ADSR::new(SAMPLE_RATE);
        adsr.set_params(0.001, 0.001, 0.5, 0.001);
        adsr.note_on();
        // Go to sustain
        for _ in 0..1000 { adsr.next_sample(); if adsr.state == ADSRState::Sustain { break; } }
        adsr.note_off();
        let mut last = adsr.current_amplitude;
        for _ in 0..1000 {
            last = adsr.next_sample();
            if adsr.state == ADSRState::Idle { break; }
        }
        assert!((last - 0.0).abs() < EPS, "Release should reach zero");
        assert_eq!(adsr.state, ADSRState::Idle);
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