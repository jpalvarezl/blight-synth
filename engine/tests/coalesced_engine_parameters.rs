use std::sync::{
    atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering},
    Arc,
};

use dsp::{
    id::{EffectId, InstrumentId, NoteEvent, NoteId},
    EffectInstallError, EffectInstallErrorKind, InstrumentTrait, MonoEffect, StereoEffect,
    SynthCmd,
};
use engine::{
    AppliedTargetStatus, CoalescedApplicationFailure, CoalescedParameterPublisher,
    CoalescedParameterStore, CoalescedTargetBinding, DropRetireSink, Engine, EngineEvent,
    EventProcessError, EventProducerId, InitialNormalizedValue, ParameterTableGenerations,
    ParameterTarget, PreparedCoalescedBindingTable, PreparedCoalescedParameterState,
    PreparedParameterBinding, PublicationResult, TimestampedEvent,
};
use param_manifest::{
    builtin::master_gain_descriptor, AutomationRate, Mapping, NodeType, ParameterDescriptor,
    ParameterId, ParameterLookup, ParameterManifest, RuntimeParamKey, SmoothingPolicy, ValueRange,
};

const SAMPLE_RATE: f32 = 48_000.0;
const MASTER_EFFECT: EffectId = EffectId::from_raw(90);
const INSTRUMENT: InstrumentId = InstrumentId::from_raw(12);
const INSTRUMENT_EFFECT: EffectId = EffectId::from_raw(13);

fn linear_descriptor(id: &str, rate: AutomationRate, node_type: NodeType) -> ParameterDescriptor {
    let mut descriptor = master_gain_descriptor();
    descriptor.id = ParameterId::from(id);
    descriptor.owner.path = format!("test/{id}");
    descriptor.owner.node_type = node_type;
    descriptor.automation_rate = rate;
    descriptor.smoothing = SmoothingPolicy::None;
    descriptor.range = ValueRange {
        min: -10.0,
        max: 10.0,
        default: 0.0,
    };
    descriptor.mapping = Mapping::Linear {
        min: -10.0,
        max: 10.0,
    };
    descriptor
}

fn master_state(
    initial: f32,
) -> (
    PreparedCoalescedParameterState,
    CoalescedParameterPublisher,
    RuntimeParamKey,
) {
    let lookup = ParameterLookup::from_manifest(&ParameterManifest::new(vec![linear_descriptor(
        "gain",
        AutomationRate::ControlCoalesced,
        NodeType::MasterEffect,
    )]))
    .unwrap();
    let key = RuntimeParamKey(0);
    let mut generations = ParameterTableGenerations::new();
    // Exercise the physically separate reset-generation seed path, not an
    // in-place dirty reset.
    let old = CoalescedParameterStore::prepare(&mut generations, lookup.table(), 1).unwrap();
    let store = old
        .prepare_reset(
            &mut generations,
            lookup.table(),
            1,
            &[InitialNormalizedValue {
                key,
                normalized: initial,
            }],
        )
        .unwrap();
    let bindings = PreparedCoalescedBindingTable::prepare(
        &store,
        lookup.table(),
        &[CoalescedTargetBinding {
            key,
            target: ParameterTarget::MasterEffect {
                effect_id: MASTER_EFFECT,
            },
        }],
    )
    .unwrap();
    let state = PreparedCoalescedParameterState::new(lookup.into_table(), store, bindings).unwrap();
    let publisher = state.publisher();
    (state, publisher, key)
}

struct ProbeEffect {
    value: Arc<AtomicU32>,
    setter_calls: Arc<AtomicUsize>,
}

impl StereoEffect for ProbeEffect {
    fn id(&self) -> EffectId {
        MASTER_EFFECT
    }

    fn process(&mut self, left: &mut [f32], right: &mut [f32], _sample_rate: f32) {
        let value = f32::from_bits(self.value.load(Ordering::Relaxed));
        for (left, right) in left.iter_mut().zip(right) {
            *left += value;
            *right += value;
        }
    }

    fn set_parameter(&mut self, index: u32, value: f32) {
        assert_eq!(index, 0);
        self.value.store(value.to_bits(), Ordering::Relaxed);
        self.setter_calls.fetch_add(1, Ordering::Relaxed);
    }
}

fn engine_with_probe(
    state: PreparedCoalescedParameterState,
) -> (Engine, Arc<AtomicU32>, Arc<AtomicUsize>) {
    let value = Arc::new(AtomicU32::new(f32::NAN.to_bits()));
    let calls = Arc::new(AtomicUsize::new(0));
    let mut engine = Engine::with_prepared_coalesced_parameters(state);
    engine.add_master_effect(
        Box::new(ProbeEffect {
            value: value.clone(),
            setter_calls: calls.clone(),
        }),
        &mut DropRetireSink,
    );
    (engine, value, calls)
}

#[test]
fn reset_seed_applies_and_confirms_on_a_valid_zero_frame_call() {
    let (state, publisher, key) = master_state(0.75);
    let (mut engine, value, calls) = engine_with_probe(state);

    engine
        .process_with_events(&mut [], &mut [], SAMPLE_RATE, &[])
        .unwrap();

    assert_eq!(calls.load(Ordering::Relaxed), 1);
    assert_eq!(f32::from_bits(value.load(Ordering::Relaxed)), 5.0);
    assert!(matches!(
        publisher.applied(key),
        AppliedTargetStatus::Applied(snapshot) if snapshot.normalized == 0.75
    ));
}

#[test]
fn valid_process_maps_and_applies_each_dirty_target_once() {
    let (state, publisher, key) = master_state(0.5);
    let (mut engine, value, calls) = engine_with_probe(state);
    engine.process(&mut [], &mut [], SAMPLE_RATE);
    assert!(matches!(
        publisher.publish(key, 0.25),
        PublicationResult::Accepted(_)
    ));
    let before = calls.load(Ordering::Relaxed);
    let mut left = [0.0; 4];
    let mut right = [0.0; 4];

    engine.process(&mut left, &mut right, SAMPLE_RATE);

    assert_eq!(calls.load(Ordering::Relaxed) - before, 1);
    assert_eq!(f32::from_bits(value.load(Ordering::Relaxed)), -5.0);
    assert_eq!(left, [-5.0; 4]);
    assert_eq!(right, left);
}

#[test]
fn events_validate_before_drain_and_offset_zero_runs_after_coalesced_application() {
    let coalesced = linear_descriptor(
        "coalesced",
        AutomationRate::ControlCoalesced,
        NodeType::MasterEffect,
    );
    let sample = linear_descriptor(
        "sample",
        AutomationRate::SampleEvent,
        NodeType::MasterEffect,
    );
    let lookup =
        ParameterLookup::from_manifest(&ParameterManifest::new(vec![coalesced, sample])).unwrap();
    let coalesced_key = RuntimeParamKey(0);
    let sample_parameter = *lookup.get(RuntimeParamKey(1)).unwrap();
    let sample_binding = PreparedParameterBinding::new(
        sample_parameter,
        ParameterTarget::MasterEffect {
            effect_id: MASTER_EFFECT,
        },
    )
    .unwrap();
    let mut generations = ParameterTableGenerations::new();
    let store = CoalescedParameterStore::prepare(&mut generations, lookup.table(), 1).unwrap();
    let bindings = PreparedCoalescedBindingTable::prepare(
        &store,
        lookup.table(),
        &[CoalescedTargetBinding {
            key: coalesced_key,
            target: ParameterTarget::MasterEffect {
                effect_id: MASTER_EFFECT,
            },
        }],
    )
    .unwrap();
    let state = PreparedCoalescedParameterState::new(lookup.into_table(), store, bindings).unwrap();
    let publisher = state.publisher();
    let (mut engine, value, calls) = engine_with_probe(state);
    engine.process(&mut [], &mut [], SAMPLE_RATE);
    let prior = publisher.applied(coalesced_key);
    assert!(matches!(
        publisher.publish(coalesced_key, 0.25),
        PublicationResult::Accepted(_)
    ));

    let invalid = [TimestampedEvent::new(
        4,
        EventProducerId::new(1),
        0,
        EngineEvent::SampleParameter {
            binding: sample_binding,
            engine_value: 9.0,
        },
    )];
    let mut left = [0.0; 4];
    let mut right = [0.0; 4];
    let before = calls.load(Ordering::Relaxed);
    assert_eq!(
        engine.process_with_events(&mut left, &mut right, SAMPLE_RATE, &invalid),
        Err(EventProcessError::OffsetOutOfRange)
    );
    assert_eq!(calls.load(Ordering::Relaxed), before);
    assert_eq!(publisher.applied(coalesced_key), prior);

    let offset_zero = [TimestampedEvent::new(
        0,
        EventProducerId::new(1),
        0,
        EngineEvent::SampleParameter {
            binding: sample_binding,
            engine_value: 7.0,
        },
    )];
    engine
        .process_with_events(&mut left, &mut right, SAMPLE_RATE, &offset_zero)
        .unwrap();
    assert_eq!(
        calls.load(Ordering::Relaxed) - before,
        2,
        "one coalesced setter plus one SampleEvent setter"
    );
    assert_eq!(f32::from_bits(value.load(Ordering::Relaxed)), 7.0);
    assert_eq!(left, [7.0; 4]);

    let after = calls.load(Ordering::Relaxed);
    engine
        .process_with_events(&mut left, &mut right, SAMPLE_RATE, &[])
        .unwrap();
    assert_eq!(
        calls.load(Ordering::Relaxed),
        after,
        "event segmentation must not recursively drain through process"
    );
}

struct InstrumentProbe {
    target_available: Arc<AtomicBool>,
    calls: Arc<AtomicUsize>,
}

impl InstrumentTrait for InstrumentProbe {
    fn id(&self) -> InstrumentId {
        INSTRUMENT
    }
    fn note_on(&mut self, _event: NoteEvent) {}
    fn note_off(&mut self, _note_id: NoteId) {}
    fn all_notes_off(&mut self) {}
    fn process(&mut self, _left: &mut [f32], _right: &mut [f32], _sample_rate: f32) {}
    fn set_pan(&mut self, _pan: f32) {}
    fn add_effect(&mut self, effect: Box<dyn MonoEffect>) -> Result<(), EffectInstallError> {
        Err(EffectInstallError::new(
            EffectInstallErrorKind::ChainFull,
            effect,
        ))
    }
    fn set_effect_parameter(&mut self, effect: EffectId, index: u32, _value: f32) {
        assert_eq!((effect, index), (INSTRUMENT_EFFECT, 0));
        self.calls.fetch_add(1, Ordering::Relaxed);
    }
    fn try_set_effect_parameter(&mut self, effect: EffectId, index: u32, value: f32) -> bool {
        if !self.target_available.load(Ordering::Relaxed) {
            return false;
        }
        self.set_effect_parameter(effect, index, value);
        true
    }
    fn try_handle_command(&mut self, _command: &SynthCmd) -> bool {
        false
    }
}

#[test]
fn missing_target_records_failure_and_preserves_prior_confirmation() {
    let lookup = ParameterLookup::from_manifest(&ParameterManifest::new(vec![linear_descriptor(
        "instrument",
        AutomationRate::ControlCoalesced,
        NodeType::InstrumentEffect,
    )]))
    .unwrap();
    let key = RuntimeParamKey(0);
    let mut generations = ParameterTableGenerations::new();
    let store = CoalescedParameterStore::prepare(&mut generations, lookup.table(), 1).unwrap();
    let bindings = PreparedCoalescedBindingTable::prepare(
        &store,
        lookup.table(),
        &[CoalescedTargetBinding {
            key,
            target: ParameterTarget::InstrumentEffect {
                instrument_id: INSTRUMENT,
                effect_id: INSTRUMENT_EFFECT,
            },
        }],
    )
    .unwrap();
    let state = PreparedCoalescedParameterState::new(lookup.into_table(), store, bindings).unwrap();
    let publisher = state.publisher();
    let target_available = Arc::new(AtomicBool::new(true));
    let calls = Arc::new(AtomicUsize::new(0));
    let mut engine = Engine::with_prepared_coalesced_parameters(state);
    engine.add_instrument(Box::new(InstrumentProbe {
        target_available: target_available.clone(),
        calls: calls.clone(),
    }));
    engine.process(&mut [], &mut [], SAMPLE_RATE);
    let prior = publisher.applied(key);
    assert_eq!(calls.load(Ordering::Relaxed), 1);

    target_available.store(false, Ordering::Relaxed);
    let failed_revision = match publisher.publish(key, 1.0) {
        PublicationResult::Accepted(accepted) => accepted.revision,
        other => panic!("unexpected publication result: {other:?}"),
    };
    engine.process(&mut [], &mut [], SAMPLE_RATE);

    assert_eq!(calls.load(Ordering::Relaxed), 1);
    assert_eq!(publisher.applied(key), prior);
    assert!(matches!(
        publisher.last_application_failure(key),
        engine::ApplicationFailureStatus::Failed(failure)
            if failure.revision == failed_revision
                && failure.code == CoalescedApplicationFailure::TargetUnavailable.code()
    ));
}
