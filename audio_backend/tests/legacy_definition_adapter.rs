use audio_backend::{
    adapt_legacy_audio_effect, adapt_legacy_instrument, build_song_hydration_commands,
    id::{EffectId, InstrumentId},
    Command, Engine, EngineCommand, InstrumentCmd, LegacyDefinitionAdapterError, NoteEvent, NoteId,
    RetireSink, RetiredState,
};
use node_registry::{kind, BuiltInRegistry, InstrumentDefinition, NrtPreparationContext};
use sequencer::models::{
    AmpEnvelopeParams, AudioEffect, DFAMParams, Envelope, HiHatParams, Instrument, InstrumentData,
    KickDrumParams, PitchEnvelopeParams, SampleParams, SimpleOscillatorParams, SnareDrumParams,
    Song, SynthParams, Waveform,
};
use serde_json::json;

fn reverb() -> AudioEffect {
    AudioEffect::Reverb {
        mix: 0.25,
        decay_time: 0.5,
        room_size: 1.5,
        diffusion: 0.75,
        damping: 0.4,
    }
}

fn delay() -> AudioEffect {
    AudioEffect::Delay {
        time: 0.2,
        num_taps: 3,
        feedback: 0.4,
        mix: 0.5,
    }
}

#[derive(Clone, Copy)]
enum Fixture {
    Oscillator,
    HiHat,
    Kick,
    Snare,
    Dfam,
}

fn instrument(fixture: Fixture, audio_effects: Vec<AudioEffect>) -> InstrumentData {
    let amp_envelope = AmpEnvelopeParams::default();
    match fixture {
        Fixture::Oscillator => InstrumentData::SimpleOscillator(SimpleOscillatorParams {
            waveform: Waveform::NesTriangle,
            audio_effects,
            amp_envelope,
        }),
        Fixture::HiHat => InstrumentData::HiHat(HiHatParams {
            audio_effects,
            amp_envelope,
        }),
        Fixture::Kick => InstrumentData::KickDrum(KickDrumParams {
            audio_effects,
            amp_envelope,
            pitch_envelope: PitchEnvelopeParams {
                freq_delta: 100.0,
                decay_time: 0.1,
            },
        }),
        Fixture::Snare => InstrumentData::SnareDrum(SnareDrumParams {
            audio_effects,
            amp_envelope,
        }),
        Fixture::Dfam => InstrumentData::DFAM(DFAMParams {
            audio_effects,
            amp_envelope,
        }),
    }
}

#[test]
fn current_legacy_instrument_and_effect_inventory_maps_or_reports_no_faithful_kind() {
    let supported = [
        (Fixture::Oscillator, kind::MONO_OSCILLATOR),
        (Fixture::HiHat, kind::HI_HAT),
        (Fixture::Kick, kind::KICK_DRUM),
        (Fixture::Snare, kind::SNARE_DRUM),
        (Fixture::Dfam, kind::MOOG_DFAM),
    ];
    for (fixture, expected_kind) in supported {
        let data = instrument(fixture, vec![]);
        let definition = adapt_legacy_instrument(InstrumentId::from_raw(7), &data).unwrap();
        assert_eq!(definition.kind.as_str(), expected_kind);
        assert_eq!(definition.schema_version, 2);
        assert_eq!(definition.parameters["pan"], json!(0.0));
        assert_eq!(
            definition.parameters["amplitude_envelope"],
            json!({
                "attack_seconds": 0.01_f32,
                "decay_seconds": 0.1_f32,
                "sustain_level": 0.8_f32,
                "release_seconds": 0.2_f32
            })
        );
        if matches!(fixture, Fixture::Oscillator) {
            assert_eq!(definition.parameters["waveform"], json!("nes_triangle"));
        }
        if matches!(fixture, Fixture::Kick) {
            assert_eq!(
                definition.parameters["pitch_envelope"],
                json!({ "frequency_delta_hz": 100.0_f32, "decay_seconds": 0.1_f32 })
            );
        }
    }

    let envelope = Envelope {
        points: vec![],
        sustain_point: 0,
        loop_start_point: 0,
        loop_end_point: 0,
        enabled: false,
    };
    let unsupported = [
        InstrumentData::Sample(SampleParams {
            note_to_sample_map: [0; 96],
            volume_envelope: envelope.clone(),
            panning_envelope: envelope.clone(),
        }),
        InstrumentData::Synth(SynthParams {
            amp_envelope: envelope.clone(),
            filter_envelope: envelope,
        }),
    ];
    for data in unsupported {
        assert!(matches!(
            adapt_legacy_instrument(InstrumentId::from_raw(7), &data),
            Err(LegacyDefinitionAdapterError::UnsupportedInstrument { .. })
        ));
    }
}

#[test]
fn repeated_effect_kinds_keep_order_and_distinct_one_based_ids() {
    let data = instrument(Fixture::HiHat, vec![reverb(), reverb(), delay()]);
    let definition = adapt_legacy_instrument(InstrumentId::from_raw(3), &data).unwrap();
    let ids: Vec<_> = definition
        .effects
        .iter()
        .map(|effect| effect.instance_id.raw())
        .collect();
    let kinds: Vec<_> = definition
        .effects
        .iter()
        .map(|effect| effect.kind.as_str())
        .collect();
    assert_eq!(ids, [1, 2, 3]);
    assert_eq!(
        kinds,
        [kind::MONO_REVERB, kind::MONO_REVERB, kind::MONO_DELAY]
    );
}

#[test]
fn dfam_implicit_ladder_precedes_user_effects() {
    let data = instrument(Fixture::Dfam, vec![delay()]);
    let definition = adapt_legacy_instrument(InstrumentId::from_raw(4), &data).unwrap();
    assert_eq!(definition.effects[0].instance_id.raw(), 1);
    assert_eq!(definition.effects[0].kind.as_str(), kind::MONO_MOOG_LADDER);
    assert_eq!(definition.effects[0].parameters["cutoff"], json!(500.0));
    assert_eq!(definition.effects[1].instance_id.raw(), 2);
    assert_eq!(definition.effects[1].kind.as_str(), kind::MONO_DELAY);
}

#[test]
fn definition_json_roundtrip_is_deterministic() {
    let data = instrument(Fixture::Oscillator, vec![delay(), reverb()]);
    let definition = adapt_legacy_instrument(InstrumentId::from_raw(11), &data).unwrap();
    let first = serde_json::to_string_pretty(&definition).unwrap();
    let decoded: InstrumentDefinition = serde_json::from_str(&first).unwrap();
    assert_eq!(definition, decoded);
    assert_eq!(first, serde_json::to_string_pretty(&decoded).unwrap());
}

fn render_adapted_instrument(data: &InstrumentData) -> (InstrumentDefinition, Vec<f32>) {
    let definition = adapt_legacy_instrument(InstrumentId::from_raw(11), data).unwrap();
    let json = serde_json::to_string(&definition).unwrap();
    let decoded: InstrumentDefinition = serde_json::from_str(&json).unwrap();
    let mut instrument = BuiltInRegistry::new()
        .prepare_instrument(&decoded, &NrtPreparationContext::new(48_000.0))
        .unwrap();
    instrument.note_on(NoteEvent {
        id: NoteId::from_pitch(36),
        pitch: 36,
        velocity: u8::MAX,
    });
    let mut left = vec![0.0; 4_096];
    let mut right = vec![0.0; left.len()];
    instrument.process(&mut left, &mut right, 48_000.0);
    (decoded, left)
}

#[test]
fn adapter_json_and_registry_apply_amplitude_and_kick_pitch_values() {
    let kick = |frequency_delta_hz, pitch_decay_seconds| {
        InstrumentData::KickDrum(KickDrumParams {
            audio_effects: Vec::new(),
            amp_envelope: AmpEnvelopeParams {
                attack: 0.0,
                decay: 0.22,
                sustain: 0.33,
                release: 0.44,
            },
            pitch_envelope: PitchEnvelopeParams {
                freq_delta: frequency_delta_hz,
                decay_time: pitch_decay_seconds,
            },
        })
    };

    let (definition, baseline) = render_adapted_instrument(&kick(123.0, 0.05));
    assert_eq!(
        definition.parameters["amplitude_envelope"],
        json!({
            "attack_seconds": 0.0_f32,
            "decay_seconds": 0.22_f32,
            "sustain_level": 0.33_f32,
            "release_seconds": 0.44_f32
        })
    );
    assert_eq!(
        definition.parameters["pitch_envelope"],
        json!({ "frequency_delta_hz": 123.0_f32, "decay_seconds": 0.05_f32 })
    );

    let (_, different_delta) = render_adapted_instrument(&kick(-123.0, 0.05));
    let (_, different_decay) = render_adapted_instrument(&kick(123.0, 0.5));
    assert_ne!(baseline, different_delta, "frequency delta must reach DSP");
    assert_ne!(baseline, different_decay, "pitch decay must reach DSP");

    let oscillator = |attack| {
        InstrumentData::SimpleOscillator(SimpleOscillatorParams {
            waveform: Waveform::Sine,
            audio_effects: Vec::new(),
            amp_envelope: AmpEnvelopeParams {
                attack,
                decay: 0.1,
                sustain: 0.8,
                release: 0.2,
            },
        })
    };
    let (_, immediate_attack) = render_adapted_instrument(&oscillator(0.0));
    let (_, slow_attack) = render_adapted_instrument(&oscillator(1.0));
    assert_ne!(
        immediate_attack, slow_attack,
        "amplitude ADSR must reach DSP"
    );
}

#[test]
fn legacy_clamps_are_explicit_and_unrepresentable_values_are_errors() {
    let clamped_reverb = AudioEffect::Reverb {
        mix: 2.0,
        decay_time: -1.0,
        room_size: 9.0,
        diffusion: 2.0,
        damping: 0.4,
    };
    let definition = adapt_legacy_audio_effect(EffectId::from_raw(1), &clamped_reverb).unwrap();
    assert_eq!(definition.parameters["mix"], json!(1.0));
    assert_eq!(definition.parameters["decay"], json!(0.0));
    assert_eq!(definition.parameters["room_size"], json!(3.0));
    assert_eq!(definition.parameters["diffusion"], json!(0.95_f32));
    assert_eq!(definition.parameters["damping"], json!(0.4_f32));

    let clamped_delay = AudioEffect::Delay {
        time: -1.0,
        num_taps: 0,
        feedback: 2.0,
        mix: -2.0,
    };
    let definition = adapt_legacy_audio_effect(EffectId::from_raw(1), &clamped_delay).unwrap();
    assert_eq!(definition.parameters["delay_seconds"], json!(0.0));
    assert_eq!(definition.parameters["num_taps"], json!(1));
    assert_eq!(definition.parameters["feedback"], json!(0.95_f32));
    assert_eq!(definition.parameters["mix"], json!(0.0));

    let mut invalid = reverb();
    if let AudioEffect::Reverb { mix, .. } = &mut invalid {
        *mix = f32::NAN;
    }
    let error = adapt_legacy_audio_effect(EffectId::from_raw(1), &invalid).unwrap_err();
    assert!(matches!(
        error,
        LegacyDefinitionAdapterError::NonFiniteParameter { field: "mix" }
    ));
    assert_eq!(error.to_string(), "legacy parameter `mix` must be finite");

    if let AudioEffect::Reverb { mix, damping, .. } = &mut invalid {
        *mix = 0.5;
        *damping = 2.0;
    }
    let definition = adapt_legacy_audio_effect(EffectId::from_raw(1), &invalid).unwrap();
    assert_eq!(definition.parameters["damping"], json!(1.0));
}

#[derive(Debug, PartialEq)]
enum HydrationStep {
    Instrument(u32),
    Effect { instrument: u32, effect: u32 },
}

fn hydration_steps(commands: &[Command]) -> Vec<HydrationStep> {
    commands
        .iter()
        .map(|command| match command {
            Command::Instrument(InstrumentCmd::AddInstrument { instrument }) => {
                HydrationStep::Instrument(instrument.id().raw())
            }
            Command::Instrument(InstrumentCmd::AddEffect {
                instrument_id,
                effect,
            }) => HydrationStep::Effect {
                instrument: instrument_id.raw(),
                effect: effect.id().raw(),
            },
            _ => panic!("envelopes must be configured before owner handoff"),
        })
        .collect()
}

#[test]
fn registry_hydration_preserves_repeated_effect_owner_and_command_order() {
    let envelope = AmpEnvelopeParams::default();
    let mut song = Song::new("same-kind effects");
    song.instrument_bank.push(Instrument {
        id: 3,
        name: "effects".to_owned(),
        data: InstrumentData::HiHat(HiHatParams {
            audio_effects: vec![reverb(), reverb(), delay()],
            amp_envelope: envelope.clone(),
        }),
    });

    let commands = build_song_hydration_commands(&song, 48_000.0).unwrap();
    let expected = vec![
        HydrationStep::Instrument(3),
        HydrationStep::Effect {
            instrument: 3,
            effect: 1,
        },
        HydrationStep::Effect {
            instrument: 3,
            effect: 2,
        },
        HydrationStep::Effect {
            instrument: 3,
            effect: 3,
        },
    ];
    assert_eq!(hydration_steps(&commands), expected);
}

#[test]
fn registry_hydration_installs_dfam_ladder_before_distinct_user_effect() {
    let envelope = AmpEnvelopeParams::default();
    let mut song = Song::new("DFAM effect slots");
    song.instrument_bank.push(Instrument {
        id: 4,
        name: "dfam".to_owned(),
        data: InstrumentData::DFAM(DFAMParams {
            audio_effects: vec![delay()],
            amp_envelope: envelope.clone(),
        }),
    });

    let commands = build_song_hydration_commands(&song, 48_000.0).unwrap();
    let expected = vec![
        HydrationStep::Instrument(4),
        HydrationStep::Effect {
            instrument: 4,
            effect: 1,
        },
        HydrationStep::Effect {
            instrument: 4,
            effect: 2,
        },
    ];
    assert_eq!(hydration_steps(&commands), expected);
}

#[test]
fn registry_hydration_hands_off_only_fully_configured_owners() {
    let envelope = AmpEnvelopeParams {
        attack: 0.11,
        decay: 0.22,
        sustain: 0.33,
        release: 0.44,
    };
    let mut song = Song::new("legacy envelopes");
    song.instrument_bank.push(Instrument {
        id: 7,
        name: "kick".to_owned(),
        data: InstrumentData::KickDrum(KickDrumParams {
            audio_effects: vec![reverb()],
            amp_envelope: envelope.clone(),
            pitch_envelope: PitchEnvelopeParams {
                freq_delta: 123.0,
                decay_time: 0.75,
            },
        }),
    });

    let commands = build_song_hydration_commands(&song, 48_000.0).unwrap();
    assert_eq!(
        hydration_steps(&commands),
        [
            HydrationStep::Instrument(7),
            HydrationStep::Effect {
                instrument: 7,
                effect: 1,
            },
        ]
    );
}

struct CollectRetired(Vec<RetiredState>);

impl RetireSink for CollectRetired {
    fn retire(&mut self, state: RetiredState) {
        self.0.push(state);
    }
}

#[test]
fn prepared_effect_command_rejection_keeps_the_existing_rt_retirement_path() {
    let mut song = Song::new("retired prepared owner");
    song.instrument_bank.push(Instrument {
        id: 8,
        name: "effect without installed instrument".to_owned(),
        data: instrument(Fixture::HiHat, vec![delay()]),
    });
    let effect_command = build_song_hydration_commands(&song, 48_000.0)
        .unwrap()
        .into_iter()
        .find_map(|command| match command {
            Command::Instrument(command @ InstrumentCmd::AddEffect { .. }) => Some(command),
            _ => None,
        })
        .unwrap();

    let mut engine = Engine::new();
    let mut retired = CollectRetired(Vec::with_capacity(1));
    engine.handle_command_with_retirement(EngineCommand::Instrument(effect_command), &mut retired);

    assert_eq!(retired.0.len(), 1);
    match &retired.0[0] {
        RetiredState::MonoEffect(effect) => assert_eq!(effect.id().raw(), 1),
        _ => panic!("rejected prepared mono effect must retire as a mono owner"),
    }
}
