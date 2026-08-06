use audio_backend::{
    adapt_legacy_audio_effect, adapt_legacy_instrument, build_song_hydration_commands,
    id::{EffectId, InstrumentId},
    Command, Engine, EngineCommand, EnvelopeCmd, InstrumentCmd, LegacyDefinitionAdapterError,
    RetireSink, RetiredState, SynthCmd,
};
use node_registry::{kind, InstrumentDefinition};
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
        assert_eq!(definition.parameters["pan"], json!(0.0));
        if matches!(fixture, Fixture::Oscillator) {
            assert_eq!(definition.parameters["waveform"], json!("nes_triangle"));
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
    Pitch { instrument: u32, delta: f32 },
    Attack { instrument: u32, value: f32 },
    Decay { instrument: u32, value: f32 },
    Sustain { instrument: u32, value: f32 },
    Release { instrument: u32, value: f32 },
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
            Command::Instrument(InstrumentCmd::PassOnSynthCmd {
                instrument_id,
                synth_cmd:
                    SynthCmd::EnvelopeCommand {
                        envelope_id: None,
                        command: EnvelopeCmd::SetPitchEnvFreqDelta { freq_delta },
                    },
            }) => HydrationStep::Pitch {
                instrument: instrument_id.raw(),
                delta: *freq_delta,
            },
            Command::Instrument(InstrumentCmd::PassOnSynthCmd {
                instrument_id,
                synth_cmd:
                    SynthCmd::EnvelopeCommand {
                        envelope_id: Some(envelope_id),
                        command,
                    },
            }) => {
                assert_eq!(envelope_id.raw(), 0);
                match command {
                    EnvelopeCmd::SetAttack { attack } => HydrationStep::Attack {
                        instrument: instrument_id.raw(),
                        value: *attack,
                    },
                    EnvelopeCmd::SetDecay { decay } => HydrationStep::Decay {
                        instrument: instrument_id.raw(),
                        value: *decay,
                    },
                    EnvelopeCmd::SetSustain { sustain } => HydrationStep::Sustain {
                        instrument: instrument_id.raw(),
                        value: *sustain,
                    },
                    EnvelopeCmd::SetRelease { release } => HydrationStep::Release {
                        instrument: instrument_id.raw(),
                        value: *release,
                    },
                    EnvelopeCmd::SetPitchEnvFreqDelta { .. } => {
                        panic!("pitch envelope must use the implicit envelope target")
                    }
                }
            }
            _ => panic!("unexpected hydration command"),
        })
        .collect()
}

fn amp_steps(instrument: u32, envelope: &AmpEnvelopeParams) -> Vec<HydrationStep> {
    vec![
        HydrationStep::Attack {
            instrument,
            value: envelope.attack,
        },
        HydrationStep::Decay {
            instrument,
            value: envelope.decay,
        },
        HydrationStep::Sustain {
            instrument,
            value: envelope.sustain,
        },
        HydrationStep::Release {
            instrument,
            value: envelope.release,
        },
    ]
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
    let mut expected = vec![
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
    expected.extend(amp_steps(3, &envelope));
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
    let mut expected = vec![
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
    expected.extend(amp_steps(4, &envelope));
    assert_eq!(hydration_steps(&commands), expected);
}

#[test]
fn registry_hydration_keeps_explicit_pitch_then_amplitude_envelope_commands() {
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
    let mut expected = vec![
        HydrationStep::Instrument(7),
        HydrationStep::Effect {
            instrument: 7,
            effect: 1,
        },
        HydrationStep::Pitch {
            instrument: 7,
            delta: 123.0,
        },
    ];
    expected.extend(amp_steps(7, &envelope));
    assert_eq!(hydration_steps(&commands), expected);
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
