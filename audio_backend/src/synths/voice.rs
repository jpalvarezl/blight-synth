use crate::synths::{adsr::ADSR, new_oscillator::Oscillator, waveform::Waveform};

pub struct Voice {
    oscillator: Oscillator,
    envelope: ADSR,
    note: Option<u8>,
    velocity: f32,
    is_active: bool,
    start_time: u64,
}

impl Voice {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            oscillator: Oscillator::new(sample_rate),
            envelope: ADSR::new(sample_rate),
            note: None,
            velocity: 0.0,
            is_active: false,
            start_time: 0,
        }
    }

    pub fn note_on(&mut self, note: u8, velocity: f32, waveform: Waveform) {
        let freq = midi_note_to_freq(note);

        self.note = Some(note);
        self.velocity = velocity;
        self.is_active = true;
        self.start_time = get_current_time(); // you provide this

        // Should we reset the oscillator phase? Probably not, as it can create clicks.
        // self.oscillator.reset();
        self.oscillator.set_waveform(waveform);
        self.oscillator.set_frequency(freq);
        
        self.envelope.reset();
        self.envelope.note_on_with_velocity(velocity);    
    }

    pub fn note_off(&mut self) {
        self.envelope.note_off();
    }

    pub fn is_active(&self) -> bool {
        self.is_active || !self.envelope.is_finished()
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
        if !self.is_active {
            return 0.0;
        }

        let env = self.envelope.next_sample();
        let sample = self.oscillator.next_sample() * env * self.velocity;

        if self.envelope.is_finished() {
            self.is_active = false;
            self.note = None;
        }

        env * sample
    }
}

fn midi_note_to_freq(note: u8) -> f32 {
    440.0 * (2.0_f32).powf((note as f32 - 69.0) / 12.0)
}

// You provide this depending on your context
fn get_current_time() -> u64 {
    // Can be a global audio frame counter, or std::time::Instant duration in frames
    0
}
