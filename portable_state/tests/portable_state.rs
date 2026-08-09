use std::sync::Arc;

use node_registry::{EffectDefinition, InstrumentDefinition};
use param_manifest::ParameterId;
use portable_state::{
    decode_canonical, migrate_v0, AssetReference, AssetResolver, NodeAddress, NormalizedValue,
    ParameterValue, PortableStateV1, ResolvedAsset, StateError, TaggedPayload,
};
use serde_json::{json, Value};

fn instrument(id: u32, effects: Vec<EffectDefinition>) -> InstrumentDefinition {
    serde_json::from_value(json!({
        "schema_version": 2,
        "instance_id": id,
        "kind": "blight.instrument.oscillator.mono",
        "parameters": {},
        "effects": effects,
    }))
    .unwrap()
}

fn effect(id: u32) -> EffectDefinition {
    serde_json::from_value(json!({
        "schema_version": 1,
        "instance_id": id,
        "kind": "blight.effect.gain.mono",
        "parameters": {},
    }))
    .unwrap()
}

fn state() -> PortableStateV1 {
    let mut state = PortableStateV1::new(
        TaggedPayload::tracker_v1(json!({"name": "test", "tempo": 120})),
        TaggedPayload::fixed_routing_v1(),
    );
    state.instruments = vec![
        instrument(2, vec![effect(9), effect(8)]),
        instrument(1, vec![]),
    ];
    state.parameter_values = vec![
        ParameterValue {
            target: NodeAddress::Instrument { instrument_id: 2 },
            parameter_id: ParameterId::from("gain"),
            normalized_value: NormalizedValue::new(0.5).unwrap(),
        },
        ParameterValue {
            target: NodeAddress::Instrument { instrument_id: 1 },
            parameter_id: ParameterId::from("pan"),
            normalized_value: NormalizedValue::new(0.25).unwrap(),
        },
    ];
    state.assets = vec![
        AssetReference {
            asset_id: "sample-b".into(),
            sha256: "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad".into(),
            media_type: "audio/wav".into(),
        },
        AssetReference {
            asset_id: "sample-a".into(),
            sha256: "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad".into(),
            media_type: "audio/wav".into(),
        },
    ];
    state
}

#[test]
fn canonical_bytes_ignore_set_and_map_construction_order_and_roundtrip() {
    let first = state();
    let mut second = first.clone();
    second.instruments.reverse();
    second.parameter_values.reverse();
    second.assets.reverse();
    second.composition.payload = json!({"tempo": 120, "name": "test"});

    let bytes = first.canonical_bytes().unwrap();
    assert_eq!(bytes, second.canonical_bytes().unwrap());
    assert!(!bytes.ends_with(b"\n"));
    let decoded = decode_canonical(Arc::from(bytes.clone())).unwrap();
    assert_eq!(decoded.canonical_bytes().unwrap(), bytes);
    assert_eq!(
        decoded.instruments[1]
            .effects
            .iter()
            .map(|item| item.instance_id.raw())
            .collect::<Vec<_>>(),
        vec![9, 8],
        "effect identity and semantic order must be preserved"
    );
}

#[test]
fn unknown_future_composition_routing_and_node_sources_are_retained() {
    let cases = [
        {
            let mut value = state();
            value.composition = TaggedPayload {
                kind: "future.composition".into(),
                schema_version: 7,
                payload: json!({"keep": [1, 2, 3]}),
            };
            value
        },
        {
            let mut value = state();
            value.routing = TaggedPayload {
                kind: "future.routing".into(),
                schema_version: 4,
                payload: json!({"graph": "keep"}),
            };
            value
        },
        {
            let mut value = state();
            value.instruments[0] = serde_json::from_value(json!({
                "schema_version": 9, "instance_id": 2, "kind": "future.node",
                "parameters": {"opaque": true}, "effects": []
            }))
            .unwrap();
            value
        },
    ];

    for value in cases {
        let bytes = serde_json_canonicalizer::to_vec(&value).unwrap();
        let diagnostic = decode_canonical(Arc::from(bytes.clone())).unwrap_err();
        assert_eq!(diagnostic.source_bytes.as_ref(), bytes);
        match diagnostic.error {
            StateError::UnsupportedPayload { source, .. }
            | StateError::UnsupportedNode { source, .. } => {
                assert!(source.to_string().contains("future"));
            }
            other => panic!("unexpected diagnostic: {other:?}"),
        }
    }
}

#[test]
fn duplicate_and_invalid_model_values_are_structured() {
    let mut duplicate_node = state();
    duplicate_node.instruments.push(instrument(2, vec![]));
    assert!(matches!(
        duplicate_node.canonical_bytes(),
        Err(StateError::DuplicateNodeId { .. })
    ));

    let mut duplicate_parameter = state();
    duplicate_parameter
        .parameter_values
        .push(duplicate_parameter.parameter_values[0].clone());
    assert!(matches!(
        duplicate_parameter.canonical_bytes(),
        Err(StateError::DuplicateParameter { .. })
    ));

    let mut duplicate_asset = state();
    duplicate_asset
        .assets
        .push(duplicate_asset.assets[0].clone());
    assert!(matches!(
        duplicate_asset.canonical_bytes(),
        Err(StateError::DuplicateAssetId { .. })
    ));

    let mut invalid: Value = serde_json::to_value(state()).unwrap();
    invalid["parameter_values"][0]["normalized_value"] = json!(1.01);
    let invalid: PortableStateV1 = serde_json::from_value(invalid).unwrap();
    assert!(matches!(
        invalid.canonical_bytes(),
        Err(StateError::InvalidNormalized { .. })
    ));

    let mut bad_digest = state();
    bad_digest.assets[0].sha256 = "ABC".into();
    assert!(matches!(
        bad_digest.canonical_bytes(),
        Err(StateError::InvalidDigest { .. })
    ));

    let mut dangling = state();
    dangling.parameter_values[0].target = NodeAddress::MasterEffect { effect_id: 99 };
    assert!(matches!(
        dangling.canonical_bytes(),
        Err(StateError::InvalidNodeReference { .. })
    ));
}

struct Resolver(Option<ResolvedAsset>);

impl AssetResolver for Resolver {
    fn resolve(&self, _: &AssetReference) -> Option<ResolvedAsset> {
        self.0.clone()
    }
}

#[test]
fn caller_asset_resolver_reports_success_missing_and_mismatch() {
    let mut value = state();
    value.assets.truncate(1);
    let success = Resolver(Some(ResolvedAsset {
        bytes: b"abc".to_vec(),
        media_type: "audio/wav".into(),
    }));
    assert_eq!(value.validate_resolved_assets(&success), Ok(()));
    assert!(matches!(
        value.validate_resolved_assets(&Resolver(None)),
        Err(StateError::AssetMissing { .. })
    ));

    let mismatch = Resolver(Some(ResolvedAsset {
        bytes: b"wrong".to_vec(),
        media_type: "audio/wav".into(),
    }));
    assert!(matches!(
        value.validate_resolved_assets(&mismatch),
        Err(StateError::AssetDigestMismatch { .. })
    ));
    let wrong_media = Resolver(Some(ResolvedAsset {
        bytes: b"abc".to_vec(),
        media_type: "application/octet-stream".into(),
    }));
    assert!(matches!(
        value.validate_resolved_assets(&wrong_media),
        Err(StateError::AssetMediaTypeMismatch { .. })
    ));
}

#[test]
fn corrupt_nonfinite_unsafe_and_noncanonical_input_is_rejected() {
    let unsupported = Arc::from(br#"{"schema_version":2}"#.as_slice());
    assert!(matches!(
        decode_canonical(unsupported).unwrap_err().error,
        StateError::UnsupportedEnvelopeVersion { requested: 2, .. }
    ));

    let canonical = state().canonical_bytes().unwrap();
    let mut with_newline = canonical.clone();
    with_newline.push(b'\n');
    assert!(matches!(
        decode_canonical(Arc::from(with_newline)).unwrap_err().error,
        StateError::NonCanonical
    ));

    for malformed in [
        br#"{"schema_version":1,"schema_version":1}"#.as_slice(),
        br#"{"schema_version":1,"value":1e400}"#.as_slice(),
        b"{".as_slice(),
    ] {
        assert!(matches!(
            decode_canonical(Arc::from(malformed)).unwrap_err().error,
            StateError::Malformed { .. }
        ));
    }

    let unsafe_number = String::from_utf8(canonical)
        .unwrap()
        .replace("\"tempo\":120", "\"tempo\":9007199254740992");
    assert!(matches!(
        decode_canonical(Arc::from(unsafe_number.into_bytes()))
            .unwrap_err()
            .error,
        StateError::InvalidNumeric { .. }
    ));
}

#[test]
fn v0_fixture_migrates_to_exact_v1_canonical_fixture() {
    let source = include_bytes!("fixtures/portable-state-v0.json");
    let expected = include_bytes!("fixtures/portable-state-v1.jcs.json");
    let migrated = migrate_v0(Arc::from(source.as_slice())).unwrap();
    assert_eq!(migrated.canonical_bytes().unwrap(), expected);
    assert_eq!(migrate_v0(Arc::from(source.as_slice())).unwrap(), migrated);
}
