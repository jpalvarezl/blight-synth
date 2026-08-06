#[cfg(feature = "device-host")]
use anyhow::{bail, Context, Result};
use node_registry::{
    BuiltInRegistry, EffectLayout, InstrumentDefinition, NrtPreparationContext, PreparationError,
    PreparedEffect, PreparedInstrumentDefinition,
};
use sequencer::models::Song;
#[cfg(feature = "device-host")]
use sequencer::{cli::FileFormat, project::open_song_from_file};
use std::{error::Error, fmt};
#[cfg(feature = "device-host")]
use std::{path::Path, sync::Arc};

use crate::{
    adapt_legacy_instrument,
    id::{EffectId, InstrumentId},
    Command, InstrumentCmd, LegacyDefinitionAdapterError,
};
#[cfg(feature = "device-host")]
use crate::{BlightAudio, CommandSubmissionErrorKind, SequencerCmd};

/// Structured NRT failure while adapting or preparing a legacy song instrument.
#[derive(Debug)]
pub enum SongHydrationError {
    UnaddressableInstrumentId {
        project_id: usize,
    },
    LegacyDefinition {
        project_id: usize,
        source: LegacyDefinitionAdapterError,
    },
    RegistryPreparation {
        project_id: usize,
        source: PreparationError,
    },
    UnsupportedPreparedEffectLayout {
        project_id: usize,
        effect_id: EffectId,
        layout: EffectLayout,
    },
}

impl fmt::Display for SongHydrationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnaddressableInstrumentId { project_id } => write!(
                formatter,
                "project instrument ID {project_id} exceeds the tracker event u8 range"
            ),
            Self::LegacyDefinition { project_id, source } => write!(
                formatter,
                "could not adapt project instrument {project_id}: {source}"
            ),
            Self::RegistryPreparation { project_id, source } => write!(
                formatter,
                "could not prepare project instrument {project_id}: {source}"
            ),
            Self::UnsupportedPreparedEffectLayout {
                project_id,
                effect_id,
                layout,
            } => write!(
                formatter,
                "project instrument {project_id} effect {} prepared as unsupported {layout:?} owner",
                effect_id.raw()
            ),
        }
    }
}

impl Error for SongHydrationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::LegacyDefinition { source, .. } => Some(source),
            Self::RegistryPreparation { source, .. } => Some(source),
            Self::UnaddressableInstrumentId { .. }
            | Self::UnsupportedPreparedEffectLayout { .. } => None,
        }
    }
}

/// Load a JSON song file and install it into the audio backend without starting playback.
///
/// This is the shared path for standalone OSC `/song/load` and examples. It queues a
/// `SequencerCmd::LoadSong` first, then queues instrument hydration commands.
#[cfg(feature = "device-host")]
pub fn load_song_file_into_audio(audio: &mut BlightAudio, path: &Path) -> Result<Song> {
    let (song, commands) = prepare_song_file_for_audio(audio, path)?;
    for command in commands {
        submit_command(audio, command)?;
    }
    Ok(song)
}

/// Parse a song and prepare its complete ordered load/hydration command batch
/// without submitting any command to RT.
#[cfg(feature = "device-host")]
pub(crate) fn prepare_song_file_for_audio(
    audio: &BlightAudio,
    path: &Path,
) -> Result<(Song, Vec<Command>)> {
    log::info!("loading song from {}", path.display());
    let song = open_song_from_file(&path.to_path_buf(), &FileFormat::Json)
        .with_context(|| format!("failed to load song from {}", path.display()))?;
    let hydration =
        build_song_hydration_commands(&song, audio.get_instrument_factory().sample_rate())?;
    let mut commands = Vec::with_capacity(hydration.len() + 1);
    commands.push(
        SequencerCmd::LoadSong {
            song: Arc::new(song.clone()),
        }
        .into(),
    );
    commands.extend(hydration);
    Ok((song, commands))
}

/// Queue commands that create backend instruments/effects from a serialized song.
#[cfg(feature = "device-host")]
pub fn hydrate_song(audio: &mut BlightAudio, song: &Song) -> Result<()> {
    let commands =
        build_song_hydration_commands(song, audio.get_instrument_factory().sample_rate())?;
    for command in commands {
        submit_command(audio, command)?;
    }
    Ok(())
}

#[cfg(feature = "device-host")]
fn submit_command(audio: &mut BlightAudio, command: Command) -> Result<()> {
    match audio.try_send_command(command) {
        Ok(()) => Ok(()),
        Err(error) => match error.kind() {
            CommandSubmissionErrorKind::Full => bail!("audio command queue is full"),
            CommandSubmissionErrorKind::Disconnected => {
                bail!("audio callback is disconnected")
            }
        },
    }
}

/// Build the same non-real-time hydration command sequence used by standalone playback.
///
/// Legacy models are first adapted into versioned definitions, then the built-in registry
/// validates and allocates every owner on NRT. The returned structural commands retain the
/// existing RT installation and deferred-retirement path.
pub fn build_song_hydration_commands(
    song: &Song,
    sample_rate: f32,
) -> std::result::Result<Vec<Command>, SongHydrationError> {
    let registry = BuiltInRegistry::new();
    let context = NrtPreparationContext::new(sample_rate);
    let mut commands = Vec::new();

    for instrument in &song.instrument_bank {
        let instrument_id = runtime_instrument_id(instrument.id)?;
        log::info!(
            "hydrating instrument {} ({})",
            instrument_id.raw(),
            instrument.name
        );
        let definition =
            adapt_legacy_instrument(instrument_id, &instrument.data).map_err(|source| {
                SongHydrationError::LegacyDefinition {
                    project_id: instrument.id,
                    source,
                }
            })?;
        let prepared = prepare_definition(&registry, &context, instrument.id, &definition)?;
        push_prepared_owner_commands(&mut commands, instrument.id, prepared)?;
    }

    Ok(commands)
}

fn prepare_definition(
    registry: &BuiltInRegistry,
    context: &NrtPreparationContext<'_>,
    project_id: usize,
    definition: &InstrumentDefinition,
) -> std::result::Result<PreparedInstrumentDefinition, SongHydrationError> {
    registry
        .prepare_definition(definition, context)
        .map_err(|source| SongHydrationError::RegistryPreparation { project_id, source })
}

fn push_prepared_owner_commands(
    commands: &mut Vec<Command>,
    project_id: usize,
    prepared: PreparedInstrumentDefinition,
) -> std::result::Result<(), SongHydrationError> {
    let instrument_id = prepared.instrument.id();
    commands.push(
        InstrumentCmd::AddInstrument {
            instrument: prepared.instrument,
        }
        .into(),
    );
    for effect in prepared.effects {
        let effect_id = effect.id();
        match effect {
            PreparedEffect::Mono(effect) => commands.push(
                InstrumentCmd::AddEffect {
                    instrument_id,
                    effect,
                }
                .into(),
            ),
            PreparedEffect::Stereo(_) => {
                return Err(SongHydrationError::UnsupportedPreparedEffectLayout {
                    project_id,
                    effect_id,
                    layout: EffectLayout::Stereo,
                });
            }
        }
    }
    Ok(())
}

fn runtime_instrument_id(model_id: usize) -> std::result::Result<InstrumentId, SongHydrationError> {
    // Tracker events persist instrument references as u8. Reject a bank ID that
    // events and the current UI cannot represent instead of hydrating a runtime
    // instrument that no tracker cell can address consistently.
    let project_id =
        u8::try_from(model_id).map_err(|_| SongHydrationError::UnaddressableInstrumentId {
            project_id: model_id,
        })?;
    Ok(InstrumentId::from_raw(u32::from(project_id)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use node_registry::{kind, InstrumentKindId, InvalidDefinitionCode, NodeCategory};
    use sequencer::models::{
        AmpEnvelopeParams, EffectType, Event, HiHatParams, Instrument, InstrumentData,
        SampleParams, SynthParams,
    };

    use crate::{EffectFactory, InstrumentFactory};

    fn project_instrument(id: usize) -> Instrument {
        Instrument {
            id,
            name: "typed ID fixture".to_owned(),
            data: InstrumentData::HiHat(HiHatParams {
                audio_effects: Vec::new(),
                amp_envelope: AmpEnvelopeParams::default(),
            }),
        }
    }

    #[test]
    fn project_numeric_ids_keep_their_json_shape_and_adapt_to_runtime_ids() {
        let instrument = project_instrument(17);
        let instrument_json = serde_json::to_value(&instrument).unwrap();
        assert_eq!(instrument_json["id"], serde_json::json!(17));
        let decoded: Instrument = serde_json::from_value(instrument_json).unwrap();
        assert_eq!(decoded.id, 17);
        assert_eq!(runtime_instrument_id(decoded.id).unwrap().raw(), 17);

        let event = Event {
            note: 60,
            volume: 100,
            instrument_id: 17,
            effect: EffectType::Arpeggio,
            effect_param: 0,
        };
        let event_json = serde_json::to_value(event).unwrap();
        assert_eq!(event_json["instrument_id"], serde_json::json!(17));
        let decoded: Event = serde_json::from_value(event_json).unwrap();
        assert_eq!(decoded.instrument_id, 17);
        assert_eq!(
            InstrumentId::from_raw(u32::from(decoded.instrument_id)).raw(),
            17
        );
    }

    #[test]
    fn hydration_rejects_instrument_ids_tracker_events_cannot_address() {
        let invalid = usize::from(u8::MAX) + 1;
        assert!(matches!(
            runtime_instrument_id(invalid),
            Err(SongHydrationError::UnaddressableInstrumentId { project_id })
                if project_id == invalid
        ));

        let mut song = Song::new("invalid tracker instrument ID");
        song.instrument_bank.push(project_instrument(invalid));
        let error = match build_song_hydration_commands(&song, 48_000.0) {
            Ok(_) => panic!("unaddressable project ID must be rejected"),
            Err(error) => error,
        };
        assert!(matches!(
            error,
            SongHydrationError::UnaddressableInstrumentId { project_id }
                if project_id == invalid
        ));
    }

    #[test]
    fn unsupported_legacy_owners_return_structured_adapter_diagnostics() {
        let envelope = sequencer::models::Envelope {
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
            let mut song = Song::new("unsupported legacy owner");
            song.instrument_bank.push(Instrument {
                id: 9,
                name: "unsupported".to_owned(),
                data,
            });
            let error = match build_song_hydration_commands(&song, 48_000.0) {
                Ok(_) => panic!("unsupported legacy owner must be rejected"),
                Err(error) => error,
            };
            assert!(matches!(
                error,
                SongHydrationError::LegacyDefinition {
                    project_id: 9,
                    source: LegacyDefinitionAdapterError::UnsupportedInstrument { .. },
                }
            ));
        }
    }

    #[test]
    fn unsupported_prepared_effect_layout_is_structured() {
        let project_id = 7_usize;
        let instrument_id = InstrumentId::from_raw(u32::try_from(project_id).unwrap());
        let effect_id = EffectId::from_raw(9);
        let prepared = PreparedInstrumentDefinition {
            instrument: InstrumentFactory::new(48_000.0).create_hihat(instrument_id, 0.0),
            effects: vec![PreparedEffect::Stereo(
                EffectFactory::new(48_000.0).create_stereo_gain(effect_id, 1.0),
            )],
        };
        let mut commands = Vec::new();

        let error = push_prepared_owner_commands(&mut commands, project_id, prepared).unwrap_err();

        assert!(matches!(
            error,
            SongHydrationError::UnsupportedPreparedEffectLayout {
                project_id: 7,
                effect_id: rejected_id,
                layout: EffectLayout::Stereo,
            } if rejected_id == effect_id
        ));
    }

    #[test]
    fn registry_unknown_and_invalid_definitions_remain_structured() {
        let instrument = project_instrument(5);
        let mut definition =
            adapt_legacy_instrument(InstrumentId::from_raw(5), &instrument.data).unwrap();
        definition.kind = InstrumentKindId::new("blight.instrument.unknown");
        let registry = BuiltInRegistry::new();
        let context = NrtPreparationContext::new(48_000.0);
        let error = match prepare_definition(&registry, &context, 5, &definition) {
            Ok(_) => panic!("unknown registry kind must be rejected"),
            Err(error) => error,
        };
        assert!(matches!(
            error,
            SongHydrationError::RegistryPreparation {
                project_id: 5,
                source: PreparationError::UnknownKind {
                    category: NodeCategory::Instrument,
                    instance_id: 5,
                    ..
                },
            }
        ));

        definition.kind = InstrumentKindId::new(kind::HI_HAT);
        definition.parameters.remove("pan");
        let error = match prepare_definition(&registry, &context, 5, &definition) {
            Ok(_) => panic!("invalid registry payload must be rejected"),
            Err(error) => error,
        };
        assert!(matches!(
            error,
            SongHydrationError::RegistryPreparation {
                project_id: 5,
                source: PreparationError::InvalidDefinition {
                    diagnostic,
                    ..
                },
            } if diagnostic.code == InvalidDefinitionCode::InvalidParameterPayload
        ));
    }
}
