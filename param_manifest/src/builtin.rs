//! Representative built-in descriptors.
//!
//! This module describes one existing parameter — the standalone master gain —
//! as a representative canonical descriptor. It is deliberately *not* wired into
//! the current engine/OSC path and is not a full catalog; consumer migration and
//! the remaining instrument/effect descriptors are follow-up work.
//!
//! The master gain today is converted ad hoc in the OSC adapter
//! (`audio_backend/src/standalone_process/osc.rs::normalized_gain_to_db`): a
//! normalized `0..1` linear amplitude is mapped to dB, then the engine `Gain`
//! effect re-converts dB to a linear factor. The descriptor below owns that
//! normalized<->dB conversion via [`Mapping::AmplitudeDecibel`], demonstrating
//! the shared conversion that future host-adapter consumers can bind to.

use crate::descriptor::{
    AutomationRate, NodeRef, NodeType, ParameterDescriptor, ParameterId, ParameterKind,
    SmoothingPolicy, Unit, ValueRange, Visibility,
};
use crate::manifest::{ParameterManifest, MANIFEST_SCHEMA_VERSION};
use crate::mapping::Mapping;

/// Stable ID of the standalone master gain parameter.
///
/// This matches the id the public OSC contract already accepts
/// (`/param/set gain <0..1>`, see `docs/osc-spec.md`), so a follow-up OSC
/// migration can call `ParameterLookup::key_for("gain")` without breaking
/// existing clients.
pub const MASTER_GAIN_ID: &str = "gain";

/// dB value treated as silence for the master gain mapping (matches the OSC
/// adapter's `GAIN_FLOOR_DB`).
pub const MASTER_GAIN_FLOOR_DB: f32 = -120.0;

/// Engine parameter slot retained for the canonical master-gain descriptor.
/// The dedicated `ParameterTarget::MasterGain` does not consume a user effect ID.
pub const MASTER_GAIN_PARAM_INDEX: u32 = 0;

/// Build the canonical descriptor for the master gain parameter.
#[must_use]
pub fn master_gain_descriptor() -> ParameterDescriptor {
    ParameterDescriptor {
        id: ParameterId::from(MASTER_GAIN_ID),
        owner: NodeRef {
            node_type: NodeType::MasterEffect,
            path: "master/effect:gain".to_string(),
            engine_param_index: MASTER_GAIN_PARAM_INDEX,
        },
        display_name: "Master Gain".to_string(),
        short_name: "Gain".to_string(),
        unit: Unit::Decibel,
        // Engine value is dB; unity (0 dB) is the default, floored at silence.
        range: ValueRange {
            min: MASTER_GAIN_FLOOR_DB,
            max: 0.0,
            default: 0.0,
        },
        mapping: Mapping::AmplitudeDecibel {
            floor_db: MASTER_GAIN_FLOOR_DB,
        },
        kind: ParameterKind::Continuous,
        automation_rate: AutomationRate::ControlCoalesced,
        // This matches the current command/OSC path: gain changes immediately.
        // Audible click/zipper reports are the trigger for DSP-local smoothing.
        smoothing: SmoothingPolicy::None,
        visibility: Visibility::default(),
        version_added: 1,
        deprecated: None,
    }
}

/// A minimal built-in manifest containing the representative descriptor(s).
#[must_use]
pub fn builtin_manifest() -> ParameterManifest {
    debug_assert_eq!(MANIFEST_SCHEMA_VERSION, 1);
    ParameterManifest::new(vec![master_gain_descriptor()])
}
