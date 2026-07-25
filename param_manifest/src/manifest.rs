//! The serializable manifest container and its validation.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use crate::descriptor::{ParameterDescriptor, ParameterId, ParameterKind, SmoothingPolicy};

/// Current manifest schema version.
///
/// Bump this only for a change to the *descriptor shape/semantics*, never for
/// adding or removing individual parameters. See ADR 0003 for the compatibility
/// rules that govern reads across versions.
pub const MANIFEST_SCHEMA_VERSION: u32 = 1;

/// A validation error for an authored manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestError {
    /// The manifest schema version is newer than this build understands.
    UnsupportedSchemaVersion { manifest: u32, supported: u32 },
    /// Two descriptors share the same stable ID.
    DuplicateId(ParameterId),
    /// A descriptor references a schema version newer than the manifest's.
    DescriptorFromFutureVersion { id: ParameterId, version_added: u32 },
    /// A descriptor carries non-finite or inconsistent numeric fields.
    InvalidNumericDescriptor { id: ParameterId, reason: &'static str },
}

impl std::fmt::Display for ManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ManifestError::UnsupportedSchemaVersion { manifest, supported } => write!(
                f,
                "manifest schema version {manifest} is newer than supported version {supported}"
            ),
            ManifestError::DuplicateId(id) => write!(f, "duplicate parameter id `{id}`"),
            ManifestError::DescriptorFromFutureVersion { id, version_added } => write!(
                f,
                "descriptor `{id}` declares version_added {version_added} above the manifest schema version"
            ),
            ManifestError::InvalidNumericDescriptor { id, reason } => {
                write!(f, "descriptor `{id}` has invalid numeric fields: {reason}")
            }
        }
    }
}

impl std::error::Error for ManifestError {}

/// The canonical, serializable parameter manifest.
///
/// It is authored/parsed off the audio thread, validated once, and then compiled
/// into a [`crate::ParameterLookup`] for real-time use.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterManifest {
    /// Descriptor-shape schema version.
    pub schema_version: u32,
    /// All parameter descriptors, in stable authored order.
    pub parameters: Vec<ParameterDescriptor>,
}

impl ParameterManifest {
    /// Construct a manifest at the current schema version.
    #[must_use]
    pub fn new(parameters: Vec<ParameterDescriptor>) -> Self {
        Self {
            schema_version: MANIFEST_SCHEMA_VERSION,
            parameters,
        }
    }

    /// Validate structural invariants: supported schema version, unique IDs, and
    /// no descriptor claiming a version newer than the manifest.
    pub fn validate(&self) -> Result<(), ManifestError> {
        if self.schema_version > MANIFEST_SCHEMA_VERSION {
            return Err(ManifestError::UnsupportedSchemaVersion {
                manifest: self.schema_version,
                supported: MANIFEST_SCHEMA_VERSION,
            });
        }

        let mut seen: BTreeSet<&ParameterId> = BTreeSet::new();
        for descriptor in &self.parameters {
            if !seen.insert(&descriptor.id) {
                return Err(ManifestError::DuplicateId(descriptor.id.clone()));
            }
            if descriptor.version_added > self.schema_version {
                return Err(ManifestError::DescriptorFromFutureVersion {
                    id: descriptor.id.clone(),
                    version_added: descriptor.version_added,
                });
            }
            validate_numeric(descriptor)?;
        }
        Ok(())
    }

    /// Whether a reader that understands `reader_schema_version` can load this
    /// manifest. Forward compatibility is intentionally not assumed: a reader
    /// must be at least as new as the manifest's descriptor schema.
    #[must_use]
    pub fn is_readable_by(&self, reader_schema_version: u32) -> bool {
        reader_schema_version >= self.schema_version
    }

    /// Look up a descriptor by its stable ID (NRT convenience).
    #[must_use]
    pub fn descriptor(&self, id: &ParameterId) -> Option<&ParameterDescriptor> {
        self.parameters.iter().find(|d| &d.id == id)
    }
}

/// Reject non-finite or inconsistent numeric descriptor data before it can reach
/// the RT conversion path (where `f32::clamp` would panic on a reversed range).
fn validate_numeric(d: &ParameterDescriptor) -> Result<(), ManifestError> {
    let invalid = |reason: &'static str| ManifestError::InvalidNumericDescriptor {
        id: d.id.clone(),
        reason,
    };

    let r = &d.range;
    if !(r.min.is_finite() && r.max.is_finite() && r.default.is_finite()) {
        return Err(invalid("range min/max/default must be finite"));
    }
    if r.min > r.max {
        return Err(invalid("range min must be <= max"));
    }
    if r.default < r.min || r.default > r.max {
        return Err(invalid("range default must be within [min, max]"));
    }

    for value in d.mapping.endpoint_values() {
        if !value.is_finite() {
            return Err(invalid("mapping parameters must be finite"));
        }
    }

    if let SmoothingPolicy::Smoothed { duration_ms, .. } = d.smoothing {
        if !duration_ms.is_finite() || duration_ms < 0.0 {
            return Err(invalid("smoothing duration_ms must be finite and non-negative"));
        }
    }

    if let ParameterKind::Discrete { steps } = &d.kind {
        if steps.is_empty() {
            return Err(invalid("discrete parameter must have at least one step"));
        }
        if steps.iter().any(|s| !s.engine_value.is_finite()) {
            return Err(invalid("discrete step engine_value must be finite"));
        }
    }

    Ok(())
}
