//! Prepared coalesced target bindings and direct map/latch application.
//!
//! This module owns the bounded application between ADR 0005's normalized store
//! and ADR 0006's scalar smoothers. It deliberately does not own render phase,
//! DSP setter delivery, engine process integration, or host lifecycle.

use std::num::NonZeroU32;

use param_manifest::{
    AutomationRate, NodeType, RuntimeParamKey, RuntimeParameter, RuntimeParameterTable,
    RuntimeParameterTableIdentity,
};

use crate::{
    ApplicationFailureCode, CoalescedDrainSummary, CoalescedParameterStore, DrainedPublication,
    ParameterApplicationResult, ParameterSnapshotStatus, ParameterTableGeneration, ParameterTarget,
    PreparedSmoother, SmootherPrepareError,
};

/// NRT-resolved concrete target for one runtime key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoalescedTargetBinding {
    pub key: RuntimeParamKey,
    pub target: ParameterTarget,
}

/// Why a complete coalesced binding table could not be prepared on NRT.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoalescedBindingPrepareError {
    /// The store was prepared from a different exact runtime table.
    RuntimeTableMismatch,
    InvalidKey(RuntimeParamKey),
    NotControlCoalesced(RuntimeParamKey),
    ReadOnly(RuntimeParamKey),
    DuplicateBinding(RuntimeParamKey),
    MissingWritableBinding(RuntimeParamKey),
    /// The manifest node class has no concrete coalesced target in this slice.
    UnsupportedTargetClass {
        key: RuntimeParamKey,
        node_type: NodeType,
    },
    /// The concrete target variant does not match the manifest node class.
    TargetClassMismatch {
        key: RuntimeParamKey,
        node_type: NodeType,
    },
    InitialSnapshotUnavailable(RuntimeParamKey),
    InitialMappingFailed(RuntimeParamKey),
    Smoother {
        key: RuntimeParamKey,
        error: SmootherPrepareError,
    },
}

/// Compact RT application defect recorded by the coalesced store.
///
/// Rich context is recoverable from the generation/key/revision carried by the
/// store's `ApplicationFailure`; the callback records only this nonzero code.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoalescedApplicationFailure {
    GenerationMismatch = 1,
    RuntimeTableMismatch = 2,
    InvalidKey = 3,
    NotControlCoalesced = 4,
    ReadOnly = 5,
    MissingBinding = 6,
    MappingFailed = 7,
    UnsupportedTarget = 8,
    SmootherRejected = 9,
}

impl CoalescedApplicationFailure {
    #[must_use]
    pub fn code(self) -> ApplicationFailureCode {
        let code = NonZeroU32::new(self as u32)
            .expect("coalesced application failure discriminants are nonzero");
        ApplicationFailureCode::new(code)
    }

    fn failed(self) -> ParameterApplicationResult {
        ParameterApplicationResult::Failed(self.code())
    }
}

/// One concrete target and its exclusively owned prepared smoother.
#[derive(Debug, Clone, Copy)]
pub struct PreparedCoalescedBinding {
    key: RuntimeParamKey,
    target: ParameterTarget,
    engine_param_index: u32,
    smoother: PreparedSmoother,
}

impl PreparedCoalescedBinding {
    #[must_use]
    pub const fn key(&self) -> RuntimeParamKey {
        self.key
    }

    #[must_use]
    pub const fn target(&self) -> ParameterTarget {
        self.target
    }

    #[must_use]
    pub const fn engine_param_index(&self) -> u32 {
        self.engine_param_index
    }

    #[must_use]
    pub const fn smoother(&self) -> &PreparedSmoother {
        &self.smoother
    }
}

/// Exact-table, exact-generation prepared coalesced application state.
///
/// Bindings are sorted by dense runtime key and contain only writable
/// `ControlCoalesced` parameters. Preparation requires exactly one supported
/// concrete target for every such table entry and seeds one smoother from the
/// store's authoritative normalized snapshot mapped by that same table.
///
/// This owner allocates during preparation and must be retired/dropped on NRT.
/// [`Self::apply`] and [`Self::drain`] are bounded and allocation-free.
#[derive(Debug)]
pub struct PreparedCoalescedBindingTable {
    generation: ParameterTableGeneration,
    table_identity: RuntimeParameterTableIdentity,
    bindings: Box<[PreparedCoalescedBinding]>,
}

impl PreparedCoalescedBindingTable {
    pub fn prepare(
        store: &CoalescedParameterStore,
        table: &RuntimeParameterTable,
        sample_rate: f32,
        targets: &[CoalescedTargetBinding],
    ) -> Result<Self, CoalescedBindingPrepareError> {
        if !store.is_for_table(table) {
            return Err(CoalescedBindingPrepareError::RuntimeTableMismatch);
        }

        let mut targets = targets.to_vec();
        targets.sort_unstable_by_key(|binding| binding.key);
        if let Some(duplicate) = targets.windows(2).find(|pair| pair[0].key == pair[1].key) {
            return Err(CoalescedBindingPrepareError::DuplicateBinding(
                duplicate[0].key,
            ));
        }

        let mut bindings = Vec::with_capacity(targets.len());
        for target in targets {
            let parameter = table
                .get(target.key)
                .copied()
                .ok_or(CoalescedBindingPrepareError::InvalidKey(target.key))?;
            validate_parameter(parameter)?;
            validate_target(parameter, target.target)?;

            let snapshot = match store.latest(target.key) {
                ParameterSnapshotStatus::Available(snapshot)
                    if snapshot.generation == store.generation() =>
                {
                    snapshot
                }
                ParameterSnapshotStatus::InvalidKey
                | ParameterSnapshotStatus::NotControlCoalesced
                | ParameterSnapshotStatus::Available(_) => {
                    return Err(CoalescedBindingPrepareError::InitialSnapshotUnavailable(
                        target.key,
                    ));
                }
            };
            let seed = table
                .normalized_to_engine(target.key, snapshot.normalized)
                .filter(|value| parameter_accepts(parameter, *value))
                .ok_or(CoalescedBindingPrepareError::InitialMappingFailed(
                    target.key,
                ))?;
            let smoother = PreparedSmoother::prepare(parameter.smoothing(), sample_rate, seed)
                .map_err(|error| CoalescedBindingPrepareError::Smoother {
                    key: target.key,
                    error,
                })?;
            bindings.push(PreparedCoalescedBinding {
                key: target.key,
                target: target.target,
                engine_param_index: parameter.engine_param_index(),
                smoother,
            });
        }

        for parameter in table.entries().iter().copied().filter(|parameter| {
            parameter.automation_rate() == AutomationRate::ControlCoalesced
                && !parameter.read_only()
        }) {
            if bindings
                .binary_search_by_key(&parameter.key(), PreparedCoalescedBinding::key)
                .is_err()
            {
                return Err(CoalescedBindingPrepareError::MissingWritableBinding(
                    parameter.key(),
                ));
            }
        }

        Ok(Self {
            generation: store.generation(),
            table_identity: table.identity(),
            bindings: bindings.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn generation(&self) -> ParameterTableGeneration {
        self.generation
    }

    #[must_use]
    pub fn entries(&self) -> &[PreparedCoalescedBinding] {
        &self.bindings
    }

    #[must_use]
    pub fn binding(&self, key: RuntimeParamKey) -> Option<&PreparedCoalescedBinding> {
        self.bindings
            .binary_search_by_key(&key, PreparedCoalescedBinding::key)
            .ok()
            .map(|index| &self.bindings[index])
    }

    /// Apply one drained normalized publication by exact-table mapping followed
    /// by a successful smoother target latch. `Applied` is returned only after
    /// both operations succeed; it means target-latched, not DSP-delivered or
    /// ramp-settled.
    pub fn apply(
        &mut self,
        table: &RuntimeParameterTable,
        publication: DrainedPublication,
    ) -> ParameterApplicationResult {
        if publication.generation != self.generation {
            return CoalescedApplicationFailure::GenerationMismatch.failed();
        }
        if !table.has_identity(&self.table_identity) {
            return CoalescedApplicationFailure::RuntimeTableMismatch.failed();
        }
        let Some(parameter) = table.get(publication.key).copied() else {
            return CoalescedApplicationFailure::InvalidKey.failed();
        };
        if parameter.automation_rate() != AutomationRate::ControlCoalesced {
            return CoalescedApplicationFailure::NotControlCoalesced.failed();
        }
        if parameter.read_only() {
            return CoalescedApplicationFailure::ReadOnly.failed();
        }
        let Ok(index) = self
            .bindings
            .binary_search_by_key(&publication.key, PreparedCoalescedBinding::key)
        else {
            return CoalescedApplicationFailure::MissingBinding.failed();
        };
        if validate_target(parameter, self.bindings[index].target).is_err() {
            return CoalescedApplicationFailure::UnsupportedTarget.failed();
        }
        let Some(engine_target) = table
            .normalized_to_engine(publication.key, publication.normalized)
            .filter(|value| parameter_accepts(parameter, *value))
        else {
            return CoalescedApplicationFailure::MappingFailed.failed();
        };
        if self.bindings[index]
            .smoother
            .latch_target(engine_target)
            .is_err()
        {
            return CoalescedApplicationFailure::SmootherRejected.failed();
        }
        ParameterApplicationResult::Applied
    }

    /// Directly drain a store through this table's application closure.
    ///
    /// Store confirmation remains owned by `CoalescedParameterStore::drain` and
    /// therefore advances only for publications for which [`Self::apply`]
    /// returns `Applied`.
    pub fn drain(
        &mut self,
        table: &RuntimeParameterTable,
        store: &CoalescedParameterStore,
    ) -> CoalescedDrainSummary {
        store.drain(|publication| self.apply(table, publication))
    }
}

fn validate_parameter(parameter: RuntimeParameter) -> Result<(), CoalescedBindingPrepareError> {
    if parameter.automation_rate() != AutomationRate::ControlCoalesced {
        return Err(CoalescedBindingPrepareError::NotControlCoalesced(
            parameter.key(),
        ));
    }
    if parameter.read_only() {
        return Err(CoalescedBindingPrepareError::ReadOnly(parameter.key()));
    }
    Ok(())
}

fn validate_target(
    parameter: RuntimeParameter,
    target: ParameterTarget,
) -> Result<(), CoalescedBindingPrepareError> {
    match parameter.node_type() {
        NodeType::MasterEffect => match target {
            ParameterTarget::MasterEffect { .. } => Ok(()),
            ParameterTarget::InstrumentEffect { .. } => {
                Err(CoalescedBindingPrepareError::TargetClassMismatch {
                    key: parameter.key(),
                    node_type: parameter.node_type(),
                })
            }
        },
        NodeType::InstrumentEffect => match target {
            ParameterTarget::InstrumentEffect { .. } => Ok(()),
            ParameterTarget::MasterEffect { .. } => {
                Err(CoalescedBindingPrepareError::TargetClassMismatch {
                    key: parameter.key(),
                    node_type: parameter.node_type(),
                })
            }
        },
        NodeType::VoiceEffect | NodeType::Instrument => {
            Err(CoalescedBindingPrepareError::UnsupportedTargetClass {
                key: parameter.key(),
                node_type: parameter.node_type(),
            })
        }
    }
}

fn parameter_accepts(parameter: RuntimeParameter, value: f32) -> bool {
    value.is_finite() && (parameter.min_engine()..=parameter.max_engine()).contains(&value)
}

const _: () = assert!(std::mem::size_of::<CoalescedApplicationFailure>() <= 4);
