//! Pure translation from the unversioned tracker model to versioned node definitions.
//!
//! This module deliberately does not prepare DSP owners or participate in active song
//! hydration. Effect identity is derived from one-based ordered legacy slots. For DFAM,
//! the implicit ladder occupies slot 1 and user-authored effects follow it.

use std::fmt;

use node_registry::{kind, EffectDefinition, InstrumentDefinition, ParameterPayload};
use sequencer::models::{
    AmpEnvelopeParams, AudioEffect, InstrumentData, PitchEnvelopeParams, Waveform,
};
use serde_json::Value;

use crate::{
    effects::{MAX_DELAY_SECONDS, MAX_TAPS},
    id::{EffectId, InstrumentId},
};

const FIRST_EFFECT_ID: u32 = 1;

/// A legacy value or variant that has no faithful versioned-definition representation.
#[derive(Clone, Debug, PartialEq)]
pub enum LegacyDefinitionAdapterError {
    UnsupportedInstrument { kind: &'static str },
    NonFiniteParameter { field: &'static str },
    EffectSlotOverflow,
}

impl fmt::Display for LegacyDefinitionAdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedInstrument { kind } => {
                write!(
                    formatter,
                    "legacy instrument kind `{kind}` has no faithful node definition"
                )
            }
            Self::NonFiniteParameter { field } => {
                write!(formatter, "legacy parameter `{field}` must be finite")
            }
            Self::EffectSlotOverflow => {
                formatter.write_str("legacy effect slot exceeds the EffectId representation")
            }
        }
    }
}

impl std::error::Error for LegacyDefinitionAdapterError {}

/// Adapt one current tracker instrument into a definition without preparing or installing it.
///
/// Amplitude and kick pitch envelopes are emitted into the current instrument payload version,
/// so registry preparation can configure the complete owner before it crosses to RT.
pub fn adapt_legacy_instrument(
    instance_id: InstrumentId,
    data: &InstrumentData,
) -> Result<InstrumentDefinition, LegacyDefinitionAdapterError> {
    let (instrument_kind, parameters, effects, has_implicit_ladder) = match data {
        InstrumentData::SimpleOscillator(parameters) => (
            kind::MONO_OSCILLATOR,
            payload([
                ("pan", Value::from(0.0)),
                ("waveform", Value::from(waveform_name(parameters.waveform))),
                (
                    "amplitude_envelope",
                    amplitude_envelope_payload(&parameters.amp_envelope)?,
                ),
            ]),
            parameters.audio_effects.as_slice(),
            false,
        ),
        InstrumentData::HiHat(parameters) => (
            kind::HI_HAT,
            pan_payload(&parameters.amp_envelope)?,
            parameters.audio_effects.as_slice(),
            false,
        ),
        InstrumentData::KickDrum(parameters) => (
            kind::KICK_DRUM,
            payload([
                ("pan", Value::from(0.0)),
                (
                    "amplitude_envelope",
                    amplitude_envelope_payload(&parameters.amp_envelope)?,
                ),
                (
                    "pitch_envelope",
                    pitch_envelope_payload(&parameters.pitch_envelope)?,
                ),
            ]),
            parameters.audio_effects.as_slice(),
            false,
        ),
        InstrumentData::SnareDrum(parameters) => (
            kind::SNARE_DRUM,
            pan_payload(&parameters.amp_envelope)?,
            parameters.audio_effects.as_slice(),
            false,
        ),
        InstrumentData::DFAM(parameters) => (
            kind::MOOG_DFAM,
            pan_payload(&parameters.amp_envelope)?,
            parameters.audio_effects.as_slice(),
            true,
        ),
        InstrumentData::Sample(_) => {
            return Err(LegacyDefinitionAdapterError::UnsupportedInstrument { kind: "sample" });
        }
        InstrumentData::Synth(_) => {
            return Err(LegacyDefinitionAdapterError::UnsupportedInstrument { kind: "synth" });
        }
    };

    let mut definitions = Vec::with_capacity(effects.len() + usize::from(has_implicit_ladder));
    if has_implicit_ladder {
        definitions.push(EffectDefinition::new(
            EffectId::from_raw(FIRST_EFFECT_ID),
            kind::MONO_MOOG_LADDER,
            payload([
                ("cutoff", Value::from(500.0)),
                ("resonance", Value::from(0.5)),
            ]),
        ));
    }
    for (index, effect) in effects.iter().enumerate() {
        let slot = index + usize::from(has_implicit_ladder);
        definitions.push(adapt_legacy_audio_effect(effect_id(slot)?, effect)?);
    }

    Ok(InstrumentDefinition::new(
        instance_id,
        instrument_kind,
        parameters,
        definitions,
    ))
}

/// Adapt one legacy effect with an already-derived stable slot identity.
pub fn adapt_legacy_audio_effect(
    instance_id: EffectId,
    effect: &AudioEffect,
) -> Result<EffectDefinition, LegacyDefinitionAdapterError> {
    let (effect_kind, parameters) = match effect {
        AudioEffect::Reverb {
            mix,
            decay_time,
            room_size,
            diffusion,
            damping,
        } => (
            kind::MONO_REVERB,
            payload([
                ("mix", normalized("mix", *mix, 0.0, 1.0)?),
                ("decay", normalized("decay_time", *decay_time, 0.0, 0.95)?),
                ("room_size", normalized("room_size", *room_size, 0.1, 3.0)?),
                ("damping", normalized("damping", *damping, 0.0, 1.0)?),
                ("diffusion", normalized("diffusion", *diffusion, 0.0, 0.95)?),
            ]),
        ),
        AudioEffect::Delay {
            time,
            num_taps,
            feedback,
            mix,
        } => (
            kind::MONO_DELAY,
            payload([
                (
                    "delay_seconds",
                    normalized("time", *time, 0.0, MAX_DELAY_SECONDS)?,
                ),
                (
                    "num_taps",
                    Value::from(usize::from(*num_taps).clamp(1, MAX_TAPS)),
                ),
                ("feedback", normalized("feedback", *feedback, 0.0, 0.95)?),
                ("mix", normalized("mix", *mix, 0.0, 1.0)?),
            ]),
        ),
    };
    Ok(EffectDefinition::new(instance_id, effect_kind, parameters))
}

fn effect_id(zero_based_slot: usize) -> Result<EffectId, LegacyDefinitionAdapterError> {
    let slot = u32::try_from(zero_based_slot)
        .ok()
        .and_then(|slot| slot.checked_add(FIRST_EFFECT_ID))
        .ok_or(LegacyDefinitionAdapterError::EffectSlotOverflow)?;
    Ok(EffectId::from_raw(slot))
}

fn normalized(
    field: &'static str,
    value: f32,
    min: f32,
    max: f32,
) -> Result<Value, LegacyDefinitionAdapterError> {
    finite(field, value).map(|value| Value::from(value.clamp(min, max)))
}

fn finite(field: &'static str, value: f32) -> Result<f32, LegacyDefinitionAdapterError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(LegacyDefinitionAdapterError::NonFiniteParameter { field })
    }
}

fn pan_payload(
    envelope: &AmpEnvelopeParams,
) -> Result<ParameterPayload, LegacyDefinitionAdapterError> {
    Ok(payload([
        ("pan", Value::from(0.0)),
        ("amplitude_envelope", amplitude_envelope_payload(envelope)?),
    ]))
}

fn amplitude_envelope_payload(
    envelope: &AmpEnvelopeParams,
) -> Result<Value, LegacyDefinitionAdapterError> {
    Ok(object(payload([
        (
            "attack_seconds",
            non_negative("amp_envelope.attack", envelope.attack)?,
        ),
        (
            "decay_seconds",
            non_negative("amp_envelope.decay", envelope.decay)?,
        ),
        (
            "sustain_level",
            normalized("amp_envelope.sustain", envelope.sustain, 0.0, 1.0)?,
        ),
        (
            "release_seconds",
            non_negative("amp_envelope.release", envelope.release)?,
        ),
    ])))
}

fn pitch_envelope_payload(
    envelope: &PitchEnvelopeParams,
) -> Result<Value, LegacyDefinitionAdapterError> {
    Ok(object(payload([
        (
            "frequency_delta_hz",
            Value::from(finite("pitch_envelope.freq_delta", envelope.freq_delta)?),
        ),
        (
            "decay_seconds",
            non_negative("pitch_envelope.decay_time", envelope.decay_time)?,
        ),
    ])))
}

fn non_negative(field: &'static str, value: f32) -> Result<Value, LegacyDefinitionAdapterError> {
    finite(field, value).map(|value| Value::from(value.max(0.0)))
}

fn object(payload: ParameterPayload) -> Value {
    Value::Object(payload.into_iter().collect())
}

fn payload<const N: usize>(entries: [(&str, Value); N]) -> ParameterPayload {
    entries
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value))
        .collect()
}

const fn waveform_name(waveform: Waveform) -> &'static str {
    match waveform {
        Waveform::Sine => "sine",
        Waveform::Square => "square",
        Waveform::Sawtooth => "sawtooth",
        Waveform::Triangle => "triangle",
        Waveform::NesTriangle => "nes_triangle",
    }
}
