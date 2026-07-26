//! Bounded, string-free runtime lookup for the audio thread.
//!
//! Preparation happens on NRT from a validated [`ParameterManifest`], allocating
//! the backing stores once. The result splits into two handles:
//!
//! * [`RuntimeParameterTable`] is the **RT handle**. It owns bounded boxed slices
//!   of `Copy` parameter entries and numeric discrete values. [`get`](Self::get),
//!   [`entries`](Self::entries), and [`normalized_to_engine`](Self::normalized_to_engine)
//!   are bounded, non-allocating operations.
//! * [`ParameterLookup`] is the **NRT owner**. It adds the `ParameterId` -> key
//!   resolver map used while preparing control bindings.
//!
//! Owning tables are intentionally not `Clone`: cloning allocates. Installation,
//! replacement, and destruction of a table are prepared-state operations, so an
//! old table must be retired to NRT rather than dropped on the audio thread.

use std::collections::HashMap;

use crate::descriptor::{
    AutomationRate, ParameterDescriptor, ParameterId, ParameterKind, SmoothingPolicy,
};
use crate::manifest::{
    ManifestError, ParameterManifest, MAX_DISCRETE_STEP_COUNT, MAX_PARAMETER_COUNT,
};
use crate::mapping::Mapping;

/// Compact numeric handle for a parameter on the real-time path.
///
/// Keys are assigned densely (`0..len`) when the lookup is built. They are stable
/// for one prepared lookup; cross-version identity remains [`ParameterId`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RuntimeParamKey(pub u32);

/// String-free kind for the RT tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeKind {
    Continuous,
    /// Discrete numeric values live in the table's string-free value arena.
    Discrete {
        step_count: u32,
    },
}

/// The private-construction, numeric `Copy` projection used on the audio thread.
///
/// Entries can only be obtained from a validated [`ParameterLookup`]. Fields are
/// private so external callers cannot bypass preparation with NaN or reversed
/// bounds. Conversion still sanitizes every bound/value defensively: even a
/// malformed internal entry has a finite, panic-free range-floor fallback.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RuntimeParameter {
    key: RuntimeParamKey,
    engine_param_index: u32,
    mapping: Mapping,
    kind: RuntimeKind,
    automation_rate: AutomationRate,
    smoothing: SmoothingPolicy,
    default_engine: f32,
    min_engine: f32,
    max_engine: f32,
    discrete_values_offset: usize,
}

impl RuntimeParameter {
    #[must_use]
    pub fn key(self) -> RuntimeParamKey {
        self.key
    }

    #[must_use]
    pub fn engine_param_index(self) -> u32 {
        self.engine_param_index
    }

    #[must_use]
    pub fn mapping(self) -> Mapping {
        self.mapping
    }

    #[must_use]
    pub fn kind(self) -> RuntimeKind {
        self.kind
    }

    #[must_use]
    pub fn automation_rate(self) -> AutomationRate {
        self.automation_rate
    }

    #[must_use]
    pub fn smoothing(self) -> SmoothingPolicy {
        self.smoothing
    }

    #[must_use]
    pub fn default_engine(self) -> f32 {
        self.default_engine
    }

    #[must_use]
    pub fn min_engine(self) -> f32 {
        self.min_engine
    }

    #[must_use]
    pub fn max_engine(self) -> f32 {
        self.max_engine
    }

    fn from_validated_descriptor(
        descriptor: &ParameterDescriptor,
        key: RuntimeParamKey,
        kind: RuntimeKind,
        discrete_values_offset: usize,
    ) -> Self {
        Self {
            key,
            engine_param_index: descriptor.owner.engine_param_index,
            mapping: descriptor.mapping,
            kind,
            automation_rate: descriptor.automation_rate,
            smoothing: descriptor.smoothing,
            default_engine: descriptor.range.default,
            min_engine: descriptor.range.min,
            max_engine: descriptor.range.max,
            discrete_values_offset,
        }
    }

    fn normalized_to_engine(self, normalized: f32, discrete_values: &[f32]) -> f32 {
        let value = match self.kind {
            RuntimeKind::Continuous => self.mapping.to_engine(normalized),
            RuntimeKind::Discrete { step_count } => {
                self.discrete_value(normalized, discrete_values, step_count)
            }
        };
        self.sanitize_engine_value(value)
    }

    fn discrete_value(self, normalized: f32, discrete_values: &[f32], step_count: u32) -> f32 {
        let Some(count) = usize::try_from(step_count).ok() else {
            return self.sanitized_bounds().0;
        };
        let Some(end) = self.discrete_values_offset.checked_add(count) else {
            return self.sanitized_bounds().0;
        };
        let Some(values) = discrete_values.get(self.discrete_values_offset..end) else {
            return self.sanitized_bounds().0;
        };
        if values.is_empty() {
            return self.sanitized_bounds().0;
        }

        let t = if normalized.is_nan() {
            0.0
        } else {
            normalized.clamp(0.0, 1.0)
        };
        let last = values.len() - 1;
        let position = f64::from(t) * last as f64;
        let index = position.round() as usize;
        values
            .get(index)
            .copied()
            .unwrap_or_else(|| self.sanitized_bounds().0)
    }

    fn sanitize_engine_value(self, value: f32) -> f32 {
        let (lo, hi) = self.sanitized_bounds();
        if value.is_nan() {
            lo
        } else {
            value.clamp(lo, hi)
        }
    }

    fn sanitized_bounds(self) -> (f32, f32) {
        match (self.min_engine.is_finite(), self.max_engine.is_finite()) {
            (true, true) if self.min_engine <= self.max_engine => {
                (self.min_engine, self.max_engine)
            }
            (true, true) => (self.max_engine, self.min_engine),
            (true, false) => (self.min_engine, self.min_engine),
            (false, true) => (self.max_engine, self.max_engine),
            (false, false) => (0.0, 0.0),
        }
    }
}

// Keep the callback entry compact and prove that API changes cannot silently
// remove its Copy property. The exact layout is compiler-owned; the upper bound
// is the contract relevant to prepared table capacity/cache use.
const fn require_copy<T: Copy>() {}
const _: () = require_copy::<RuntimeParameter>();
const _: () = assert!(std::mem::size_of::<RuntimeParameter>() <= 64);

/// A bounded, string-free table of runtime parameters (the RT handle).
///
/// Read/conversion methods are RT-safe. Moving, replacing, or dropping this owner
/// is an NRT prepared-state lifecycle operation because its boxed slices deallocate.
#[derive(Debug)]
pub struct RuntimeParameterTable {
    entries: Box<[RuntimeParameter]>,
    discrete_values: Box<[f32]>,
}

impl RuntimeParameterTable {
    /// Fetch a runtime parameter by key (RT-safe, bounded O(1) slice index).
    #[must_use]
    pub fn get(&self, key: RuntimeParamKey) -> Option<&RuntimeParameter> {
        let index = usize::try_from(key.0).ok()?;
        self.entries.get(index)
    }

    /// Convert a normalized value using the prepared parameter and, for a
    /// discrete parameter, its exact numeric value arena (RT-safe and bounded).
    #[must_use]
    pub fn normalized_to_engine(&self, key: RuntimeParamKey, normalized: f32) -> Option<f32> {
        self.get(key)
            .map(|entry| entry.normalized_to_engine(normalized, &self.discrete_values))
    }

    /// Number of parameters in the table (RT-safe).
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the table is empty (RT-safe).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The entries as a bounded slice (RT-safe).
    #[must_use]
    pub fn entries(&self) -> &[RuntimeParameter] {
        &self.entries
    }
}

/// The NRT owner: a prepared table plus a stable-ID resolver.
///
/// This type allocates during construction and deallocates on drop. Resolve all
/// string IDs on NRT, then hand only the table to RT under the prepared-state
/// installation/retirement protocol.
#[derive(Debug)]
pub struct ParameterLookup {
    table: RuntimeParameterTable,
    index: HashMap<ParameterId, RuntimeParamKey>,
}

impl ParameterLookup {
    /// Prepare a lookup from a manifest (NRT). Descriptors keep authored order,
    /// so keys are dense. Validation runs before any prepared state is returned.
    pub fn from_manifest(manifest: &ParameterManifest) -> Result<Self, ManifestError> {
        manifest.validate()?;

        let total_discrete_values = manifest
            .parameters
            .iter()
            .map(|descriptor| match &descriptor.kind {
                ParameterKind::Continuous => 0,
                ParameterKind::Discrete { steps } => steps.len(),
            })
            .sum();
        let mut entries = Vec::with_capacity(manifest.parameters.len());
        let mut discrete_values = Vec::with_capacity(total_discrete_values);
        let mut index = HashMap::with_capacity(manifest.parameters.len());

        for (i, descriptor) in manifest.parameters.iter().enumerate() {
            let key_index = u32::try_from(i).map_err(|_| ManifestError::CapacityExceeded {
                what: "parameter key",
                count: manifest.parameters.len(),
                max: MAX_PARAMETER_COUNT,
            })?;
            let key = RuntimeParamKey(key_index);
            let discrete_values_offset = discrete_values.len();
            let kind = match &descriptor.kind {
                ParameterKind::Continuous => RuntimeKind::Continuous,
                ParameterKind::Discrete { steps } => {
                    let step_count = u32::try_from(steps.len()).map_err(|_| {
                        ManifestError::CapacityExceeded {
                            what: "discrete step count",
                            count: steps.len(),
                            max: MAX_DISCRETE_STEP_COUNT,
                        }
                    })?;
                    discrete_values.extend(steps.iter().map(|step| step.engine_value));
                    RuntimeKind::Discrete { step_count }
                }
            };
            entries.push(RuntimeParameter::from_validated_descriptor(
                descriptor,
                key,
                kind,
                discrete_values_offset,
            ));
            index.insert(descriptor.id.clone(), key);
        }

        Ok(Self {
            table: RuntimeParameterTable {
                entries: entries.into_boxed_slice(),
                discrete_values: discrete_values.into_boxed_slice(),
            },
            index,
        })
    }

    /// Resolve a stable ID to its runtime key (NRT only).
    #[must_use]
    pub fn key_for(&self, id: &ParameterId) -> Option<RuntimeParamKey> {
        self.index.get(id).copied()
    }

    /// Borrow the string-free RT table.
    #[must_use]
    pub fn table(&self) -> &RuntimeParameterTable {
        &self.table
    }

    /// Take the RT table while dropping the NRT resolver on the calling NRT thread.
    /// The returned table must still be retired to NRT before destruction.
    #[must_use]
    pub fn into_table(self) -> RuntimeParameterTable {
        self.table
    }

    /// Fetch a runtime parameter by key without hashing.
    ///
    /// The operation is bounded, but this NRT owner must not cross to RT; hand
    /// [`RuntimeParameterTable`] to the callback instead.
    #[must_use]
    pub fn get(&self, key: RuntimeParamKey) -> Option<&RuntimeParameter> {
        self.table.get(key)
    }

    /// Convert through the prepared table without hashing or allocation.
    ///
    /// The operation is bounded, but this NRT owner must not cross to RT; hand
    /// [`RuntimeParameterTable`] to the callback instead.
    #[must_use]
    pub fn normalized_to_engine(&self, key: RuntimeParamKey, normalized: f32) -> Option<f32> {
        self.table.normalized_to_engine(key, normalized)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.table.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    #[must_use]
    pub fn entries(&self) -> &[RuntimeParameter] {
        self.table.entries()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::descriptor::{SmoothingCurve, SmoothingPolicy};

    #[test]
    fn malformed_internal_entry_has_finite_panic_free_fallback() {
        // Private fields prevent this construction outside the crate. This test
        // keeps the final RT defense honest if internal preparation is refactored.
        let malformed = RuntimeParameter {
            key: RuntimeParamKey(0),
            engine_param_index: 0,
            mapping: Mapping::Linear {
                min: f32::NAN,
                max: f32::INFINITY,
            },
            kind: RuntimeKind::Discrete {
                step_count: u32::MAX,
            },
            automation_rate: AutomationRate::ControlCoalesced,
            smoothing: SmoothingPolicy::Smoothed {
                duration_ms: f32::NAN,
                curve: SmoothingCurve::Linear,
            },
            default_engine: f32::NAN,
            min_engine: f32::NAN,
            max_engine: -1.0,
            discrete_values_offset: usize::MAX,
        };

        let output = malformed.normalized_to_engine(f32::NAN, &[]);
        assert!(output.is_finite());
        assert_eq!(output, -1.0);

        let reversed = RuntimeParameter {
            mapping: Mapping::Linear {
                min: -1.0,
                max: 1.0,
            },
            kind: RuntimeKind::Continuous,
            min_engine: 1.0,
            max_engine: -1.0,
            ..malformed
        };
        assert_eq!(reversed.normalized_to_engine(0.5, &[]), 0.0);
    }
}
