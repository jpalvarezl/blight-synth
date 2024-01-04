use core::{synths::oscillator::{Oscillator, Waveform}, harmony::note::Note};
use std::sync::Arc;

pub struct OscillatorViewModel {
    oscillator: Arc<Oscillator>,
}

impl OscillatorViewModel {
    pub fn new() -> Self {
        Self {
            oscillator: Arc::new(get_oscillator(&Note::new())),
        }
    }

    pub fn get_oscillator(&self) -> &Arc<Oscillator> {
        &self.oscillator
    }
}

pub(crate) fn get_oscillator(_note: &Note) -> Oscillator {
    Oscillator {
        waveform: Waveform::Sine,
        frequency_hz: 440.0,
    }
}
