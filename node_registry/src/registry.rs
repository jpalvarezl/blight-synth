#![allow(
    deprecated,
    reason = "the registry inventories existing deprecated DSP factory kinds"
)]

use std::sync::Arc;

use dsp::{
    effects::{DistortionType, FilterType, ReverbParameter, MAX_DELAY_SECONDS, MAX_TAPS},
    id::{EffectId, SampleId},
    instruments::Waveform,
    EffectFactory, InstrumentFactory, InstrumentTrait, MonoEffect, SampleData, StereoEffect,
};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::{Map, Value};

use crate::{
    EffectDefinition, InstrumentDefinition, InvalidDefinitionCode, InvalidDefinitionDiagnostic,
    NodeCategory, PreparationError, NODE_DEFINITION_SCHEMA_VERSION,
};

/// Stable built-in kind IDs. These strings are persistence/protocol identities:
/// never rename or reuse one for a different implementation.
pub mod kind {
    pub const MONO_OSCILLATOR: &str = "blight.instrument.oscillator.mono";
    pub const POLYPHONIC_OSCILLATOR: &str = "blight.instrument.oscillator.polyphonic";
    pub const HI_HAT: &str = "blight.instrument.hi_hat";
    pub const KICK_DRUM: &str = "blight.instrument.kick_drum";
    pub const SNARE_DRUM: &str = "blight.instrument.snare_drum";
    pub const MOOG_DFAM: &str = "blight.instrument.moog_dfam";
    pub const ONE_SHOT_SAMPLE_PLAYER: &str = "blight.instrument.sample_player.one_shot";
    pub const LOOP_SAMPLE_PLAYER: &str = "blight.instrument.sample_player.loop";

    pub const MONO_REVERB: &str = "blight.effect.reverb.mono";
    pub const STEREO_REVERB: &str = "blight.effect.reverb.stereo";
    pub const MONO_DELAY: &str = "blight.effect.delay.mono";
    pub const MONO_DISTORTION: &str = "blight.effect.distortion.mono";
    pub const MONO_FILTER: &str = "blight.effect.filter.mono";
    pub const MONO_GAIN: &str = "blight.effect.gain.mono";
    pub const STEREO_GAIN: &str = "blight.effect.gain.stereo";
    pub const MONO_MOOG_LADDER: &str = "blight.effect.moog_ladder.mono";
}

const V1: &[u32] = &[NODE_DEFINITION_SCHEMA_VERSION];

/// Resolves a typed sample resource ID to already decoded immutable sample data.
/// Implementations and all calls are NRT-only.
pub trait SampleResolver {
    fn resolve_sample(&self, id: SampleId) -> Option<Arc<SampleData>>;
}

/// Inputs required while allocating and validating prepared DSP owners.
///
/// This type is deliberately named NRT: creating nodes can allocate large delay
/// lines, clone `Arc`s, validate JSON payloads, and fail richly. Neither this
/// context nor [`BuiltInRegistry`] belongs in a callback-reachable owner.
pub struct NrtPreparationContext<'a> {
    sample_rate: f32,
    samples: Option<&'a dyn SampleResolver>,
}

impl<'a> NrtPreparationContext<'a> {
    /// Creates NRT preparation inputs. Invalid sample rates are reported against
    /// the definition passed to a registry preparation method.
    #[must_use]
    pub const fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            samples: None,
        }
    }

    /// Adds an NRT resolver for sample-player definitions.
    #[must_use]
    pub const fn with_sample_resolver(mut self, samples: &'a dyn SampleResolver) -> Self {
        self.samples = Some(samples);
        self
    }

    #[must_use]
    pub const fn sample_rate(&self) -> f32 {
        self.sample_rate
    }
}

/// Channel ownership of one prepared effect implementation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectLayout {
    Mono,
    Stereo,
}

/// NRT-prepared ownership of either current DSP effect trait.
pub enum PreparedEffect {
    Mono(Box<dyn MonoEffect>),
    Stereo(Box<dyn StereoEffect>),
}

impl PreparedEffect {
    #[must_use]
    pub fn id(&self) -> EffectId {
        match self {
            Self::Mono(effect) => effect.id(),
            Self::Stereo(effect) => effect.id(),
        }
    }

    #[must_use]
    pub const fn layout(&self) -> EffectLayout {
        match self {
            Self::Mono(_) => EffectLayout::Mono,
            Self::Stereo(_) => EffectLayout::Stereo,
        }
    }
}

/// NRT-prepared owners for one instrument definition. Effects remain ordered but
/// are not installed or routed; those semantics belong to issue #136.
pub struct PreparedInstrumentDefinition {
    pub instrument: Box<dyn InstrumentTrait>,
    pub effects: Vec<PreparedEffect>,
}

/// Public read-only inventory item derived from a single static registration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuiltInKindDescriptor {
    pub category: NodeCategory,
    pub kind: &'static str,
    pub supported_schema_versions: &'static [u32],
    pub effect_layout: Option<EffectLayout>,
    /// Existing deprecated DSP factories remain discoverable for compatibility,
    /// but callers should not author new definitions using them.
    pub deprecated: bool,
}

type InstrumentBuilder = fn(
    &InstrumentDefinition,
    &NrtPreparationContext<'_>,
) -> Result<Box<dyn InstrumentTrait>, InvalidDefinitionDiagnostic>;
type EffectBuilder = fn(
    &EffectDefinition,
    &NrtPreparationContext<'_>,
) -> Result<PreparedEffect, InvalidDefinitionDiagnostic>;

struct InstrumentRegistration {
    kind: &'static str,
    versions: &'static [u32],
    deprecated: bool,
    builder: InstrumentBuilder,
}

struct EffectRegistration {
    kind: &'static str,
    versions: &'static [u32],
    layout: EffectLayout,
    deprecated: bool,
    builder: EffectBuilder,
}

// Adding a statically compiled instrument/effect requires one registration here
// (plus its local payload parser/builder). Host, OSC, UI, and tracker type
// switches are intentionally absent.
static INSTRUMENTS: &[InstrumentRegistration] = &[
    InstrumentRegistration {
        kind: kind::MONO_OSCILLATOR,
        versions: V1,
        deprecated: false,
        builder: build_mono_oscillator,
    },
    InstrumentRegistration {
        kind: kind::POLYPHONIC_OSCILLATOR,
        versions: V1,
        deprecated: false,
        builder: build_polyphonic_oscillator,
    },
    InstrumentRegistration {
        kind: kind::HI_HAT,
        versions: V1,
        deprecated: false,
        builder: build_hi_hat,
    },
    InstrumentRegistration {
        kind: kind::KICK_DRUM,
        versions: V1,
        deprecated: false,
        builder: build_kick_drum,
    },
    InstrumentRegistration {
        kind: kind::SNARE_DRUM,
        versions: V1,
        deprecated: false,
        builder: build_snare_drum,
    },
    InstrumentRegistration {
        kind: kind::MOOG_DFAM,
        versions: V1,
        deprecated: false,
        builder: build_moog_dfam,
    },
    InstrumentRegistration {
        kind: kind::ONE_SHOT_SAMPLE_PLAYER,
        versions: V1,
        deprecated: false,
        builder: build_one_shot_sample_player,
    },
    InstrumentRegistration {
        kind: kind::LOOP_SAMPLE_PLAYER,
        versions: V1,
        deprecated: false,
        builder: build_loop_sample_player,
    },
];

static EFFECTS: &[EffectRegistration] = &[
    EffectRegistration {
        kind: kind::MONO_REVERB,
        versions: V1,
        layout: EffectLayout::Mono,
        deprecated: false,
        builder: build_mono_reverb,
    },
    EffectRegistration {
        kind: kind::STEREO_REVERB,
        versions: V1,
        layout: EffectLayout::Stereo,
        deprecated: false,
        builder: build_stereo_reverb,
    },
    EffectRegistration {
        kind: kind::MONO_DELAY,
        versions: V1,
        layout: EffectLayout::Mono,
        deprecated: false,
        builder: build_mono_delay,
    },
    EffectRegistration {
        kind: kind::MONO_DISTORTION,
        versions: V1,
        layout: EffectLayout::Mono,
        deprecated: true,
        builder: build_mono_distortion,
    },
    EffectRegistration {
        kind: kind::MONO_FILTER,
        versions: V1,
        layout: EffectLayout::Mono,
        deprecated: true,
        builder: build_mono_filter,
    },
    EffectRegistration {
        kind: kind::MONO_GAIN,
        versions: V1,
        layout: EffectLayout::Mono,
        deprecated: false,
        builder: build_mono_gain,
    },
    EffectRegistration {
        kind: kind::STEREO_GAIN,
        versions: V1,
        layout: EffectLayout::Stereo,
        deprecated: false,
        builder: build_stereo_gain,
    },
    EffectRegistration {
        kind: kind::MONO_MOOG_LADDER,
        versions: V1,
        layout: EffectLayout::Mono,
        deprecated: false,
        builder: build_moog_ladder,
    },
];

/// Statically compiled resolver for built-in definitions.
///
/// The registry has no dynamic loading path and stores no host-specific type
/// knowledge. Every preparation method is NRT-only and returns allocated DSP
/// owners ready for a later bounded handoff/installation.
#[derive(Clone, Copy, Debug, Default)]
pub struct BuiltInRegistry;

impl BuiltInRegistry {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    pub fn instrument_kinds() -> impl ExactSizeIterator<Item = BuiltInKindDescriptor> {
        INSTRUMENTS
            .iter()
            .map(|registration| BuiltInKindDescriptor {
                category: NodeCategory::Instrument,
                kind: registration.kind,
                supported_schema_versions: registration.versions,
                effect_layout: None,
                deprecated: registration.deprecated,
            })
    }

    pub fn effect_kinds() -> impl ExactSizeIterator<Item = BuiltInKindDescriptor> {
        EFFECTS.iter().map(|registration| BuiltInKindDescriptor {
            category: NodeCategory::Effect,
            kind: registration.kind,
            supported_schema_versions: registration.versions,
            effect_layout: Some(registration.layout),
            deprecated: registration.deprecated,
        })
    }

    /// Allocates and validates one instrument owner on NRT.
    pub fn prepare_instrument(
        &self,
        definition: &InstrumentDefinition,
        context: &NrtPreparationContext<'_>,
    ) -> Result<Box<dyn InstrumentTrait>, PreparationError> {
        validate_kind_id(
            NodeCategory::Instrument,
            definition.kind.as_str(),
            definition.instance_id.raw(),
            definition.schema_version,
        )?;
        let registration = INSTRUMENTS
            .iter()
            .find(|registration| registration.kind == definition.kind.as_str())
            .ok_or_else(|| PreparationError::UnknownKind {
                category: NodeCategory::Instrument,
                kind: definition.kind.to_string(),
                instance_id: definition.instance_id.raw(),
            })?;
        validate_version(
            NodeCategory::Instrument,
            registration.kind,
            definition.instance_id.raw(),
            definition.schema_version,
            registration.versions,
        )?;
        validate_context(
            NodeCategory::Instrument,
            registration.kind,
            definition.instance_id.raw(),
            definition.schema_version,
            context,
        )?;
        (registration.builder)(definition, context).map_err(|diagnostic| {
            invalid_error(
                NodeCategory::Instrument,
                registration.kind,
                definition.instance_id.raw(),
                definition.schema_version,
                diagnostic,
            )
        })
    }

    /// Allocates and validates one effect owner on NRT.
    pub fn prepare_effect(
        &self,
        definition: &EffectDefinition,
        context: &NrtPreparationContext<'_>,
    ) -> Result<PreparedEffect, PreparationError> {
        validate_kind_id(
            NodeCategory::Effect,
            definition.kind.as_str(),
            definition.instance_id.raw(),
            definition.schema_version,
        )?;
        let registration = EFFECTS
            .iter()
            .find(|registration| registration.kind == definition.kind.as_str())
            .ok_or_else(|| PreparationError::UnknownKind {
                category: NodeCategory::Effect,
                kind: definition.kind.to_string(),
                instance_id: definition.instance_id.raw(),
            })?;
        validate_version(
            NodeCategory::Effect,
            registration.kind,
            definition.instance_id.raw(),
            definition.schema_version,
            registration.versions,
        )?;
        validate_context(
            NodeCategory::Effect,
            registration.kind,
            definition.instance_id.raw(),
            definition.schema_version,
            context,
        )?;
        (registration.builder)(definition, context).map_err(|diagnostic| {
            invalid_error(
                NodeCategory::Effect,
                registration.kind,
                definition.instance_id.raw(),
                definition.schema_version,
                diagnostic,
            )
        })
    }

    /// Prepares an ordered chain. Same-kind entries are valid; reusing an
    /// `EffectId` is rejected because it would make parameter targeting
    /// ambiguous. No routing or installation occurs here.
    pub fn prepare_effect_chain(
        &self,
        definitions: &[EffectDefinition],
        context: &NrtPreparationContext<'_>,
    ) -> Result<Vec<PreparedEffect>, PreparationError> {
        for (index, definition) in definitions.iter().enumerate() {
            if definitions[..index]
                .iter()
                .any(|previous| previous.instance_id == definition.instance_id)
            {
                return Err(invalid_error(
                    NodeCategory::Effect,
                    definition.kind.as_str(),
                    definition.instance_id.raw(),
                    definition.schema_version,
                    InvalidDefinitionDiagnostic::new(
                        InvalidDefinitionCode::DuplicateInstanceId,
                        Some("instance_id"),
                        format!(
                            "effect instance ID {} appears more than once in the ordered chain",
                            definition.instance_id.raw()
                        ),
                    ),
                ));
            }
        }
        definitions
            .iter()
            .map(|definition| self.prepare_effect(definition, context))
            .collect()
    }

    /// Prepares the instrument and its effect definitions without installing or
    /// routing the returned owners.
    pub fn prepare_definition(
        &self,
        definition: &InstrumentDefinition,
        context: &NrtPreparationContext<'_>,
    ) -> Result<PreparedInstrumentDefinition, PreparationError> {
        let instrument = self.prepare_instrument(definition, context)?;
        let effects = self.prepare_effect_chain(&definition.effects, context)?;
        Ok(PreparedInstrumentDefinition {
            instrument,
            effects,
        })
    }
}

fn validate_kind_id(
    category: NodeCategory,
    kind: &str,
    instance_id: u32,
    schema_version: u32,
) -> Result<(), PreparationError> {
    if !kind.is_empty()
        && kind.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"._-".contains(&byte)
        })
    {
        return Ok(());
    }
    Err(invalid_error(
        category,
        kind,
        instance_id,
        schema_version,
        InvalidDefinitionDiagnostic::new(
            InvalidDefinitionCode::InvalidKindId,
            Some("kind"),
            "kind IDs must be non-empty lowercase ASCII names containing only letters, digits, `.`, `_`, or `-`",
        ),
    ))
}

fn validate_version(
    category: NodeCategory,
    kind: &str,
    instance_id: u32,
    requested: u32,
    supported: &'static [u32],
) -> Result<(), PreparationError> {
    if supported.contains(&requested) {
        Ok(())
    } else {
        Err(PreparationError::UnsupportedSchemaVersion {
            category,
            kind: kind.to_owned(),
            instance_id,
            requested,
            supported,
        })
    }
}

fn validate_context(
    category: NodeCategory,
    kind: &str,
    instance_id: u32,
    schema_version: u32,
    context: &NrtPreparationContext<'_>,
) -> Result<(), PreparationError> {
    if context.sample_rate.is_finite() && context.sample_rate > 0.0 {
        Ok(())
    } else {
        Err(invalid_error(
            category,
            kind,
            instance_id,
            schema_version,
            InvalidDefinitionDiagnostic::new(
                InvalidDefinitionCode::InvalidPreparationContext,
                Some("sample_rate"),
                "sample rate must be finite and greater than zero",
            ),
        ))
    }
}

fn invalid_error(
    category: NodeCategory,
    kind: &str,
    instance_id: u32,
    schema_version: u32,
    diagnostic: InvalidDefinitionDiagnostic,
) -> PreparationError {
    PreparationError::InvalidDefinition {
        category,
        kind: kind.to_owned(),
        instance_id,
        schema_version,
        diagnostic,
    }
}

fn parse_payload<T: DeserializeOwned>(
    parameters: &crate::ParameterPayload,
) -> Result<T, InvalidDefinitionDiagnostic> {
    let object: Map<String, Value> = parameters
        .iter()
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect();
    serde_json::from_value(Value::Object(object)).map_err(|error| {
        InvalidDefinitionDiagnostic::new(
            InvalidDefinitionCode::InvalidParameterPayload,
            None::<String>,
            error.to_string(),
        )
    })
}

fn validate_finite(name: &str, value: f32) -> Result<(), InvalidDefinitionDiagnostic> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(out_of_range(name, "must be finite"))
    }
}

fn validate_range(
    name: &str,
    value: f32,
    min: f32,
    max: f32,
) -> Result<(), InvalidDefinitionDiagnostic> {
    if value.is_finite() && (min..=max).contains(&value) {
        Ok(())
    } else {
        Err(out_of_range(
            name,
            format!("must be finite and in {min}..={max}"),
        ))
    }
}

fn out_of_range(field: &str, message: impl Into<String>) -> InvalidDefinitionDiagnostic {
    InvalidDefinitionDiagnostic::new(
        InvalidDefinitionCode::ParameterOutOfRange,
        Some(field),
        message,
    )
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum WaveformPayload {
    Sine,
    Square,
    Sawtooth,
    Triangle,
    NesTriangle,
}

impl From<WaveformPayload> for Waveform {
    fn from(value: WaveformPayload) -> Self {
        match value {
            WaveformPayload::Sine => Self::Sine,
            WaveformPayload::Square => Self::Square,
            WaveformPayload::Sawtooth => Self::Sawtooth,
            WaveformPayload::Triangle => Self::Triangle,
            WaveformPayload::NesTriangle => Self::NesTriangle,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OscillatorParameters {
    pan: f32,
    waveform: WaveformPayload,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PolyphonicOscillatorParameters {
    pan: f32,
    max_polyphony: u8,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PanParameters {
    pan: f32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SamplePlayerParameters {
    pan: f32,
    sample_id: u32,
}

fn validated_pan(pan: f32) -> Result<f32, InvalidDefinitionDiagnostic> {
    validate_range("pan", pan, -1.0, 1.0)?;
    Ok(pan)
}

fn build_mono_oscillator(
    definition: &InstrumentDefinition,
    context: &NrtPreparationContext<'_>,
) -> Result<Box<dyn InstrumentTrait>, InvalidDefinitionDiagnostic> {
    let parameters: OscillatorParameters = parse_payload(&definition.parameters)?;
    let pan = validated_pan(parameters.pan)?;
    Ok(
        InstrumentFactory::new(context.sample_rate).create_oscillator_with_waveform(
            definition.instance_id,
            pan,
            parameters.waveform.into(),
        ),
    )
}

fn build_polyphonic_oscillator(
    definition: &InstrumentDefinition,
    context: &NrtPreparationContext<'_>,
) -> Result<Box<dyn InstrumentTrait>, InvalidDefinitionDiagnostic> {
    let parameters: PolyphonicOscillatorParameters = parse_payload(&definition.parameters)?;
    let pan = validated_pan(parameters.pan)?;
    if !(1..=64).contains(&parameters.max_polyphony) {
        return Err(out_of_range("max_polyphony", "must be in 1..=64"));
    }
    Ok(
        InstrumentFactory::new(context.sample_rate).create_polyphonic_oscillator(
            definition.instance_id,
            pan,
            parameters.max_polyphony,
        ),
    )
}

macro_rules! pan_instrument_builder {
    ($name:ident, $method:ident) => {
        fn $name(
            definition: &InstrumentDefinition,
            context: &NrtPreparationContext<'_>,
        ) -> Result<Box<dyn InstrumentTrait>, InvalidDefinitionDiagnostic> {
            let parameters: PanParameters = parse_payload(&definition.parameters)?;
            let pan = validated_pan(parameters.pan)?;
            Ok(InstrumentFactory::new(context.sample_rate).$method(definition.instance_id, pan))
        }
    };
}

pan_instrument_builder!(build_hi_hat, create_hihat);
pan_instrument_builder!(build_kick_drum, create_kick_drum);
pan_instrument_builder!(build_snare_drum, create_snare_drum);
pan_instrument_builder!(build_moog_dfam, create_dfam);

fn resolve_sample(
    parameters: &SamplePlayerParameters,
    context: &NrtPreparationContext<'_>,
) -> Result<Arc<SampleData>, InvalidDefinitionDiagnostic> {
    let resolver = context.samples.ok_or_else(|| {
        InvalidDefinitionDiagnostic::new(
            InvalidDefinitionCode::MissingResource,
            Some("sample_id"),
            "sample-player preparation requires an NRT SampleResolver",
        )
    })?;
    let sample = resolver
        .resolve_sample(SampleId::from_raw(parameters.sample_id))
        .ok_or_else(|| {
            InvalidDefinitionDiagnostic::new(
                InvalidDefinitionCode::MissingResource,
                Some("sample_id"),
                format!("sample resource {} was not found", parameters.sample_id),
            )
        })?;
    if !sample.sample_rate.is_finite() || sample.sample_rate <= 0.0 {
        return Err(InvalidDefinitionDiagnostic::new(
            InvalidDefinitionCode::InvalidResource,
            Some("sample_id"),
            "sample must have a positive finite sample rate",
        ));
    }
    if !matches!(sample.channels, 1 | 2) || sample.data.len() % usize::from(sample.channels) != 0 {
        return Err(InvalidDefinitionDiagnostic::new(
            InvalidDefinitionCode::InvalidResource,
            Some("sample_id"),
            "sample must be mono or stereo with channel-aligned interleaved data",
        ));
    }
    Ok(sample)
}

fn build_one_shot_sample_player(
    definition: &InstrumentDefinition,
    context: &NrtPreparationContext<'_>,
) -> Result<Box<dyn InstrumentTrait>, InvalidDefinitionDiagnostic> {
    let parameters: SamplePlayerParameters = parse_payload(&definition.parameters)?;
    let pan = validated_pan(parameters.pan)?;
    let sample = resolve_sample(&parameters, context)?;
    Ok(
        InstrumentFactory::new(context.sample_rate).create_one_shot_sample_player(
            definition.instance_id,
            pan,
            sample,
        ),
    )
}

fn build_loop_sample_player(
    definition: &InstrumentDefinition,
    context: &NrtPreparationContext<'_>,
) -> Result<Box<dyn InstrumentTrait>, InvalidDefinitionDiagnostic> {
    let parameters: SamplePlayerParameters = parse_payload(&definition.parameters)?;
    let pan = validated_pan(parameters.pan)?;
    let sample = resolve_sample(&parameters, context)?;
    let (Some(start), Some(end)) = (sample.loop_start, sample.loop_end) else {
        return Err(InvalidDefinitionDiagnostic::new(
            InvalidDefinitionCode::InvalidResource,
            Some("sample_id"),
            "loop sample requires both loop_start and loop_end",
        ));
    };
    let frame_count = sample.data.len() / usize::from(sample.channels);
    if start >= end || end as usize > frame_count {
        return Err(InvalidDefinitionDiagnostic::new(
            InvalidDefinitionCode::InvalidResource,
            Some("sample_id"),
            format!("loop range {start}..{end} is outside {frame_count} sample frames"),
        ));
    }
    Ok(
        InstrumentFactory::new(context.sample_rate).create_loop_sample_player(
            definition.instance_id,
            pan,
            sample,
        ),
    )
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReverbParameters {
    mix: f32,
    decay: f32,
    room_size: f32,
    damping: f32,
    diffusion: f32,
}

fn validate_reverb(parameters: &ReverbParameters) -> Result<(), InvalidDefinitionDiagnostic> {
    validate_range("mix", parameters.mix, 0.0, 1.0)?;
    validate_range("decay", parameters.decay, 0.0, 0.95)?;
    validate_range("room_size", parameters.room_size, 0.1, 3.0)?;
    validate_range("damping", parameters.damping, 0.0, 1.0)?;
    validate_range("diffusion", parameters.diffusion, 0.0, 0.95)
}

fn apply_mono_reverb_parameters(effect: &mut dyn MonoEffect, parameters: &ReverbParameters) {
    // Preserve current tracker hydration order. The existing DSP setters derive
    // `comb_feedback` independently, so damping currently wins over decay; the
    // task packet records that pre-existing fidelity risk rather than silently
    // changing DSP/golden behavior in this schema issue.
    for (parameter, value) in [
        (ReverbParameter::Mix, parameters.mix),
        (ReverbParameter::Decay, parameters.decay),
        (ReverbParameter::RoomSize, parameters.room_size),
        (ReverbParameter::Damping, parameters.damping),
        (ReverbParameter::Diffusion, parameters.diffusion),
    ] {
        effect.set_parameter(parameter.as_index(), value);
    }
}

fn apply_stereo_reverb_parameters(effect: &mut dyn StereoEffect, parameters: &ReverbParameters) {
    for (parameter, value) in [
        (ReverbParameter::Mix, parameters.mix),
        (ReverbParameter::Decay, parameters.decay),
        (ReverbParameter::RoomSize, parameters.room_size),
        (ReverbParameter::Damping, parameters.damping),
        (ReverbParameter::Diffusion, parameters.diffusion),
    ] {
        effect.set_parameter(parameter.as_index(), value);
    }
}

fn build_mono_reverb(
    definition: &EffectDefinition,
    context: &NrtPreparationContext<'_>,
) -> Result<PreparedEffect, InvalidDefinitionDiagnostic> {
    let parameters: ReverbParameters = parse_payload(&definition.parameters)?;
    validate_reverb(&parameters)?;
    let mut effect =
        EffectFactory::new(context.sample_rate).create_mono_reverb(definition.instance_id);
    apply_mono_reverb_parameters(&mut *effect, &parameters);
    Ok(PreparedEffect::Mono(effect))
}

fn build_stereo_reverb(
    definition: &EffectDefinition,
    context: &NrtPreparationContext<'_>,
) -> Result<PreparedEffect, InvalidDefinitionDiagnostic> {
    let parameters: ReverbParameters = parse_payload(&definition.parameters)?;
    validate_reverb(&parameters)?;
    let mut effect =
        EffectFactory::new(context.sample_rate).create_stereo_reverb(definition.instance_id);
    apply_stereo_reverb_parameters(&mut *effect, &parameters);
    Ok(PreparedEffect::Stereo(effect))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DelayParameters {
    delay_seconds: f32,
    num_taps: usize,
    feedback: f32,
    mix: f32,
}

fn build_mono_delay(
    definition: &EffectDefinition,
    context: &NrtPreparationContext<'_>,
) -> Result<PreparedEffect, InvalidDefinitionDiagnostic> {
    let parameters: DelayParameters = parse_payload(&definition.parameters)?;
    validate_range(
        "delay_seconds",
        parameters.delay_seconds,
        0.0,
        MAX_DELAY_SECONDS,
    )?;
    if !(1..=MAX_TAPS).contains(&parameters.num_taps) {
        return Err(out_of_range(
            "num_taps",
            format!("must be in 1..={MAX_TAPS}"),
        ));
    }
    validate_range("feedback", parameters.feedback, 0.0, 0.95)?;
    validate_range("mix", parameters.mix, 0.0, 1.0)?;
    let effect = EffectFactory::new(context.sample_rate).create_mono_delay(
        definition.instance_id,
        parameters.delay_seconds,
        parameters.num_taps,
        parameters.feedback,
        parameters.mix,
    );
    Ok(PreparedEffect::Mono(effect))
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum DistortionTypePayload {
    Soft,
    Hard,
    Tube,
    Foldback,
}

impl From<DistortionTypePayload> for DistortionType {
    fn from(value: DistortionTypePayload) -> Self {
        match value {
            DistortionTypePayload::Soft => Self::Soft,
            DistortionTypePayload::Hard => Self::Hard,
            DistortionTypePayload::Tube => Self::Tube,
            DistortionTypePayload::Foldback => Self::Foldback,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DistortionParameters {
    distortion_type: DistortionTypePayload,
    drive: f32,
    level: f32,
    mix: f32,
}

fn build_mono_distortion(
    definition: &EffectDefinition,
    context: &NrtPreparationContext<'_>,
) -> Result<PreparedEffect, InvalidDefinitionDiagnostic> {
    let parameters: DistortionParameters = parse_payload(&definition.parameters)?;
    validate_finite("drive", parameters.drive)?;
    validate_finite("level", parameters.level)?;
    validate_range("mix", parameters.mix, 0.0, 1.0)?;
    Ok(PreparedEffect::Mono(
        EffectFactory::new(context.sample_rate).create_distortion(
            definition.instance_id,
            parameters.distortion_type.into(),
            parameters.drive,
            parameters.level,
            parameters.mix,
        ),
    ))
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum FilterTypePayload {
    LowPass,
    HighPass,
    BandPass,
    Notch,
}

impl From<FilterTypePayload> for FilterType {
    fn from(value: FilterTypePayload) -> Self {
        match value {
            FilterTypePayload::LowPass => Self::LowPass,
            FilterTypePayload::HighPass => Self::HighPass,
            FilterTypePayload::BandPass => Self::BandPass,
            FilterTypePayload::Notch => Self::Notch,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FilterParameters {
    filter_type: FilterTypePayload,
    cutoff: f32,
    resonance: f32,
}

fn build_mono_filter(
    definition: &EffectDefinition,
    context: &NrtPreparationContext<'_>,
) -> Result<PreparedEffect, InvalidDefinitionDiagnostic> {
    let parameters: FilterParameters = parse_payload(&definition.parameters)?;
    validate_range("cutoff", parameters.cutoff, 20.0, context.sample_rate / 2.0)?;
    validate_range("resonance", parameters.resonance, 0.5, 10.0)?;
    Ok(PreparedEffect::Mono(
        EffectFactory::new(context.sample_rate).create_filter(
            definition.instance_id,
            parameters.filter_type.into(),
            parameters.cutoff,
            parameters.resonance,
        ),
    ))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GainParameters {
    gain: f32,
}

fn validate_gain(gain: f32) -> Result<(), InvalidDefinitionDiagnostic> {
    if gain.is_finite() && gain >= 0.0 {
        Ok(())
    } else {
        Err(out_of_range("gain", "must be finite and non-negative"))
    }
}

fn build_mono_gain(
    definition: &EffectDefinition,
    context: &NrtPreparationContext<'_>,
) -> Result<PreparedEffect, InvalidDefinitionDiagnostic> {
    let parameters: GainParameters = parse_payload(&definition.parameters)?;
    validate_gain(parameters.gain)?;
    Ok(PreparedEffect::Mono(
        EffectFactory::new(context.sample_rate)
            .create_mono_gain(definition.instance_id, parameters.gain),
    ))
}

fn build_stereo_gain(
    definition: &EffectDefinition,
    context: &NrtPreparationContext<'_>,
) -> Result<PreparedEffect, InvalidDefinitionDiagnostic> {
    let parameters: GainParameters = parse_payload(&definition.parameters)?;
    validate_gain(parameters.gain)?;
    Ok(PreparedEffect::Stereo(
        EffectFactory::new(context.sample_rate)
            .create_stereo_gain(definition.instance_id, parameters.gain),
    ))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MoogLadderParameters {
    cutoff: f32,
    resonance: f32,
}

fn build_moog_ladder(
    definition: &EffectDefinition,
    context: &NrtPreparationContext<'_>,
) -> Result<PreparedEffect, InvalidDefinitionDiagnostic> {
    let parameters: MoogLadderParameters = parse_payload(&definition.parameters)?;
    validate_range("cutoff", parameters.cutoff, 20.0, context.sample_rate / 2.0)?;
    validate_range("resonance", parameters.resonance, 0.0, 4.0)?;
    Ok(PreparedEffect::Mono(
        EffectFactory::new(context.sample_rate).create_moog_ladder(
            definition.instance_id,
            parameters.cutoff,
            parameters.resonance,
        ),
    ))
}
