use dsp::id::{EffectId, InstrumentId};
use engine::{
    CoalescedApplicationFailure, CoalescedBindingPrepareError, CoalescedParameterStore,
    CoalescedTargetBinding, DrainedPublication, ParameterApplicationResult,
    ParameterTableGenerations, ParameterTarget, PreparedCoalescedBindingTable,
    PreparedCoalescedParameterState, PreparedCoalescedParameterStateError, PublicationResult,
};
use param_manifest::{
    builtin::master_gain_descriptor, AutomationRate, Mapping, NodeType, ParameterDescriptor,
    ParameterId, ParameterLookup, ParameterManifest, RuntimeParamKey, SmoothingCurve,
    SmoothingPolicy, ValueRange, Visibility,
};

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

fn descriptor(id: &str, smoothing: SmoothingPolicy) -> ParameterDescriptor {
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
    ParameterLookup::from_manifest(&ParameterManifest::new(descriptors)).unwrap()
}

#[test]
fn none_binding_maps_exactly_and_invokes_target_once() {
    let lookup = make_lookup(vec![descriptor("gain", SmoothingPolicy::None)]);
    let key = RuntimeParamKey(0);
    let mut generations = ParameterTableGenerations::new();
    let store = CoalescedParameterStore::prepare(&mut generations, lookup.table(), 1).unwrap();
    let bindings =
        PreparedCoalescedBindingTable::prepare(&store, lookup.table(), &[target(key, 7)]).unwrap();
    let mut calls = Vec::new();

    let seed = bindings.drain(lookup.table(), &store, |binding, value| {
        calls.push((binding, value));
        true
    });
    assert_eq!((seed.applied, seed.failed), (1, 0));
    assert_eq!(calls[0].1, 0.0);

    assert!(matches!(
        store.publisher().publish(key, 0.75),
        PublicationResult::Accepted(_)
    ));
    let summary = bindings.drain(lookup.table(), &store, |binding, value| {
        calls.push((binding, value));
        true
    });
    assert_eq!(
        (summary.dirty_slots, summary.applied, summary.failed),
        (1, 1, 0)
    );
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[1].0.target(), target(key, 7).target);
    assert_eq!(calls[1].0.engine_param_index(), 0);
    assert_eq!(calls[1].1, 5.0);
}

#[test]
fn generic_preparation_rejects_smoothed_policy_compactly() {
    let lookup = make_lookup(vec![descriptor(
        "gain",
        SmoothingPolicy::Smoothed {
            duration_ms: 10.0,
            curve: SmoothingCurve::Linear,
        },
    )]);
    let key = RuntimeParamKey(0);
    let mut generations = ParameterTableGenerations::new();
    let store = CoalescedParameterStore::prepare(&mut generations, lookup.table(), 1).unwrap();

    assert_eq!(
        PreparedCoalescedBindingTable::prepare(&store, lookup.table(), &[target(key, 1)])
            .unwrap_err(),
        CoalescedBindingPrepareError::UnsupportedSmoothingPolicy(key)
    );
}

#[test]
fn preparation_keeps_exact_table_key_class_writability_and_coverage_validation() {
    let writable = descriptor("writable", SmoothingPolicy::None);
    let mut sample = descriptor("sample", SmoothingPolicy::None);
    sample.automation_rate = AutomationRate::SampleEvent;
    let mut read_only = descriptor("meter", SmoothingPolicy::None);
    read_only.visibility = Visibility {
        host_visible: true,
        automatable: false,
        read_only: true,
    };
    let lookup = make_lookup(vec![writable, sample, read_only]);
    let mut generations = ParameterTableGenerations::new();
    let store = CoalescedParameterStore::prepare(&mut generations, lookup.table(), 2).unwrap();
    let other = make_lookup(vec![descriptor("writable", SmoothingPolicy::None)]);

    assert_eq!(
        PreparedCoalescedBindingTable::prepare(
            &store,
            other.table(),
            &[target(RuntimeParamKey(0), 1)]
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
            vec![target(RuntimeParamKey(1), 1)],
            CoalescedBindingPrepareError::NotControlCoalesced(RuntimeParamKey(1)),
        ),
        (
            vec![target(RuntimeParamKey(2), 1)],
            CoalescedBindingPrepareError::ReadOnly(RuntimeParamKey(2)),
        ),
        (
            vec![target(RuntimeParamKey(0), 1), target(RuntimeParamKey(0), 2)],
            CoalescedBindingPrepareError::DuplicateBinding(RuntimeParamKey(0)),
        ),
        (
            vec![],
            CoalescedBindingPrepareError::MissingWritableBinding(RuntimeParamKey(0)),
        ),
    ] {
        assert_eq!(
            PreparedCoalescedBindingTable::prepare(&store, lookup.table(), &targets).unwrap_err(),
            expected
        );
    }
}

#[test]
fn preparation_validates_supported_node_and_target_classes() {
    let mut instrument = descriptor("instrument-effect", SmoothingPolicy::None);
    instrument.owner.node_type = NodeType::InstrumentEffect;
    let lookup = make_lookup(vec![instrument]);
    let key = RuntimeParamKey(0);
    let mut generations = ParameterTableGenerations::new();
    let store = CoalescedParameterStore::prepare(&mut generations, lookup.table(), 1).unwrap();
    let instrument_target = CoalescedTargetBinding {
        key,
        target: ParameterTarget::InstrumentEffect {
            instrument_id: InstrumentId::from_raw(3),
            effect_id: effect_id(4),
        },
    };
    assert!(
        PreparedCoalescedBindingTable::prepare(&store, lookup.table(), &[instrument_target])
            .is_ok()
    );
    assert_eq!(
        PreparedCoalescedBindingTable::prepare(&store, lookup.table(), &[target(key, 1)])
            .unwrap_err(),
        CoalescedBindingPrepareError::TargetClassMismatch {
            key,
            node_type: NodeType::InstrumentEffect
        }
    );

    let mut unsupported = descriptor("instrument", SmoothingPolicy::None);
    unsupported.owner.node_type = NodeType::Instrument;
    let lookup = make_lookup(vec![unsupported]);
    let store = CoalescedParameterStore::prepare(&mut generations, lookup.table(), 1).unwrap();
    assert_eq!(
        PreparedCoalescedBindingTable::prepare(&store, lookup.table(), &[target(key, 1)])
            .unwrap_err(),
        CoalescedBindingPrepareError::UnsupportedTargetClass {
            key,
            node_type: NodeType::Instrument
        }
    );
}

#[test]
fn prepared_state_rejects_table_identity_and_generation_mismatches_on_nrt() {
    let first = make_lookup(vec![descriptor("gain", SmoothingPolicy::None)]);
    let mut generations = ParameterTableGenerations::new();
    let first_store = CoalescedParameterStore::prepare(&mut generations, first.table(), 1).unwrap();
    let first_bindings = PreparedCoalescedBindingTable::prepare(
        &first_store,
        first.table(),
        &[target(RuntimeParamKey(0), 1)],
    )
    .unwrap();
    let other = make_lookup(vec![descriptor("gain", SmoothingPolicy::None)]);
    assert_eq!(
        PreparedCoalescedParameterState::new(other.into_table(), first_store, first_bindings)
            .unwrap_err(),
        PreparedCoalescedParameterStateError::RuntimeTableMismatch
    );

    let lookup = make_lookup(vec![descriptor("gain", SmoothingPolicy::None)]);
    let old_store = CoalescedParameterStore::prepare(&mut generations, lookup.table(), 1).unwrap();
    let old_bindings = PreparedCoalescedBindingTable::prepare(
        &old_store,
        lookup.table(),
        &[target(RuntimeParamKey(0), 1)],
    )
    .unwrap();
    let new_store = CoalescedParameterStore::prepare(&mut generations, lookup.table(), 1).unwrap();
    assert_eq!(
        PreparedCoalescedParameterState::new(lookup.into_table(), new_store, old_bindings)
            .unwrap_err(),
        PreparedCoalescedParameterStateError::GenerationMismatch
    );
}

#[test]
fn direct_application_preserves_compact_validation_failures() {
    let lookup = make_lookup(vec![descriptor("gain", SmoothingPolicy::None)]);
    let key = RuntimeParamKey(0);
    let mut generations = ParameterTableGenerations::new();
    let store = CoalescedParameterStore::prepare(&mut generations, lookup.table(), 1).unwrap();
    let bindings =
        PreparedCoalescedBindingTable::prepare(&store, lookup.table(), &[target(key, 1)]).unwrap();
    let mut publication = None;
    store.drain(|value| {
        publication = Some(value);
        ParameterApplicationResult::Applied
    });
    let publication = publication.unwrap();

    assert_eq!(
        bindings.apply(
            lookup.table(),
            DrainedPublication {
                key: RuntimeParamKey(u32::MAX),
                ..publication
            },
            |_, _| true
        ),
        ParameterApplicationResult::Failed(CoalescedApplicationFailure::InvalidKey.code())
    );
    assert_eq!(
        bindings.apply(lookup.table(), publication, |_, _| false),
        ParameterApplicationResult::Failed(CoalescedApplicationFailure::TargetUnavailable.code())
    );
}
