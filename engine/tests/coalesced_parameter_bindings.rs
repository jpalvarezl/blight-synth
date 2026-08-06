use dsp::id::{EffectId, InstrumentId};
use engine::{
    AppliedTargetStatus, CoalescedApplicationFailure, CoalescedBindingPrepareError,
    CoalescedParameterStore, CoalescedTargetBinding, DrainedPublication, InitialNormalizedValue,
    ParameterApplicationResult, ParameterTableGenerations, ParameterTarget,
    PreparedCoalescedBindingTable, PublicationResult,
};
use param_manifest::{
    builtin::master_gain_descriptor, AutomationRate, Mapping, NodeType, ParameterDescriptor,
    ParameterId, ParameterLookup, ParameterManifest, RuntimeParamKey, SmoothingCurve,
    SmoothingPolicy, ValueRange, Visibility,
};

const SAMPLE_RATE: f32 = 1_000.0;

fn effect_id(raw: u32) -> EffectId {
    EffectId::from_raw(raw)
}

fn target(key: RuntimeParamKey, raw: u32) -> CoalescedTargetBinding {
    CoalescedTargetBinding {
        key,
        target: ParameterTarget::MasterEffect {
            effect_id: effect_id(raw),
        },
    }
}

fn linear_descriptor(id: &str, smoothing: SmoothingPolicy) -> ParameterDescriptor {
    let mut descriptor = master_gain_descriptor();
    descriptor.id = ParameterId::from(id);
    descriptor.owner.path = format!("master/effect:{id}");
    descriptor.range = ValueRange {
        min: -10.0,
        max: 10.0,
        default: 0.0,
    };
    descriptor.mapping = Mapping::Linear {
        min: -10.0,
        max: 10.0,
    };
    descriptor.smoothing = smoothing;
    descriptor
}

fn make_lookup(descriptors: Vec<ParameterDescriptor>) -> ParameterLookup {
    ParameterLookup::from_manifest(&ParameterManifest::new(descriptors)).expect("valid fixture")
}

fn accepted_revision(
    store: &CoalescedParameterStore,
    key: RuntimeParamKey,
    normalized: f32,
) -> engine::PublicationRevision {
    match store.publisher().publish(key, normalized) {
        PublicationResult::Accepted(accepted) => accepted.revision,
        result => panic!("unexpected publication result: {result:?}"),
    }
}

#[test]
fn direct_drain_maps_and_latches_none_linear_and_exponential_bindings() {
    let descriptors = vec![
        linear_descriptor("none", SmoothingPolicy::None),
        linear_descriptor(
            "linear",
            SmoothingPolicy::Smoothed {
                duration_ms: 10.0,
                curve: SmoothingCurve::Linear,
            },
        ),
        linear_descriptor(
            "exponential",
            SmoothingPolicy::Smoothed {
                duration_ms: 10.0,
                curve: SmoothingCurve::Exponential,
            },
        ),
    ];
    let lookup = make_lookup(descriptors);
    let keys: Vec<_> = lookup.entries().iter().map(|entry| entry.key()).collect();
    let seeds: Vec<_> = keys
        .iter()
        .copied()
        .map(|key| InitialNormalizedValue {
            key,
            normalized: 0.25,
        })
        .collect();
    let targets: Vec<_> = keys
        .iter()
        .copied()
        .enumerate()
        .map(|(index, key)| target(key, index as u32 + 1))
        .collect();
    let mut generations = ParameterTableGenerations::new();
    let store = CoalescedParameterStore::prepare_with_initial_values(
        &mut generations,
        lookup.table(),
        3,
        &seeds,
    )
    .unwrap();
    let mut bindings =
        PreparedCoalescedBindingTable::prepare(&store, lookup.table(), SAMPLE_RATE, &targets)
            .unwrap();

    for key in &keys {
        let smoother = bindings.binding(*key).unwrap().smoother();
        assert_eq!(smoother.current(), -5.0);
        assert_eq!(smoother.target(), -5.0);
        assert!(smoother.is_settled());
    }
    assert_eq!(bindings.drain(lookup.table(), &store).applied, 3);

    for key in &keys {
        accepted_revision(&store, *key, 0.75);
    }
    let summary = bindings.drain(lookup.table(), &store);
    assert_eq!(summary.dirty_slots, 3);
    assert_eq!(summary.applied, 3);
    assert_eq!(summary.failed, 0);

    let jump = bindings.binding(keys[0]).unwrap().smoother();
    assert_eq!(jump.current(), 5.0);
    assert_eq!(jump.target(), 5.0);
    assert!(jump.is_settled());
    for key in &keys[1..] {
        let smoother = bindings.binding(*key).unwrap().smoother();
        assert_eq!(smoother.current(), -5.0);
        assert_eq!(smoother.target(), 5.0);
        assert!(!smoother.is_settled());
    }
}

#[test]
fn exact_runtime_table_owns_mapping_and_normalized_bounds() {
    let lookup = make_lookup(vec![master_gain_descriptor()]);
    let key = lookup.entries()[0].key();
    let mut generations = ParameterTableGenerations::new();
    let store = CoalescedParameterStore::prepare(&mut generations, lookup.table(), 1).unwrap();
    let mut bindings =
        PreparedCoalescedBindingTable::prepare(&store, lookup.table(), 48_000.0, &[target(key, 1)])
            .unwrap();
    bindings.drain(lookup.table(), &store);

    accepted_revision(&store, key, 0.5);
    bindings.drain(lookup.table(), &store);
    let mapped = lookup.table().normalized_to_engine(key, 0.5).unwrap();
    assert!((mapped - -6.020_6).abs() < 0.001);
    assert_eq!(bindings.binding(key).unwrap().smoother().target(), mapped);

    accepted_revision(&store, key, -100.0);
    bindings.drain(lookup.table(), &store);
    assert_eq!(bindings.binding(key).unwrap().smoother().target(), -120.0);
    accepted_revision(&store, key, 100.0);
    bindings.drain(lookup.table(), &store);
    assert_eq!(bindings.binding(key).unwrap().smoother().target(), 0.0);
}

#[test]
fn preparation_rejects_wrong_table_keys_classes_writability_and_coverage() {
    let writable = linear_descriptor("writable", SmoothingPolicy::None);
    let mut sample = linear_descriptor("sample", SmoothingPolicy::None);
    sample.automation_rate = AutomationRate::SampleEvent;
    let mut read_only = linear_descriptor("meter", SmoothingPolicy::None);
    read_only.visibility = Visibility {
        host_visible: true,
        automatable: false,
        read_only: true,
    };
    let lookup = make_lookup(vec![writable, sample, read_only]);
    let keys: Vec<_> = lookup.entries().iter().map(|entry| entry.key()).collect();
    let mut generations = ParameterTableGenerations::new();
    let store = CoalescedParameterStore::prepare(&mut generations, lookup.table(), 2).unwrap();

    let equal_but_distinct =
        make_lookup(vec![linear_descriptor("writable", SmoothingPolicy::None)]);
    assert_eq!(
        PreparedCoalescedBindingTable::prepare(
            &store,
            equal_but_distinct.table(),
            SAMPLE_RATE,
            &[target(RuntimeParamKey(0), 1)],
        )
        .unwrap_err(),
        CoalescedBindingPrepareError::RuntimeTableMismatch
    );

    for (targets, expected) in [
        (
            vec![target(RuntimeParamKey(u32::MAX), 1)],
            CoalescedBindingPrepareError::InvalidKey(RuntimeParamKey(u32::MAX)),
        ),
        (
            vec![target(keys[1], 1)],
            CoalescedBindingPrepareError::NotControlCoalesced(keys[1]),
        ),
        (
            vec![target(keys[2], 1)],
            CoalescedBindingPrepareError::ReadOnly(keys[2]),
        ),
        (
            vec![target(keys[0], 1), target(keys[0], 2)],
            CoalescedBindingPrepareError::DuplicateBinding(keys[0]),
        ),
        (
            vec![],
            CoalescedBindingPrepareError::MissingWritableBinding(keys[0]),
        ),
    ] {
        assert_eq!(
            PreparedCoalescedBindingTable::prepare(&store, lookup.table(), SAMPLE_RATE, &targets,)
                .unwrap_err(),
            expected
        );
    }
}

#[test]
fn preparation_accepts_only_supported_matching_concrete_targets() {
    let mut instrument_effect = linear_descriptor("instrument-effect", SmoothingPolicy::None);
    instrument_effect.owner.node_type = NodeType::InstrumentEffect;
    let instrument_lookup = make_lookup(vec![instrument_effect]);
    let key = instrument_lookup.entries()[0].key();
    let mut generations = ParameterTableGenerations::new();
    let store =
        CoalescedParameterStore::prepare(&mut generations, instrument_lookup.table(), 1).unwrap();
    assert!(PreparedCoalescedBindingTable::prepare(
        &store,
        instrument_lookup.table(),
        SAMPLE_RATE,
        &[CoalescedTargetBinding {
            key,
            target: ParameterTarget::InstrumentEffect {
                instrument_id: InstrumentId::from_raw(7),
                effect_id: effect_id(8),
            },
        }],
    )
    .is_ok());
    assert_eq!(
        PreparedCoalescedBindingTable::prepare(
            &store,
            instrument_lookup.table(),
            SAMPLE_RATE,
            &[target(key, 1)],
        )
        .unwrap_err(),
        CoalescedBindingPrepareError::TargetClassMismatch {
            key,
            node_type: NodeType::InstrumentEffect,
        }
    );

    let mut unsupported = linear_descriptor("instrument", SmoothingPolicy::None);
    unsupported.owner.node_type = NodeType::Instrument;
    let unsupported_lookup = make_lookup(vec![unsupported]);
    let unsupported_key = unsupported_lookup.entries()[0].key();
    let unsupported_store =
        CoalescedParameterStore::prepare(&mut generations, unsupported_lookup.table(), 1).unwrap();
    assert_eq!(
        PreparedCoalescedBindingTable::prepare(
            &unsupported_store,
            unsupported_lookup.table(),
            SAMPLE_RATE,
            &[target(unsupported_key, 1)],
        )
        .unwrap_err(),
        CoalescedBindingPrepareError::UnsupportedTargetClass {
            key: unsupported_key,
            node_type: NodeType::Instrument,
        }
    );
}

#[test]
fn failure_preserves_prior_confirmation_and_records_exact_revision() {
    let lookup = make_lookup(vec![linear_descriptor("gain", SmoothingPolicy::None)]);
    let wrong_table = make_lookup(vec![linear_descriptor("gain", SmoothingPolicy::None)]);
    let key = lookup.entries()[0].key();
    let mut generations = ParameterTableGenerations::new();
    let store = CoalescedParameterStore::prepare(&mut generations, lookup.table(), 1).unwrap();
    let mut bindings = PreparedCoalescedBindingTable::prepare(
        &store,
        lookup.table(),
        SAMPLE_RATE,
        &[target(key, 1)],
    )
    .unwrap();
    bindings.drain(lookup.table(), &store);
    let prior = match store.applied(key) {
        AppliedTargetStatus::Applied(snapshot) => snapshot,
        status => panic!("unexpected status: {status:?}"),
    };

    let failed_revision = accepted_revision(&store, key, 0.75);
    let summary = bindings.drain(wrong_table.table(), &store);
    assert_eq!(summary.failed, 1);
    assert_eq!(summary.applied, 0);
    assert!(matches!(
        store.applied(key),
        AppliedTargetStatus::Applied(snapshot) if snapshot == prior
    ));
    assert!(matches!(
        store.last_application_failure(key),
        engine::ApplicationFailureStatus::Failed(failure)
            if failure.revision == failed_revision
                && failure.code == CoalescedApplicationFailure::RuntimeTableMismatch.code()
    ));
}

#[test]
fn direct_application_rejects_generation_key_class_and_read_only_injection() {
    let writable = linear_descriptor("writable", SmoothingPolicy::None);
    let mut sample = linear_descriptor("sample", SmoothingPolicy::None);
    sample.automation_rate = AutomationRate::SampleEvent;
    let mut read_only = linear_descriptor("meter", SmoothingPolicy::None);
    read_only.visibility.automatable = false;
    read_only.visibility.read_only = true;
    let lookup = make_lookup(vec![writable, sample, read_only]);
    let keys: Vec<_> = lookup.entries().iter().map(|entry| entry.key()).collect();
    let mut generations = ParameterTableGenerations::new();
    let store = CoalescedParameterStore::prepare(&mut generations, lookup.table(), 2).unwrap();
    let mut bindings = PreparedCoalescedBindingTable::prepare(
        &store,
        lookup.table(),
        SAMPLE_RATE,
        &[target(keys[0], 1)],
    )
    .unwrap();
    let mut publication = None;
    store.drain(|drained| {
        if drained.key == keys[0] {
            publication = Some(drained);
        }
        ParameterApplicationResult::Applied
    });
    let publication = publication.unwrap();

    for (injected, expected) in [
        (
            DrainedPublication {
                key: RuntimeParamKey(u32::MAX),
                ..publication
            },
            CoalescedApplicationFailure::InvalidKey,
        ),
        (
            DrainedPublication {
                key: keys[1],
                ..publication
            },
            CoalescedApplicationFailure::NotControlCoalesced,
        ),
        (
            DrainedPublication {
                key: keys[2],
                ..publication
            },
            CoalescedApplicationFailure::ReadOnly,
        ),
    ] {
        assert_eq!(
            bindings.apply(lookup.table(), injected),
            ParameterApplicationResult::Failed(expected.code())
        );
    }

    let other = CoalescedParameterStore::prepare(&mut generations, lookup.table(), 2).unwrap();
    let mut other_publication = None;
    other.drain(|drained| {
        if drained.key == keys[0] {
            other_publication = Some(drained);
        }
        ParameterApplicationResult::Applied
    });
    assert_eq!(
        bindings.apply(lookup.table(), other_publication.unwrap()),
        ParameterApplicationResult::Failed(CoalescedApplicationFailure::GenerationMismatch.code())
    );
}

#[test]
fn reset_builds_fresh_key_state_seeded_from_the_new_snapshot() {
    let first_lookup = make_lookup(vec![
        linear_descriptor("a", SmoothingPolicy::None),
        linear_descriptor("b", SmoothingPolicy::None),
    ]);
    let a_old = first_lookup.key_for(&ParameterId::from("a")).unwrap();
    let b_old = first_lookup.key_for(&ParameterId::from("b")).unwrap();
    let mut generations = ParameterTableGenerations::new();
    let old_store =
        CoalescedParameterStore::prepare(&mut generations, first_lookup.table(), 2).unwrap();
    let mut old_bindings = PreparedCoalescedBindingTable::prepare(
        &old_store,
        first_lookup.table(),
        SAMPLE_RATE,
        &[target(a_old, 1), target(b_old, 2)],
    )
    .unwrap();
    old_bindings.drain(first_lookup.table(), &old_store);
    accepted_revision(&old_store, a_old, 1.0);
    old_bindings.drain(first_lookup.table(), &old_store);
    assert_eq!(
        old_bindings.binding(a_old).unwrap().smoother().current(),
        10.0
    );

    let second_lookup = make_lookup(vec![
        linear_descriptor("b", SmoothingPolicy::None),
        linear_descriptor("a", SmoothingPolicy::None),
    ]);
    let a_new = second_lookup.key_for(&ParameterId::from("a")).unwrap();
    let b_new = second_lookup.key_for(&ParameterId::from("b")).unwrap();
    assert_eq!(a_old, b_new, "dense positions intentionally changed owner");
    let new_store = old_store
        .prepare_reset(
            &mut generations,
            second_lookup.table(),
            2,
            &[
                InitialNormalizedValue {
                    key: a_new,
                    normalized: 0.8,
                },
                InitialNormalizedValue {
                    key: b_new,
                    normalized: 0.2,
                },
            ],
        )
        .unwrap();
    let mut new_bindings = PreparedCoalescedBindingTable::prepare(
        &new_store,
        second_lookup.table(),
        SAMPLE_RATE,
        &[target(a_new, 1), target(b_new, 2)],
    )
    .unwrap();

    let a = new_bindings.binding(a_new).unwrap().smoother();
    let b = new_bindings.binding(b_new).unwrap().smoother();
    assert_eq!((a.current(), a.target(), a.is_settled()), (6.0, 6.0, true));
    assert_eq!(
        (b.current(), b.target(), b.is_settled()),
        (-6.0, -6.0, true)
    );
    let summary = new_bindings.drain(second_lookup.table(), &new_store);
    assert_eq!((summary.applied, summary.failed), (2, 0));
}
