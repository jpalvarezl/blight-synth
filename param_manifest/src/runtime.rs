//! Bounded, string-free runtime lookup for the audio thread.
//!
//! Preparation happens on NRT from a validated [`ParameterManifest`], allocating
//! the backing store once. The result splits into two handles so the audio thread
//! never touches a `String` or a hash map:
//!
//! * [`RuntimeParameterTable`] is the **RT handle** — a bounded `Box<[RuntimeParameter]>`
//!   addressed by a compact [`RuntimeParamKey`] with an O(1) slice index. It owns
//!   no strings and never allocates on `get`.
//! * [`ParameterLookup`] is the **NRT owner** — it holds the table plus the
//!   `ParameterId` -> key resolver map, and hands `&RuntimeParameterTable` (or an
//!   owned table) to the callback.
//!
//! This matches the "Prepared-state rule" and "Continuous parameters" traffic
//! class of the real-time audio contract.

use std::collections::HashMap;

use crate::descriptor::{AutomationRate, ParameterId, ParameterKind, SmoothingPolicy};
use crate::manifest::{ManifestError, ParameterManifest};
use crate::mapping::Mapping;

/// Compact numeric handle for a parameter on the real-time path.
///
/// Keys are assigned densely (`0..len`) when the lookup is built, so they double
/// as the backing-store index. They are stable for the lifetime of a given
/// prepared lookup; the stable *cross-version* identity remains [`ParameterId`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RuntimeParamKey(pub u32);

/// String-free kind for the RT tier (discrete steps collapse to a count).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeKind {
    Continuous,
    Discrete { step_count: u32 },
}

/// The numeric, `Copy` projection of a descriptor used on the audio thread.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RuntimeParameter {
    /// Dense key / backing-store index.
    pub key: RuntimeParamKey,
    /// Engine parameter slot passed to the DSP node's `set_parameter`.
    pub engine_param_index: u32,
    /// Normalized `0..1` <-> engine-value mapping.
    pub mapping: Mapping,
    /// Continuous or discrete (step count only).
    pub kind: RuntimeKind,
    /// Real-time traffic class.
    pub automation_rate: AutomationRate,
    /// Smoothing policy for coalesced changes.
    pub smoothing: SmoothingPolicy,
    /// Default engine value.
    pub default_engine: f32,
    /// Minimum engine value.
    pub min_engine: f32,
    /// Maximum engine value.
    pub max_engine: f32,
}

impl RuntimeParameter {
    /// Convert a normalized `0..1` value to a clamped engine value.
    ///
    /// A validated manifest guarantees `min_engine <= max_engine`, but this type
    /// is `Copy` and could be constructed directly, so we normalize the clamp
    /// bounds defensively. `f32::clamp` panics when `min > max`; ordering the
    /// bounds here keeps this RT-facing conversion panic-free without allocating.
    #[must_use]
    pub fn normalized_to_engine(&self, normalized: f32) -> f32 {
        let (lo, hi) = if self.min_engine <= self.max_engine {
            (self.min_engine, self.max_engine)
        } else {
            (self.max_engine, self.min_engine)
        };
        self.mapping.to_engine(normalized).clamp(lo, hi)
    }
}

/// A bounded, string-free table of runtime parameters (the RT handle).
///
/// Addressed by [`RuntimeParamKey`] with an O(1) slice index. It owns no `String`
/// and no hash map, so it is safe to index on the audio thread.
#[derive(Debug, Clone)]
pub struct RuntimeParameterTable {
    entries: Box<[RuntimeParameter]>,
}

impl RuntimeParameterTable {
    /// Fetch a runtime parameter by key (RT-safe, bounded O(1) slice index).
    #[must_use]
    pub fn get(&self, key: RuntimeParamKey) -> Option<&RuntimeParameter> {
        self.entries.get(key.0 as usize)
    }

    /// Number of parameters in the table.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the table is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The entries as a bounded slice.
    #[must_use]
    pub fn entries(&self) -> &[RuntimeParameter] {
        &self.entries
    }
}

/// The NRT owner: a prepared table plus a stable-ID resolver.
///
/// * [`table`](ParameterLookup::table) yields the string-free RT handle.
/// * [`key_for`](ParameterLookup::key_for) resolves a stable [`ParameterId`] to a
///   key; this map is never touched on the audio thread.
#[derive(Debug, Clone)]
pub struct ParameterLookup {
    table: RuntimeParameterTable,
    index: HashMap<ParameterId, RuntimeParamKey>,
}

impl ParameterLookup {
    /// Prepare a lookup from a manifest (NRT). Descriptors keep their authored
    /// order, so keys are `0..parameters.len()`.
    ///
    /// The manifest is validated first via [`ParameterManifest::validate`], so
    /// duplicate IDs and inconsistent numeric fields (e.g. a reversed
    /// `min > max` range that would later panic `f32::clamp` on the RT path)
    /// are rejected here rather than silently reaching the audio thread.
    ///
    /// # Errors
    ///
    /// Returns the [`ManifestError`] produced by validation if the manifest is
    /// structurally invalid.
    pub fn from_manifest(manifest: &ParameterManifest) -> Result<Self, ManifestError> {
        manifest.validate()?;

        let mut entries = Vec::with_capacity(manifest.parameters.len());
        let mut index = HashMap::with_capacity(manifest.parameters.len());

        for (i, descriptor) in manifest.parameters.iter().enumerate() {
            let key = RuntimeParamKey(i as u32);
            let kind = match &descriptor.kind {
                ParameterKind::Continuous => RuntimeKind::Continuous,
                ParameterKind::Discrete { steps } => RuntimeKind::Discrete {
                    step_count: steps.len() as u32,
                },
            };
            entries.push(RuntimeParameter {
                key,
                engine_param_index: descriptor.owner.engine_param_index,
                mapping: descriptor.mapping,
                kind,
                automation_rate: descriptor.automation_rate,
                smoothing: descriptor.smoothing,
                default_engine: descriptor.range.default,
                min_engine: descriptor.range.min,
                max_engine: descriptor.range.max,
            });
            index.insert(descriptor.id.clone(), key);
        }

        Ok(Self {
            table: RuntimeParameterTable {
                entries: entries.into_boxed_slice(),
            },
            index,
        })
    }

    /// Resolve a stable ID to its runtime key (NRT).
    #[must_use]
    pub fn key_for(&self, id: &ParameterId) -> Option<RuntimeParamKey> {
        self.index.get(id).copied()
    }

    /// Borrow the string-free RT table handed to the audio thread.
    #[must_use]
    pub fn table(&self) -> &RuntimeParameterTable {
        &self.table
    }

    /// Take the owned string-free RT table (drops the NRT resolver).
    #[must_use]
    pub fn into_table(self) -> RuntimeParameterTable {
        self.table
    }

    /// Fetch a runtime parameter by key. Convenience delegating to the table.
    #[must_use]
    pub fn get(&self, key: RuntimeParamKey) -> Option<&RuntimeParameter> {
        self.table.get(key)
    }

    /// Number of parameters in the lookup.
    #[must_use]
    pub fn len(&self) -> usize {
        self.table.len()
    }

    /// Whether the lookup is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    /// The RT-facing entries as a bounded slice.
    #[must_use]
    pub fn entries(&self) -> &[RuntimeParameter] {
        self.table.entries()
    }
}
