use anyhow::{bail, Context, Result};
use sequencer::{
    cli::FileFormat,
    models::{AmpEnvelopeParams, AudioEffect, InstrumentData, Song, Waveform},
    project::open_song_from_file,
};
use std::{path::Path, sync::Arc};

use crate::{
    effects::{DelayParameter as DP, ReverbParameter as RP},
    id::{EffectId, InstrumentId},
    instruments::Waveform as BackendWaveform,
    BlightAudio, EnvelopeCmd, InstrumentCmd, MonoEffect, SequencerCmd, SynthCmd,
};

const DEFAULT_INSTRUMENT_EFFECT_ID: EffectId = 1;

/// Load a JSON song file and install it into the audio backend without starting playback.
///
/// This is the shared path for standalone OSC `/song/load` and examples. It queues a
/// `SequencerCmd::LoadSong` first, then queues instrument hydration commands.
pub fn load_song_file_into_audio(audio: &mut BlightAudio, path: &Path) -> Result<Song> {
    log::info!("loading song from {}", path.display());
    let song = open_song_from_file(&path.to_path_buf(), &FileFormat::Json)
        .with_context(|| format!("failed to load song from {}", path.display()))?;

    audio.send_command(
        SequencerCmd::LoadSong {
            song: Arc::new(song.clone()),
        }
        .into(),
    );
    hydrate_song(audio, &song)?;

    Ok(song)
}

/// Queue commands that create backend instruments/effects from a serialized song.
pub fn hydrate_song(audio: &mut BlightAudio, song: &Song) -> Result<()> {
    for instrument in &song.instrument_bank {
        let instrument_id = instrument.id as InstrumentId;
        log::info!(
            "hydrating instrument {} ({})",
            instrument_id,
            instrument.name
        );
        match &instrument.data {
            InstrumentData::SimpleOscillator(params) => {
                let instrument = audio
                    .get_instrument_factory()
                    .create_oscillator_with_waveform(
                        instrument_id,
                        0.0,
                        map_waveform_to_backend(params.waveform),
                    );
                audio.send_command(SequencerCmd::AddTrackInstrument { instrument }.into());
                apply_effects(audio, instrument_id, &params.audio_effects);
                send_amp_envelope(audio, instrument_id, &params.amp_envelope);
            }
            InstrumentData::HiHat(params) => {
                let instrument = audio
                    .get_instrument_factory()
                    .create_hihat(instrument_id, 0.0);
                audio.send_command(SequencerCmd::AddTrackInstrument { instrument }.into());
                apply_effects(audio, instrument_id, &params.audio_effects);
                send_amp_envelope(audio, instrument_id, &params.amp_envelope);
            }
            InstrumentData::KickDrum(params) => {
                let instrument = audio
                    .get_instrument_factory()
                    .create_kick_drum(instrument_id, 0.0);
                audio.send_command(SequencerCmd::AddTrackInstrument { instrument }.into());
                apply_effects(audio, instrument_id, &params.audio_effects);

                audio.send_command(
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

                send_amp_envelope(audio, instrument_id, &params.amp_envelope);
            }
            InstrumentData::SnareDrum(params) => {
                let instrument = audio
                    .get_instrument_factory()
                    .create_snare_drum(instrument_id, 0.0);
                audio.send_command(SequencerCmd::AddTrackInstrument { instrument }.into());
                apply_effects(audio, instrument_id, &params.audio_effects);
                send_amp_envelope(audio, instrument_id, &params.amp_envelope);
            }
            InstrumentData::DFAM(params) => {
                let instrument = audio
                    .get_instrument_factory()
                    .create_dfam(instrument_id, 0.0);
                audio.send_command(SequencerCmd::AddTrackInstrument { instrument }.into());

                let ladder = audio.get_effect_factory().create_moog_ladder(
                    DEFAULT_INSTRUMENT_EFFECT_ID,
                    500.0,
                    0.5,
                );
                audio.send_command(
                    SequencerCmd::AddEffectToInstrument {
                        instrument_id,
                        effect: ladder,
                    }
                    .into(),
                );

                apply_effects(audio, instrument_id, &params.audio_effects);
                send_amp_envelope(audio, instrument_id, &params.amp_envelope);
            }
            unsupported => {
                bail!("unsupported instrument type in song hydration: {unsupported:?}");
            }
        }
    }

    Ok(())
}

fn send_amp_envelope(
    audio: &mut BlightAudio,
    instrument_id: InstrumentId,
    env: &AmpEnvelopeParams,
) {
    for command in [
        EnvelopeCmd::SetAttack { attack: env.attack },
        EnvelopeCmd::SetDecay { decay: env.decay },
        EnvelopeCmd::SetSustain {
            sustain: env.sustain,
        },
        EnvelopeCmd::SetRelease {
            release: env.release,
        },
    ] {
        audio.send_command(
            InstrumentCmd::PassOnSynthCmd {
                instrument_id,
                synth_cmd: SynthCmd::EnvelopeCommand {
                    envelope_id: Some(0),
                    command,
                },
            }
            .into(),
        );
    }
}

fn apply_effects(audio: &mut BlightAudio, instrument_id: InstrumentId, effects: &[AudioEffect]) {
    for effect in effects {
        match effect {
            AudioEffect::Reverb {
                mix,
                decay_time,
                room_size,
                diffusion,
                damping,
            } => {
                let mut reverb = audio
                    .get_effect_factory()
                    .create_mono_reverb(DEFAULT_INSTRUMENT_EFFECT_ID);
                MonoEffect::set_parameter(&mut *reverb, RP::Mix.as_index(), (*mix).clamp(0.0, 1.0));
                MonoEffect::set_parameter(&mut *reverb, RP::Decay.as_index(), *decay_time);
                MonoEffect::set_parameter(&mut *reverb, RP::RoomSize.as_index(), *room_size);
                MonoEffect::set_parameter(&mut *reverb, RP::Damping.as_index(), *damping);
                MonoEffect::set_parameter(&mut *reverb, RP::Diffusion.as_index(), *diffusion);

                audio.send_command(
                    SequencerCmd::AddEffectToInstrument {
                        instrument_id,
                        effect: reverb,
                    }
                    .into(),
                );
            }
            AudioEffect::Delay {
                time,
                num_taps,
                feedback,
                mix,
            } => {
                let mut delay = audio.get_effect_factory().create_mono_delay(
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

                audio.send_command(
                    SequencerCmd::AddEffectToInstrument {
                        instrument_id,
                        effect: delay,
                    }
                    .into(),
                );
            }
        }
    }
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
