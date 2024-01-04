use core::{synths::oscillator::Oscillator, harmony::note::Note};

pub struct OscillatorViewModel {
    oscillator: Oscillator,
}

impl OscillatorViewModel {
    pub fn new() -> Self {
        Self {
            oscillator: get_oscillator(&Note::new()),
        }
    }

    pub fn get_oscillator(&self) -> &Oscillator {
        &self.oscillator
    }
}

pub(crate) fn get_oscillator(_note: &Note) -> Oscillator {
    Oscillator {
        waveform: Waveform::Sine,
        frequency_hz: 440.0,
    }
}
