//! Host-neutral portable project state and RFC 8785 canonical byte boundary.
//!
//! This crate owns authored/reconstructible data only. It cannot name an audio
//! device, filesystem path, sequencer `Song`, live engine, prepared DSP owner,
//! transport, callback queue, or UI state. Parsing, migration, asset validation,
//! and diagnostics are NRT operations; engine preparation/restore is issue #243.

mod canonical;
mod model;

pub use canonical::{
    decode_canonical, migrate_v0, NodeDefinitionCategory, PayloadCategory, StateDiagnostic,
    StateError,
};
pub use model::{
    AssetReference, AssetResolver, CheckpointRecord, DeterministicSeed, NodeAddress,
    NormalizedValue, ParameterValue, PortableStateV1, ResolvedAsset, TaggedPayload,
    FIXED_ROUTING_KIND, FIXED_ROUTING_SCHEMA_VERSION, PORTABLE_STATE_SCHEMA_VERSION,
    TRACKER_COMPOSITION_KIND, TRACKER_COMPOSITION_SCHEMA_VERSION,
};
