#![cfg(feature = "device-host")]

use std::sync::Arc;

use audio_backend::{
    id::InstrumentId, prepare_initial_parameter_generation, ApplicationFailureStatus,
    AppliedTargetStatus, AudioProcessor, Command, DesiredParameterValue, InstrumentCmd,
    InstrumentFactory, MeterState, PublicationRejection, StableParameterPublication,
    StableParameterQuery,
};
use engine::RetiredState;
use param_manifest::{builtin::MASTER_GAIN_ID, ParameterId};
use ringbuf::{
    storage::Heap,
    traits::{Consumer, Observer, Producer, Split},
    SharedRb,
};

#[test]
fn facade_resolves_stable_ids_and_exposes_initial_generation_state() {
    let (state, facade) = prepare_initial_parameter_generation()
        .expect("built-in parameter generation prepares")
        .into_parts();
    let gain = ParameterId::from(MASTER_GAIN_ID);
    let unknown = ParameterId::from("unknown.parameter");

    assert!(matches!(
        facade.latest(&gain),
        StableParameterQuery::Value(snapshot)
            if snapshot.generation == facade.generation()
                && snapshot.normalized == 1.0
    ));
    assert!(matches!(
        facade.applied(&gain),
        StableParameterQuery::Value(AppliedTargetStatus::Pending { generation, .. })
            if generation == facade.generation()
    ));
    assert_eq!(
        facade.last_application_failure(&gain),
        StableParameterQuery::Value(ApplicationFailureStatus::None)
    );

    assert_eq!(
        facade.publish(&unknown, 0.5),
        StableParameterPublication::UnknownParameter
    );
    assert_eq!(
        facade.latest(&unknown),
        StableParameterQuery::UnknownParameter
    );
    assert_eq!(
        facade.applied(&unknown),
        StableParameterQuery::UnknownParameter
    );
    assert_eq!(
        facade.last_application_failure(&unknown),
        StableParameterQuery::UnknownParameter
    );

    // Both owners belong to this NRT scope. No callback can observe or destroy
    // either owner before the initial state is installed by AudioProcessor.
    drop(state);
    drop(facade);
}

#[test]
fn invalid_closed_and_disconnected_publications_are_compact_and_counted() {
    let (state, facade) = prepare_initial_parameter_generation()
        .expect("built-in parameter generation prepares")
        .into_parts();
    let gain = ParameterId::from(MASTER_GAIN_ID);

    assert_eq!(
        facade.publish(&gain, f32::NAN),
        StableParameterPublication::Rejected(PublicationRejection::InvalidValue)
    );
    let accepted = match facade.publish(&gain, 0.25) {
        StableParameterPublication::Accepted(accepted) => accepted,
        other => panic!("unexpected publication result: {other:?}"),
    };
    assert!(matches!(
        facade.latest(&gain),
        StableParameterQuery::Value(snapshot)
            if snapshot.revision == accepted.revision && snapshot.normalized == 0.25
    ));

    facade.close();
    assert_eq!(
        facade.publish(&gain, 0.75),
        StableParameterPublication::Rejected(PublicationRejection::Closed)
    );
    let surviving_facade = facade.clone();
    facade.disconnect();
    drop(state);
    drop(facade);
    assert_eq!(
        surviving_facade.publish(&gain, 0.75),
        StableParameterPublication::Rejected(PublicationRejection::Disconnected)
    );
    assert_eq!(
        surviving_facade.latest(&gain),
        StableParameterQuery::Disconnected
    );
    assert_eq!(
        surviving_facade.applied(&gain),
        StableParameterQuery::Disconnected
    );
    assert_eq!(
        surviving_facade.last_application_failure(&gain),
        StableParameterQuery::Disconnected
    );

    let counters = surviving_facade.counters();
    assert_eq!(counters.invalid_writes, 1);
    assert_eq!(counters.stale_writes, 1);
    assert_eq!(counters.disconnected_writes, 1);
    assert_eq!(counters.coalesced_writes, 1);

    // This outliving clone performs the final prepared-owner release on NRT.
    drop(surviving_facade);
}

#[test]
fn replacement_rebinds_replays_and_confirms_at_the_block_boundary() {
    let (initial_state, mut lifecycle) = prepare_initial_parameter_generation()
        .unwrap()
        .into_lifecycle_parts();
    let old = lifecycle.facade();
    let gain = ParameterId::from(MASTER_GAIN_ID);
    let commands = SharedRb::<Heap<Command>>::new(4);
    let (mut command_tx, command_rx) = commands.split();
    let retirements = SharedRb::<Heap<RetiredState>>::new(4);
    let (retirement_tx, mut retirement_rx) = retirements.split();
    let mut processor = AudioProcessor::new(
        command_rx,
        retirement_tx,
        48_000.0,
        2,
        Arc::new(MeterState::new()),
        initial_state,
    );
    let instrument_id = InstrumentId::from_raw(1);
    command_tx
        .try_push(
            InstrumentCmd::AddInstrument {
                instrument: InstrumentFactory::new(48_000.0)
                    .create_simple_oscillator(instrument_id, 0.0),
            }
            .into(),
        )
        .unwrap_or_else(|_| panic!("instrument command fits"));
    command_tx
        .try_push(
            InstrumentCmd::NoteOn {
                instrument_id,
                note: 60,
                velocity: 127,
            }
            .into(),
        )
        .unwrap_or_else(|_| panic!("note command fits"));
    let mut output = [0.0; 64];
    processor.process(&mut output);
    let unity_peak = output.iter().copied().map(f32::abs).fold(0.0, f32::max);
    assert!(unity_peak > 0.0);
    let prior = old.applied(&gain);

    let prepared = lifecycle
        .prepare_builtin_replacement(&[
            DesiredParameterValue {
                id: gain.clone(),
                normalized: 0.25,
            },
            DesiredParameterValue {
                id: gain.clone(),
                normalized: 0.0,
            },
        ])
        .unwrap();
    assert_eq!(
        old.publish(&gain, 0.75),
        StableParameterPublication::Rejected(PublicationRejection::Closed)
    );
    let (state, new, transition) = prepared.into_parts();
    assert!(transition.current > transition.previous);
    assert_eq!(transition.rebound.as_slice(), std::slice::from_ref(&gain));
    assert!(transition.removed_or_missing.is_empty());
    assert!(matches!(
        new.applied(&gain),
        StableParameterQuery::Value(AppliedTargetStatus::Pending { generation, .. })
            if generation == transition.current
    ));
    command_tx
        .try_push(state)
        .unwrap_or_else(|_| panic!("replacement command fits"));
    // Enqueue/transition does not rewrite the prior generation's confirmation.
    assert_eq!(old.applied(&gain), prior);
    output.fill(0.0);
    processor.process(&mut output);
    assert!(matches!(
        new.applied(&gain),
        StableParameterQuery::Value(AppliedTargetStatus::Applied(snapshot))
            if snapshot.generation == transition.current && snapshot.normalized == 0.0
    ));
    let seeded_peak = output.iter().copied().map(f32::abs).fold(0.0, f32::max);
    assert!(
        seeded_peak < unity_peak * 2.0e-6,
        "replacement seed must apply before the block renders"
    );
    assert!(matches!(
        retirement_rx.try_pop(),
        Some(RetiredState::Prepared(_))
    ));
    assert!(matches!(
        new.latest(&gain),
        StableParameterQuery::Value(snapshot) if snapshot.normalized == 0.0
    ));
    old.disconnect();
    assert!(matches!(
        new.publish(&gain, 0.1),
        StableParameterPublication::Accepted(_)
    ));
}

#[test]
fn removed_desired_ids_are_reported_without_dense_key_reinterpretation() {
    let (initial_state, mut lifecycle) = prepare_initial_parameter_generation()
        .unwrap()
        .into_lifecycle_parts();
    let gain = ParameterId::from(MASTER_GAIN_ID);
    let missing = ParameterId::from("removed.parameter");
    let old = lifecycle.facade();
    assert!(lifecycle
        .prepare_builtin_replacement(&[DesiredParameterValue {
            id: gain.clone(),
            normalized: f32::NAN,
        }])
        .is_err());
    assert_eq!(
        old.publish(&gain, 0.3),
        StableParameterPublication::Rejected(PublicationRejection::Closed)
    );
    let prepared = lifecycle
        .prepare_builtin_replacement(&[DesiredParameterValue {
            id: missing.clone(),
            normalized: 0.4,
        }])
        .unwrap();
    let (_state, facade, transition) = prepared.into_parts();

    assert!(transition.rebound.is_empty());
    assert_eq!(
        transition.removed_or_missing.as_slice(),
        std::slice::from_ref(&missing)
    );
    assert_eq!(
        facade.publish(&missing, 0.9),
        StableParameterPublication::UnknownParameter
    );
    drop(initial_state);
}

#[test]
fn saturated_retirement_pauses_replacements_then_recovers_fairly() {
    let (initial_state, mut lifecycle) = prepare_initial_parameter_generation()
        .unwrap()
        .into_lifecycle_parts();
    let commands = SharedRb::<Heap<Command>>::new(4);
    let (mut command_tx, command_rx) = commands.split();
    let retirements = SharedRb::<Heap<RetiredState>>::new(1);
    let (retirement_tx, mut retirement_rx) = retirements.split();
    let mut processor = AudioProcessor::new(
        command_rx,
        retirement_tx,
        48_000.0,
        2,
        Arc::new(MeterState::new()),
        initial_state,
    );
    let gain = ParameterId::from(MASTER_GAIN_ID);
    let mut final_facade = lifecycle.facade();

    for normalized in [0.2, 0.4, 0.6] {
        let prepared = lifecycle
            .prepare_builtin_replacement(&[DesiredParameterValue {
                id: gain.clone(),
                normalized,
            }])
            .unwrap();
        let (state, facade, _transition) = prepared.into_parts();
        command_tx
            .try_push(state)
            .unwrap_or_else(|_| panic!("replacement command fits"));
        final_facade = facade;
    }

    processor.process(&mut [0.0; 16]);
    assert_eq!(command_tx.occupied_len(), 2);
    processor.process(&mut [0.0; 16]);
    assert_eq!(
        command_tx.occupied_len(),
        2,
        "full retirement handoff pauses replacement"
    );

    for expected_queued in [1, 0] {
        drop(
            retirement_rx
                .try_pop()
                .expect("displaced generation reached NRT"),
        );
        processor.process(&mut [0.0; 16]);
        assert_eq!(
            command_tx.occupied_len(),
            expected_queued,
            "one replacement advances per reclaimed retirement slot"
        );
    }
    assert!(matches!(
        final_facade.applied(&gain),
        StableParameterQuery::Value(AppliedTargetStatus::Applied(snapshot))
            if snapshot.normalized == 0.6
    ));
    while let Some(retired) = retirement_rx.try_pop() {
        drop(retired);
        processor.process(&mut [0.0; 16]);
    }
}

#[test]
fn shutdown_disconnects_old_and_new_outliving_facades_and_reclaims_states() {
    let (initial_state, mut lifecycle) = prepare_initial_parameter_generation()
        .unwrap()
        .into_lifecycle_parts();
    let old = lifecycle.facade();
    let gain = ParameterId::from(MASTER_GAIN_ID);
    let first = lifecycle.prepare_builtin_replacement(&[]).unwrap();
    let (first_state, first_facade, _first_transition) = first.into_parts();
    let second = lifecycle.prepare_builtin_replacement(&[]).unwrap();
    let (second_state, second_facade, _second_transition) = second.into_parts();

    let commands = SharedRb::<Heap<Command>>::new(2);
    let (mut command_tx, command_rx) = commands.split();
    command_tx
        .try_push(first_state)
        .unwrap_or_else(|_| panic!("first replacement fits"));
    command_tx
        .try_push(second_state)
        .unwrap_or_else(|_| panic!("second replacement fits"));
    let retirements = SharedRb::<Heap<RetiredState>>::new(2);
    let (retirement_tx, mut retirement_rx) = retirements.split();
    let mut processor = AudioProcessor::new(
        command_rx,
        retirement_tx,
        48_000.0,
        2,
        Arc::new(MeterState::new()),
        initial_state,
    );
    processor.process(&mut [0.0; 16]);

    lifecycle.disconnect();
    for facade in [&old, &first_facade, &second_facade] {
        assert_eq!(facade.latest(&gain), StableParameterQuery::Disconnected);
    }
    drop(processor);
    while let Some(retired) = retirement_rx.try_pop() {
        drop(retired);
    }
}
