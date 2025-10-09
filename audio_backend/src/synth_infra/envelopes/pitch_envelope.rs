use crate::Envelope;

pub struct PitchEnvelope {
    adsr: Envelope,
    freq_delta: f32,
    start_freq: f32,
}

impl PitchEnvelope {
    pub fn new(freq_delta_hz: f32, adsr: Envelope) -> Self {
        Self {
            adsr,
            freq_delta: freq_delta_hz,
            start_freq: 0.0,
        }
    }

    pub fn note_on(&mut self, start_freq: f32) {
        self.start_freq = start_freq;
        self.adsr.gate(true);
    }

    pub fn note_off(&mut self) {
        self.adsr.gate(false);
    }

    #[inline(always)]
    pub fn next_freq(&mut self) -> f32 {
        let env_val = self.adsr.process();
        let end_freq = self.start_freq + self.freq_delta;
        self.start_freq * (1.0 - env_val) + end_freq * env_val
    }

    pub fn is_active(&self) -> bool {
        self.adsr.is_active()
    }

    pub fn set_freq_delta(&mut self, freq_delta: f32) {
        self.freq_delta = freq_delta;
    }

    pub fn set_decay_time(&mut self, decay_time: f32) {
        self.adsr.set_decay(decay_time);
        self.adsr.set_attack(0.0); // Instant attack for pitch sweep
        self.adsr.set_sustain(1.0); // Full sweep completion
        self.adsr.set_release(0.0); // No release for pitch
    }

    pub fn set_parameters(&mut self, a: f32, d: f32, s: f32, r: f32) {
        self.adsr.set_parameters(a, d, s, r);
    }
}
