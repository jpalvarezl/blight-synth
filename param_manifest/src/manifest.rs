//! The serializable manifest container and its validation.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use crate::descriptor::{ParameterDescriptor, ParameterId, ParameterKind, SmoothingPolicy};
use crate::mapping::{Mapping, MAX_SKEW, MIN_SKEW};

/// Current manifest schema version.
///
/// Bump this only for a change to the *descriptor shape/semantics*, never for
/// adding or removing individual parameters. See ADR 0004 for the compatibility
/// rules that govern reads across versions.
pub const MANIFEST_SCHEMA_VERSION: u32 = 1;

/// Maximum number of entries in one prepared runtime parameter table.
pub const MAX_PARAMETER_COUNT: usize = 16_384;
/// Maximum number of choices carried by one discrete parameter.
pub const MAX_DISCRETE_STEP_COUNT: usize = 4_096;
/// Maximum total discrete numeric choices carried by one runtime table.
pub const MAX_TOTAL_DISCRETE_VALUES: usize = 1_048_576;

/// A validation error for an authored manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestError {
    /// The manifest schema version is not one this build supports.
    UnsupportedSchemaVersion { manifest: u32, supported: u32 },
    /// A manifest exceeds a practical prepared-state/key-space capacity.
    CapacityExceeded {
        what: &'static str,
        count: usize,
        max: usize,
    },
    /// Two descriptors share the same stable ID.
    DuplicateId(ParameterId),
    /// A descriptor's introduction version is outside the defined schema range.
    InvalidDescriptorVersion {
        id: ParameterId,
        version_added: u32,
        schema_version: u32,
    },
    /// A descriptor is simultaneously writable by automation and read-only.
    ContradictoryVisibility(ParameterId),
    /// A descriptor carries non-finite or inconsistent numeric fields.
    InvalidNumericDescriptor {
        id: ParameterId,
        reason: &'static str,
    },
}

impl std::fmt::Display for ManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ManifestError::UnsupportedSchemaVersion { manifest, supported } => write!(
                f,
                "manifest schema version {manifest} is unsupported; this build requires version {supported}"
            ),
            ManifestError::CapacityExceeded { what, count, max } => {
                write!(f, "{what} {count} exceeds prepared-state capacity {max}")
            }
            ManifestError::DuplicateId(id) => write!(f, "duplicate parameter id `{id}`"),
            ManifestError::InvalidDescriptorVersion {
                id,
                version_added,
                schema_version,
            } => write!(
                f,
                "descriptor `{id}` declares version_added {version_added} outside 1..={schema_version}"
            ),
            ManifestError::ContradictoryVisibility(id) => write!(
                f,
                "descriptor `{id}` cannot be both automatable and read-only"
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

    /// Validate structural, numeric, mapping, and prepared-capacity invariants.
    pub fn validate(&self) -> Result<(), ManifestError> {
        // Version 1 is the first and only defined schema. Supporting an older
        // shape requires an explicit migration rather than accepting it as v1.
        if self.schema_version != MANIFEST_SCHEMA_VERSION {
            return Err(ManifestError::UnsupportedSchemaVersion {
                manifest: self.schema_version,
                supported: MANIFEST_SCHEMA_VERSION,
            });
        }
        if self.parameters.len() > MAX_PARAMETER_COUNT {
            return Err(ManifestError::CapacityExceeded {
                what: "parameter count",
                count: self.parameters.len(),
                max: MAX_PARAMETER_COUNT,
            });
        }

        let mut total_discrete_values = 0_usize;
        let mut seen: BTreeSet<&ParameterId> = BTreeSet::new();
        for descriptor in &self.parameters {
            if !seen.insert(&descriptor.id) {
                return Err(ManifestError::DuplicateId(descriptor.id.clone()));
            }
            if !(1..=self.schema_version).contains(&descriptor.version_added) {
                return Err(ManifestError::InvalidDescriptorVersion {
                    id: descriptor.id.clone(),
                    version_added: descriptor.version_added,
                    schema_version: self.schema_version,
                });
            }
            if descriptor.visibility.automatable && descriptor.visibility.read_only {
                return Err(ManifestError::ContradictoryVisibility(
                    descriptor.id.clone(),
                ));
            }
            validate_numeric(descriptor)?;
            if let ParameterKind::Discrete { steps } = &descriptor.kind {
                total_discrete_values = total_discrete_values.checked_add(steps.len()).ok_or(
                    ManifestError::CapacityExceeded {
                        what: "total discrete value count",
                        count: usize::MAX,
                        max: MAX_TOTAL_DISCRETE_VALUES,
                    },
                )?;
                if total_discrete_values > MAX_TOTAL_DISCRETE_VALUES {
                    return Err(ManifestError::CapacityExceeded {
                        what: "total discrete value count",
                        count: total_discrete_values,
                        max: MAX_TOTAL_DISCRETE_VALUES,
                    });
                }
            }
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

/// Reject non-finite, non-invertible, or inconsistent numeric data before NRT
/// preparation. The RT tier still sanitizes malformed values as a final defense.
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

    let (mapping_min, mapping_max) = d.mapping.engine_bounds();
    if !(mapping_min.is_finite() && mapping_max.is_finite()) {
        return Err(invalid("mapping bounds must be finite"));
    }
    if mapping_min >= mapping_max {
        return Err(invalid("mapping bounds must be strictly increasing"));
    }
    if mapping_min != r.min || mapping_max != r.max {
        return Err(invalid("mapping bounds must equal range min/max"));
    }

    match d.mapping {
        Mapping::Linear { .. } => {}
        Mapping::Exponential { min, max } => {
            if !(min > 0.0 && min < max) {
                return Err(invalid("exponential mapping requires finite 0 < min < max"));
            }
        }
        Mapping::Skewed { skew, .. } => {
            if !skew.is_finite() || !(MIN_SKEW..=MAX_SKEW).contains(&skew) {
                return Err(invalid("mapping skew is outside the supported range"));
            }
            if !d.mapping.has_representable_skew_round_trip() {
                return Err(invalid(
                    "mapping skew/range loses required f32 round-trip precision",
                ));
            }
        }
        Mapping::AmplitudeDecibel { floor_db } => {
            if !floor_db.is_finite() || floor_db >= 0.0 {
                return Err(invalid("amplitude-decibel floor must be finite and < 0 dB"));
            }
        }
    }

    if let SmoothingPolicy::Smoothed { duration_ms, .. } = d.smoothing {
        if !duration_ms.is_finite() || duration_ms < 0.0 {
            return Err(invalid(
                "smoothing duration_ms must be finite and non-negative",
            ));
        }
    }

    if let ParameterKind::Discrete { steps } = &d.kind {
        if steps.len() < 2 {
            return Err(invalid("discrete parameter must have at least two steps"));
        }
        if steps.len() > MAX_DISCRETE_STEP_COUNT {
            return Err(ManifestError::CapacityExceeded {
                what: "discrete step count",
                count: steps.len(),
                max: MAX_DISCRETE_STEP_COUNT,
            });
        }
        if steps.iter().any(|s| {
            !s.engine_value.is_finite() || s.engine_value < r.min || s.engine_value > r.max
        }) {
            return Err(invalid(
                "discrete step engine_value must be finite and within range",
            ));
        }
        if !steps.iter().any(|s| s.engine_value == r.default) {
            return Err(invalid("discrete default must equal one authored step"));
        }
    }

    Ok(())
}
