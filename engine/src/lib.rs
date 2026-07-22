mod commands;

pub use commands::*;
use dsp::{
    id::{EffectId, InstrumentId},
    InstrumentTrait, MonoEffect, StereoEffect, StereoEffectChain, SynthCmd, VoiceEffects,
};

const DEFAULT_INSTRUMENT_CAPACITY: usize = 64;
const DEFAULT_MASTER_EFFECT_CAPACITY: usize = 8;

struct InstrumentSlot {
    id: InstrumentId,
    instrument: Box<dyn InstrumentTrait>,
}

/// Heap-owning engine state displaced on RT and requiring NRT destruction.
#[non_exhaustive]
pub enum RetiredState {
    Instrument(Box<dyn InstrumentTrait>),
}

/// Host-independent runtime for instrument dispatch, mixing, and master effects.
///
/// `Engine` owns live sound-producing state but no audio device, composition
/// document, clock, file loader, network socket, or UI. Hosts provide planar
/// buffers and composition adapters decide which methods to call and when.
pub struct Engine {
    // Sorted, contiguous slots provide deterministic mix order and avoid
    // per-block hashing/tree traversal. A fixed hard capacity/stealing policy
    // remains part of #137; this vector is preallocated to the current limit.
    instruments: Vec<InstrumentSlot>,
    master_effects: StereoEffectChain,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    pub fn new() -> Self {
        Self {
            instruments: Vec::with_capacity(DEFAULT_INSTRUMENT_CAPACITY),
            master_effects: StereoEffectChain::new(DEFAULT_MASTER_EFFECT_CAPACITY),
        }
    }

    pub fn handle_command(&mut self, command: EngineCommand) -> Option<RetiredState> {
        match command {
            EngineCommand::Instrument(command) => self.handle_instrument_command(command),
            EngineCommand::Mixer(command) => {
                self.handle_mixer_command(command);
                None
            }
        }
    }

    fn handle_instrument_command(&mut self, command: InstrumentCmd) -> Option<RetiredState> {
        match command {
            InstrumentCmd::AddInstrument { instrument } => return self.add_instrument(instrument),
            InstrumentCmd::AddEffect {
                instrument_id,
                effect,
            } => self.add_effect_to_instrument(instrument_id, effect),
            InstrumentCmd::AddVoiceEffects {
                instrument_id,
                effects,
            } => self.add_voice_effects_to_instrument(instrument_id, effects),
            InstrumentCmd::NoteOn {
                instrument_id,
                note,
                velocity,
            } => self.note_on(instrument_id, note, velocity),
            InstrumentCmd::NoteOff { instrument_id } => self.note_off(instrument_id),
            InstrumentCmd::PassOnSynthCmd {
                instrument_id,
                synth_cmd,
            } => {
                self.try_handle_synth_command(instrument_id, &synth_cmd);
            }
            InstrumentCmd::SetEffectParameter {
                instrument_id,
                effect_id,
                param_index,
                value,
            } => self.set_instrument_effect_parameter(instrument_id, effect_id, param_index, value),
        }
        None
    }

    fn handle_mixer_command(&mut self, command: MixerCmd) {
        match command {
            MixerCmd::AddMasterEffect { effect } => self.add_master_effect(effect),
            MixerCmd::SetMasterEffectParameter {
                effect_id,
                param_index,
                value,
            } => self.set_master_effect_parameter(effect_id, param_index, value),
            // Effect-chain mutation semantics are intentionally deferred to #136.
            MixerCmd::RemoveMasterEffect { .. } | MixerCmd::ReorderMasterEffects { .. } => {}
        }
    }

    pub fn note_on(&mut self, instrument_id: InstrumentId, note: u8, velocity: u8) {
        if let Some(instrument) = self.instrument_mut(instrument_id) {
            instrument.note_on(note, velocity);
        }
    }

    pub fn note_off(&mut self, instrument_id: InstrumentId) {
        if let Some(instrument) = self.instrument_mut(instrument_id) {
            instrument.note_off();
        }
    }

    /// Adds every instrument output to the caller-provided planar buffers and
    /// then applies the master effect chain.
    ///
    /// If channel lengths differ, only complete stereo frames in their common
    /// prefix are rendered. The longer channel's tail is left untouched. Host
    /// adapters should still provide equal lengths, but malformed input must
    /// not panic in an audio callback.
    pub fn process(&mut self, left: &mut [f32], right: &mut [f32], sample_rate: f32) {
        let frame_count = left.len().min(right.len());
        let left = &mut left[..frame_count];
        let right = &mut right[..frame_count];

        for slot in &mut self.instruments {
            slot.instrument.process(left, right, sample_rate);
        }
        self.master_effects.process(left, right, sample_rate);
    }

    pub fn add_instrument(&mut self, instrument: Box<dyn InstrumentTrait>) -> Option<RetiredState> {
        let id = instrument.id();
        match self.instruments.binary_search_by_key(&id, |slot| slot.id) {
            Ok(index) => {
                let retired =
                    std::mem::replace(&mut self.instruments[index].instrument, instrument);
                Some(RetiredState::Instrument(retired))
            }
            Err(index) => {
                self.instruments
                    .insert(index, InstrumentSlot { id, instrument });
                None
            }
        }
    }

    pub fn clear_instruments(&mut self) {
        self.stop_all_notes();
        self.instruments.clear();
    }

    pub fn add_effect_to_instrument(
        &mut self,
        instrument_id: InstrumentId,
        effect: Box<dyn MonoEffect>,
    ) {
        if let Some(instrument) = self.instrument_mut(instrument_id) {
            instrument.add_effect(effect);
        }
    }

    pub fn add_voice_effects_to_instrument(
        &mut self,
        instrument_id: InstrumentId,
        effects: VoiceEffects,
    ) {
        if let Some(instrument) = self.instrument_mut(instrument_id) {
            instrument.add_voice_effects(effects);
        }
    }

    pub fn stop_all_notes(&mut self) {
        for slot in &mut self.instruments {
            slot.instrument.note_off();
        }
    }

    pub fn set_instrument_pan(&mut self, instrument_id: InstrumentId, pan: f32) {
        if let Some(instrument) = self.instrument_mut(instrument_id) {
            instrument.set_pan(pan);
        }
    }

    pub fn try_handle_synth_command(
        &mut self,
        instrument_id: InstrumentId,
        command: &SynthCmd,
    ) -> bool {
        self.instrument_mut(instrument_id)
            .is_some_and(|instrument| instrument.try_handle_command(command))
    }

    pub fn set_instrument_effect_parameter(
        &mut self,
        instrument_id: InstrumentId,
        effect_id: EffectId,
        param_index: u32,
        value: f32,
    ) {
        if let Some(instrument) = self.instrument_mut(instrument_id) {
            instrument.set_effect_parameter(effect_id, param_index, value);
        }
    }

    fn instrument_mut(
        &mut self,
        instrument_id: InstrumentId,
    ) -> Option<&mut (dyn InstrumentTrait + '_)> {
        let index = self
            .instruments
            .binary_search_by_key(&instrument_id, |slot| slot.id)
            .ok()?;
        Some(self.instruments[index].instrument.as_mut())
    }

    pub fn add_master_effect(&mut self, effect: Box<dyn StereoEffect>) {
        self.master_effects.add_effect(effect);
    }

    pub fn set_master_effect_parameter(
        &mut self,
        effect_id: EffectId,
        param_index: u32,
        value: f32,
    ) {
        self.master_effects
            .set_effect_parameter(effect_id, param_index, value);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{AtomicU32, AtomicUsize, Ordering},
        Arc, Mutex,
    };

    use super::*;

    struct TestInstrument {
        id: InstrumentId,
        note_ons: Arc<AtomicUsize>,
        note_offs: Arc<AtomicUsize>,
        effect_value: Arc<AtomicU32>,
    }

    impl InstrumentTrait for TestInstrument {
        fn id(&self) -> InstrumentId {
            self.id
        }

        fn note_on(&mut self, _note: u8, _velocity: u8) {
            self.note_ons.fetch_add(1, Ordering::Relaxed);
        }

        fn note_off(&mut self) {
            self.note_offs.fetch_add(1, Ordering::Relaxed);
        }

        fn process(&mut self, left: &mut [f32], right: &mut [f32], _sample_rate: f32) {
            for (left_sample, right_sample) in left.iter_mut().zip(right) {
                *left_sample += 0.25;
                *right_sample += 0.5;
            }
        }

        fn set_pan(&mut self, _pan: f32) {}

        fn add_effect(&mut self, _effect: Box<dyn MonoEffect>) {}

        fn add_voice_effects(&mut self, _effects: VoiceEffects) {}

        fn set_effect_parameter(&mut self, _effect_id: EffectId, _param_index: u32, value: f32) {
            self.effect_value.store(value.to_bits(), Ordering::Relaxed);
        }

        fn try_handle_command(&mut self, _command: &SynthCmd) -> bool {
            false
        }
    }

    struct DropProbeInstrument {
        id: InstrumentId,
        drop_thread: Arc<Mutex<Option<std::thread::ThreadId>>>,
    }

    impl Drop for DropProbeInstrument {
        fn drop(&mut self) {
            *self.drop_thread.lock().unwrap() = Some(std::thread::current().id());
        }
    }

    impl InstrumentTrait for DropProbeInstrument {
        fn id(&self) -> InstrumentId {
            self.id
        }
        fn note_on(&mut self, _note: u8, _velocity: u8) {}
        fn note_off(&mut self) {}
        fn process(&mut self, _left: &mut [f32], _right: &mut [f32], _sample_rate: f32) {}
        fn set_pan(&mut self, _pan: f32) {}
        fn add_effect(&mut self, _effect: Box<dyn MonoEffect>) {}
        fn add_voice_effects(&mut self, _effects: VoiceEffects) {}
        fn set_effect_parameter(&mut self, _effect_id: EffectId, _param_index: u32, _value: f32) {}
        fn try_handle_command(&mut self, _command: &SynthCmd) -> bool {
            false
        }
    }

    struct ScaleEffect {
        id: EffectId,
        scale: f32,
    }

    impl StereoEffect for ScaleEffect {
        fn id(&self) -> EffectId {
            self.id
        }

        fn process(&mut self, left: &mut [f32], right: &mut [f32], _sample_rate: f32) {
            for (left_sample, right_sample) in left.iter_mut().zip(right) {
                *left_sample *= self.scale;
                *right_sample *= self.scale;
            }
        }

        fn set_parameter(&mut self, _index: u32, value: f32) {
            self.scale = value;
        }
    }

    #[test]
    fn renders_instruments_and_master_effects_without_a_host() {
        let note_ons = Arc::new(AtomicUsize::new(0));
        let note_offs = Arc::new(AtomicUsize::new(0));
        let effect_value = Arc::new(AtomicU32::new(0));
        let mut engine = Engine::new();
        let _ = engine.handle_command(
            InstrumentCmd::AddInstrument {
                instrument: Box::new(TestInstrument {
                    id: 3,
                    note_ons: note_ons.clone(),
                    note_offs: note_offs.clone(),
                    effect_value: effect_value.clone(),
                }),
            }
            .into(),
        );
        let _ = engine.handle_command(
            MixerCmd::AddMasterEffect {
                effect: Box::new(ScaleEffect { id: 9, scale: 2.0 }),
            }
            .into(),
        );

        let _ = engine.handle_command(
            InstrumentCmd::NoteOn {
                instrument_id: 3,
                note: 60,
                velocity: 127,
            }
            .into(),
        );
        let _ = engine.handle_command(
            InstrumentCmd::SetEffectParameter {
                instrument_id: 3,
                effect_id: 4,
                param_index: 0,
                value: 0.75,
            }
            .into(),
        );
        let mut left = [0.0; 4];
        let mut right = [0.0; 4];
        engine.process(&mut left, &mut right, 48_000.0);
        let _ = engine.handle_command(InstrumentCmd::NoteOff { instrument_id: 3 }.into());

        assert_eq!(left, [0.5; 4]);
        assert_eq!(right, [1.0; 4]);
        assert_eq!(note_ons.load(Ordering::Relaxed), 1);
        assert_eq!(note_offs.load(Ordering::Relaxed), 1);
        assert_eq!(f32::from_bits(effect_value.load(Ordering::Relaxed)), 0.75);
    }

    #[test]
    fn instrument_slots_remain_sorted_and_replace_duplicate_ids() {
        let counters = || {
            (
                Arc::new(AtomicUsize::new(0)),
                Arc::new(AtomicUsize::new(0)),
                Arc::new(AtomicU32::new(0)),
            )
        };
        let mut engine = Engine::new();
        for id in [3, 1, 2, 2] {
            let (note_ons, note_offs, effect_value) = counters();
            let _ = engine.add_instrument(Box::new(TestInstrument {
                id,
                note_ons,
                note_offs,
                effect_value,
            }));
        }

        assert_eq!(
            engine
                .instruments
                .iter()
                .map(|slot| slot.id)
                .collect::<Vec<_>>(),
            [1, 2, 3]
        );
    }

    #[test]
    fn duplicate_instrument_replacement_returns_ownership_for_nrt_drop() {
        let drop_thread = Arc::new(Mutex::new(None));
        let mut engine = Engine::new();
        assert!(engine
            .add_instrument(Box::new(DropProbeInstrument {
                id: 7,
                drop_thread: drop_thread.clone(),
            }))
            .is_none());

        let retired = match engine.add_instrument(Box::new(TestInstrument {
            id: 7,
            note_ons: Arc::new(AtomicUsize::new(0)),
            note_offs: Arc::new(AtomicUsize::new(0)),
            effect_value: Arc::new(AtomicU32::new(0)),
        })) {
            Some(retired) => retired,
            None => panic!("duplicate id must return retired ownership"),
        };
        assert!(drop_thread.lock().unwrap().is_none());

        let nrt_thread = std::thread::spawn(move || drop(retired));
        let expected_thread = nrt_thread.thread().id();
        nrt_thread.join().unwrap();
        assert_eq!(*drop_thread.lock().unwrap(), Some(expected_thread));
    }

    #[test]
    fn renders_only_complete_frames_when_channel_lengths_differ() {
        let mut engine = Engine::new();
        let _ = engine.add_instrument(Box::new(TestInstrument {
            id: 3,
            note_ons: Arc::new(AtomicUsize::new(0)),
            note_offs: Arc::new(AtomicUsize::new(0)),
            effect_value: Arc::new(AtomicU32::new(0)),
        }));
        let mut left = [0.0; 3];
        let mut right = [0.0; 2];

        engine.process(&mut left, &mut right, 48_000.0);

        assert_eq!(left, [0.25, 0.25, 0.0]);
        assert_eq!(right, [0.5, 0.5]);
    }

    #[test]
    fn missing_instrument_ids_are_no_ops() {
        let mut engine = Engine::new();
        engine.note_on(99, 60, 127);
        engine.note_off(99);
        engine.set_instrument_pan(99, 0.5);
        engine.set_instrument_effect_parameter(99, 1, 0, 0.5);

        let mut left = [0.0; 2];
        let mut right = [0.0; 2];
        engine.process(&mut left, &mut right, 48_000.0);

        assert_eq!(left, [0.0; 2]);
        assert_eq!(right, [0.0; 2]);
    }
}
