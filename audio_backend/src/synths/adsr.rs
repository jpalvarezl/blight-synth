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
