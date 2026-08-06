use std::sync::Arc;

use crate::{
    id::InstrumentId,
    instruments::{
        HiHat, KickDrum, LoopRegion, MonophonicOscillator, MoogDFAM, PolyphonicOscillator,
        SnareDrum, Waveform,
    },
    InstrumentTrait, SampleData,
};

pub struct InstrumentFactory {
    sample_rate: f32,
}

impl InstrumentFactory {
    pub fn new(sample_rate: f32) -> Self {
        InstrumentFactory { sample_rate }
    }

    /// Returns the sample rate used by this factory.
    #[must_use]
    pub const fn sample_rate(&self) -> f32 {
        self.sample_rate
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

    pub fn create_dfam(&self, instrument_id: InstrumentId, pan: f32) -> Box<dyn InstrumentTrait> {
        Box::new(MoogDFAM::new(instrument_id, pan, self.sample_rate))
    }

    pub fn create_one_shot_sample_player(
        &self,
        instrument_id: InstrumentId,
        pan: f32,
        sample_data: Arc<SampleData>,
    ) -> Box<dyn InstrumentTrait> {
        Box::new(crate::instruments::SamplePlayer::new_one_shot(
            instrument_id,
            sample_data.clone(),
            self.sample_rate,
            pan,
        ))
    }

    pub fn create_loop_sample_player(
        &self,
        instrument_id: InstrumentId,
        pan: f32,
        sample_data: Arc<SampleData>,
    ) -> Box<dyn InstrumentTrait> {
        let start_frame = sample_data
            .loop_start
            .expect("This sample doesn't have loop data");
        let end_frame = sample_data
            .loop_end
            .expect("This sample doesn't have loop data");

        let loop_region = LoopRegion::new(start_frame as f64, end_frame as f64);
        Box::new(crate::instruments::SamplePlayer::new_with_loop(
            instrument_id,
            sample_data.clone(),
            self.sample_rate,
            pan,
            loop_region,
        ))
    }
}
