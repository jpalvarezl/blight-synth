use crate::audio::{AudioManager, TRACKER_EFFECT_ID};
use crate::audio_utils::map_waveform_to_backend;
use audio_backend::effects::{DelayParameter as DP, ReverbParameter as RP};
use audio_backend::{EnvelopeCmd, SequencerCmd};
use sequencer::models::{
    AmpEnvelopeParams, AudioEffect, HiHatParams, InstrumentData, KickDrumParams,
    SimpleOscillatorParams, SnareDrumParams,
};

pub fn ensure_backend_instrument(audio_mgr: &mut AudioManager, id_u8: u8, data: &InstrumentData) {
    match data {
        InstrumentData::SimpleOscillator(params) => {
            ensure_backend_osc_with_params(audio_mgr, id_u8, params);
        }
        InstrumentData::HiHat(params) => {
            ensure_backend_hihat_with_params(audio_mgr, id_u8, params);
        }
        InstrumentData::KickDrum(params) => {
            ensure_backend_kick_with_params(audio_mgr, id_u8, params);
        }
        InstrumentData::SnareDrum(params) => {
            ensure_backend_snare_with_params(audio_mgr, id_u8, params);
        }
        InstrumentData::DFAM(params) => {
            ensure_backend_dfam_with_params(audio_mgr, id_u8, params);
        }
        _ => {}
    }
}

pub fn send_amp_envelope_to_backend(
    audio_mgr: &mut AudioManager,
    instrument_id: u8,
    env: &AmpEnvelopeParams,
) {
    if let Some(audio) = &mut audio_mgr.audio {
        let id = audio_backend::id::InstrumentId::from(instrument_id as u32);
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
                audio_backend::InstrumentCmd::PassOnSynthCmd {
                    instrument_id: id,
                    synth_cmd: audio_backend::SynthCmd::EnvelopeCommand {
                        envelope_id: Some(0),
                        command,
                    },
                }
                .into(),
            );
        }
    }
}

fn ensure_backend_osc_with_params(
    audio_mgr: &mut AudioManager,
    id_u8: u8,
    params: &SimpleOscillatorParams,
) {
    if let Some(audio) = &mut audio_mgr.audio {
        let backend_wave = map_waveform_to_backend(params.waveform);
        let id = audio_backend::id::InstrumentId::from(id_u8 as u32);
        let instrument = audio
            .get_instrument_factory()
            .create_oscillator_with_waveform(id, 0.0, backend_wave);
        audio.send_command(SequencerCmd::AddTrackInstrument { instrument }.into());
        apply_effects(audio, id, &params.audio_effects);
    }

    send_amp_envelope_to_backend(audio_mgr, id_u8, &params.amp_envelope);
}

fn ensure_backend_hihat_with_params(audio_mgr: &mut AudioManager, id_u8: u8, params: &HiHatParams) {
    if let Some(audio) = &mut audio_mgr.audio {
        let id = audio_backend::id::InstrumentId::from(id_u8 as u32);
        let instrument = audio.get_instrument_factory().create_hihat(id, 0.0);
        audio.send_command(SequencerCmd::AddTrackInstrument { instrument }.into());
        apply_effects(audio, id, &params.audio_effects);
    }

    send_amp_envelope_to_backend(audio_mgr, id_u8, &params.amp_envelope);
}

fn ensure_backend_kick_with_params(
    audio_mgr: &mut AudioManager,
    id_u8: u8,
    params: &KickDrumParams,
) {
    if let Some(audio) = &mut audio_mgr.audio {
        let id = audio_backend::id::InstrumentId::from(id_u8 as u32);
        let instrument = audio.get_instrument_factory().create_kick_drum(id, 0.0);
        audio.send_command(SequencerCmd::AddTrackInstrument { instrument }.into());
        apply_effects(audio, id, &params.audio_effects);

        audio.send_command(
            audio_backend::InstrumentCmd::PassOnSynthCmd {
                instrument_id: id,
                synth_cmd: audio_backend::SynthCmd::EnvelopeCommand {
                    envelope_id: None,
                    command: EnvelopeCmd::SetPitchEnvFreqDelta {
                        freq_delta: params.pitch_envelope.freq_delta,
                    },
                },
            }
            .into(),
        );
    }

    send_amp_envelope_to_backend(audio_mgr, id_u8, &params.amp_envelope);
}

fn ensure_backend_snare_with_params(
    audio_mgr: &mut AudioManager,
    id_u8: u8,
    params: &SnareDrumParams,
) {
    if let Some(audio) = &mut audio_mgr.audio {
        let id = audio_backend::id::InstrumentId::from(id_u8 as u32);
        let instrument = audio.get_instrument_factory().create_snare_drum(id, 0.0);
        audio.send_command(SequencerCmd::AddTrackInstrument { instrument }.into());
        apply_effects(audio, id, &params.audio_effects);
    }

    send_amp_envelope_to_backend(audio_mgr, id_u8, &params.amp_envelope);
}

fn ensure_backend_dfam_with_params(
    audio_mgr: &mut AudioManager,
    id_u8: u8,
    params: &sequencer::models::DFAMParams,
) {
    if let Some(audio) = &mut audio_mgr.audio {
        let id = audio_backend::id::InstrumentId::from(id_u8 as u32);
        let instrument = audio.get_instrument_factory().create_dfam(id, 0.0);
        audio.send_command(SequencerCmd::AddTrackInstrument { instrument }.into());

        let ladder = audio
            .get_effect_factory()
            .create_moog_ladder(TRACKER_EFFECT_ID, 500.0, 0.5);
        audio.send_command(
            SequencerCmd::AddEffectToInstrument {
                instrument_id: id,
                effect: ladder,
            }
            .into(),
        );

        apply_effects(audio, id, &params.audio_effects);
    }

    send_amp_envelope_to_backend(audio_mgr, id_u8, &params.amp_envelope);
}

fn apply_effects(
    audio: &mut audio_backend::BlightAudio,
    instrument_id: audio_backend::id::InstrumentId,
    effects: &[AudioEffect],
) {
    for eff in effects {
        match eff {
            AudioEffect::Reverb {
                mix,
                decay_time,
                room_size,
                diffusion,
                damping,
            } => {
                let mut r = audio
                    .get_effect_factory()
                    .create_mono_reverb(TRACKER_EFFECT_ID);
                audio_backend::MonoEffect::set_parameter(
                    &mut *r,
                    RP::Mix.as_index(),
                    (*mix).clamp(0.0, 1.0),
                );
                audio_backend::MonoEffect::set_parameter(
                    &mut *r,
                    RP::Decay.as_index(),
                    *decay_time,
                );
                audio_backend::MonoEffect::set_parameter(
                    &mut *r,
                    RP::RoomSize.as_index(),
                    *room_size,
                );
                audio_backend::MonoEffect::set_parameter(&mut *r, RP::Damping.as_index(), *damping);
                audio_backend::MonoEffect::set_parameter(
                    &mut *r,
                    RP::Diffusion.as_index(),
                    *diffusion,
                );

                audio.send_command(
                    SequencerCmd::AddEffectToInstrument {
                        instrument_id,
                        effect: r,
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
                let mut d = audio.get_effect_factory().create_mono_delay(
                    TRACKER_EFFECT_ID,
                    *time,
                    *num_taps as usize,
                    *feedback,
                    *mix,
                );
                audio_backend::MonoEffect::set_parameter(&mut *d, DP::Time.as_index(), *time);
                audio_backend::MonoEffect::set_parameter(
                    &mut *d,
                    DP::NumTaps.as_index(),
                    *num_taps as f32,
                );
                audio_backend::MonoEffect::set_parameter(
                    &mut *d,
                    DP::Feedback.as_index(),
                    *feedback,
                );
                audio_backend::MonoEffect::set_parameter(&mut *d, DP::Mix.as_index(), *mix);

                audio.send_command(
                    SequencerCmd::AddEffectToInstrument {
                        instrument_id,
                        effect: d,
                    }
                    .into(),
                );
            }
        }
    }
}
