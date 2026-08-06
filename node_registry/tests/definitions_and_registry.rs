use std::{collections::BTreeMap, sync::Arc, thread};

use dsp::id::{EffectId, InstrumentId, SampleId};
use node_registry::{
    kind, BuiltInRegistry, EffectDefinition, EffectKindId, EffectLayout, InstrumentDefinition,
    InstrumentKindId, InvalidDefinitionCode, NodeCategory, NrtPreparationContext, ParameterPayload,
    PreparationError, SampleResolver, LEGACY_NODE_DEFINITION_SCHEMA_VERSION,
    NODE_DEFINITION_SCHEMA_VERSION,
};
use serde_json::{json, Value};

const V1_FIXTURE: &str = include_str!("fixtures/instrument-definition-v1.json");
const V2_FIXTURE: &str = include_str!("fixtures/instrument-definition-v2.json");

fn payload(value: Value) -> ParameterPayload {
    let Value::Object(object) = value else {
        panic!("test payload must be a JSON object");
    };
    object.into_iter().collect::<BTreeMap<_, _>>()
}

fn amplitude_envelope() -> Value {
    json!({
        "attack_seconds": 0.1,
        "decay_seconds": 0.1,
        "sustain_level": 0.8,
        "release_seconds": 0.5
    })
}

fn mono_gain(instance_id: u32, gain: f32) -> EffectDefinition {
    EffectDefinition::new(
        EffectId::from_raw(instance_id),
        kind::MONO_GAIN,
        payload(json!({ "gain": gain })),
    )
}

fn oscillator(effects: Vec<EffectDefinition>) -> InstrumentDefinition {
    InstrumentDefinition::new(
        InstrumentId::from_raw(7),
        kind::MONO_OSCILLATOR,
        payload(json!({
            "pan": 0.25,
            "waveform": "sawtooth",
            "amplitude_envelope": amplitude_envelope()
        })),
        effects,
    )
}

#[test]
fn v1_json_fixture_migrates_to_canonical_v2_deterministically() {
    let definition: InstrumentDefinition = serde_json::from_str(V1_FIXTURE).unwrap();

    assert_eq!(
        definition.schema_version,
        LEGACY_NODE_DEFINITION_SCHEMA_VERSION
    );
    assert_eq!(definition.instance_id, InstrumentId::from_raw(7));
    assert_eq!(definition.kind.as_str(), kind::MONO_OSCILLATOR);
    assert_eq!(definition.effects.len(), 2);
    assert_eq!(definition.effects[0].instance_id, EffectId::from_raw(41));
    assert_eq!(definition.effects[1].instance_id, EffectId::from_raw(42));
    assert_eq!(definition.effects[0].kind, definition.effects[1].kind);

    let migrated = BuiltInRegistry::new()
        .migrate_instrument_definition(&definition)
        .unwrap();
    assert_eq!(migrated.schema_version, NODE_DEFINITION_SCHEMA_VERSION);
    assert_eq!(migrated.effects, definition.effects);
    let encoded = format!("{}\n", serde_json::to_string_pretty(&migrated).unwrap());
    assert_eq!(encoded, V2_FIXTURE);
    let decoded_again: InstrumentDefinition = serde_json::from_str(&encoded).unwrap();
    assert_eq!(decoded_again, migrated);
    assert_eq!(
        BuiltInRegistry::new()
            .migrate_instrument_definition(&migrated)
            .unwrap(),
        migrated,
        "migration must be idempotent at the supported version"
    );
}

#[test]
fn unknown_definition_data_is_retained_for_diagnostics_and_migration() {
    let unknown_json = json!({
        "schema_version": 77,
        "instance_id": 99,
        "kind": "vendor.instrument.future",
        "parameters": {
            "nested": { "z": 1, "a": true },
            "opaque": [3, 2, 1]
        },
        "effects": []
    });
    let definition: InstrumentDefinition = serde_json::from_value(unknown_json.clone()).unwrap();

    assert_eq!(
        definition.kind,
        InstrumentKindId::from("vendor.instrument.future")
    );
    assert_eq!(definition.schema_version, 77);
    assert_eq!(serde_json::to_value(&definition).unwrap(), unknown_json);

    let before = serde_json::to_value(&definition).unwrap();
    let error = BuiltInRegistry::new()
        .migrate_instrument_definition(&definition)
        .unwrap_err();
    assert!(matches!(
        error,
        PreparationError::UnknownKind {
            category: NodeCategory::Instrument,
            instance_id: 99,
            ..
        }
    ));
    assert_eq!(serde_json::to_value(definition).unwrap(), before);
}

#[test]
fn migration_preserves_unrecognized_payload_data_for_preparation_diagnostics() {
    let mut definition: InstrumentDefinition = serde_json::from_str(V1_FIXTURE).unwrap();
    definition.parameters.insert(
        "vendor_extension".to_owned(),
        json!({ "opaque": [3, 2, 1] }),
    );

    let migrated = BuiltInRegistry::new()
        .migrate_instrument_definition(&definition)
        .unwrap();
    assert_eq!(
        migrated.parameters["vendor_extension"],
        definition.parameters["vendor_extension"]
    );
    let error = BuiltInRegistry::new()
        .prepare_instrument(&migrated, &NrtPreparationContext::new(48_000.0))
        .err()
        .unwrap();
    assert!(matches!(
        error,
        PreparationError::InvalidDefinition { diagnostic, .. }
            if diagnostic.code == InvalidDefinitionCode::InvalidParameterPayload
    ));
}

#[test]
fn same_kind_effects_keep_order_and_independent_typed_identity() {
    let definition: InstrumentDefinition = serde_json::from_str(V2_FIXTURE).unwrap();
    let prepared = BuiltInRegistry::new()
        .prepare_definition(&definition, &NrtPreparationContext::new(48_000.0))
        .unwrap();

    assert_eq!(prepared.instrument.id(), InstrumentId::from_raw(7));
    let ids: Vec<_> = prepared.effects.iter().map(|effect| effect.id()).collect();
    assert_eq!(ids, [EffectId::from_raw(41), EffectId::from_raw(42)]);
    assert!(prepared
        .effects
        .iter()
        .all(|effect| effect.layout() == EffectLayout::Mono));
}

#[test]
fn duplicate_effect_instance_id_is_invalid_even_when_kinds_differ() {
    let duplicate = EffectId::from_raw(41);
    let effects = vec![
        mono_gain(duplicate.raw(), 0.5),
        EffectDefinition::new(
            duplicate,
            kind::MONO_MOOG_LADDER,
            payload(json!({ "cutoff": 500.0, "resonance": 0.5 })),
        ),
    ];

    let error = match BuiltInRegistry::new()
        .prepare_effect_chain(&effects, &NrtPreparationContext::new(48_000.0))
    {
        Ok(_) => panic!("duplicate IDs must fail"),
        Err(error) => error,
    };
    assert!(matches!(
        error,
        PreparationError::InvalidDefinition {
            category: NodeCategory::Effect,
            instance_id: 41,
            diagnostic,
            ..
        } if diagnostic.code == InvalidDefinitionCode::DuplicateInstanceId
            && diagnostic.field.as_deref() == Some("instance_id")
    ));
}

#[test]
fn unknown_instrument_and_effect_kinds_are_structured() {
    let instrument = InstrumentDefinition::new(
        InstrumentId::from_raw(12),
        "vendor.instrument.missing",
        ParameterPayload::new(),
        Vec::new(),
    );
    let instrument_error = BuiltInRegistry::new()
        .prepare_instrument(&instrument, &NrtPreparationContext::new(48_000.0))
        .err()
        .unwrap();
    assert_eq!(
        instrument_error,
        PreparationError::UnknownKind {
            category: NodeCategory::Instrument,
            kind: "vendor.instrument.missing".to_owned(),
            instance_id: 12,
        }
    );

    let effect = EffectDefinition::new(
        EffectId::from_raw(13),
        "vendor.effect.missing",
        ParameterPayload::new(),
    );
    let effect_error = BuiltInRegistry::new()
        .prepare_effect(&effect, &NrtPreparationContext::new(48_000.0))
        .err()
        .unwrap();
    assert_eq!(
        effect_error,
        PreparationError::UnknownKind {
            category: NodeCategory::Effect,
            kind: "vendor.effect.missing".to_owned(),
            instance_id: 13,
        }
    );
}

#[test]
fn unsupported_instrument_and_effect_versions_report_supported_versions() {
    let mut instrument = oscillator(Vec::new());
    instrument.schema_version = 3;
    let error = BuiltInRegistry::new()
        .prepare_instrument(&instrument, &NrtPreparationContext::new(48_000.0))
        .err()
        .unwrap();
    assert!(matches!(
        error,
        PreparationError::UnsupportedSchemaVersion {
            category: NodeCategory::Instrument,
            requested: 3,
            supported: &[2],
            ..
        }
    ));

    let mut effect = mono_gain(1, 1.0);
    effect.schema_version = 0;
    let error = BuiltInRegistry::new()
        .prepare_effect(&effect, &NrtPreparationContext::new(48_000.0))
        .err()
        .unwrap();
    assert!(matches!(
        error,
        PreparationError::UnsupportedSchemaVersion {
            category: NodeCategory::Effect,
            requested: 0,
            supported: &[1],
            ..
        }
    ));
}

#[test]
fn malformed_and_out_of_range_payloads_have_machine_readable_diagnostics() {
    let malformed = EffectDefinition::new(
        EffectId::from_raw(5),
        kind::MONO_DELAY,
        payload(json!({ "mix": 0.5 })),
    );
    let error = BuiltInRegistry::new()
        .prepare_effect(&malformed, &NrtPreparationContext::new(48_000.0))
        .err()
        .unwrap();
    assert!(matches!(
        error,
        PreparationError::InvalidDefinition { diagnostic, .. }
            if diagnostic.code == InvalidDefinitionCode::InvalidParameterPayload
    ));

    let invalid_pan = InstrumentDefinition::new(
        InstrumentId::from_raw(6),
        kind::HI_HAT,
        payload(json!({
            "pan": 2.0,
            "amplitude_envelope": amplitude_envelope()
        })),
        Vec::new(),
    );
    let error = BuiltInRegistry::new()
        .prepare_instrument(&invalid_pan, &NrtPreparationContext::new(48_000.0))
        .err()
        .unwrap();
    assert!(matches!(
        error,
        PreparationError::InvalidDefinition { diagnostic, .. }
            if diagnostic.code == InvalidDefinitionCode::ParameterOutOfRange
                && diagnostic.field.as_deref() == Some("pan")
    ));
}

#[test]
fn built_in_inventory_is_static_complete_and_distinguishes_effect_layout() {
    let instruments: Vec<_> = BuiltInRegistry::instrument_kinds().collect();
    assert_eq!(
        instruments
            .iter()
            .map(|descriptor| descriptor.kind)
            .collect::<Vec<_>>(),
        [
            kind::MONO_OSCILLATOR,
            kind::POLYPHONIC_OSCILLATOR,
            kind::HI_HAT,
            kind::KICK_DRUM,
            kind::SNARE_DRUM,
            kind::MOOG_DFAM,
            kind::ONE_SHOT_SAMPLE_PLAYER,
            kind::LOOP_SAMPLE_PLAYER,
        ]
    );
    assert!(instruments.iter().all(|descriptor| {
        descriptor.category == NodeCategory::Instrument
            && descriptor.effect_layout.is_none()
            && descriptor.supported_schema_versions == [2]
    }));

    let effects: Vec<_> = BuiltInRegistry::effect_kinds().collect();
    assert_eq!(
        effects
            .iter()
            .map(|descriptor| descriptor.kind)
            .collect::<Vec<_>>(),
        [
            kind::MONO_REVERB,
            kind::STEREO_REVERB,
            kind::MONO_DELAY,
            kind::MONO_DISTORTION,
            kind::MONO_FILTER,
            kind::MONO_GAIN,
            kind::STEREO_GAIN,
            kind::MONO_MOOG_LADDER,
        ]
    );
    assert_eq!(
        effects
            .iter()
            .filter(|descriptor| descriptor.effect_layout == Some(EffectLayout::Stereo))
            .map(|descriptor| descriptor.kind)
            .collect::<Vec<_>>(),
        [kind::STEREO_REVERB, kind::STEREO_GAIN]
    );
    assert_eq!(
        effects
            .iter()
            .filter(|descriptor| descriptor.deprecated)
            .map(|descriptor| descriptor.kind)
            .collect::<Vec<_>>(),
        [kind::MONO_DISTORTION, kind::MONO_FILTER]
    );
}

struct OneSample;

impl SampleResolver for OneSample {
    fn resolve_sample(&self, id: SampleId) -> Option<Arc<dsp::SampleData>> {
        (id == SampleId::from_raw(23)).then(|| {
            Arc::new(dsp::SampleData {
                data: vec![0.0; 16],
                sample_rate: 48_000.0,
                channels: 1,
                loop_start: Some(2),
                loop_end: Some(8),
            })
        })
    }
}

#[test]
fn sample_resolution_and_owner_construction_run_on_an_explicit_nrt_thread() {
    let prepared = thread::Builder::new()
        .name("node-preparation-nrt".to_owned())
        .spawn(|| {
            assert_eq!(
                thread::current().name(),
                Some("node-preparation-nrt"),
                "the allocation/factory phase is explicitly outside an audio callback"
            );
            let resolver = OneSample;
            let context = NrtPreparationContext::new(48_000.0).with_sample_resolver(&resolver);
            let definition = InstrumentDefinition::new(
                InstrumentId::from_raw(31),
                kind::LOOP_SAMPLE_PLAYER,
                payload(json!({
                    "pan": 0.0,
                    "sample_id": 23,
                    "amplitude_envelope": amplitude_envelope()
                })),
                vec![mono_gain(51, 1.0)],
            );
            BuiltInRegistry::new()
                .prepare_definition(&definition, &context)
                .unwrap()
        })
        .unwrap()
        .join()
        .unwrap();

    // Only already-prepared owners cross the NRT thread boundary. No definition,
    // registry, JSON payload, or resource resolver is needed to use their IDs.
    assert_eq!(prepared.instrument.id(), InstrumentId::from_raw(31));
    assert_eq!(prepared.effects[0].id(), EffectId::from_raw(51));
}

struct FixedSample(Arc<dsp::SampleData>);

impl SampleResolver for FixedSample {
    fn resolve_sample(&self, id: SampleId) -> Option<Arc<dsp::SampleData>> {
        (id == SampleId::from_raw(23)).then(|| self.0.clone())
    }
}

#[test]
fn sample_resources_require_supported_channels_and_aligned_data() {
    for (channels, sample_count) in [(0, 16), (3, 18), (2, 15)] {
        let resolver = FixedSample(Arc::new(dsp::SampleData {
            data: vec![0.0; sample_count],
            sample_rate: 48_000.0,
            channels,
            loop_start: None,
            loop_end: None,
        }));
        let context = NrtPreparationContext::new(48_000.0).with_sample_resolver(&resolver);
        let definition = InstrumentDefinition::new(
            InstrumentId::from_raw(32),
            kind::ONE_SHOT_SAMPLE_PLAYER,
            payload(json!({
                "pan": 0.0,
                "sample_id": 23,
                "amplitude_envelope": amplitude_envelope()
            })),
            Vec::new(),
        );

        let error = BuiltInRegistry::new()
            .prepare_instrument(&definition, &context)
            .err()
            .unwrap();
        assert!(matches!(
            error,
            PreparationError::InvalidDefinition { diagnostic, .. }
                if diagnostic.code == InvalidDefinitionCode::InvalidResource
                    && diagnostic.field.as_deref() == Some("sample_id")
        ));
    }
}

#[test]
fn missing_sample_resource_is_an_invalid_definition_not_a_panic() {
    let definition = InstrumentDefinition::new(
        InstrumentId::from_raw(32),
        kind::LOOP_SAMPLE_PLAYER,
        payload(json!({
            "pan": 0.0,
            "sample_id": 404,
            "amplitude_envelope": amplitude_envelope()
        })),
        Vec::new(),
    );
    let error = BuiltInRegistry::new()
        .prepare_instrument(&definition, &NrtPreparationContext::new(48_000.0))
        .err()
        .unwrap();
    assert!(matches!(
        error,
        PreparationError::InvalidDefinition { diagnostic, .. }
            if diagnostic.code == InvalidDefinitionCode::MissingResource
                && diagnostic.field.as_deref() == Some("sample_id")
    ));
}

#[test]
fn kind_id_domains_are_distinct_even_when_text_matches() {
    let instrument = InstrumentKindId::from("vendor.same.text");
    let effect = EffectKindId::from("vendor.same.text");
    assert_eq!(instrument.as_str(), effect.as_str());
    // The two wrapper types intentionally have no cross-domain conversion or
    // PartialEq implementation, just as InstrumentId and EffectId are distinct.
}
