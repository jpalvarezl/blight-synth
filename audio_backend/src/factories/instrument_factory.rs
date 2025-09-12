use crate::{
    id::InstrumentId, HiHat, InstrumentTrait, KickDrum, MonophonicOscillator, PolyphonicOscillator,
    SnareDrum, Waveform,
};

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

    pub fn create_oscillator_with_waveform(
        &self,
        instrument_id: InstrumentId,
        pan: f32,
        waveform: Waveform,
    ) -> Box<dyn InstrumentTrait> {
        Box::new(MonophonicOscillator::new_with_waveform(
            instrument_id,
            pan,
            self.sample_rate,
            waveform,
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

    pub fn create_hihat(&self, instrument_id: InstrumentId, pan: f32) -> Box<dyn InstrumentTrait> {
        Box::new(HiHat::new(instrument_id, pan, self.sample_rate))
    }

    pub fn create_kick_drum(
        &self,
        instrument_id: InstrumentId,
        pan: f32,
    ) -> Box<dyn InstrumentTrait> {
        Box::new(KickDrum::new(instrument_id, pan, self.sample_rate))
    }

    pub fn create_snare_drum(
        &self,
        instrument_id: InstrumentId,
        pan: f32,
    ) -> Box<dyn InstrumentTrait> {
        Box::new(SnareDrum::new(instrument_id, pan, self.sample_rate))
    }
}
