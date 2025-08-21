#![cfg(feature = "tracker")]

use crate::{id::InstrumentId, InstrumentTrait, MonophonicOscillator, PolyphonicOscillator};

pub struct InstrumentFactory {
    sample_rate: f32,
}

impl InstrumentFactory {
    pub fn new(sample_rate: f32) -> Self {
        InstrumentFactory { sample_rate }
    }

    pub fn create_simple_oscillator(
        &self,
        instrument_id: InstrumentId,
        pan: f32,
    ) -> Box<dyn InstrumentTrait> {
        Box::new(MonophonicOscillator::new(
            instrument_id,
            pan,
            self.sample_rate,
        ))
    }

    pub fn create_polyphonic_oscillator(
        &self,
        instrument_id: InstrumentId,
        pan: f32,
        max_polyphony: u8,
    ) -> Box<dyn InstrumentTrait> {
        Box::new(PolyphonicOscillator::new(
            instrument_id,
            pan,
            self.sample_rate,
            max_polyphony,
        ))
    }
}
