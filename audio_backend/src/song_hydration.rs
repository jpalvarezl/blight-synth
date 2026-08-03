#[cfg(feature = "device-host")]
use anyhow::Context;
use anyhow::{bail, Result};
use sequencer::models::{AmpEnvelopeParams, AudioEffect, InstrumentData, Song, Waveform};
#[cfg(feature = "device-host")]
use sequencer::{cli::FileFormat, project::open_song_from_file};
#[cfg(feature = "device-host")]
use std::{path::Path, sync::Arc};

use crate::{
    effects::{DelayParameter as DP, ReverbParameter as RP},
    id::{EffectId, EnvelopeId, InstrumentId},
    instruments::Waveform as BackendWaveform,
    Command, EffectFactory, EnvelopeCmd, InstrumentCmd, InstrumentFactory, MonoEffect, SynthCmd,
};
#[cfg(feature = "device-host")]
use crate::{BlightAudio, CommandSubmissionErrorKind, SequencerCmd};

const DEFAULT_INSTRUMENT_EFFECT_ID: EffectId = EffectId::from_raw(1);

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
    let hydration = build_hydration_commands_with_factories(
        &song,
        audio.get_instrument_factory(),
        audio.get_effect_factory(),
    )?;
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
    let commands = build_hydration_commands_with_factories(
        song,
        audio.get_instrument_factory(),
        audio.get_effect_factory(),
    )?;
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
/// Offline hosts use this entry point so they do not need to construct a CPAL-backed
/// [`BlightAudio`] instance merely to hydrate instruments and effects.
pub fn build_song_hydration_commands(song: &Song, sample_rate: f32) -> Result<Vec<Command>> {
    let instrument_factory = InstrumentFactory::new(sample_rate);
    let effect_factory = EffectFactory::new(sample_rate);
    build_hydration_commands_with_factories(song, &instrument_factory, &effect_factory)
}

fn build_hydration_commands_with_factories(
    song: &Song,
    instrument_factory: &InstrumentFactory,
    effect_factory: &EffectFactory,
) -> Result<Vec<Command>> {
    let mut commands = Vec::new();

    for instrument in &song.instrument_bank {
        let instrument_id = runtime_instrument_id(instrument.id)?;
        log::info!(
            "hydrating instrument {} ({})",
            instrument_id.raw(),
            instrument.name
        );
        match &instrument.data {
            InstrumentData::SimpleOscillator(params) => {
                commands.push(
                    InstrumentCmd::AddInstrument {
                        instrument: instrument_factory.create_oscillator_with_waveform(
                            instrument_id,
                            0.0,
                            map_waveform_to_backend(params.waveform),
                        ),
                    }
                    .into(),
                );
                push_effect_commands(
                    &mut commands,
                    effect_factory,
                    instrument_id,
                    &params.audio_effects,
                );
                push_amp_envelope_commands(&mut commands, instrument_id, &params.amp_envelope);
            }
            InstrumentData::HiHat(params) => {
                commands.push(
                    InstrumentCmd::AddInstrument {
                        instrument: instrument_factory.create_hihat(instrument_id, 0.0),
                    }
                    .into(),
                );
                push_effect_commands(
                    &mut commands,
                    effect_factory,
                    instrument_id,
                    &params.audio_effects,
                );
                push_amp_envelope_commands(&mut commands, instrument_id, &params.amp_envelope);
            }
            InstrumentData::KickDrum(params) => {
                commands.push(
                    InstrumentCmd::AddInstrument {
                        instrument: instrument_factory.create_kick_drum(instrument_id, 0.0),
                    }
                    .into(),
                );
                push_effect_commands(
                    &mut commands,
                    effect_factory,
                    instrument_id,
                    &params.audio_effects,
                );
                commands.push(
                    InstrumentCmd::PassOnSynthCmd {
                        instrument_id,
                        synth_cmd: SynthCmd::EnvelopeCommand {
                            envelope_id: None,
                            command: EnvelopeCmd::SetPitchEnvFreqDelta {
                                freq_delta: params.pitch_envelope.freq_delta,
                            },
                        },
                    }
                    .into(),
                );
                push_amp_envelope_commands(&mut commands, instrument_id, &params.amp_envelope);
            }
            InstrumentData::SnareDrum(params) => {
                commands.push(
                    InstrumentCmd::AddInstrument {
                        instrument: instrument_factory.create_snare_drum(instrument_id, 0.0),
                    }
                    .into(),
                );
                push_effect_commands(
                    &mut commands,
                    effect_factory,
                    instrument_id,
                    &params.audio_effects,
                );
                push_amp_envelope_commands(&mut commands, instrument_id, &params.amp_envelope);
            }
            InstrumentData::DFAM(params) => {
                commands.push(
                    InstrumentCmd::AddInstrument {
                        instrument: instrument_factory.create_dfam(instrument_id, 0.0),
                    }
                    .into(),
                );
                commands.push(
                    InstrumentCmd::AddEffect {
                        instrument_id,
                        effect: effect_factory.create_moog_ladder(
                            DEFAULT_INSTRUMENT_EFFECT_ID,
                            500.0,
                            0.5,
                        ),
                    }
                    .into(),
                );
                push_effect_commands(
                    &mut commands,
                    effect_factory,
                    instrument_id,
                    &params.audio_effects,
                );
                push_amp_envelope_commands(&mut commands, instrument_id, &params.amp_envelope);
            }
            unsupported => {
                bail!("unsupported instrument type in song hydration: {unsupported:?}");
            }
        }
    }

    Ok(commands)
}

fn push_amp_envelope_commands(
    commands: &mut Vec<Command>,
    instrument_id: InstrumentId,
    envelope: &AmpEnvelopeParams,
) {
    for command in [
        EnvelopeCmd::SetAttack {
            attack: envelope.attack,
        },
        EnvelopeCmd::SetDecay {
            decay: envelope.decay,
        },
        EnvelopeCmd::SetSustain {
            sustain: envelope.sustain,
        },
        EnvelopeCmd::SetRelease {
            release: envelope.release,
        },
    ] {
        commands.push(
            InstrumentCmd::PassOnSynthCmd {
                instrument_id,
                synth_cmd: SynthCmd::EnvelopeCommand {
                    envelope_id: Some(EnvelopeId::from_raw(0)),
                    command,
                },
            }
            .into(),
        );
    }
}

fn push_effect_commands(
    commands: &mut Vec<Command>,
    effect_factory: &EffectFactory,
    instrument_id: InstrumentId,
    effects: &[AudioEffect],
) {
    for effect in effects {
        let effect = match effect {
            AudioEffect::Reverb {
                mix,
                decay_time,
                room_size,
                diffusion,
                damping,
            } => {
                let mut reverb = effect_factory.create_mono_reverb(DEFAULT_INSTRUMENT_EFFECT_ID);
                MonoEffect::set_parameter(&mut *reverb, RP::Mix.as_index(), (*mix).clamp(0.0, 1.0));
                MonoEffect::set_parameter(&mut *reverb, RP::Decay.as_index(), *decay_time);
                MonoEffect::set_parameter(&mut *reverb, RP::RoomSize.as_index(), *room_size);
                MonoEffect::set_parameter(&mut *reverb, RP::Damping.as_index(), *damping);
                MonoEffect::set_parameter(&mut *reverb, RP::Diffusion.as_index(), *diffusion);
                reverb
            }
            AudioEffect::Delay {
                time,
                num_taps,
                feedback,
                mix,
            } => {
                let mut delay = effect_factory.create_mono_delay(
                    DEFAULT_INSTRUMENT_EFFECT_ID,
                    *time,
                    *num_taps as usize,
                    *feedback,
                    *mix,
                );
                MonoEffect::set_parameter(&mut *delay, DP::Time.as_index(), *time);
                MonoEffect::set_parameter(&mut *delay, DP::NumTaps.as_index(), *num_taps as f32);
                MonoEffect::set_parameter(&mut *delay, DP::Feedback.as_index(), *feedback);
                MonoEffect::set_parameter(&mut *delay, DP::Mix.as_index(), *mix);
                delay
            }
        };
        commands.push(
            InstrumentCmd::AddEffect {
                instrument_id,
                effect,
            }
            .into(),
        );
    }
}

fn runtime_instrument_id(model_id: usize) -> Result<InstrumentId> {
    let raw = u32::try_from(model_id).map_err(|_| {
        anyhow::anyhow!("project instrument ID {model_id} exceeds the runtime u32 range")
    })?;
    Ok(InstrumentId::from_raw(raw))
}

fn map_waveform_to_backend(waveform: Waveform) -> BackendWaveform {
    match waveform {
        Waveform::Sine => BackendWaveform::Sine,
        Waveform::Square => BackendWaveform::Square,
        Waveform::Sawtooth => BackendWaveform::Sawtooth,
        Waveform::Triangle => BackendWaveform::Triangle,
        Waveform::NesTriangle => BackendWaveform::NesTriangle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sequencer::models::{EffectType, Event, HiHatParams, Instrument};

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

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn hydration_rejects_project_instrument_ids_wider_than_runtime_ids() {
        let invalid = usize::try_from(u64::from(u32::MAX) + 1).unwrap();
        assert!(runtime_instrument_id(invalid).is_err());

        let mut song = Song::new("invalid runtime ID");
        song.instrument_bank.push(project_instrument(invalid));
        let error = match build_song_hydration_commands(&song, 48_000.0) {
            Ok(_) => panic!("oversized project ID must be rejected"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("exceeds the runtime u32 range"));
    }
}
