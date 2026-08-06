use std::{collections::BTreeMap, fmt};

use dsp::id::{EffectId, InstrumentId};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

/// Version of the definition envelope introduced by issue #210.
///
/// Each built-in kind independently declares the envelope versions it can
/// interpret. Adding a kind does not bump this value; changing the meaning or
/// shape of an existing kind's payload requires a new version and migration on
/// NRT before preparation.
pub const LEGACY_NODE_DEFINITION_SCHEMA_VERSION: u32 = 1;
pub const NODE_DEFINITION_SCHEMA_VERSION: u32 = 2;

/// An opaque, kind-versioned constructor payload.
///
/// Keys and values are intentionally retained even when a kind is unknown so a
/// caller can diagnose or migrate the definition. This is not a replacement for
/// `param_manifest`: manifests describe stable automatable parameters, while
/// this payload carries the complete NRT constructor input owned by one node
/// kind and schema version.
pub type ParameterPayload = BTreeMap<String, Value>;

macro_rules! kind_id {
    ($name:ident, $description:literal) => {
        #[doc = $description]
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            #[must_use]
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self::new(value)
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self::new(value)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(formatter)
            }
        }
    };
}

kind_id!(
    InstrumentKindId,
    "Stable identity of an instrument implementation, not an instrument instance."
);
kind_id!(
    EffectKindId,
    "Stable identity of an effect implementation, not an effect instance or slot."
);

/// Serializable definition of one instrument instance and its ordered effects.
///
/// `kind` selects a statically compiled implementation. `instance_id` identifies
/// this particular prepared owner and is deliberately independent of `kind`.
/// The numeric JSON form preserves the current compact typed-ID representation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InstrumentDefinition {
    pub schema_version: u32,
    #[serde(
        serialize_with = "serialize_instrument_id",
        deserialize_with = "deserialize_instrument_id"
    )]
    pub instance_id: InstrumentId,
    pub kind: InstrumentKindId,
    pub parameters: ParameterPayload,
    /// Effect order is data: index zero is prepared before index one.
    /// Routing/installation semantics remain owned by issue #136.
    pub effects: Vec<EffectDefinition>,
}

impl InstrumentDefinition {
    /// Creates a definition at the current envelope version.
    #[must_use]
    pub fn new(
        instance_id: InstrumentId,
        kind: impl Into<InstrumentKindId>,
        parameters: ParameterPayload,
        effects: Vec<EffectDefinition>,
    ) -> Self {
        Self {
            schema_version: NODE_DEFINITION_SCHEMA_VERSION,
            instance_id,
            kind: kind.into(),
            parameters,
            effects,
        }
    }
}

/// Serializable definition of one independently addressable effect slot.
///
/// Two entries may have the same `kind`; their typed `instance_id` values keep
/// them distinct. A chain rejects duplicate instance IDs, not duplicate kinds.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectDefinition {
    pub schema_version: u32,
    #[serde(
        serialize_with = "serialize_effect_id",
        deserialize_with = "deserialize_effect_id"
    )]
    pub instance_id: EffectId,
    pub kind: EffectKindId,
    pub parameters: ParameterPayload,
}

impl EffectDefinition {
    /// Creates an effect definition at its unchanged v1 payload version.
    #[must_use]
    pub fn new(
        instance_id: EffectId,
        kind: impl Into<EffectKindId>,
        parameters: ParameterPayload,
    ) -> Self {
        Self {
            schema_version: LEGACY_NODE_DEFINITION_SCHEMA_VERSION,
            instance_id,
            kind: kind.into(),
            parameters,
        }
    }
}

fn serialize_instrument_id<S>(id: &InstrumentId, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u32(id.raw())
}

fn deserialize_instrument_id<'de, D>(deserializer: D) -> Result<InstrumentId, D::Error>
where
    D: Deserializer<'de>,
{
    u32::deserialize(deserializer).map(InstrumentId::from_raw)
}

fn serialize_effect_id<S>(id: &EffectId, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u32(id.raw())
}

fn deserialize_effect_id<'de, D>(deserializer: D) -> Result<EffectId, D::Error>
where
    D: Deserializer<'de>,
{
    u32::deserialize(deserializer).map(EffectId::from_raw)
}
