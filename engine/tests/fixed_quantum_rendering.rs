use std::sync::{
    atomic::{AtomicU32, AtomicUsize, Ordering},
    Arc,
};

use dsp::{id::EffectId, StereoEffect};
use engine::{
    AppliedTargetStatus, CoalescedParameterPublisher, CoalescedParameterStore,
    CoalescedTargetBinding, Engine, EngineEvent, EventProcessError, EventProducerId,
    InitialNormalizedValue, ParameterTableGenerations, ParameterTarget,
    PreparedCoalescedBindingTable, PreparedCoalescedParameterState, PreparedParameterBinding,
    PublicationResult, TimestampedEvent, COALESCED_CONTROL_QUANTUM_FRAMES,
};
use param_manifest::{
    builtin::master_gain_descriptor, AutomationRate, Mapping, ParameterDescriptor, ParameterId,
    ParameterLookup, ParameterManifest, RuntimeParamKey, SmoothingCurve, SmoothingPolicy,
    ValueRange,
};

const SAMPLE_RATE: f32 = 1_000.0;
const EFFECT_ID: EffectId = EffectId::from_raw(91);
const PRODUCER: EventProducerId = EventProducerId::new(1);
const MAX_RECORDED_SETTER_VALUES: usize = 64;

type DeferredPublication = (CoalescedParameterPublisher, RuntimeParamKey, f32);

struct SetterTrace {
    values: [AtomicU32; MAX_RECORDED_SETTER_VALUES],
}

impl SetterTrace {
    fn new() -> Self {
        Self {
            values: std::array::from_fn(|_| AtomicU32::new(0)),
        }
    }

    fn record(&self, index: usize, value: f32) {
        if let Some(slot) = self.values.get(index) {
            slot.store(value.to_bits(), Ordering::Relaxed);
        }
    }

    fn snapshot(&self, count: usize) -> Vec<f32> {
        assert!(count <= MAX_RECORDED_SETTER_VALUES);
        self.values[..count]
            .iter()
            .map(|value| f32::from_bits(value.load(Ordering::Relaxed)))
            .collect()
    }
}

struct ProbeGain {
    value: f32,
    setter_calls: Arc<AtomicUsize>,
    setter_values: Arc<SetterTrace>,
    publish_once: Option<DeferredPublication>,
}

impl StereoEffect for ProbeGain {
    fn id(&self) -> EffectId {
        EFFECT_ID
    }

    fn process(&mut self, left: &mut [f32], right: &mut [f32], _sample_rate: f32) {
        for (left, right) in left.iter_mut().zip(right) {
            *left *= self.value;
            *right *= self.value;
        }
    }

    fn set_parameter(&mut self, index: u32, value: f32) {
        if index != 0 {
            return;
        }
        self.value = value;
        let call_index = self.setter_calls.fetch_add(1, Ordering::Relaxed);
        self.setter_values.record(call_index, value);
        if let Some((publisher, key, normalized)) = self.publish_once.take() {
            assert!(matches!(
                publisher.publish(key, normalized),
                PublicationResult::Accepted(_)
            ));
        }
    }
}

struct Fixture {
    engine: Engine,
    publisher: CoalescedParameterPublisher,
    key: RuntimeParamKey,
    setter_calls: Arc<AtomicUsize>,
    setter_values: Arc<SetterTrace>,
}

fn descriptor(id: &str, smoothing: SmoothingPolicy) -> ParameterDescriptor {
    let mut descriptor = master_gain_descriptor();
    descriptor.id = ParameterId::from(id);
    descriptor.owner.path = format!("master/effect:{id}");
    descriptor.range = ValueRange {
        min: 0.0,
        max: 1.0,
        default: 0.0,
    };
    descriptor.mapping = Mapping::Linear { min: 0.0, max: 1.0 };
    descriptor.smoothing = smoothing;
    descriptor
}

fn fixture(smoothing: SmoothingPolicy, publish_once: Option<f32>) -> Fixture {
    let lookup = ParameterLookup::from_manifest(&ParameterManifest::new(vec![descriptor(
        "gain", smoothing,
    )]))
    .unwrap();
    let key = lookup.entries()[0].key();
    let mut generations = ParameterTableGenerations::new();
    let store = CoalescedParameterStore::prepare_with_initial_values(
        &mut generations,
        lookup.table(),
        1,
        &[InitialNormalizedValue {
            key,
            normalized: 0.0,
        }],
    )
    .unwrap();
    let bindings = PreparedCoalescedBindingTable::prepare(
        &store,
        lookup.table(),
        SAMPLE_RATE,
        &[CoalescedTargetBinding {
            key,
            target: ParameterTarget::MasterEffect {
                effect_id: EFFECT_ID,
            },
        }],
    )
    .unwrap();
    let state = PreparedCoalescedParameterState::new(lookup.into_table(), store, bindings).unwrap();
    let publisher = state.publisher();
    let setter_calls = Arc::new(AtomicUsize::new(0));
    let setter_values = Arc::new(SetterTrace::new());
    let mut engine = Engine::with_prepared_parameter_state(state);
    engine.add_master_effect(
        Box::new(ProbeGain {
            value: 99.0,
            setter_calls: setter_calls.clone(),
            setter_values: setter_values.clone(),
            publish_once: publish_once.map(|value| (publisher.clone(), key, value)),
        }),
        &mut engine::DropRetireSink,
    );
    Fixture {
        engine,
        publisher,
        key,
        setter_calls,
        setter_values,
    }
}

fn publish(fixture: &Fixture, normalized: f32) -> engine::AcceptedPublication {
    match fixture.publisher.publish(fixture.key, normalized) {
        PublicationResult::Accepted(accepted) => accepted,
        result => panic!("unexpected publication: {result:?}"),
    }
}

fn applied(fixture: &Fixture) -> engine::ParameterSnapshot {
    match fixture.publisher.applied(fixture.key) {
        AppliedTargetStatus::Applied(snapshot) => snapshot,
        status => panic!("unexpected applied status: {status:?}"),
    }
}

fn no_op_event(offset: usize, sequence: u64) -> TimestampedEvent {
    TimestampedEvent::new(offset, PRODUCER, sequence, EngineEvent::AllNotesOff)
}

fn sample_binding(value: f32, offset: usize) -> TimestampedEvent {
    let mut descriptor = descriptor("sample", SmoothingPolicy::None);
    descriptor.automation_rate = AutomationRate::SampleEvent;
    let lookup = ParameterLookup::from_manifest(&ParameterManifest::new(vec![descriptor])).unwrap();
    let parameter = *lookup.get(RuntimeParamKey(0)).unwrap();
    let binding = PreparedParameterBinding::new(
        parameter,
        ParameterTarget::MasterEffect {
            effect_id: EFFECT_ID,
        },
    )
    .unwrap();
    TimestampedEvent::new(
        offset,
        PRODUCER,
        0,
        EngineEvent::SampleParameter {
            binding,
            engine_value: value,
        },
    )
}

#[test]
fn process_latches_confirms_and_delivers_once_before_rendering() {
    let mut fixture = fixture(SmoothingPolicy::None, None);
    let accepted = publish(&fixture, 0.75);
    let mut left = [1.0; 4];
    let mut right = [1.0; 4];

    fixture.engine.process(&mut left, &mut right, SAMPLE_RATE);

    assert_eq!(left, [0.75; 4]);
    assert_eq!(right, left);
    assert_eq!(applied(&fixture).revision, accepted.revision);
    assert_eq!(fixture.engine.coalesced_parameter_phase(), Some(4));
    let work = fixture.engine.last_coalesced_render_work().unwrap();
    assert_eq!(
        (work.drain.applied, work.delivery_sweeps, work.setter_calls),
        (1, 1, 1)
    );
}

#[test]
fn initial_coalesced_delivery_precedes_an_offset_zero_parameter_event() {
    let mut fixture = fixture(SmoothingPolicy::None, None);
    publish(&fixture, 0.75);
    let mut left = [1.0];
    let mut right = [1.0];

    fixture
        .engine
        .process_with_events(
            &mut left,
            &mut right,
            SAMPLE_RATE,
            &[sample_binding(0.25, 0)],
        )
        .unwrap();

    assert_eq!(
        fixture
            .setter_values
            .snapshot(fixture.setter_calls.load(Ordering::Relaxed)),
        [0.75, 0.25]
    );
    assert_eq!(left, [0.25], "the offset-zero event wins before rendering");
    assert_eq!(right, left);
}

#[test]
fn events_validate_before_latch_and_leave_the_prior_success_observable() {
    let mut fixture = fixture(SmoothingPolicy::None, None);
    let mut warm_left = [1.0; 1];
    let mut warm_right = [1.0; 1];
    fixture
        .engine
        .process(&mut warm_left, &mut warm_right, SAMPLE_RATE);
    let prior_applied = applied(&fixture);
    let prior_phase = fixture.engine.coalesced_parameter_phase();
    let prior_work = fixture.engine.last_coalesced_render_work();
    let prior_setters = fixture.setter_calls.load(Ordering::Relaxed);
    let pending = publish(&fixture, 0.8);
    let mut left = [0.25; 4];
    let mut right = [0.5; 4];

    assert_eq!(
        fixture.engine.process_with_events(
            &mut left,
            &mut right,
            SAMPLE_RATE,
            &[no_op_event(4, 0)],
        ),
        Err(EventProcessError::OffsetOutOfRange)
    );
    assert_eq!(left, [0.25; 4]);
    assert_eq!(right, [0.5; 4]);
    assert_eq!(applied(&fixture), prior_applied);
    assert_ne!(applied(&fixture).revision, pending.revision);
    assert_eq!(fixture.engine.coalesced_parameter_phase(), prior_phase);
    assert_eq!(fixture.engine.last_coalesced_render_work(), prior_work);
    assert_eq!(fixture.setter_calls.load(Ordering::Relaxed), prior_setters);
}

#[test]
fn process_with_events_does_not_recursively_relatch_between_segments() {
    let mut fixture = fixture(SmoothingPolicy::None, Some(0.8));
    let mut left = [1.0; 8];
    let mut right = [1.0; 8];
    let events = [no_op_event(2, 0), no_op_event(5, 1)];

    fixture
        .engine
        .process_with_events(&mut left, &mut right, SAMPLE_RATE, &events)
        .unwrap();

    assert_eq!(applied(&fixture).normalized, 0.0);
    assert!(matches!(
        fixture.publisher.latest(fixture.key),
        engine::ParameterSnapshotStatus::Available(snapshot) if snapshot.normalized == 0.8
    ));
    assert_eq!(left, [0.0; 8]);
    assert_eq!(
        fixture
            .engine
            .last_coalesced_render_work()
            .unwrap()
            .drain
            .applied,
        1
    );

    let mut next_left = [1.0; 1];
    let mut next_right = [1.0; 1];
    fixture
        .engine
        .process(&mut next_left, &mut next_right, SAMPLE_RATE);
    assert_eq!(applied(&fixture).normalized, 0.8);
    assert_eq!(next_left, [0.8]);
}

#[test]
fn zero_frame_calls_confirm_without_delivery_or_phase_advance() {
    let mut fixture = fixture(SmoothingPolicy::None, None);
    let accepted = publish(&fixture, 0.6);
    let mut left = [];
    let mut right = [];

    fixture.engine.process(&mut left, &mut right, SAMPLE_RATE);

    assert_eq!(applied(&fixture).revision, accepted.revision);
    assert_eq!(fixture.setter_calls.load(Ordering::Relaxed), 0);
    assert_eq!(fixture.engine.coalesced_parameter_phase(), Some(0));
    assert_eq!(
        fixture
            .engine
            .last_coalesced_render_work()
            .unwrap()
            .delivery_sweeps,
        0
    );

    let mut next_left = [1.0];
    let mut next_right = [1.0];
    fixture
        .engine
        .process(&mut next_left, &mut next_right, SAMPLE_RATE);
    assert_eq!(next_left, [0.6]);
    assert_eq!(fixture.setter_calls.load(Ordering::Relaxed), 1);
}

fn render_partitioned(partitions: &[usize], event_offsets: &[usize]) -> (Vec<f32>, u8) {
    let mut fixture = fixture(
        SmoothingPolicy::Smoothed {
            duration_ms: 32.0,
            curve: SmoothingCurve::Linear,
        },
        None,
    );
    publish(&fixture, 1.0);
    let mut output = Vec::new();
    let mut global_cursor = 0;
    for &frames in partitions {
        let end = global_cursor + frames;
        let events: Vec<_> = event_offsets
            .iter()
            .enumerate()
            .filter(|(_, offset)| (global_cursor..end).contains(offset))
            .map(|(sequence, offset)| no_op_event(offset - global_cursor, sequence as u64))
            .collect();
        let mut left = vec![1.0; frames];
        let mut right = vec![1.0; frames];
        fixture
            .engine
            .process_with_events(&mut left, &mut right, SAMPLE_RATE, &events)
            .unwrap();
        output.extend(left);
        global_cursor = end;
    }
    (output, fixture.engine.coalesced_parameter_phase().unwrap())
}

#[test]
fn absolute_phase_and_trajectory_ignore_callback_and_event_partitions() {
    let event_offsets = [0, 7, 16, 31, 48];
    let whole = render_partitioned(&[65], &event_offsets);
    let partitioned = render_partitioned(&[5, 12, 1, 19, 28], &event_offsets);

    assert_eq!(partitioned, whole);
    assert_eq!(whole.1, 1);
}

#[test]
fn quantum_delivery_precedes_coincident_events_and_nonquantum_events_do_not_sweep() {
    let smoothing = SmoothingPolicy::Smoothed {
        duration_ms: 32.0,
        curve: SmoothingCurve::Linear,
    };
    let mut coincident = fixture(smoothing, None);
    publish(&coincident, 1.0);
    let mut left = [1.0; 17];
    let mut right = [1.0; 17];
    coincident
        .engine
        .process_with_events(
            &mut left,
            &mut right,
            SAMPLE_RATE,
            &[sample_binding(0.25, 16)],
        )
        .unwrap();
    assert_eq!(&left[..16], &[0.0; 16]);
    assert_eq!(
        left[16], 0.25,
        "the timestamped event wins after quantum delivery"
    );
    let work = coincident.engine.last_coalesced_render_work().unwrap();
    assert_eq!((work.delivery_sweeps, work.setter_calls), (2, 2));
    assert_eq!(coincident.setter_calls.load(Ordering::Relaxed), 3);

    let mut nonquantum = fixture(smoothing, None);
    publish(&nonquantum, 1.0);
    let mut left = [1.0; 8];
    let mut right = [1.0; 8];
    nonquantum
        .engine
        .process_with_events(
            &mut left,
            &mut right,
            SAMPLE_RATE,
            &[sample_binding(0.25, 7)],
        )
        .unwrap();
    let work = nonquantum.engine.last_coalesced_render_work().unwrap();
    assert_eq!((work.delivery_sweeps, work.setter_calls), (1, 1));
    assert_eq!(nonquantum.setter_calls.load(Ordering::Relaxed), 2);
}

#[test]
fn dense_nonquantum_events_do_not_add_smoother_advance_scans() {
    let mut fixture = fixture(
        SmoothingPolicy::Smoothed {
            duration_ms: 64.0,
            curve: SmoothingCurve::Linear,
        },
        None,
    );
    publish(&fixture, 1.0);
    let events: Vec<_> = (1..47)
        .filter(|offset| offset % COALESCED_CONTROL_QUANTUM_FRAMES != 0)
        .enumerate()
        .map(|(sequence, offset)| no_op_event(offset, sequence as u64))
        .collect();
    let mut left = [1.0; 47];
    let mut right = [1.0; 47];

    fixture
        .engine
        .process_with_events(&mut left, &mut right, SAMPLE_RATE, &events)
        .unwrap();

    let work = fixture.engine.last_coalesced_render_work().unwrap();
    assert_eq!(work.smoother_advance_segments, 3);
    assert_eq!(
        work.smoother_workset_words,
        3 * engine::COALESCED_DIRTY_WORD_COUNT
    );
    assert_eq!(work.smoother_advances, 3);
}

#[test]
fn a_smoother_settled_between_boundaries_delivers_exact_target_at_the_next_boundary() {
    let mut fixture = fixture(
        SmoothingPolicy::Smoothed {
            duration_ms: 20.0,
            curve: SmoothingCurve::Linear,
        },
        None,
    );
    publish(&fixture, 1.0);
    let mut left = [1.0; 33];
    let mut right = [1.0; 33];

    fixture.engine.process(&mut left, &mut right, SAMPLE_RATE);

    assert_eq!(&left[..16], &[0.0; 16]);
    assert!(left[16..32]
        .iter()
        .all(|value| (*value - 0.8).abs() < f32::EPSILON));
    assert_eq!(left[32], 1.0);
    assert_eq!(
        fixture
            .setter_values
            .snapshot(fixture.setter_calls.load(Ordering::Relaxed)),
        [0.0, 0.8, 1.0]
    );
}

#[test]
fn none_linear_and_exponential_trajectories_use_quantized_scalar_delivery() {
    let cases = [
        (SmoothingPolicy::None, 1.0),
        (
            SmoothingPolicy::Smoothed {
                duration_ms: 32.0,
                curve: SmoothingCurve::Linear,
            },
            0.5,
        ),
        (
            SmoothingPolicy::Smoothed {
                duration_ms: 32.0,
                curve: SmoothingCurve::Exponential,
            },
            1.0 - 10.0_f32.powf(-2.5),
        ),
    ];

    for (policy, middle) in cases {
        let mut fixture = fixture(policy, None);
        publish(&fixture, 1.0);
        let mut left = [1.0; 33];
        let mut right = [1.0; 33];
        fixture.engine.process(&mut left, &mut right, SAMPLE_RATE);

        if policy == SmoothingPolicy::None {
            assert_eq!(left, [1.0; 33]);
        } else {
            assert_eq!(&left[..16], &[0.0; 16]);
            assert!(left[16..32]
                .iter()
                .all(|value| (*value - middle).abs() < 1.0e-6));
            assert_eq!(left[32], 1.0);
        }
        assert_eq!(
            fixture
                .engine
                .last_coalesced_render_work()
                .unwrap()
                .setter_calls,
            if policy == SmoothingPolicy::None {
                1
            } else {
                3
            }
        );
    }
}

#[test]
fn exact_frame_count_boundary_has_no_extra_sweep() {
    let mut fixture = fixture(
        SmoothingPolicy::Smoothed {
            duration_ms: 32.0,
            curve: SmoothingCurve::Linear,
        },
        None,
    );
    publish(&fixture, 1.0);
    let mut left = [1.0; 32];
    let mut right = [1.0; 32];

    fixture.engine.process(&mut left, &mut right, SAMPLE_RATE);

    let work = fixture.engine.last_coalesced_render_work().unwrap();
    assert_eq!((work.delivery_sweeps, work.setter_calls), (2, 2));
    assert_eq!(fixture.engine.coalesced_parameter_phase(), Some(0));
}

#[test]
fn maximum_binding_set_and_sweep_work_remain_fixed_and_observable() {
    let descriptors: Vec<_> = (0..engine::MAX_COALESCED_PARAMETER_COUNT)
        .map(|index| {
            descriptor(
                &format!("gain.{index}"),
                SmoothingPolicy::Smoothed {
                    duration_ms: 32.0,
                    curve: SmoothingCurve::Linear,
                },
            )
        })
        .collect();
    let lookup = ParameterLookup::from_manifest(&ParameterManifest::new(descriptors)).unwrap();
    let keys: Vec<_> = lookup.entries().iter().map(|entry| entry.key()).collect();
    let mut generations = ParameterTableGenerations::new();
    let store = CoalescedParameterStore::prepare(
        &mut generations,
        lookup.table(),
        engine::MAX_COALESCED_PARAMETER_COUNT,
    )
    .unwrap();
    let targets: Vec<_> = keys
        .iter()
        .copied()
        .map(|key| CoalescedTargetBinding {
            key,
            target: ParameterTarget::MasterEffect {
                effect_id: EFFECT_ID,
            },
        })
        .collect();
    let bindings =
        PreparedCoalescedBindingTable::prepare(&store, lookup.table(), SAMPLE_RATE, &targets)
            .unwrap();
    let state = PreparedCoalescedParameterState::new(lookup.into_table(), store, bindings).unwrap();
    let publisher = state.publisher();
    for key in keys {
        assert!(matches!(
            publisher.publish(key, 1.0),
            PublicationResult::Accepted(_)
        ));
    }
    let setter_calls = Arc::new(AtomicUsize::new(0));
    let mut engine = Engine::with_prepared_parameter_state(state);
    engine.add_master_effect(
        Box::new(ProbeGain {
            value: 0.0,
            setter_calls: setter_calls.clone(),
            setter_values: Arc::new(SetterTrace::new()),
            publish_once: None,
        }),
        &mut engine::DropRetireSink,
    );
    let mut left = [1.0; 17];
    let mut right = [1.0; 17];

    engine.process(&mut left, &mut right, SAMPLE_RATE);

    let work = engine.last_coalesced_render_work().unwrap();
    assert_eq!(
        work.drain.applied as usize,
        engine::MAX_COALESCED_PARAMETER_COUNT
    );
    assert_eq!(work.delivery_sweeps, 2);
    assert_eq!(
        work.delivery_workset_words,
        2 * engine::COALESCED_DIRTY_WORD_COUNT
    );
    assert_eq!(work.setter_calls, 2 * engine::MAX_COALESCED_PARAMETER_COUNT);
    assert_eq!(setter_calls.load(Ordering::Relaxed), work.setter_calls);
    assert!(work.delivery_sweeps <= 1 + left.len().div_ceil(COALESCED_CONTROL_QUANTUM_FRAMES));
}
