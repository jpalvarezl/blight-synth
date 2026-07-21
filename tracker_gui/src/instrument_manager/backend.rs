use crate::audio::{AudioManager, TRACKER_EFFECT_ID, submit_command};
use crate::audio_utils::map_waveform_to_backend;
use audio_backend::effects::{DelayParameter as DP, ReverbParameter as RP};
use audio_backend::{BlightAudio, EnvelopeCmd, InstrumentCmd};
use sequencer::models::{
    AmpEnvelopeParams, AudioEffect, HiHatParams, InstrumentData, KickDrumParams,
    SimpleOscillatorParams, SnareDrumParams,
};

pub fn ensure_backend_instrument(audio_mgr: &mut AudioManager, id_u8: u8, data: &InstrumentData) {
    if let Some(audio) = &mut audio_mgr.audio {
        hydrate_instrument(audio, id_u8, data);
    }
}

pub fn hydrate_instrument(audio: &mut BlightAudio, id_u8: u8, data: &InstrumentData) {
    match data {
        InstrumentData::SimpleOscillator(params) => {
            hydrate_osc_with_params(audio, id_u8, params);
        }
        InstrumentData::HiHat(params) => {
            hydrate_hihat_with_params(audio, id_u8, params);
        }
        InstrumentData::KickDrum(params) => {
            hydrate_kick_with_params(audio, id_u8, params);
        }
        InstrumentData::SnareDrum(params) => {
            hydrate_snare_with_params(audio, id_u8, params);
        }
        InstrumentData::DFAM(params) => {
            hydrate_dfam_with_params(audio, id_u8, params);
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
        send_amp_envelope(audio, instrument_id, env);
    }
}

fn send_amp_envelope(audio: &mut BlightAudio, instrument_id: u8, env: &AmpEnvelopeParams) {
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
        submit_command(
            audio,
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

fn hydrate_osc_with_params(audio: &mut BlightAudio, id_u8: u8, params: &SimpleOscillatorParams) {
    let backend_wave = map_waveform_to_backend(params.waveform);
    let id = audio_backend::id::InstrumentId::from(id_u8 as u32);
    let instrument = audio
        .get_instrument_factory()
        .create_oscillator_with_waveform(id, 0.0, backend_wave);
    submit_command(audio, InstrumentCmd::AddInstrument { instrument }.into());
    apply_effects(audio, id, &params.audio_effects);

    send_amp_envelope(audio, id_u8, &params.amp_envelope);
}

fn hydrate_hihat_with_params(audio: &mut BlightAudio, id_u8: u8, params: &HiHatParams) {
    let id = audio_backend::id::InstrumentId::from(id_u8 as u32);
    let instrument = audio.get_instrument_factory().create_hihat(id, 0.0);
    submit_command(audio, InstrumentCmd::AddInstrument { instrument }.into());
    apply_effects(audio, id, &params.audio_effects);

    send_amp_envelope(audio, id_u8, &params.amp_envelope);
}

fn hydrate_kick_with_params(audio: &mut BlightAudio, id_u8: u8, params: &KickDrumParams) {
    let id = audio_backend::id::InstrumentId::from(id_u8 as u32);
    let instrument = audio.get_instrument_factory().create_kick_drum(id, 0.0);
    submit_command(audio, InstrumentCmd::AddInstrument { instrument }.into());
    apply_effects(audio, id, &params.audio_effects);

    submit_command(
        audio,
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

    send_amp_envelope(audio, id_u8, &params.amp_envelope);
}

fn hydrate_snare_with_params(audio: &mut BlightAudio, id_u8: u8, params: &SnareDrumParams) {
    let id = audio_backend::id::InstrumentId::from(id_u8 as u32);
    let instrument = audio.get_instrument_factory().create_snare_drum(id, 0.0);
    submit_command(audio, InstrumentCmd::AddInstrument { instrument }.into());
    apply_effects(audio, id, &params.audio_effects);

    send_amp_envelope(audio, id_u8, &params.amp_envelope);
}

fn hydrate_dfam_with_params(
    audio: &mut BlightAudio,
    id_u8: u8,
    params: &sequencer::models::DFAMParams,
) {
    let id = audio_backend::id::InstrumentId::from(id_u8 as u32);
    let instrument = audio.get_instrument_factory().create_dfam(id, 0.0);
    submit_command(audio, InstrumentCmd::AddInstrument { instrument }.into());

    let ladder = audio
        .get_effect_factory()
        .create_moog_ladder(TRACKER_EFFECT_ID, 500.0, 0.5);
    submit_command(
        audio,
        InstrumentCmd::AddEffect {
            instrument_id: id,
            effect: ladder,
        }
        .into(),
    );

    apply_effects(audio, id, &params.audio_effects);

    send_amp_envelope(audio, id_u8, &params.amp_envelope);
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

                submit_command(
                    audio,
                    InstrumentCmd::AddEffect {
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

                submit_command(
                    audio,
                    InstrumentCmd::AddEffect {
                        instrument_id,
                        effect: d,
                    }
                    .into(),
                );
            }
        }
    }
}
