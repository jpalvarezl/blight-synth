use engine::{Engine, EngineCommand, EventProcessError, RetireSink, TimestampedEvent};
use sequencer::models::{MAX_TRACKS, NO_INSTRUMENT};

use crate::id::InstrumentId;

fn no_instrument_id() -> InstrumentId {
    InstrumentId::from_raw(u32::from(NO_INSTRUMENT))
}

/// Tracker-specific adapter around the host-independent render engine.
///
/// Track-to-last-instrument state remains here because it belongs to tracker
/// event interpretation, not generic sound rendering. The indexed array makes
/// the `MAX_TRACKS` bound structural and keeps lookup/update work constant.
pub struct TrackerEngineAdapter {
    engine: Engine,
    track_last_instrument: [InstrumentId; MAX_TRACKS],
}

impl TrackerEngineAdapter {
    pub fn new() -> Self {
        Self::with_engine(Engine::new())
    }

    #[cfg(feature = "device-host")]
    pub fn with_prepared_coalesced_parameters(
        state: engine::PreparedCoalescedParameterState,
        sample_rate: f32,
    ) -> Self {
        let mut engine = Engine::with_prepared_coalesced_parameters(state);
        // Constructor value is only neutral prepared storage. The manifest's
        // authoritative normalized seed is mapped/set before the first render.
        let gain = dsp::EffectFactory::new(sample_rate)
            .create_stereo_gain(crate::device_host::MASTER_GAIN_EFFECT_ID, 1.0);
        engine.add_master_effect(gain, &mut engine::DropRetireSink);
        Self::with_engine(engine)
    }

    fn with_engine(engine: Engine) -> Self {
        Self {
            engine,
            track_last_instrument: [no_instrument_id(); MAX_TRACKS],
        }
    }

    /// Hard instrument-slot capacity of the wrapped engine. Non-RT accessor used
    /// to validate retirement-ring sizing invariants at construction time.
    #[cfg(feature = "device-host")]
    pub fn instrument_capacity(&self) -> usize {
        self.engine.instrument_capacity()
    }

    pub fn process_with_events(
        &mut self,
        left: &mut [f32],
        right: &mut [f32],
        sample_rate: f32,
        events: &[TimestampedEvent],
    ) -> Result<(), EventProcessError> {
        self.engine
            .process_with_events(left, right, sample_rate, events)
    }

    pub fn process(&mut self, left: &mut [f32], right: &mut [f32], sample_rate: f32) {
        self.engine.process(left, right, sample_rate);
    }

    pub fn clear_instruments(&mut self, retired: &mut impl RetireSink) {
        self.engine.clear_instruments(retired);
        self.track_last_instrument.fill(no_instrument_id());
    }

    pub fn handle_engine_command(&mut self, command: EngineCommand, retired: &mut impl RetireSink) {
        self.engine.handle_command_with_retirement(command, retired);
    }

    pub fn track_instruments(&self) -> [InstrumentId; MAX_TRACKS] {
        self.track_last_instrument
    }

    pub fn set_track_instruments(&mut self, state: [InstrumentId; MAX_TRACKS]) {
        self.track_last_instrument = state;
    }
}
