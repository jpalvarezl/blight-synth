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
    MonoEffect(Box<dyn MonoEffect>),
    StereoEffect(Box<dyn StereoEffect>),
}

/// Receives heap-owning state displaced by engine operations.
///
/// RT hosts provide a bounded handoff sink; offline/NRT callers may use
/// [`DropRetireSink`] to destroy ownership immediately on their current thread.
pub trait RetireSink {
    fn retire(&mut self, state: RetiredState);
}

/// Immediate destruction policy for callers known to run outside RT.
pub struct DropRetireSink;

impl RetireSink for DropRetireSink {
    fn retire(&mut self, state: RetiredState) {
        drop(state);
    }
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

    /// Handles a command on a caller known to be outside RT, immediately
    /// destroying any displaced ownership on that caller.
    pub fn handle_command(&mut self, command: EngineCommand) {
        self.handle_command_with_retirement(command, &mut DropRetireSink);
    }

    /// Handles a command while routing displaced ownership to the supplied
    /// retirement policy. RT hosts must use this method.
    pub fn handle_command_with_retirement(
        &mut self,
        command: EngineCommand,
        retired: &mut impl RetireSink,
    ) {
        match command {
            EngineCommand::Instrument(command) => self.handle_instrument_command(command, retired),
            EngineCommand::Mixer(command) => self.handle_mixer_command(command, retired),
        }
    }

    fn handle_instrument_command(&mut self, command: InstrumentCmd, retired: &mut impl RetireSink) {
        match command {
            InstrumentCmd::AddInstrument { instrument } => {
                self.add_instrument_with_retirement(instrument, retired)
            }
            InstrumentCmd::AddEffect {
                instrument_id,
                effect,
            } => self.add_effect_to_instrument(instrument_id, effect, retired),
            InstrumentCmd::AddVoiceEffects {
                instrument_id,
                effects,
            } => self.add_voice_effects_to_instrument(instrument_id, effects, retired),
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
    }

    fn handle_mixer_command(&mut self, command: MixerCmd, retired: &mut impl RetireSink) {
        match command {
            MixerCmd::AddMasterEffect { effect } => self.add_master_effect(effect, retired),
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

    /// Adds an instrument on a caller known to be outside RT.
    pub fn add_instrument(&mut self, instrument: Box<dyn InstrumentTrait>) {
        self.add_instrument_with_retirement(instrument, &mut DropRetireSink);
    }

    /// Adds or replaces an instrument using the supplied retirement policy.
    pub fn add_instrument_with_retirement(
        &mut self,
        instrument: Box<dyn InstrumentTrait>,
        retired: &mut impl RetireSink,
    ) {
        let id = instrument.id();
        match self.instruments.binary_search_by_key(&id, |slot| slot.id) {
            Ok(index) => {
                let displaced =
                    std::mem::replace(&mut self.instruments[index].instrument, instrument);
                retired.retire(RetiredState::Instrument(displaced));
            }
            Err(index) => self
                .instruments
                .insert(index, InstrumentSlot { id, instrument }),
        }
    }

    pub fn clear_instruments(&mut self, retired: &mut impl RetireSink) {
        debug_assert!(
            self.instruments.len() <= DEFAULT_INSTRUMENT_CAPACITY,
            "#137 must enforce/update the hard instrument bound before retirement sizing changes"
        );
        self.stop_all_notes();
        for slot in self.instruments.drain(..) {
            retired.retire(RetiredState::Instrument(slot.instrument));
        }
    }

    pub fn add_effect_to_instrument(
        &mut self,
        instrument_id: InstrumentId,
        effect: Box<dyn MonoEffect>,
        retired: &mut impl RetireSink,
    ) {
        if let Some(instrument) = self.instrument_mut(instrument_id) {
            if let Err(error) = instrument.add_effect(effect) {
                // #136 will surface error.kind() through an NRT command result;
                // this slice owns only safe retirement of the rejected effect.
                retired.retire(RetiredState::MonoEffect(error.into_effect()));
            }
        } else {
            retired.retire(RetiredState::MonoEffect(effect));
        }
    }

    pub fn add_voice_effects_to_instrument(
        &mut self,
        instrument_id: InstrumentId,
        effects: VoiceEffects,
        retired: &mut impl RetireSink,
    ) {
        let rejected = if let Some(instrument) = self.instrument_mut(instrument_id) {
            instrument.add_voice_effects(effects)
        } else {
            effects
        };
        for effect in rejected {
            retired.retire(RetiredState::MonoEffect(effect));
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

    pub fn add_master_effect(
        &mut self,
        effect: Box<dyn StereoEffect>,
        retired: &mut impl RetireSink,
    ) {
        if let Err(effect) = self.master_effects.add_effect(effect) {
            retired.retire(RetiredState::StereoEffect(effect));
        }
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
    use dsp::{EffectFactory, EffectInstallError, EffectInstallErrorKind, InstrumentFactory};
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

        fn add_effect(&mut self, effect: Box<dyn MonoEffect>) -> Result<(), EffectInstallError> {
            Err(EffectInstallError::new(
                EffectInstallErrorKind::UnsupportedForPolyphonicInstrument,
                effect,
            ))
        }

        fn add_voice_effects(&mut self, effects: VoiceEffects) -> VoiceEffects {
            effects
        }

        fn set_effect_parameter(&mut self, _effect_id: EffectId, _param_index: u32, value: f32) {
            self.effect_value.store(value.to_bits(), Ordering::Relaxed);
        }

        fn try_handle_command(&mut self, _command: &SynthCmd) -> bool {
            false
        }
    }

    struct CollectRetired(Vec<RetiredState>);

    impl RetireSink for CollectRetired {
        fn retire(&mut self, state: RetiredState) {
            self.0.push(state);
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
        fn add_effect(&mut self, effect: Box<dyn MonoEffect>) -> Result<(), EffectInstallError> {
            Err(EffectInstallError::new(
                EffectInstallErrorKind::UnsupportedForPolyphonicInstrument,
                effect,
            ))
        }
        fn add_voice_effects(&mut self, effects: VoiceEffects) -> VoiceEffects {
            effects
        }
        fn set_effect_parameter(&mut self, _effect_id: EffectId, _param_index: u32, _value: f32) {}
        fn try_handle_command(&mut self, _command: &SynthCmd) -> bool {
            false
        }
    }

    struct DropMonoEffect {
        id: EffectId,
        drops: Arc<AtomicUsize>,
    }

    impl Drop for DropMonoEffect {
        fn drop(&mut self) {
            self.drops.fetch_add(1, Ordering::Relaxed);
        }
    }

    impl MonoEffect for DropMonoEffect {
        fn id(&self) -> EffectId {
            self.id
        }
        fn process(&mut self, _buf: &mut [f32], _sample_rate: f32) {}
        fn set_parameter(&mut self, _index: u32, _value: f32) {}
    }

    struct DropStereoEffect {
        id: EffectId,
        drops: Arc<AtomicUsize>,
    }

    impl Drop for DropStereoEffect {
        fn drop(&mut self) {
            self.drops.fetch_add(1, Ordering::Relaxed);
        }
    }

    impl StereoEffect for DropStereoEffect {
        fn id(&self) -> EffectId {
            self.id
        }
        fn process(&mut self, _left: &mut [f32], _right: &mut [f32], _sample_rate: f32) {}
        fn set_parameter(&mut self, _index: u32, _value: f32) {}
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
        engine.handle_command(
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
        engine.handle_command(
            MixerCmd::AddMasterEffect {
                effect: Box::new(ScaleEffect { id: 9, scale: 2.0 }),
            }
            .into(),
        );

        engine.handle_command(
            InstrumentCmd::NoteOn {
                instrument_id: 3,
                note: 60,
                velocity: 127,
            }
            .into(),
        );
        engine.handle_command(
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
        engine.handle_command(InstrumentCmd::NoteOff { instrument_id: 3 }.into());

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
            engine.add_instrument(Box::new(TestInstrument {
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
    fn duplicate_instrument_replacement_routes_ownership_for_nrt_drop() {
        let drop_thread = Arc::new(Mutex::new(None));
        let mut engine = Engine::new();
        let mut retired = CollectRetired(Vec::new());
        engine.add_instrument_with_retirement(
            Box::new(DropProbeInstrument {
                id: 7,
                drop_thread: drop_thread.clone(),
            }),
            &mut retired,
        );
        assert!(retired.0.is_empty());

        engine.add_instrument_with_retirement(
            Box::new(TestInstrument {
                id: 7,
                note_ons: Arc::new(AtomicUsize::new(0)),
                note_offs: Arc::new(AtomicUsize::new(0)),
                effect_value: Arc::new(AtomicU32::new(0)),
            }),
            &mut retired,
        );
        let retired = retired.0.pop().expect("duplicate id must retire ownership");
        assert!(drop_thread.lock().unwrap().is_none());

        let nrt_thread = std::thread::spawn(move || drop(retired));
        let expected_thread = nrt_thread.thread().id();
        nrt_thread.join().unwrap();
        assert_eq!(*drop_thread.lock().unwrap(), Some(expected_thread));
    }

    #[test]
    fn clear_and_effect_rejections_route_every_object_to_retirement() {
        let instrument_drops = Arc::new(Mutex::new(None));
        let mono_drops = Arc::new(AtomicUsize::new(0));
        let stereo_drops = Arc::new(AtomicUsize::new(0));
        let mut engine = Engine::new();
        let mut retired = CollectRetired(Vec::with_capacity(16));

        for id in [1, 2, 3] {
            engine.add_instrument_with_retirement(
                Box::new(DropProbeInstrument {
                    id,
                    drop_thread: instrument_drops.clone(),
                }),
                &mut retired,
            );
        }
        engine.clear_instruments(&mut retired);
        assert_eq!(retired.0.len(), 3);

        engine.add_effect_to_instrument(
            99,
            Box::new(DropMonoEffect {
                id: 1,
                drops: mono_drops.clone(),
            }),
            &mut retired,
        );
        let mut voice_effects = VoiceEffects::new();
        voice_effects.push(Box::new(DropMonoEffect {
            id: 2,
            drops: mono_drops.clone(),
        }));
        voice_effects.push(Box::new(DropMonoEffect {
            id: 3,
            drops: mono_drops.clone(),
        }));
        engine.add_voice_effects_to_instrument(99, voice_effects, &mut retired);

        for id in 0..=DEFAULT_MASTER_EFFECT_CAPACITY as EffectId {
            engine.add_master_effect(
                Box::new(DropStereoEffect {
                    id,
                    drops: stereo_drops.clone(),
                }),
                &mut retired,
            );
        }
        assert_eq!(mono_drops.load(Ordering::Relaxed), 0);
        assert_eq!(stereo_drops.load(Ordering::Relaxed), 0);

        drop(retired);
        assert_eq!(mono_drops.load(Ordering::Relaxed), 3);
        assert_eq!(stereo_drops.load(Ordering::Relaxed), 1);
        assert!(instrument_drops.lock().unwrap().is_some());
    }

    #[test]
    fn polyphonic_single_effect_rejection_reports_typed_reason_and_returns_effect() {
        let mut instrument =
            InstrumentFactory::new(48_000.0).create_polyphonic_oscillator(4, 0.0, 2);
        let effect = EffectFactory::new(48_000.0).create_mono_gain(9, 1.0);

        let error = instrument
            .add_effect(effect)
            .expect_err("polyphonic instruments require per-voice effects");

        assert_eq!(
            error.kind(),
            EffectInstallErrorKind::UnsupportedForPolyphonicInstrument
        );
        drop(error.into_effect());
    }

    #[test]
    fn renders_only_complete_frames_when_channel_lengths_differ() {
        let mut engine = Engine::new();
        engine.add_instrument(Box::new(TestInstrument {
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
