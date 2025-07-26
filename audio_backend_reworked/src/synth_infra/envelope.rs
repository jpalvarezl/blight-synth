#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnvelopeState {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Debug)]
pub struct Envelope {
    state: EnvelopeState,
    sample_rate: f32,
    output: f32,
    target: f32,
    coefficient: f32,
    attack_coef: f32,
    decay_coef: f32,
    release_coef: f32,
    sustain_level: f32,
}

impl Envelope {
    pub fn new(sample_rate: f32) -> Self {
        // Default values
        Self {
            state: EnvelopeState::Idle,
            sample_rate,
            output: 0.0,
            target: 0.0,
            coefficient: 0.0,
            attack_coef: 0.0,
            decay_coef: 0.0,
            release_coef: 0.0,
            sustain_level: 1.0,
        }
    }

    pub fn set_parameters(&mut self, attack_s: f32, decay_s: f32, sustain: f32, release_s: f32) {
        // Pre-calculate coefficients to avoid expensive math in the audio loop.
        // This formula creates an exponential curve.
        fn time_to_coef(time_s: f32, sample_rate: f32) -> f32 {
            if time_s <= 0.0 {
                return 0.0;
            }
            (-1.0 / (time_s * sample_rate)).exp()
        }
        self.attack_coef = time_to_coef(attack_s, self.sample_rate);
        self.decay_coef = time_to_coef(decay_s, self.sample_rate);
        self.release_coef = time_to_coef(release_s, self.sample_rate);
        self.sustain_level = sustain.clamp(0.0, 1.0);
    }

    pub fn gate(&mut self, is_on: bool) {
        if is_on {
            self.state = EnvelopeState::Attack;
            self.target = 1.0;
            self.coefficient = self.attack_coef;
        } else {
            self.state = EnvelopeState::Release;
            self.target = 0.0;
            self.coefficient = self.release_coef;
        }
    }

    pub fn process(&mut self) -> f32 {
        // State transition logic (infrequent branches)
        match self.state {
            EnvelopeState::Attack if self.output >= 0.999 => {
                self.state = EnvelopeState::Decay;
                self.target = self.sustain_level;
                self.coefficient = self.decay_coef;
            }
            EnvelopeState::Decay if self.output <= self.sustain_level => {
                self.state = EnvelopeState::Sustain;
                self.target = self.sustain_level;
                self.coefficient = 0.0; // Stay at sustain level
            }
            EnvelopeState::Release if self.output <= 0.001 => {
                self.state = EnvelopeState::Idle;
                self.coefficient = 0.0;
            }
            _ => {}
        }

        // The core branchless calculation
        self.output = self.target + self.coefficient * (self.output - self.target);

        if self.state == EnvelopeState::Idle {
            0.0
        } else {
            self.output
        }
    }

    /// Returns `true` if the envelope is currently in any state other than `Idle`.
    /// This is the primary indicator of whether a voice should be kept alive.
    pub fn is_active(&self) -> bool {
        self.state != EnvelopeState::Idle
    }
}
