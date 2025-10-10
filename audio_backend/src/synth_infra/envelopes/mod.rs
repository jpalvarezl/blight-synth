mod adsr_envelope;
mod pitch_envelope;

pub use adsr_envelope::*;
pub use pitch_envelope::*;

pub trait EnvelopeLike: Send + Sync {
    fn gate(&mut self, on: bool);
    fn process(&mut self) -> f32;
    fn is_active(&self) -> bool;
    fn set_parameters(&mut self, a: f32, d: f32, s: f32, r: f32);
    fn set_attack(&mut self, attack: f32);
    fn set_decay(&mut self, decay: f32);
    fn set_sustain(&mut self, sustain: f32);
    fn set_release(&mut self, release: f32);
}

pub trait PitchEnvLike: Send + Sync {
    fn note_on(&mut self, start_freq: f32);
    fn note_off(&mut self);
    fn next_freq(&mut self) -> f32;
    fn is_active(&self) -> bool;
    fn set_freq_delta(&mut self, freq_delta: f32);
    fn set_decay_time(&mut self, decay_time: f32);
    fn set_parameters(&mut self, a: f32, d: f32, s: f32, r: f32);
}

impl EnvelopeLike for Envelope {
    fn gate(&mut self, on: bool) {
        self.gate(on)
    }
    fn process(&mut self) -> f32 {
        self.process()
    }
    fn is_active(&self) -> bool {
        self.is_active()
    }
    fn set_parameters(&mut self, a: f32, d: f32, s: f32, r: f32) {
        self.set_parameters(a, d, s, r)
    }
    fn set_attack(&mut self, attack: f32) {
        self.set_attack(attack)
    }
    fn set_decay(&mut self, decay: f32) {
        self.set_decay(decay)
    }
    fn set_sustain(&mut self, sustain: f32) {
        self.set_sustain(sustain)
    }
    fn set_release(&mut self, release: f32) {
        self.set_release(release)
    }
}

impl PitchEnvLike for PitchEnvelope {
    fn note_on(&mut self, start_freq: f32) {
        self.note_on(start_freq)
    }
    fn note_off(&mut self) {
        self.note_off()
    }
    fn next_freq(&mut self) -> f32 {
        self.next_freq()
    }
    fn is_active(&self) -> bool {
        self.is_active()
    }
    fn set_freq_delta(&mut self, freq_delta: f32) {
        self.set_freq_delta(freq_delta)
    }
    fn set_decay_time(&mut self, decay_time: f32) {
        self.set_decay_time(decay_time)
    }

    fn set_parameters(&mut self, a: f32, d: f32, s: f32, r: f32) {
        self.set_parameters(a, d, s, r)
    }
}
