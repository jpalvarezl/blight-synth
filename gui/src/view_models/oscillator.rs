use core::synths::oscillator::{Oscillator, Waveform};
use harmony::note::Note;
use std::sync::{Arc, RwLock};

pub struct OscillatorViewModel {
    oscillator: Arc<RwLock<Oscillator>>,
}

impl OscillatorViewModel {
    pub fn new() -> Self {
        Self {
            oscillator: Arc::new(RwLock::new(get_oscillator(&Note::new()))),
        }
    }

    pub fn get_oscillator(&self) -> Arc<RwLock<Oscillator>> {
        self.oscillator.clone()
    }

    pub fn set_waveform(&self, waveform: Waveform) {
        let mut oscillator = self
            .oscillator
            .write()
            .expect("Couldn't get oscillator lock");
        oscillator.waveform = waveform;
    }

    pub fn set_frequency(&self, frequency: f32) {
        let mut oscillator = self
            .oscillator
            .write()
            .expect("Couldn't get oscillator lock");
        oscillator.frequency_hz = frequency;
    }
}

pub(crate) fn get_oscillator(_note: &Note) -> Oscillator {
    Oscillator {
        waveform: Waveform::Silence,
        frequency_hz: 0.0,
    }
}
