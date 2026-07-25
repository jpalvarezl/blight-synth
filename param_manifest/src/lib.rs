//! Canonical parameter manifest and runtime lookup.
//!
//! This crate is the single source of truth for parameter *metadata* across the
//! Blight Synth boundaries: the Rust DSP/engine, project state, the TypeScript /
//! Svelte UI, the OSC transport, and a future JUCE/APVTS plugin host. It owns the
//! descriptor shape, the normalized `0..1` <-> engine-value mapping, the unit and
//! smoothing policy, and the automation-rate classification. Host adapters bind to
//! a descriptor by its stable parameter ID and reuse the shared mapping/smoothing
//! rather than re-deriving unit conversion per transport.
//!
//! The crate has two tiers, mirroring the
//! [real-time audio contract](../../../docs/architecture/realtime-contract.md)
//! "Prepared-state rule":
//!
//! * A serializable, human-authored **manifest** ([`ParameterManifest`] of
//!   [`ParameterDescriptor`]s) that carries all descriptive strings, labels, and
//!   versioning metadata. It is authored/parsed/validated exclusively off the
//!   audio thread (NRT).
//! * A bounded, string-free **runtime lookup** ([`ParameterLookup`] of
//!   [`RuntimeParameter`]s) prepared on NRT and consumed on the audio thread by a
//!   compact numeric [`RuntimeParamKey`]. The RT tier holds only `Copy` numeric
//!   data, so it never allocates, hashes, or touches a `String` on the callback.
//!
//! See ADR 0003 (`docs/decisions/0003-parameter-manifest.md`) for the design
//! rationale and compatibility rules.

mod compatibility;
mod descriptor;
mod manifest;
mod mapping;
mod runtime;

pub mod builtin;

pub use compatibility::{CompatibilityBreak, CompatibilityReport};
pub use descriptor::{
    AutomationRate, DiscreteStep, NodeRef, NodeType, ParameterDescriptor, ParameterId,
    ParameterKind, SmoothingCurve, SmoothingPolicy, Unit, ValueRange, Visibility,
};
pub use manifest::{ManifestError, ParameterManifest, MANIFEST_SCHEMA_VERSION};
pub use mapping::Mapping;
pub use runtime::{ParameterLookup, RuntimeKind, RuntimeParamKey, RuntimeParameter, RuntimeParameterTable};
