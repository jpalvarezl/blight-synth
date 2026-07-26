//! Integration tests for the parameter manifest crate.

use param_manifest::{
    builtin::{builtin_manifest, master_gain_descriptor, MASTER_GAIN_FLOOR_DB, MASTER_GAIN_ID},
    AutomationRate, CompatibilityBreak, Mapping, ParameterDescriptor, ParameterId,
    ParameterLookup, ParameterManifest, RuntimeKind, MANIFEST_SCHEMA_VERSION,
};

#[test]
fn manifest_json_round_trip_is_lossless() {
    let manifest = builtin_manifest();
    manifest.validate().expect("builtin manifest is valid");

    let json = serde_json::to_string_pretty(&manifest).expect("serialize");
    let restored: ParameterManifest = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(manifest, restored);
    assert_eq!(restored.schema_version, MANIFEST_SCHEMA_VERSION);
    assert_eq!(restored.parameters.len(), 1);
    assert_eq!(restored.parameters[0].id.as_str(), MASTER_GAIN_ID);
}

#[test]
fn duplicate_ids_are_rejected() {
    let manifest = ParameterManifest::new(vec![master_gain_descriptor(), master_gain_descriptor()]);
    let err = manifest.validate().expect_err("duplicate ids must fail");
    assert!(format!("{err}").contains("duplicate"));
}

#[test]
fn newer_schema_version_is_unsupported() {
    let mut manifest = builtin_manifest();
    manifest.schema_version = MANIFEST_SCHEMA_VERSION + 1;
    assert!(manifest.validate().is_err());
    assert!(!manifest.is_readable_by(MANIFEST_SCHEMA_VERSION));
    assert!(manifest.is_readable_by(MANIFEST_SCHEMA_VERSION + 1));
}

#[test]
fn removing_a_parameter_without_deprecation_is_a_breaking_change() {
    let previous = builtin_manifest();
    // New manifest drops the parameter entirely.
    let new = ParameterManifest::new(vec![]);

    let report = new.compatibility_against(&previous);
    assert!(!report.is_compatible());
    assert_eq!(
        report.breaks,
        vec![CompatibilityBreak::RemovedWithoutDeprecation(
            ParameterId::from(MASTER_GAIN_ID)
        )]
    );
}

#[test]
fn deprecating_then_removing_is_compatible() {
    // Step 1: previous still has it live; new marks it deprecated -> compatible.
    let previous = builtin_manifest();
    let mut deprecated_desc = master_gain_descriptor();
    deprecated_desc.deprecated = Some("superseded by master.gain.v2".to_string());
    let deprecated_manifest = ParameterManifest::new(vec![deprecated_desc]);

    let report = deprecated_manifest.compatibility_against(&previous);
    assert!(report.is_compatible());
    assert_eq!(report.newly_deprecated, vec![ParameterId::from(MASTER_GAIN_ID)]);

    // Step 2: removing an already-deprecated parameter is compatible.
    let removed = ParameterManifest::new(vec![]);
    let report = removed.compatibility_against(&deprecated_manifest);
    assert!(report.is_compatible());
}

#[test]
fn adding_a_parameter_is_compatible() {
    let previous = ParameterManifest::new(vec![]);
    let new = builtin_manifest();
    let report = new.compatibility_against(&previous);
    assert!(report.is_compatible());
    assert_eq!(report.added, vec![ParameterId::from(MASTER_GAIN_ID)]);
}

#[test]
fn changing_automation_rate_is_a_breaking_change() {
    let previous = builtin_manifest();
    let mut changed = master_gain_descriptor();
    changed.automation_rate = AutomationRate::Structural;
    let new = ParameterManifest::new(vec![changed]);

    let report = new.compatibility_against(&previous);
    assert_eq!(
        report.breaks,
        vec![CompatibilityBreak::AutomationRateChanged(ParameterId::from(
            MASTER_GAIN_ID
        ))]
    );
}

#[test]
fn changing_mapping_under_same_id_is_a_breaking_change() {
    let previous = builtin_manifest();
    let mut changed = master_gain_descriptor();
    changed.mapping = Mapping::Linear { min: 0.0, max: 1.0 };
    let new = ParameterManifest::new(vec![changed]);

    let report = new.compatibility_against(&previous);
    assert!(report.breaks.contains(&CompatibilityBreak::SemanticsChanged(
        ParameterId::from(MASTER_GAIN_ID)
    )));
}

#[test]
fn reversed_range_is_rejected_before_reaching_rt() {
    use param_manifest::ValueRange;
    let mut bad = master_gain_descriptor();
    // A reversed range would panic f32::clamp on the RT conversion path.
    bad.range = ValueRange {
        min: 0.0,
        max: -120.0,
        default: 0.0,
    };
    let manifest = ParameterManifest::new(vec![bad]);
    assert!(manifest.validate().is_err());
}

#[test]
fn building_lookup_from_reversed_range_returns_error_not_panic() {
    use param_manifest::ValueRange;
    let mut bad = master_gain_descriptor();
    // A reversed range slipping into the lookup would let a later
    // `normalized_to_engine` reach `f32::clamp(min, max)` with min > max.
    // Construction must validate and reject it instead.
    bad.range = ValueRange {
        min: 0.0,
        max: -120.0,
        default: 0.0,
    };
    let manifest = ParameterManifest::new(vec![bad]);
    assert!(ParameterLookup::from_manifest(&manifest).is_err());
}

#[test]
fn non_finite_default_is_rejected() {
    use param_manifest::ValueRange;
    let mut bad = master_gain_descriptor();
    bad.range = ValueRange {
        min: -120.0,
        max: 0.0,
        default: f32::NAN,
    };
    let manifest = ParameterManifest::new(vec![bad]);
    assert!(manifest.validate().is_err());
}

#[test]
fn runtime_table_is_the_string_free_rt_handle() {
    let manifest = builtin_manifest();
    let lookup = ParameterLookup::from_manifest(&manifest).expect("valid manifest");
    let key = lookup
        .key_for(&ParameterId::from(MASTER_GAIN_ID))
        .expect("resolves");

    // The RT handle indexes by key without any resolver map.
    let table = lookup.table();
    assert_eq!(table.len(), 1);
    assert_eq!(
        table.get(key).unwrap().engine_param_index,
        lookup.get(key).unwrap().engine_param_index
    );

    // Ownership can be transferred to the callback, dropping the NRT resolver.
    let owned = lookup.into_table();
    assert!(owned.get(key).is_some());
    assert!(owned.get(param_manifest::RuntimeParamKey(999)).is_none());
}

#[test]
fn master_gain_mapping_matches_the_osc_normalized_db_convention() {
    let descriptor = master_gain_descriptor();
    let mapping = descriptor.mapping;

    // Unity amplitude -> 0 dB.
    assert!((mapping.to_engine(1.0) - 0.0).abs() < 1e-4);
    // 0.5 linear amplitude -> ~-6.02 dB (the exact value the OSC adapter asserts).
    assert!((mapping.to_engine(0.5) - (-6.0206)).abs() < 1e-3);
    // Silence floors at the configured floor.
    assert_eq!(mapping.to_engine(0.0), MASTER_GAIN_FLOOR_DB);
    // Above unity clamps to unity.
    assert!((mapping.to_engine(2.0) - 0.0).abs() < 1e-4);

    // Inverse round-trips within range.
    assert!((mapping.to_normalized(0.0) - 1.0).abs() < 1e-4);
    assert!((mapping.to_normalized(-6.0206) - 0.5).abs() < 1e-3);
}

#[test]
fn linear_mapping_round_trips() {
    let mapping = Mapping::Linear {
        min: 20.0,
        max: 20_000.0,
    };
    let engine = mapping.to_engine(0.25);
    let normalized = mapping.to_normalized(engine);
    assert!((normalized - 0.25).abs() < 1e-5);
}

#[test]
fn exponential_mapping_round_trips_and_is_perceptually_geometric() {
    // 20 Hz .. 20 kHz over a 0..1 knob: the exponential curve is geometric, so
    // the midpoint of the knob lands on the geometric mean (sqrt(20 * 20000) =
    // ~632 Hz), not the arithmetic mean (~10 kHz).
    let mapping = Mapping::Exponential {
        min: 20.0,
        max: 20_000.0,
    };
    assert!((mapping.to_engine(0.0) - 20.0).abs() < 1e-3);
    assert!((mapping.to_engine(1.0) - 20_000.0).abs() < 1.0);
    let mid = mapping.to_engine(0.5);
    assert!((mid - 632.4555).abs() < 0.5, "midpoint should be geometric mean, got {mid}");
    // Round-trip several points through the inverse.
    for &t in &[0.0_f32, 0.1, 0.37, 0.5, 0.9, 1.0] {
        let engine = mapping.to_engine(t);
        let back = mapping.to_normalized(engine);
        assert!((back - t).abs() < 1e-4, "round-trip failed at t={t}: back={back}");
    }
}

#[test]
fn to_normalized_clamps_engine_values_outside_the_range() {
    // Values below the range map to 0.0, above the range to 1.0, for every curve.
    let linear = Mapping::Linear { min: 0.0, max: 10.0 };
    assert_eq!(linear.to_normalized(-5.0), 0.0);
    assert_eq!(linear.to_normalized(50.0), 1.0);

    let exp = Mapping::Exponential {
        min: 20.0,
        max: 20_000.0,
    };
    assert_eq!(exp.to_normalized(1.0), 0.0); // below min
    assert_eq!(exp.to_normalized(1_000_000.0), 1.0); // above max
    // Non-positive engine input on a positive-endpoint exponential must not
    // produce NaN/inf; it clamps to 0.0.
    assert_eq!(exp.to_normalized(0.0), 0.0);
    assert_eq!(exp.to_normalized(-3.0), 0.0);

    let db = Mapping::AmplitudeDecibel { floor_db: -120.0 };
    assert_eq!(db.to_normalized(-200.0), 0.0); // at/below floor
    assert_eq!(db.to_normalized(6.0), 1.0); // above 0 dB clamps to 1.0
}

#[test]
fn exponential_with_non_positive_endpoints_falls_back_to_linear() {
    // Guarded configuration: a non-positive endpoint can't use the geometric
    // formula, so both directions degrade to linear interpolation rather than
    // returning NaN.
    let mapping = Mapping::Exponential { min: 0.0, max: 100.0 };
    assert!((mapping.to_engine(0.5) - 50.0).abs() < 1e-3);
    assert!((mapping.to_normalized(50.0) - 0.5).abs() < 1e-3);
}

#[test]
fn skewed_mapping_with_skew_one_matches_linear() {
    let skewed = Mapping::Skewed {
        min: 0.0,
        max: 10.0,
        skew: 1.0,
    };
    let linear = Mapping::Linear { min: 0.0, max: 10.0 };
    for &t in &[0.0_f32, 0.1, 0.37, 0.5, 0.9, 1.0] {
        assert!(
            (skewed.to_engine(t) - linear.to_engine(t)).abs() < 1e-6,
            "to_engine diverges from linear at t={t}"
        );
        let engine = linear.to_engine(t);
        assert!(
            (skewed.to_normalized(engine) - linear.to_normalized(engine)).abs() < 1e-6,
            "to_normalized diverges from linear at t={t}"
        );
    }
}

#[test]
fn skewed_skew_biases_midpoint_relative_to_arithmetic_mean() {
    // min=0, max=10 -> arithmetic mean of the endpoints is 5.0.
    let mean = 5.0_f32;
    // skew < 1.0 biases toward max: fast rise, so the knob midpoint sits above
    // the arithmetic mean.
    let fast = Mapping::Skewed {
        min: 0.0,
        max: 10.0,
        skew: 0.4,
    };
    assert!(
        fast.to_engine(0.5) > mean,
        "skew<1 midpoint should exceed arithmetic mean, got {}",
        fast.to_engine(0.5)
    );
    // skew > 1.0 biases toward min: slow rise, so the knob midpoint sits below
    // the arithmetic mean.
    let slow = Mapping::Skewed {
        min: 0.0,
        max: 10.0,
        skew: 2.5,
    };
    assert!(
        slow.to_engine(0.5) < mean,
        "skew>1 midpoint should be below arithmetic mean, got {}",
        slow.to_engine(0.5)
    );
}

#[test]
fn skewed_mapping_round_trips_for_several_skews() {
    for &skew in &[0.4_f32, 2.5] {
        let mapping = Mapping::Skewed {
            min: -3.0,
            max: 7.0,
            skew,
        };
        for &t in &[0.0_f32, 0.1, 0.37, 0.5, 0.9, 1.0] {
            let engine = mapping.to_engine(t);
            let back = mapping.to_normalized(engine);
            assert!(
                (back - t).abs() < 1e-4,
                "round-trip failed at t={t} skew={skew}: back={back}"
            );
        }
    }
}

#[test]
fn skewed_to_normalized_clamps_out_of_range_engine() {
    let mapping = Mapping::Skewed {
        min: 0.0,
        max: 10.0,
        skew: 2.5,
    };
    assert_eq!(mapping.to_normalized(-5.0), 0.0); // below min
    assert_eq!(mapping.to_normalized(50.0), 1.0); // above max
}

#[test]
fn skewed_with_invalid_skew_is_rejected_by_validation() {
    for bad_skew in [0.0_f32, -1.0, f32::NAN, f32::INFINITY] {
        let mut d = master_gain_descriptor();
        d.mapping = Mapping::Skewed {
            min: -120.0,
            max: 0.0,
            skew: bad_skew,
        };
        let manifest = ParameterManifest::new(vec![d]);
        assert!(
            manifest.validate().is_err(),
            "validation should reject skew={bad_skew}"
        );
    }
}

#[test]
fn skewed_with_valid_skew_passes_validation() {
    let mut d = master_gain_descriptor();
    d.mapping = Mapping::Skewed {
        min: -120.0,
        max: 0.0,
        skew: 2.5,
    };
    let manifest = ParameterManifest::new(vec![d]);
    assert!(manifest.validate().is_ok());
}

#[test]
fn lookup_resolves_by_stable_id_and_indexes_by_key() {
    let manifest = builtin_manifest();
    let lookup = ParameterLookup::from_manifest(&manifest).expect("valid manifest");

    assert_eq!(lookup.len(), 1);

    // NRT: resolve stable ID -> compact key.
    let id = ParameterId::from(MASTER_GAIN_ID);
    let key = lookup.key_for(&id).expect("id resolves to a key");

    // RT: bounded index by key, no strings involved.
    let entry = lookup.get(key).expect("key resolves to an entry");
    assert_eq!(entry.engine_param_index, 0);
    assert!(matches!(entry.kind, RuntimeKind::Continuous));
    assert_eq!(entry.automation_rate, AutomationRate::ControlCoalesced);

    // The runtime mapping reproduces the same engine value as the descriptor.
    assert!((entry.normalized_to_engine(0.5) - (-6.0206)).abs() < 1e-3);

    // Out-of-range key yields None (bounded, no panic).
    assert!(lookup.get(param_manifest::RuntimeParamKey(999)).is_none());

    // Unknown ID does not resolve.
    assert!(lookup.key_for(&ParameterId::from("nope")).is_none());
}

#[test]
fn discrete_kind_collapses_to_step_count_on_rt() {
    use param_manifest::{
        DiscreteStep, NodeRef, NodeType, ParameterKind, SmoothingPolicy, Unit, ValueRange,
        Visibility,
    };

    let descriptor = ParameterDescriptor {
        id: ParameterId::from("delay.mode"),
        owner: NodeRef {
            node_type: NodeType::InstrumentEffect,
            path: "instrument/effect:delay".to_string(),
            engine_param_index: 1,
        },
        display_name: "Delay Mode".to_string(),
        short_name: "Mode".to_string(),
        unit: Unit::Count,
        range: ValueRange {
            min: 0.0,
            max: 2.0,
            default: 0.0,
        },
        mapping: Mapping::Linear { min: 0.0, max: 2.0 },
        kind: ParameterKind::Discrete {
            steps: vec![
                DiscreteStep {
                    label: "Off".to_string(),
                    engine_value: 0.0,
                },
                DiscreteStep {
                    label: "Slap".to_string(),
                    engine_value: 1.0,
                },
                DiscreteStep {
                    label: "Ping-Pong".to_string(),
                    engine_value: 2.0,
                },
            ],
        },
        automation_rate: AutomationRate::ControlCoalesced,
        smoothing: SmoothingPolicy::None,
        visibility: Visibility::default(),
        version_added: 1,
        deprecated: None,
    };

    let manifest = ParameterManifest::new(vec![descriptor]);
    manifest.validate().expect("valid");
    let lookup = ParameterLookup::from_manifest(&manifest).expect("valid manifest");
    let key = lookup
        .key_for(&ParameterId::from("delay.mode"))
        .expect("resolves");
    assert_eq!(
        lookup.get(key).unwrap().kind,
        RuntimeKind::Discrete { step_count: 3 }
    );
}
