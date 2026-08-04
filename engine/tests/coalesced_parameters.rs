use std::{num::NonZeroU32, sync::Arc};

use engine::{
    ApplicationFailureCode, ApplicationFailureStatus, AppliedTargetStatus, CoalescedParameterStore,
    CoalescedStorePrepareError, DrainedPublication, InitialNormalizedValue,
    ParameterApplicationResult, ParameterSnapshotStatus, ParameterTableGenerations,
    PublicationRejection, PublicationResult, COALESCED_DIRTY_WORD_COUNT,
    MAX_COALESCED_PARAMETER_COUNT,
};
use param_manifest::{
    builtin::master_gain_descriptor, AutomationRate, ParameterId, ParameterLookup,
    ParameterManifest, RuntimeParamKey, SmoothingPolicy, Visibility,
};

fn lookup_with(descriptors: Vec<param_manifest::ParameterDescriptor>) -> ParameterLookup {
    ParameterLookup::from_manifest(&ParameterManifest::new(descriptors)).expect("valid fixture")
}

fn one_store() -> (
    ParameterLookup,
    CoalescedParameterStore,
    RuntimeParamKey,
    ParameterTableGenerations,
) {
    let lookup = lookup_with(vec![master_gain_descriptor()]);
    let key = lookup.entries()[0].key();
    let mut generations = ParameterTableGenerations::new();
    let store = CoalescedParameterStore::prepare(&mut generations, lookup.table(), 1).unwrap();
    (lookup, store, key, generations)
}

fn apply_all(store: &CoalescedParameterStore) -> Vec<DrainedPublication> {
    let mut publications = Vec::new();
    let summary = store.drain(|publication| {
        publications.push(publication);
        ParameterApplicationResult::Applied
    });
    assert_eq!(
        summary.scanned_dirty_words,
        COALESCED_DIRTY_WORD_COUNT as u8
    );
    publications
}

#[test]
fn canonical_publication_coalesces_and_confirms_one_coherent_revision_value() {
    let (_lookup, store, key, _generations) = one_store();
    let initial = apply_all(&store);
    assert_eq!(initial.len(), 1);
    assert_eq!(initial[0].revision.get(), 1);
    assert_eq!(initial[0].normalized, 1.0);

    let publisher = store.publisher();
    let first = match publisher.publish(key, -1.0) {
        PublicationResult::Accepted(accepted) => accepted,
        result => panic!("unexpected result: {result:?}"),
    };
    assert_eq!(first.canonical_normalized.to_bits(), 0.0_f32.to_bits());
    assert!(!first.replaced_pending);
    assert_eq!(first.revision.get(), 2);

    let second = match publisher.publish(key, 2.0) {
        PublicationResult::Accepted(accepted) => accepted,
        result => panic!("unexpected result: {result:?}"),
    };
    assert_eq!(second.canonical_normalized, 1.0);
    assert!(second.replaced_pending);
    assert_eq!(second.revision.get(), 3);
    assert_eq!(publisher.counters().coalesced_writes, 1);

    let drained = apply_all(&store);
    assert_eq!(drained.len(), 1, "one application per dirty key per block");
    assert_eq!(drained[0].revision, second.revision);
    assert_eq!(drained[0].normalized, second.canonical_normalized);
    assert_eq!(
        publisher.applied(key),
        AppliedTargetStatus::Applied(engine::ParameterSnapshot {
            generation: store.generation(),
            key,
            revision: second.revision,
            normalized: 1.0,
        })
    );
    assert!(apply_all(&store).is_empty());
}

#[test]
fn invalid_key_class_read_only_and_values_do_not_dirty_active_slots() {
    let mut sample = master_gain_descriptor();
    sample.id = ParameterId::from("sample");
    sample.owner.path = "master/effect:sample".into();
    sample.automation_rate = AutomationRate::SampleEvent;
    sample.smoothing = SmoothingPolicy::None;

    let mut structural = master_gain_descriptor();
    structural.id = ParameterId::from("structural");
    structural.owner.path = "master/effect:structural".into();
    structural.automation_rate = AutomationRate::Structural;
    structural.smoothing = SmoothingPolicy::None;

    let mut read_only = master_gain_descriptor();
    read_only.id = ParameterId::from("meter");
    read_only.owner.path = "master/meter".into();
    read_only.visibility = Visibility {
        host_visible: true,
        automatable: false,
        read_only: true,
    };

    let lookup = lookup_with(vec![
        sample,
        master_gain_descriptor(),
        structural,
        read_only,
    ]);
    let keys: Vec<_> = lookup.entries().iter().map(|entry| entry.key()).collect();
    let mut generations = ParameterTableGenerations::new();
    let store = CoalescedParameterStore::prepare(&mut generations, lookup.table(), 2).unwrap();
    apply_all(&store);
    let publisher = store.publisher();

    let cases = [
        (
            publisher.publish(RuntimeParamKey(u32::MAX), 0.5),
            PublicationRejection::InvalidKey,
        ),
        (
            publisher.publish(keys[0], 0.5),
            PublicationRejection::NotControlCoalesced,
        ),
        (
            publisher.publish(keys[2], 0.5),
            PublicationRejection::NotControlCoalesced,
        ),
        (
            publisher.publish(keys[3], 0.5),
            PublicationRejection::ReadOnly,
        ),
        (
            publisher.publish(keys[1], f32::NAN),
            PublicationRejection::InvalidValue,
        ),
        (
            publisher.publish(keys[1], f32::INFINITY),
            PublicationRejection::InvalidValue,
        ),
    ];
    for (result, expected) in cases {
        assert_eq!(result, PublicationResult::Rejected(expected));
    }
    assert_eq!(publisher.counters().invalid_writes, 6);
    assert!(apply_all(&store).is_empty());
}

#[test]
fn failed_application_is_observable_and_does_not_advance_confirmation() {
    let (_lookup, store, key, _generations) = one_store();
    let code = ApplicationFailureCode::new(NonZeroU32::new(7).unwrap());
    let summary = store.drain(|_| ParameterApplicationResult::Failed(code));
    assert_eq!(summary.failed, 1);
    assert_eq!(summary.applied, 0);
    assert!(matches!(
        store.applied(key),
        AppliedTargetStatus::Pending { .. }
    ));
    assert!(matches!(
        store.last_application_failure(key),
        ApplicationFailureStatus::Failed(failure)
            if failure.revision.get() == 1 && failure.code == code
    ));

    let accepted = match store.publisher().publish(key, 0.25) {
        PublicationResult::Accepted(accepted) => accepted,
        result => panic!("unexpected result: {result:?}"),
    };
    let summary = store.drain(|_| ParameterApplicationResult::Applied);
    assert_eq!(summary.applied, 1);
    assert!(matches!(
        store.applied(key),
        AppliedTargetStatus::Applied(snapshot)
            if snapshot.revision == accepted.revision && snapshot.normalized == 0.25
    ));
}

#[test]
fn reset_closes_old_handles_and_seeds_a_physically_separate_generation() {
    let (lookup, old, key, mut generations) = one_store();
    apply_all(&old);
    let old_publisher = old.publisher();
    let old_generation = old.generation();

    let replacement = old
        .prepare_reset(
            &mut generations,
            lookup.table(),
            1,
            &[InitialNormalizedValue {
                key,
                normalized: 0.25,
            }],
        )
        .unwrap();
    assert_ne!(replacement.generation(), old_generation);
    assert!(!old.is_accepting());
    assert_eq!(
        old_publisher.publish(key, 0.75),
        PublicationResult::Rejected(PublicationRejection::Closed)
    );
    assert_eq!(old_publisher.counters().stale_writes, 1);

    let seeded = apply_all(&replacement);
    assert_eq!(seeded.len(), 1);
    assert_eq!(seeded[0].revision.get(), 1);
    assert_eq!(seeded[0].normalized, 0.25);
    assert_eq!(seeded[0].generation, replacement.generation());
    assert!(apply_all(&old).is_empty());
}

#[test]
fn disconnection_is_distinct_and_never_dirties() {
    let (_lookup, store, key, _generations) = one_store();
    apply_all(&store);
    let publisher = store.publisher();
    store.disconnect();
    assert_eq!(
        publisher.publish(key, 0.5),
        PublicationResult::Rejected(PublicationRejection::Disconnected)
    );
    assert_eq!(publisher.counters().disconnected_writes, 1);
    assert!(apply_all(&store).is_empty());
}

#[test]
fn preparation_enforces_configured_and_hard_coalesced_caps() {
    let descriptors = (0..=MAX_COALESCED_PARAMETER_COUNT)
        .map(|index| {
            let mut descriptor = master_gain_descriptor();
            descriptor.id = ParameterId::from(format!("gain.{index}"));
            descriptor.owner.path = format!("master/effect:gain.{index}");
            descriptor
        })
        .collect();
    let lookup = lookup_with(descriptors);
    let mut generations = ParameterTableGenerations::new();
    let error = CoalescedParameterStore::prepare(
        &mut generations,
        lookup.table(),
        MAX_COALESCED_PARAMETER_COUNT,
    )
    .expect_err("1,025 entries exceed the hard cap");
    assert_eq!(
        error,
        CoalescedStorePrepareError::CoalescedCapacityExceeded {
            count: MAX_COALESCED_PARAMETER_COUNT + 1,
            limit: MAX_COALESCED_PARAMETER_COUNT,
        }
    );

    let empty = lookup_with(vec![]);
    let error = CoalescedParameterStore::prepare(
        &mut generations,
        empty.table(),
        MAX_COALESCED_PARAMETER_COUNT + 1,
    )
    .expect_err("configuration cannot raise the hard cap");
    assert!(matches!(
        error,
        CoalescedStorePrepareError::CoalescedLimitTooHigh { .. }
    ));

    let one = lookup_with(vec![master_gain_descriptor()]);
    let error = CoalescedParameterStore::prepare(&mut generations, one.table(), 0)
        .expect_err("a lower configured cap is enforced");
    assert_eq!(
        error,
        CoalescedStorePrepareError::CoalescedCapacityExceeded { count: 1, limit: 0 }
    );
}

#[test]
fn full_capacity_drain_covers_all_sixteen_dirty_words_and_last_slot() {
    let descriptors = (0..MAX_COALESCED_PARAMETER_COUNT)
        .map(|index| {
            let mut descriptor = master_gain_descriptor();
            descriptor.id = ParameterId::from(format!("gain.{index}"));
            descriptor.owner.path = format!("master/effect:gain.{index}");
            descriptor
        })
        .collect();
    let lookup = lookup_with(descriptors);
    let last_key = lookup.entries()[MAX_COALESCED_PARAMETER_COUNT - 1].key();
    let mut generations = ParameterTableGenerations::new();
    let store = CoalescedParameterStore::prepare(
        &mut generations,
        lookup.table(),
        MAX_COALESCED_PARAMETER_COUNT,
    )
    .unwrap();

    let mut saw_first = false;
    let mut saw_last = false;
    let summary = store.drain(|publication| {
        saw_first |= publication.key == RuntimeParamKey(0);
        saw_last |= publication.key == last_key;
        ParameterApplicationResult::Applied
    });
    assert_eq!(summary.scanned_dirty_words, 16);
    assert_eq!(summary.dirty_slots, 1_024);
    assert_eq!(summary.applied, 1_024);
    assert!(saw_first && saw_last);

    assert!(matches!(
        store.publisher().publish(last_key, 0.125),
        PublicationResult::Accepted(_)
    ));
    let mut redrained_key = None;
    let summary = store.drain(|publication| {
        redrained_key = Some(publication.key);
        ParameterApplicationResult::Applied
    });
    assert_eq!(summary.dirty_slots, 1);
    assert_eq!(redrained_key, Some(last_key));
}

#[test]
fn invalid_initial_snapshots_are_rejected_before_a_store_is_exposed() {
    let mut sample = master_gain_descriptor();
    sample.id = ParameterId::from("sample");
    sample.owner.path = "master/effect:sample".into();
    sample.automation_rate = AutomationRate::SampleEvent;
    sample.smoothing = SmoothingPolicy::None;
    let lookup = lookup_with(vec![master_gain_descriptor(), sample]);
    let coalesced_key = lookup.entries()[0].key();
    let sample_key = lookup.entries()[1].key();
    let mut generations = ParameterTableGenerations::new();

    for (initial_values, expected) in [
        (
            vec![InitialNormalizedValue {
                key: RuntimeParamKey(u32::MAX),
                normalized: 0.5,
            }],
            CoalescedStorePrepareError::InvalidInitialKey(RuntimeParamKey(u32::MAX)),
        ),
        (
            vec![InitialNormalizedValue {
                key: sample_key,
                normalized: 0.5,
            }],
            CoalescedStorePrepareError::InitialKeyNotCoalesced(sample_key),
        ),
        (
            vec![InitialNormalizedValue {
                key: coalesced_key,
                normalized: f32::NAN,
            }],
            CoalescedStorePrepareError::InvalidInitialValue(coalesced_key),
        ),
        (
            vec![
                InitialNormalizedValue {
                    key: coalesced_key,
                    normalized: 0.25,
                },
                InitialNormalizedValue {
                    key: coalesced_key,
                    normalized: 0.75,
                },
            ],
            CoalescedStorePrepareError::DuplicateInitialKey(coalesced_key),
        ),
    ] {
        assert_eq!(
            CoalescedParameterStore::prepare_with_initial_values(
                &mut generations,
                lookup.table(),
                1,
                &initial_values,
            )
            .unwrap_err(),
            expected
        );
    }
}

#[test]
fn every_drain_scans_the_fixed_words_even_for_an_empty_table() {
    let lookup = lookup_with(vec![]);
    let mut generations = ParameterTableGenerations::new();
    let store = CoalescedParameterStore::prepare(&mut generations, lookup.table(), 0).unwrap();
    let summary = store.drain(|_| unreachable!("empty store cannot apply"));
    assert_eq!(summary.scanned_dirty_words, 16);
    assert_eq!(summary.dirty_slots, 0);
}

#[test]
fn same_slot_mpsc_stress_applies_eventual_latest_after_quiescence() {
    let (_lookup, store, key, _generations) = one_store();
    apply_all(&store);
    let publisher = Arc::new(store.publisher());
    let mut workers = Vec::new();
    for producer in 0..4_u32 {
        let publisher = Arc::clone(&publisher);
        workers.push(std::thread::spawn(move || {
            let mut last = None;
            for sequence in 0..1_000_u32 {
                let value = ((producer * 1_000 + sequence) % 1_001) as f32 / 1_000.0;
                let PublicationResult::Accepted(accepted) = publisher.publish(key, value) else {
                    panic!("active valid publication must be accepted");
                };
                last = Some(accepted);
            }
            last.unwrap()
        }));
    }
    let expected = workers
        .into_iter()
        .map(|worker| worker.join().unwrap())
        .max_by_key(|publication| publication.revision)
        .unwrap();

    let drained = apply_all(&store);
    assert_eq!(drained.len(), 1);
    assert_eq!(drained[0].revision, expected.revision);
    assert_eq!(drained[0].normalized, expected.canonical_normalized);
    assert!(matches!(
        store.latest(key),
        ParameterSnapshotStatus::Available(snapshot)
            if snapshot.revision == expected.revision
                && snapshot.normalized == expected.canonical_normalized
    ));
    assert!(matches!(
        store.applied(key),
        AppliedTargetStatus::Applied(snapshot)
            if snapshot.revision == expected.revision
                && snapshot.normalized == expected.canonical_normalized
    ));
}

#[test]
fn closure_races_return_only_defined_statuses_and_cannot_touch_replacement() {
    for _ in 0..100 {
        let (lookup, store, key, mut generations) = one_store();
        apply_all(&store);
        let publisher = store.publisher();
        let publishing = std::thread::spawn(move || publisher.publish(key, 0.75));
        store.close();
        let result = publishing.join().unwrap();
        assert!(matches!(
            result,
            PublicationResult::Accepted(_)
                | PublicationResult::Rejected(PublicationRejection::Closed)
                | PublicationResult::Rejected(PublicationRejection::StaleGeneration)
        ));

        let replacement =
            CoalescedParameterStore::prepare(&mut generations, lookup.table(), 1).unwrap();
        let seeded = apply_all(&replacement);
        assert_eq!(seeded[0].normalized, 1.0);
        assert_eq!(seeded[0].revision.get(), 1);
    }
}
