use std::{collections::BTreeMap, fmt, sync::Arc};

use node_registry::{BuiltInRegistry, EffectDefinition, InstrumentDefinition};
use serde::{
    de::{self, MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer,
};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::{
    model::PortableStateV0, AssetResolver, NodeAddress, PortableStateV1, FIXED_ROUTING_KIND,
    FIXED_ROUTING_SCHEMA_VERSION, PORTABLE_STATE_SCHEMA_VERSION, TRACKER_COMPOSITION_KIND,
    TRACKER_COMPOSITION_SCHEMA_VERSION,
};

const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PayloadCategory {
    Composition,
    Routing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeDefinitionCategory {
    Instrument,
    InstrumentEffect,
    MasterEffect,
}

/// Machine-readable portable-state failure. Unknown source variants retain the
/// parsed nested value; [`StateDiagnostic`] additionally owns exact input bytes.
#[derive(Clone, Debug, PartialEq)]
pub enum StateError {
    UnsupportedEnvelopeVersion {
        requested: u32,
        supported: u32,
    },
    UnsupportedPayload {
        category: PayloadCategory,
        kind: String,
        schema_version: u32,
        source: Value,
    },
    UnsupportedNode {
        category: NodeDefinitionCategory,
        kind: String,
        schema_version: u32,
        instance_id: u32,
        source: Value,
    },
    DuplicateNodeId {
        scope: String,
        instance_id: u32,
    },
    DuplicateParameter {
        target: NodeAddress,
        parameter_id: String,
    },
    DuplicateAssetId {
        asset_id: String,
    },
    InvalidNodeReference {
        target: NodeAddress,
    },
    InvalidNormalized {
        target: NodeAddress,
        parameter_id: String,
        value: f64,
    },
    InvalidNumeric {
        pointer: String,
    },
    InvalidDigest {
        asset_id: String,
        digest: String,
    },
    AssetMissing {
        asset_id: String,
    },
    AssetDigestMismatch {
        asset_id: String,
        expected: String,
        actual: String,
    },
    AssetMediaTypeMismatch {
        asset_id: String,
        expected: String,
        actual: String,
    },
    Malformed {
        message: String,
    },
    Canonicalization {
        message: String,
    },
    NonCanonical,
}

impl fmt::Display for StateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for StateError {}

#[derive(Clone, Debug, PartialEq)]
pub struct StateDiagnostic {
    pub error: StateError,
    pub source_bytes: Arc<[u8]>,
}

impl PortableStateV1 {
    /// Validates semantics and emits the one RFC 8785 representation. Set-like
    /// vectors are sorted on a clone; ordered effect vectors are never changed.
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, StateError> {
        self.validate()?;
        let normalized = self.normalized();
        serde_json_canonicalizer::to_vec(&normalized).map_err(|error| {
            StateError::Canonicalization {
                message: error.to_string(),
            }
        })
    }

    pub fn validate(&self) -> Result<(), StateError> {
        if self.schema_version != PORTABLE_STATE_SCHEMA_VERSION {
            return Err(StateError::UnsupportedEnvelopeVersion {
                requested: self.schema_version,
                supported: PORTABLE_STATE_SCHEMA_VERSION,
            });
        }
        validate_payload(
            PayloadCategory::Composition,
            &self.composition,
            TRACKER_COMPOSITION_KIND,
            TRACKER_COMPOSITION_SCHEMA_VERSION,
            false,
        )?;
        validate_payload(
            PayloadCategory::Routing,
            &self.routing,
            FIXED_ROUTING_KIND,
            FIXED_ROUTING_SCHEMA_VERSION,
            true,
        )?;
        validate_nodes(self)?;
        validate_parameters(self)?;
        validate_assets(self)?;
        let value = serde_json::to_value(self).map_err(|error| StateError::Canonicalization {
            message: error.to_string(),
        })?;
        validate_numbers(&value, "")
    }

    pub fn validate_resolved_assets(&self, resolver: &dyn AssetResolver) -> Result<(), StateError> {
        self.validate()?;
        for reference in &self.assets {
            let resolved = resolver
                .resolve(reference)
                .ok_or_else(|| StateError::AssetMissing {
                    asset_id: reference.asset_id.clone(),
                })?;
            if resolved.media_type != reference.media_type {
                return Err(StateError::AssetMediaTypeMismatch {
                    asset_id: reference.asset_id.clone(),
                    expected: reference.media_type.clone(),
                    actual: resolved.media_type,
                });
            }
            let actual = format!("{:x}", Sha256::digest(&resolved.bytes));
            if actual != reference.sha256 {
                return Err(StateError::AssetDigestMismatch {
                    asset_id: reference.asset_id.clone(),
                    expected: reference.sha256.clone(),
                    actual,
                });
            }
        }
        Ok(())
    }

    fn normalized(&self) -> Self {
        let mut state = self.clone();
        state.instruments.sort_by_key(|item| item.instance_id.raw());
        state.parameter_values.sort_by(|left, right| {
            (&left.target, &left.parameter_id).cmp(&(&right.target, &right.parameter_id))
        });
        state
            .assets
            .sort_by(|left, right| left.asset_id.cmp(&right.asset_id));
        state
    }
}

pub fn decode_canonical(source_bytes: Arc<[u8]>) -> Result<PortableStateV1, StateDiagnostic> {
    let result = (|| {
        let source = parse_strict(&source_bytes)?;
        validate_numbers(&source, "")?;
        let version = envelope_version(&source)?;
        if version != PORTABLE_STATE_SCHEMA_VERSION {
            return Err(StateError::UnsupportedEnvelopeVersion {
                requested: version,
                supported: PORTABLE_STATE_SCHEMA_VERSION,
            });
        }
        let state: PortableStateV1 =
            serde_json::from_value(source).map_err(|error| StateError::Malformed {
                message: error.to_string(),
            })?;
        let canonical = state.canonical_bytes()?;
        if canonical.as_slice() != source_bytes.as_ref() {
            return Err(StateError::NonCanonical);
        }
        Ok(state.normalized())
    })();
    result.map_err(|error| StateDiagnostic {
        error,
        source_bytes,
    })
}

/// Deterministic envelope-only migration. This deliberately does not recognize
/// or adapt the legacy sequencer `Song` format owned by issue #250.
pub fn migrate_v0(source_bytes: Arc<[u8]>) -> Result<PortableStateV1, StateDiagnostic> {
    let result = (|| {
        let source = parse_strict(&source_bytes)?;
        validate_numbers(&source, "")?;
        let version = envelope_version(&source)?;
        if version != 0 {
            return Err(StateError::UnsupportedEnvelopeVersion {
                requested: version,
                supported: 0,
            });
        }
        let old: PortableStateV0 =
            serde_json::from_value(source).map_err(|error| StateError::Malformed {
                message: error.to_string(),
            })?;
        let state = PortableStateV1 {
            schema_version: PORTABLE_STATE_SCHEMA_VERSION,
            composition: old.composition,
            instruments: old.instruments,
            master_effects: old.master_effects,
            parameter_values: old.parameter_values,
            routing: old.routing,
            assets: Vec::new(),
        };
        state.validate()?;
        Ok(state.normalized())
    })();
    result.map_err(|error| StateDiagnostic {
        error,
        source_bytes,
    })
}

fn validate_payload(
    category: PayloadCategory,
    tagged: &crate::TaggedPayload,
    known_kind: &str,
    known_version: u32,
    must_be_empty: bool,
) -> Result<(), StateError> {
    if tagged.kind != known_kind || tagged.schema_version != known_version {
        return Err(StateError::UnsupportedPayload {
            category,
            kind: tagged.kind.clone(),
            schema_version: tagged.schema_version,
            source: serde_json::to_value(tagged).expect("tagged payload is JSON data"),
        });
    }
    if !tagged.payload.is_object()
        || must_be_empty && tagged.payload.as_object().is_some_and(|v| !v.is_empty())
    {
        return Err(StateError::Malformed {
            message: format!(
                "{category:?} v1 payload must be {}JSON object",
                if must_be_empty { "an empty " } else { "a " }
            ),
        });
    }
    Ok(())
}

fn validate_nodes(state: &PortableStateV1) -> Result<(), StateError> {
    reject_duplicate_ids(
        "instruments",
        state.instruments.iter().map(|node| node.instance_id.raw()),
    )?;
    validate_effects(
        "master_effects",
        &state.master_effects,
        NodeDefinitionCategory::MasterEffect,
    )?;
    for instrument in &state.instruments {
        validate_node_kind(instrument, NodeDefinitionCategory::Instrument)?;
        validate_effects(
            &format!("instrument:{}.effects", instrument.instance_id.raw()),
            &instrument.effects,
            NodeDefinitionCategory::InstrumentEffect,
        )?;
    }
    Ok(())
}

fn validate_effects(
    scope: &str,
    effects: &[EffectDefinition],
    category: NodeDefinitionCategory,
) -> Result<(), StateError> {
    reject_duplicate_ids(scope, effects.iter().map(|node| node.instance_id.raw()))?;
    for effect in effects {
        let known = BuiltInRegistry::effect_kinds().any(|item| {
            item.kind == effect.kind.as_str()
                && item
                    .supported_schema_versions
                    .contains(&effect.schema_version)
        });
        if !known {
            return Err(unsupported_node(
                category,
                effect.kind.as_str(),
                effect.schema_version,
                effect.instance_id.raw(),
                effect,
            ));
        }
    }
    Ok(())
}

fn validate_node_kind(
    node: &InstrumentDefinition,
    category: NodeDefinitionCategory,
) -> Result<(), StateError> {
    let known = BuiltInRegistry::instrument_kinds().any(|item| {
        item.kind == node.kind.as_str()
            && item
                .supported_schema_versions
                .contains(&node.schema_version)
    });
    if known {
        Ok(())
    } else {
        Err(unsupported_node(
            category,
            node.kind.as_str(),
            node.schema_version,
            node.instance_id.raw(),
            node,
        ))
    }
}

fn unsupported_node<T: serde::Serialize>(
    category: NodeDefinitionCategory,
    kind: &str,
    schema_version: u32,
    instance_id: u32,
    source: &T,
) -> StateError {
    StateError::UnsupportedNode {
        category,
        kind: kind.into(),
        schema_version,
        instance_id,
        source: serde_json::to_value(source).expect("node definition is JSON data"),
    }
}

fn reject_duplicate_ids(scope: &str, ids: impl IntoIterator<Item = u32>) -> Result<(), StateError> {
    let mut seen = std::collections::BTreeSet::new();
    for id in ids {
        if !seen.insert(id) {
            return Err(StateError::DuplicateNodeId {
                scope: scope.into(),
                instance_id: id,
            });
        }
    }
    Ok(())
}

fn validate_parameters(state: &PortableStateV1) -> Result<(), StateError> {
    let mut seen = std::collections::BTreeSet::new();
    for parameter in &state.parameter_values {
        let key = (parameter.target.clone(), parameter.parameter_id.clone());
        if !seen.insert(key) {
            return Err(StateError::DuplicateParameter {
                target: parameter.target.clone(),
                parameter_id: parameter.parameter_id.to_string(),
            });
        }
        let value = parameter.normalized_value.get();
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            return Err(StateError::InvalidNormalized {
                target: parameter.target.clone(),
                parameter_id: parameter.parameter_id.to_string(),
                value,
            });
        }
        if !target_exists(state, &parameter.target) {
            return Err(StateError::InvalidNodeReference {
                target: parameter.target.clone(),
            });
        }
    }
    Ok(())
}

fn target_exists(state: &PortableStateV1, target: &NodeAddress) -> bool {
    match *target {
        NodeAddress::Instrument { instrument_id } => state
            .instruments
            .iter()
            .any(|item| item.instance_id.raw() == instrument_id),
        NodeAddress::InstrumentEffect {
            instrument_id,
            effect_id,
        }
        | NodeAddress::VoiceEffect {
            instrument_id,
            effect_id,
        } => state.instruments.iter().any(|item| {
            item.instance_id.raw() == instrument_id
                && item
                    .effects
                    .iter()
                    .any(|effect| effect.instance_id.raw() == effect_id)
        }),
        NodeAddress::MasterEffect { effect_id } => state
            .master_effects
            .iter()
            .any(|effect| effect.instance_id.raw() == effect_id),
    }
}

fn validate_assets(state: &PortableStateV1) -> Result<(), StateError> {
    let mut seen = std::collections::BTreeSet::new();
    for asset in &state.assets {
        if !seen.insert(&asset.asset_id) {
            return Err(StateError::DuplicateAssetId {
                asset_id: asset.asset_id.clone(),
            });
        }
        if asset.asset_id.is_empty() || asset.media_type.is_empty() {
            return Err(StateError::Malformed {
                message: "asset_id and media_type must be non-empty".into(),
            });
        }
        if asset.sha256.len() != 64
            || !asset
                .sha256
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(StateError::InvalidDigest {
                asset_id: asset.asset_id.clone(),
                digest: asset.sha256.clone(),
            });
        }
    }
    Ok(())
}

fn envelope_version(value: &Value) -> Result<u32, StateError> {
    value
        .get("schema_version")
        .and_then(Value::as_u64)
        .and_then(|version| u32::try_from(version).ok())
        .ok_or_else(|| StateError::Malformed {
            message: "schema_version must be an unsigned 32-bit integer".into(),
        })
}

fn validate_numbers(value: &Value, pointer: &str) -> Result<(), StateError> {
    match value {
        Value::Number(number) => {
            let unsafe_integer = number
                .as_u64()
                .is_some_and(|value| value > MAX_SAFE_INTEGER)
                || number
                    .as_i64()
                    .is_some_and(|value| value.unsigned_abs() > MAX_SAFE_INTEGER);
            let invalid_float = number.as_f64().is_none_or(|value| {
                !value.is_finite() || value.fract() == 0.0 && value.abs() > MAX_SAFE_INTEGER as f64
            });
            if unsafe_integer || invalid_float {
                return Err(StateError::InvalidNumeric {
                    pointer: pointer.into(),
                });
            }
        }
        Value::Array(values) => {
            for (index, child) in values.iter().enumerate() {
                validate_numbers(child, &format!("{pointer}/{index}"))?;
            }
        }
        Value::Object(values) => {
            for (key, child) in values {
                let key = key.replace('~', "~0").replace('/', "~1");
                validate_numbers(child, &format!("{pointer}/{key}"))?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn parse_strict(bytes: &[u8]) -> Result<Value, StateError> {
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let value =
        StrictValue::deserialize(&mut deserializer).map_err(|error| StateError::Malformed {
            message: error.to_string(),
        })?;
    deserializer.end().map_err(|error| StateError::Malformed {
        message: error.to_string(),
    })?;
    Ok(value.0)
}

struct StrictValue(Value);

impl<'de> Deserialize<'de> for StrictValue {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(StrictVisitor)
    }
}

struct StrictVisitor;

impl<'de> Visitor<'de> for StrictVisitor {
    type Value = StrictValue;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("I-JSON value")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::Bool(value)))
    }
    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
        Ok(StrictValue(value.into()))
    }
    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
        Ok(StrictValue(value.into()))
    }
    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        serde_json::Number::from_f64(value)
            .map(Value::Number)
            .map(StrictValue)
            .ok_or_else(|| E::custom("non-finite number"))
    }
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_string(value.into())
    }
    fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::String(value)))
    }
    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::Null))
    }
    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::Null))
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut sequence: A) -> Result<Self::Value, A::Error> {
        let mut values = Vec::new();
        while let Some(value) = sequence.next_element::<StrictValue>()? {
            values.push(value.0);
        }
        Ok(StrictValue(Value::Array(values)))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut values = BTreeMap::new();
        while let Some((key, value)) = map.next_entry::<String, StrictValue>()? {
            if values.insert(key.clone(), value.0).is_some() {
                return Err(de::Error::custom(format!("duplicate object key `{key}`")));
            }
        }
        Ok(StrictValue(Value::Object(values.into_iter().collect())))
    }
}
