use node_registry::{EffectDefinition, InstrumentDefinition};
use param_manifest::ParameterId;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const PORTABLE_STATE_SCHEMA_VERSION: u32 = 1;
pub const TRACKER_COMPOSITION_KIND: &str = "blight.composition.tracker";
pub const FIXED_ROUTING_KIND: &str = "blight.routing.fixed_bus";
pub const TRACKER_COMPOSITION_SCHEMA_VERSION: u32 = 1;
pub const FIXED_ROUTING_SCHEMA_VERSION: u32 = 1;

/// Host-neutral authored project state. The type has no place for device, DSP
/// history, transport, filesystem, prepared-owner, or UI fields.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortableStateV1 {
    pub schema_version: u32,
    pub composition: TaggedPayload,
    pub instruments: Vec<InstrumentDefinition>,
    pub master_effects: Vec<EffectDefinition>,
    pub parameter_values: Vec<ParameterValue>,
    pub routing: TaggedPayload,
    pub assets: Vec<AssetReference>,
}

impl PortableStateV1 {
    #[must_use]
    pub fn new(composition: TaggedPayload, routing: TaggedPayload) -> Self {
        Self {
            schema_version: PORTABLE_STATE_SCHEMA_VERSION,
            composition,
            instruments: Vec::new(),
            master_effects: Vec::new(),
            parameter_values: Vec::new(),
            routing,
            assets: Vec::new(),
        }
    }
}

/// Independently versioned composition or routing source. `payload` stays
/// opaque until an NRT kind-specific adapter handles it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaggedPayload {
    pub kind: String,
    pub schema_version: u32,
    pub payload: Value,
}

impl TaggedPayload {
    #[must_use]
    pub fn tracker_v1(payload: Value) -> Self {
        Self {
            kind: TRACKER_COMPOSITION_KIND.into(),
            schema_version: TRACKER_COMPOSITION_SCHEMA_VERSION,
            payload,
        }
    }

    #[must_use]
    pub fn fixed_routing_v1() -> Self {
        Self {
            kind: FIXED_ROUTING_KIND.into(),
            schema_version: FIXED_ROUTING_SCHEMA_VERSION,
            payload: Value::Object(Default::default()),
        }
    }
}

/// Stable instance address, deliberately distinct from manifest `NodeRef`: a
/// manifest describes a kind-local binding while this identifies one saved
/// project instance without persisting a vector index or runtime table key.
/// Instrument/voice effect categories identify distinct insertion semantics;
/// #136's routing preparation validates that category beyond ID existence.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "node_type", rename_all = "snake_case")]
pub enum NodeAddress {
    Instrument { instrument_id: u32 },
    InstrumentEffect { instrument_id: u32, effect_id: u32 },
    VoiceEffect { instrument_id: u32, effect_id: u32 },
    MasterEffect { effect_id: u32 },
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NormalizedValue(f64);

impl NormalizedValue {
    /// Constructs a finite normalized value. Derived serde permits diagnostics
    /// to retain an invalid number; `PortableStateV1::validate` and the canonical
    /// decode boundary reject it before the model can be accepted.
    #[must_use]
    pub fn new(value: f64) -> Option<Self> {
        (value.is_finite() && (0.0..=1.0).contains(&value)).then_some(Self(value))
    }

    #[must_use]
    pub const fn get(self) -> f64 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParameterValue {
    pub target: NodeAddress,
    pub parameter_id: ParameterId,
    pub normalized_value: NormalizedValue,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetReference {
    pub asset_id: String,
    pub sha256: String,
    pub media_type: String,
}

/// Resolver output remains caller-owned input to validation; the core never
/// opens a path or interprets host packaging.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedAsset {
    pub bytes: Vec<u8>,
    pub media_type: String,
}

pub trait AssetResolver {
    fn resolve(&self, reference: &AssetReference) -> Option<ResolvedAsset>;
}

/// Optional deterministic replay data belongs inside a composition payload,
/// never in the top-level envelope. Values use strings so 64/128-bit seeds do
/// not violate JSON's safe-integer boundary.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeterministicSeed {
    pub schema_version: u32,
    pub algorithm: String,
    pub value: String,
}

/// A runtime-specific checkpoint is likewise plain versioned payload data. It
/// carries no live DSP owner, callback state, or host handle.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckpointRecord {
    pub schema_version: u32,
    pub payload: Value,
}

/// Explicit prior envelope used only by the deterministic v0-to-v1 migration.
/// V0 predates portable asset references; legacy tracker `Song` is not accepted.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PortableStateV0 {
    pub schema_version: u32,
    pub composition: TaggedPayload,
    pub instruments: Vec<InstrumentDefinition>,
    pub master_effects: Vec<EffectDefinition>,
    pub parameter_values: Vec<ParameterValue>,
    pub routing: TaggedPayload,
}
