#![cfg(feature = "device-host")]

use audio_backend::{
    prepare_initial_parameter_generation, ApplicationFailureStatus, AppliedTargetStatus,
    PublicationRejection, StableParameterPublication, StableParameterQuery,
};
use param_manifest::{builtin::MASTER_GAIN_ID, ParameterId};

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
