//! Integration tests for the parameter manifest crate.

use param_manifest::{
    builtin::{builtin_manifest, master_gain_descriptor, MASTER_GAIN_FLOOR_DB, MASTER_GAIN_ID},
    AutomationRate, CompatibilityBreak, Mapping, ParameterDescriptor, ParameterId, ParameterLookup,
    ParameterManifest, RuntimeKind, RuntimeParameter, SmoothingCurve, SmoothingPolicy, ValueRange,
    MANIFEST_SCHEMA_VERSION, MAX_DISCRETE_STEP_COUNT, MAX_PARAMETER_COUNT, MAX_SKEW, MIN_SKEW,
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
    assert_eq!(restored.parameters[0].smoothing, SmoothingPolicy::None);
    assert!(json.contains("\"smoothing\": \"none\""));
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
    assert_eq!(
        report.newly_deprecated,
        vec![ParameterId::from(MASTER_GAIN_ID)]
    );

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
        vec![CompatibilityBreak::AutomationRateChanged(
            ParameterId::from(MASTER_GAIN_ID)
        )]
    );
}

#[test]
fn smoothing_is_valid_only_for_control_coalesced_parameters() {
    for automation_rate in [AutomationRate::SampleEvent, AutomationRate::Structural] {
        let mut descriptor = master_gain_descriptor();
        descriptor.automation_rate = automation_rate;
        descriptor.smoothing = SmoothingPolicy::Smoothed {
            duration_ms: 10.0,
            curve: SmoothingCurve::Linear,
        };
        let error = ParameterManifest::new(vec![descriptor])
            .validate()
            .expect_err("non-coalesced smoothing must be rejected");
        assert!(matches!(
            error,
            param_manifest::ManifestError::ContradictorySmoothingClass(_)
        ));
    }

    let mut coalesced = master_gain_descriptor();
    coalesced.smoothing = SmoothingPolicy::Smoothed {
        duration_ms: 10.0,
        curve: SmoothingCurve::Exponential,
    };
    ParameterManifest::new(vec![coalesced])
        .validate()
        .expect("ControlCoalesced retains ADR 0004 smoothing compatibility");

    for automation_rate in [
        AutomationRate::SampleEvent,
        AutomationRate::ControlCoalesced,
        AutomationRate::Structural,
    ] {
        let mut descriptor = master_gain_descriptor();
        descriptor.automation_rate = automation_rate;
        descriptor.smoothing = SmoothingPolicy::None;
        ParameterManifest::new(vec![descriptor])
            .validate()
            .expect("ADR 0004 None fixtures remain valid for every traffic class");
    }
}

#[test]
fn changing_mapping_under_same_id_is_a_breaking_change() {
    let previous = builtin_manifest();
    let mut changed = master_gain_descriptor();
    changed.mapping = Mapping::Linear { min: 0.0, max: 1.0 };
    let new = ParameterManifest::new(vec![changed]);

    let report = new.compatibility_against(&previous);
    assert!(report
        .breaks
        .contains(&CompatibilityBreak::SemanticsChanged(ParameterId::from(
            MASTER_GAIN_ID
        ))));
}

#[test]
fn reversed_range_is_rejected_before_reaching_rt() {
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
        table.get(key).unwrap().engine_param_index(),
        lookup.get(key).unwrap().engine_param_index()
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
    assert!(
        (mid - 632.4555).abs() < 0.5,
        "midpoint should be geometric mean, got {mid}"
    );
    // Round-trip several points through the inverse.
    for &t in &[0.0_f32, 0.1, 0.37, 0.5, 0.9, 1.0] {
        let engine = mapping.to_engine(t);
        let back = mapping.to_normalized(engine);
        assert!(
            (back - t).abs() < 1e-4,
            "round-trip failed at t={t}: back={back}"
        );
    }
}

#[test]
fn to_normalized_clamps_engine_values_outside_the_range() {
    // Values below the range map to 0.0, above the range to 1.0, for every curve.
    let linear = Mapping::Linear {
        min: 0.0,
        max: 10.0,
    };
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
    let mapping = Mapping::Exponential {
        min: 0.0,
        max: 100.0,
    };
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
    let linear = Mapping::Linear {
        min: 0.0,
        max: 10.0,
    };
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
    assert_eq!(entry.engine_param_index(), 0);
    assert!(matches!(entry.kind(), RuntimeKind::Continuous));
    assert_eq!(entry.automation_rate(), AutomationRate::ControlCoalesced);

    // The runtime table reproduces the same engine value as the descriptor.
    let engine = lookup.normalized_to_engine(key, 0.5).expect("key converts");
    assert!((engine - (-6.0206)).abs() < 1e-3);

    // Out-of-range key yields None (bounded, no panic).
    assert!(lookup.get(param_manifest::RuntimeParamKey(999)).is_none());

    // Unknown ID does not resolve.
    assert!(lookup.key_for(&ParameterId::from("nope")).is_none());
}

#[test]
fn discrete_kind_carries_exact_numeric_values_on_rt() {
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
            default: 1.25,
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
                    // Deliberately non-uniform: the RT tier must carry this exact
                    // numeric value rather than reconstructing it from a count.
                    engine_value: 1.25,
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

    assert_eq!(descriptor.default_normalized(), 0.5);
    let manifest = ParameterManifest::new(vec![descriptor]);
    manifest.validate().expect("valid");
    let lookup = ParameterLookup::from_manifest(&manifest).expect("valid manifest");
    let key = lookup
        .key_for(&ParameterId::from("delay.mode"))
        .expect("resolves");
    assert_eq!(
        lookup.get(key).unwrap().kind(),
        RuntimeKind::Discrete { step_count: 3 }
    );
    assert_eq!(lookup.normalized_to_engine(key, 0.49), Some(1.25));
    assert_eq!(lookup.normalized_to_engine(key, f32::NAN), Some(0.0));
}

#[test]
fn schema_version_zero_is_rejected() {
    let mut manifest = builtin_manifest();
    manifest.schema_version = 0;
    let error = manifest.validate().expect_err("schema v0 is undefined");
    assert!(matches!(
        error,
        param_manifest::ManifestError::UnsupportedSchemaVersion { manifest: 0, .. }
    ));
}

#[test]
fn descriptor_version_added_zero_is_rejected() {
    let mut descriptor = master_gain_descriptor();
    descriptor.version_added = 0;
    let error = ParameterManifest::new(vec![descriptor])
        .validate()
        .expect_err("descriptor schema version 0 is undefined");
    assert!(matches!(
        error,
        param_manifest::ManifestError::InvalidDescriptorVersion {
            version_added: 0,
            schema_version: MANIFEST_SCHEMA_VERSION,
            ..
        }
    ));
}

#[test]
fn mapping_bounds_must_match_descriptor_range() {
    let mut descriptor = master_gain_descriptor();
    descriptor.mapping = Mapping::Linear { min: 0.0, max: 1.0 };
    let error = ParameterManifest::new(vec![descriptor])
        .validate()
        .expect_err("mapping/range disagreement must fail");
    assert!(format!("{error}").contains("mapping bounds must equal"));
}

#[test]
fn reversed_and_equal_mapping_endpoints_are_rejected() {
    for mapping in [
        Mapping::Linear { min: 1.0, max: 0.0 },
        Mapping::Linear { min: 0.0, max: 0.0 },
        Mapping::Exponential { min: 1.0, max: 1.0 },
        Mapping::Skewed {
            min: 1.0,
            max: 0.0,
            skew: 1.0,
        },
    ] {
        let (min, max) = mapping.engine_bounds();
        let mut descriptor = master_gain_descriptor();
        descriptor.range = ValueRange {
            min,
            max,
            default: min,
        };
        descriptor.mapping = mapping;
        assert!(
            ParameterManifest::new(vec![descriptor]).validate().is_err(),
            "mapping {mapping:?} must be rejected"
        );
    }

    // Direct malformed use remains deterministic and finite despite rejection.
    let equal = Mapping::Linear { min: 2.0, max: 2.0 };
    assert_eq!(equal.to_engine(0.5), 2.0);
    assert_eq!(equal.to_normalized(2.0), 0.0);
}

#[test]
fn tiny_linear_span_is_valid_and_invertible() {
    let mapping = Mapping::Linear {
        min: 0.0,
        max: 1.0e-8,
    };
    let mut descriptor = master_gain_descriptor();
    descriptor.range = ValueRange {
        min: 0.0,
        max: 1.0e-8,
        default: 0.0,
    };
    descriptor.mapping = mapping;
    ParameterManifest::new(vec![descriptor])
        .validate()
        .expect("representable non-zero spans are valid");

    assert_eq!(mapping.to_engine(1.0), 1.0e-8);
    assert_eq!(mapping.to_normalized(1.0e-8), 1.0);
    let engine = mapping.to_engine(0.37);
    assert!((mapping.to_normalized(engine) - 0.37).abs() < 1.0e-6);
}

#[test]
fn extreme_linear_span_uses_finite_stable_arithmetic() {
    let mapping = Mapping::Linear {
        min: -f32::MAX,
        max: f32::MAX,
    };
    let mut descriptor = master_gain_descriptor();
    descriptor.range = ValueRange {
        min: -f32::MAX,
        max: f32::MAX,
        default: 0.0,
    };
    descriptor.mapping = mapping;
    ParameterManifest::new(vec![descriptor])
        .validate()
        .expect("full finite f32 span is supported");

    assert_eq!(mapping.to_engine(0.5), 0.0);
    assert_eq!(mapping.to_normalized(0.0), 0.5);
    for t in [0.0, 0.1, 0.9, 1.0] {
        assert!(mapping.to_engine(t).is_finite());
    }
}

#[test]
fn exponential_extreme_ratio_does_not_overflow_or_underflow_intermediates() {
    let mapping = Mapping::Exponential {
        min: f32::MIN_POSITIVE,
        max: f32::MAX,
    };
    let mut descriptor = master_gain_descriptor();
    descriptor.range = ValueRange {
        min: f32::MIN_POSITIVE,
        max: f32::MAX,
        default: 1.0,
    };
    descriptor.mapping = mapping;
    ParameterManifest::new(vec![descriptor])
        .validate()
        .expect("extreme positive endpoints are supported");

    assert_eq!(mapping.to_engine(0.0), f32::MIN_POSITIVE);
    assert_eq!(mapping.to_engine(1.0), f32::MAX);
    let midpoint = mapping.to_engine(0.5);
    assert!(midpoint.is_finite() && midpoint > 0.0);
    assert!((mapping.to_normalized(midpoint) - 0.5).abs() < 1.0e-5);
}

#[test]
fn skew_bounds_reject_collapsing_shapes_and_round_trip_representative_values() {
    for skew in [MIN_SKEW / 2.0, MAX_SKEW * 2.0] {
        let mut descriptor = master_gain_descriptor();
        descriptor.mapping = Mapping::Skewed {
            min: -120.0,
            max: 0.0,
            skew,
        };
        assert!(ParameterManifest::new(vec![descriptor]).validate().is_err());
    }

    // mapping.rs documents 1e-4 as the accepted f32 normalized round-trip
    // tolerance. Exercise both the unit range from the original underflow report
    // and the translated -120..=0 range where endpoint rounding occurs sooner.
    const ROUND_TRIP_TOLERANCE: f32 = 1.0e-4;
    for skew in [MIN_SKEW, MAX_SKEW] {
        for (min, max, default) in [(0.0, 1.0, 0.5), (-120.0, 0.0, 0.0)] {
            let mapping = Mapping::Skewed { min, max, skew };
            let mut descriptor = master_gain_descriptor();
            descriptor.range = ValueRange { min, max, default };
            descriptor.mapping = mapping;
            ParameterManifest::new(vec![descriptor])
                .validate()
                .expect("boundary skew/range pair is representable");

            for normalized in [0.1_f32, 0.25, 0.5, 0.9] {
                let engine = mapping.to_engine(normalized);
                let round_trip = mapping.to_normalized(engine);
                assert!(
                    (round_trip - normalized).abs() <= ROUND_TRIP_TOLERANCE,
                    "round-trip failed for range {min}..={max}, skew={skew}, normalized={normalized}: {round_trip}"
                );
            }
        }
    }

    // Bounds alone cannot guarantee precision when endpoints are adjacent f32s;
    // validation therefore rejects an authored skew/range pair that collapses.
    let min = 1.0e20_f32;
    let max = f32::from_bits(min.to_bits() + 1);
    let mut descriptor = master_gain_descriptor();
    descriptor.range = ValueRange {
        min,
        max,
        default: min,
    };
    descriptor.mapping = Mapping::Skewed {
        min,
        max,
        skew: MAX_SKEW,
    };
    let error = ParameterManifest::new(vec![descriptor])
        .validate()
        .expect_err("collapsed skew/range pair must be rejected");
    assert!(format!("{error}").contains("round-trip precision"));
}

#[test]
fn invalid_amplitude_decibel_floors_are_rejected() {
    for floor_db in [0.0, 1.0, f32::NAN, f32::INFINITY] {
        let mut descriptor = master_gain_descriptor();
        descriptor.mapping = Mapping::AmplitudeDecibel { floor_db };
        assert!(
            ParameterManifest::new(vec![descriptor]).validate().is_err(),
            "floor {floor_db} must fail"
        );
    }
}

#[test]
fn nan_mapping_inputs_use_finite_range_floor_fallback() {
    let mappings = [
        Mapping::Linear {
            min: -2.0,
            max: 3.0,
        },
        Mapping::Exponential {
            min: 1.0,
            max: 100.0,
        },
        Mapping::Skewed {
            min: -2.0,
            max: 3.0,
            skew: 2.0,
        },
        Mapping::AmplitudeDecibel { floor_db: -120.0 },
    ];

    for mapping in mappings {
        let (floor, _) = mapping.engine_bounds();
        assert_eq!(mapping.to_engine(f32::NAN), floor, "{mapping:?}");
        assert_eq!(mapping.to_normalized(f32::NAN), 0.0, "{mapping:?}");
        assert!(mapping.to_engine(f32::NAN).is_finite());
        assert!(mapping.to_normalized(f32::NAN).is_finite());
        assert_eq!(mapping.to_engine(f32::NEG_INFINITY), floor);
        assert_eq!(mapping.to_engine(f32::INFINITY), mapping.engine_bounds().1);
    }
}

#[test]
fn discrete_values_must_be_in_range_and_default_must_be_a_step() {
    use param_manifest::{DiscreteStep, ParameterKind};

    let mut descriptor = master_gain_descriptor();
    descriptor.kind = ParameterKind::Discrete {
        steps: vec![
            DiscreteStep {
                label: "floor".into(),
                engine_value: -120.0,
            },
            DiscreteStep {
                label: "invalid".into(),
                engine_value: 1.0,
            },
        ],
    };
    assert!(ParameterManifest::new(vec![descriptor]).validate().is_err());

    let mut descriptor = master_gain_descriptor();
    descriptor.range.default = -6.0;
    descriptor.kind = ParameterKind::Discrete {
        steps: vec![
            DiscreteStep {
                label: "floor".into(),
                engine_value: -120.0,
            },
            DiscreteStep {
                label: "unity".into(),
                engine_value: 0.0,
            },
        ],
    };
    assert!(ParameterManifest::new(vec![descriptor]).validate().is_err());
}

#[test]
fn practical_parameter_and_step_capacities_are_enforced() {
    use param_manifest::{DiscreteStep, ParameterKind};

    let descriptors = vec![master_gain_descriptor(); MAX_PARAMETER_COUNT + 1];
    let error = ParameterManifest::new(descriptors)
        .validate()
        .expect_err("parameter capacity must be bounded");
    assert!(matches!(
        error,
        param_manifest::ManifestError::CapacityExceeded { .. }
    ));

    let mut descriptor = master_gain_descriptor();
    descriptor.kind = ParameterKind::Discrete {
        steps: vec![
            DiscreteStep {
                label: String::new(),
                engine_value: 0.0,
            };
            MAX_DISCRETE_STEP_COUNT + 1
        ],
    };
    assert!(matches!(
        ParameterManifest::new(vec![descriptor]).validate(),
        Err(param_manifest::ManifestError::CapacityExceeded { .. })
    ));
}

#[test]
fn moving_stable_id_to_another_owner_is_breaking() {
    use param_manifest::NodeType;

    let previous = builtin_manifest();
    let mut changed = master_gain_descriptor();
    changed.owner.node_type = NodeType::InstrumentEffect;
    changed.owner.path = "instrument/effect:gain".to_string();
    let report = ParameterManifest::new(vec![changed]).compatibility_against(&previous);
    assert!(report
        .breaks
        .contains(&CompatibilityBreak::SemanticsChanged(ParameterId::from(
            MASTER_GAIN_ID
        ))));
}

#[test]
fn automatable_read_only_visibility_is_rejected() {
    use param_manifest::Visibility;

    let mut descriptor = master_gain_descriptor();
    descriptor.visibility = Visibility {
        host_visible: true,
        automatable: true,
        read_only: true,
    };
    let error = ParameterManifest::new(vec![descriptor])
        .validate()
        .expect_err("read-only parameters cannot accept automation writes");
    assert!(matches!(
        error,
        param_manifest::ManifestError::ContradictoryVisibility(_)
    ));
}

#[test]
fn visibility_is_a_binding_break_but_smoothing_is_compatible() {
    use param_manifest::{SmoothingPolicy, Visibility};

    let previous = builtin_manifest();
    let mut changed = master_gain_descriptor();
    changed.visibility = Visibility {
        host_visible: false,
        automatable: false,
        read_only: true,
    };
    let report = ParameterManifest::new(vec![changed]).compatibility_against(&previous);
    assert!(report
        .breaks
        .contains(&CompatibilityBreak::HostBindingChanged(ParameterId::from(
            MASTER_GAIN_ID
        ))));

    let mut changed = master_gain_descriptor();
    changed.smoothing = SmoothingPolicy::Smoothed {
        duration_ms: 15.0,
        curve: SmoothingCurve::Linear,
    };
    let report = ParameterManifest::new(vec![changed]).compatibility_against(&previous);
    assert!(report.is_compatible());
}

#[test]
fn runtime_parameter_is_copy_and_compact() {
    const fn require_copy<T: Copy>() {}
    const _: () = require_copy::<RuntimeParameter>();
    const _: () = assert!(std::mem::size_of::<RuntimeParameter>() <= 64);

    assert!(std::mem::size_of::<RuntimeParameter>() <= 64);
}
