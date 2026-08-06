//! Pure translation from the unversioned tracker model to versioned node definitions.
//!
//! This module deliberately does not prepare DSP owners or participate in active song
//! hydration. Effect identity is derived from one-based ordered legacy slots. For DFAM,
//! the implicit ladder occupies slot 1 and user-authored effects follow it.

use node_registry::{kind, EffectDefinition, InstrumentDefinition, ParameterPayload};
use sequencer::models::{AudioEffect, InstrumentData, Waveform};
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

/// Adapt one current tracker instrument into a definition without preparing or installing it.
///
/// Registry constructor payloads do not contain legacy amplitude/pitch envelopes. Those remain
/// explicit hydration commands for #222 rather than being silently invented as unknown fields.
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
            ]),
            parameters.audio_effects.as_slice(),
            false,
        ),
        InstrumentData::HiHat(parameters) => (
            kind::HI_HAT,
            pan_payload(),
            parameters.audio_effects.as_slice(),
            false,
        ),
        InstrumentData::KickDrum(parameters) => (
            kind::KICK_DRUM,
            pan_payload(),
            parameters.audio_effects.as_slice(),
            false,
        ),
        InstrumentData::SnareDrum(parameters) => (
            kind::SNARE_DRUM,
            pan_payload(),
            parameters.audio_effects.as_slice(),
            false,
        ),
        InstrumentData::DFAM(parameters) => (
            kind::MOOG_DFAM,
            pan_payload(),
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

fn pan_payload() -> ParameterPayload {
    payload([("pan", Value::from(0.0))])
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
