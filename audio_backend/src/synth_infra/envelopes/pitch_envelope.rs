use crate::Envelope;

pub struct PitchEnvelope {
    adsr: Envelope,
    freq_delta: f32,
    start_freq: f32,
}

impl PitchEnvelope {
    pub fn new(freq_delta: f32, adsr: Envelope) -> Self {
        Self {
            adsr,
            freq_delta,
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
}
