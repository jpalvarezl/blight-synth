//! Serializable parameter descriptor fields.
//!
//! A [`ParameterDescriptor`] is the authored, off-audio-thread record for one
//! parameter. It carries every descriptive string, unit, mapping, automation
//! rate, smoothing policy, host-visibility flag, and versioning field. The
//! real-time tier ([`crate::RuntimeParameter`]) is derived from it and keeps only
//! the numeric fields.

use serde::{Deserialize, Serialize};

use crate::mapping::Mapping;

/// Stable, human-readable parameter identity.
///
/// The string is the canonical key every boundary agrees on (OSC address arg,
/// APVTS parameter ID, Svelte store key, project-state field). It is **stable
/// across schema versions**: an ID is never renamed or reused for a different
/// meaning. Renames are modeled as a new ID plus a deprecation of the old one.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ParameterId(pub String);

impl ParameterId {
    /// Borrow the underlying stable ID string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for ParameterId {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<String> for ParameterId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for ParameterId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// The kind of engine node that owns the parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    /// A master-bus effect (targeted by `engine::MixerCmd`).
    MasterEffect,
    /// An instrument-owned effect (targeted by `engine::InstrumentCmd`).
    InstrumentEffect,
    /// A per-voice effect.
    VoiceEffect,
    /// An instrument/synth node parameter.
    Instrument,
}

/// Stable reference to the owning node and its engine-facing parameter slot.
///
/// `path` is a stable structural path (e.g. `"master/effect:gain"`); the concrete
/// runtime `EffectId`/`InstrumentId` is resolved by the host adapter at prepare
/// time. `engine_param_index` is the index passed to the DSP node's
/// `set_parameter(index, value)`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeRef {
    pub node_type: NodeType,
    pub path: String,
    pub engine_param_index: u32,
}

/// Physical/semantic unit of the *engine* value (post-mapping).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Unit {
    /// Dimensionless linear factor.
    Linear,
    Decibel,
    Hertz,
    Seconds,
    Milliseconds,
    Percent,
    Semitones,
    /// An integer count (e.g. number of taps/voices).
    Count,
    /// A named unit not covered above.
    Custom(String),
}

/// Engine-value range and default, expressed in engine units.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ValueRange {
    pub min: f32,
    pub max: f32,
    pub default: f32,
}

/// One choice of a discrete parameter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiscreteStep {
    /// Display label (kept off the RT path).
    pub label: String,
    /// Engine value selected by this step.
    pub engine_value: f32,
}

/// Whether the parameter is continuous or a discrete set of labeled steps.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ParameterKind {
    Continuous,
    Discrete { steps: Vec<DiscreteStep> },
}

/// Real-time traffic class, matching the "Control traffic classes" section of the
/// [real-time audio contract](../../../docs/architecture/realtime-contract.md).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AutomationRate {
    /// Sample-accurate timestamped events (ordered, bounded). Owned by #134/#145.
    SampleEvent,
    /// Continuous latest-value-wins, coalesced at a control rate. Owned by #101.
    ControlCoalesced,
    /// Infrequent structural prepared-state replacement. Owned by #174/#138.
    Structural,
}

/// Smoothing interpolation curve applied when the engine value changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SmoothingCurve {
    Linear,
    Exponential,
}

/// Smoothing policy applied by the engine when a coalesced value moves.
///
/// Owning smoothing here (not in each host adapter) guarantees OSC, APVTS, and
/// the Svelte UI all get identical de-zipper behavior for the same parameter.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "smoothing", rename_all = "snake_case")]
pub enum SmoothingPolicy {
    /// No smoothing; the engine value jumps to the target.
    None,
    /// Smooth over `duration_ms` using `curve`.
    Smoothed {
        duration_ms: f32,
        curve: SmoothingCurve,
    },
}

/// Host-facing visibility and automation flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Visibility {
    /// Shown in host/plugin generic parameter lists and the UI.
    pub host_visible: bool,
    /// May be automated by a host/DAW (APVTS `isAutomatable`).
    pub automatable: bool,
    /// Read-only meter/telemetry value; hosts must not write it.
    pub read_only: bool,
}

impl Default for Visibility {
    fn default() -> Self {
        Self {
            host_visible: true,
            automatable: true,
            read_only: false,
        }
    }
}

/// The complete authored descriptor for one parameter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterDescriptor {
    /// Stable identity used by every boundary.
    pub id: ParameterId,
    /// Owning node type/path and engine parameter slot.
    pub owner: NodeRef,
    /// Full display name.
    pub display_name: String,
    /// Short/abbreviated name (e.g. for compact UI or APVTS short name).
    pub short_name: String,
    /// Engine-value unit.
    pub unit: Unit,
    /// Engine-value range and default.
    pub range: ValueRange,
    /// Normalized `0..1` <-> engine-value mapping (unit conversion owner).
    pub mapping: Mapping,
    /// Continuous or discrete.
    pub kind: ParameterKind,
    /// Real-time traffic class.
    pub automation_rate: AutomationRate,
    /// Smoothing policy for coalesced changes.
    pub smoothing: SmoothingPolicy,
    /// Host visibility/automation flags.
    pub visibility: Visibility,
    /// Manifest schema version in which this descriptor was introduced.
    pub version_added: u32,
    /// If set, the descriptor is deprecated; the string explains the successor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<String>,
}

impl ParameterDescriptor {
    /// Default engine value expressed as a normalized `0..1` control value.
    ///
    /// Continuous parameters invert their mapping. Discrete parameters use the
    /// authored step's ordinal position because their numeric engine values may
    /// be non-uniform; validation guarantees the default equals one step. On an
    /// unvalidated malformed discrete descriptor, this falls back to the mapping
    /// inverse rather than panicking.
    #[must_use]
    pub fn default_normalized(&self) -> f32 {
        if let ParameterKind::Discrete { steps } = &self.kind {
            if steps.len() >= 2 {
                if let Some(index) = steps
                    .iter()
                    .position(|step| step.engine_value == self.range.default)
                {
                    return (index as f64 / (steps.len() - 1) as f64) as f32;
                }
            }
        }
        self.mapping.to_normalized(self.range.default)
    }
}
