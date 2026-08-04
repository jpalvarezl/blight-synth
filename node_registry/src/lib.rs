//! Versioned node definitions and the statically compiled built-in DSP registry.
//!
//! This crate is an NRT control-plane layer above `dsp`:
//!
//! 1. Deserialize [`InstrumentDefinition`] / [`EffectDefinition`] on NRT.
//! 2. Resolve stable kind IDs through [`BuiltInRegistry`] on NRT.
//! 3. Hand the returned prepared owners to the engine through its bounded
//!    prepared-state lifecycle.
//!
//! Registry calls parse JSON values, allocate DSP owners and buffers, resolve
//! resources, and build rich diagnostics. They must never run in an audio
//! callback. The workspace architecture checker enforces that callback-owning
//! `dsp` and `engine` cannot depend on this crate. There is no runtime module
//! loading or third-party DSP ABI.
//!
//! Definitions deliberately stop before project snapshots, tracker hydration,
//! and routing. An instrument's `effects` vector records deterministic order and
//! independently typed slot identity only; installation topology remains #136.

mod definitions;
mod diagnostics;
mod registry;

pub use definitions::{
    EffectDefinition, EffectKindId, InstrumentDefinition, InstrumentKindId, ParameterPayload,
    NODE_DEFINITION_SCHEMA_VERSION,
};
pub use diagnostics::{
    InvalidDefinitionCode, InvalidDefinitionDiagnostic, NodeCategory, PreparationError,
};
pub use registry::{
    kind, BuiltInKindDescriptor, BuiltInRegistry, EffectLayout, NrtPreparationContext,
    PreparedEffect, PreparedInstrumentDefinition, SampleResolver,
};
