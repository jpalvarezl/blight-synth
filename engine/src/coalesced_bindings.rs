//! Prepared coalesced target bindings and direct block-start application.
//!
//! This module joins ADR 0005's normalized store to concrete Engine targets.
//! It owns no render phase or smoother: the exact runtime table maps each dirty
//! value once, then Engine invokes the prepared scalar target once.

use std::num::NonZeroU32;

use param_manifest::{
    AutomationRate, NodeType, RuntimeParamKey, RuntimeParameter, RuntimeParameterTable,
    RuntimeParameterTableIdentity, SmoothingPolicy,
};

use crate::{
    ApplicationFailureCode, CoalescedDrainSummary, CoalescedParameterPublisher,
    CoalescedParameterStore, DrainedPublication, ParameterApplicationResult,
    ParameterTableGeneration, ParameterTarget,
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
    /// Generic Engine bindings currently support only immediate application.
    UnsupportedSmoothingPolicy(RuntimeParamKey),
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
}

/// Why exact table/store/binding owners could not become one prepared state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreparedCoalescedParameterStateError {
    RuntimeTableMismatch,
    GenerationMismatch,
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
    TargetUnavailable = 8,
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

/// One concrete immediate-application target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PreparedCoalescedBinding {
    key: RuntimeParamKey,
    target: ParameterTarget,
    engine_param_index: u32,
}

impl PreparedCoalescedBinding {
    #[must_use]
    pub const fn key(self) -> RuntimeParamKey {
        self.key
    }

    #[must_use]
    pub const fn target(self) -> ParameterTarget {
        self.target
    }

    #[must_use]
    pub const fn engine_param_index(self) -> u32 {
        self.engine_param_index
    }
}

/// Exact-table, exact-generation prepared coalesced binding state.
///
/// Bindings are sorted by dense runtime key and contain only writable
/// `ControlCoalesced` parameters. Preparation requires exactly one supported
/// concrete target for every such table entry. Generic preparation accepts only
/// `SmoothingPolicy::None`; reviewed DSP-local capability can be added later.
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
            bindings.push(PreparedCoalescedBinding {
                key: target.key,
                target: target.target,
                engine_param_index: parameter.engine_param_index(),
            });
        }

        for parameter in table.entries().iter().copied().filter(|parameter| {
            parameter.automation_rate() == AutomationRate::ControlCoalesced
                && !parameter.read_only()
        }) {
            if bindings
                .binary_search_by_key(&parameter.key(), |binding| binding.key())
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
    pub fn is_for_table(&self, table: &RuntimeParameterTable) -> bool {
        table.has_identity(&self.table_identity)
    }

    #[must_use]
    pub fn entries(&self) -> &[PreparedCoalescedBinding] {
        &self.bindings
    }

    #[must_use]
    pub fn binding(&self, key: RuntimeParamKey) -> Option<PreparedCoalescedBinding> {
        self.bindings
            .binary_search_by_key(&key, |binding| binding.key())
            .ok()
            .map(|index| self.bindings[index])
    }

    /// Map one drained publication and invoke its concrete scalar target.
    ///
    /// `Applied` means the target resolver found the concrete target and invoked
    /// its existing infallible setter. A missing target is never confirmed.
    pub fn apply(
        &self,
        table: &RuntimeParameterTable,
        publication: DrainedPublication,
        mut apply_target: impl FnMut(PreparedCoalescedBinding, f32) -> bool,
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
        let Some(binding) = self.binding(publication.key) else {
            return CoalescedApplicationFailure::MissingBinding.failed();
        };
        if validate_target(parameter, binding.target).is_err() {
            return CoalescedApplicationFailure::TargetUnavailable.failed();
        }
        let Some(engine_target) = table
            .normalized_to_engine(publication.key, publication.normalized)
            .filter(|value| parameter_accepts(parameter, *value))
        else {
            return CoalescedApplicationFailure::MappingFailed.failed();
        };
        if !apply_target(binding, engine_target) {
            return CoalescedApplicationFailure::TargetUnavailable.failed();
        }
        ParameterApplicationResult::Applied
    }

    /// Drain the store exactly once through mapping and concrete application.
    pub fn drain(
        &self,
        table: &RuntimeParameterTable,
        store: &CoalescedParameterStore,
        mut apply_target: impl FnMut(PreparedCoalescedBinding, f32) -> bool,
    ) -> CoalescedDrainSummary {
        store.drain(|publication| self.apply(table, publication, &mut apply_target))
    }
}

/// Minimal constructor-time owner moved into one Engine.
///
/// This type intentionally has no live swap or retirement API. Create the NRT
/// publisher before moving the state into Engine; #215 owns replacement.
#[derive(Debug)]
pub struct PreparedCoalescedParameterState {
    table: RuntimeParameterTable,
    store: CoalescedParameterStore,
    bindings: PreparedCoalescedBindingTable,
}

impl PreparedCoalescedParameterState {
    pub fn new(
        table: RuntimeParameterTable,
        store: CoalescedParameterStore,
        bindings: PreparedCoalescedBindingTable,
    ) -> Result<Self, PreparedCoalescedParameterStateError> {
        if !store.is_for_table(&table) || !bindings.is_for_table(&table) {
            return Err(PreparedCoalescedParameterStateError::RuntimeTableMismatch);
        }
        if store.generation() != bindings.generation() {
            return Err(PreparedCoalescedParameterStateError::GenerationMismatch);
        }
        Ok(Self {
            table,
            store,
            bindings,
        })
    }

    #[must_use]
    pub fn publisher(&self) -> CoalescedParameterPublisher {
        self.store.publisher()
    }

    pub(crate) fn drain(
        &self,
        apply_target: impl FnMut(PreparedCoalescedBinding, f32) -> bool,
    ) -> CoalescedDrainSummary {
        self.bindings.drain(&self.table, &self.store, apply_target)
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
    if parameter.smoothing() != SmoothingPolicy::None {
        return Err(CoalescedBindingPrepareError::UnsupportedSmoothingPolicy(
            parameter.key(),
        ));
    }
    Ok(())
}

fn validate_target(
    parameter: RuntimeParameter,
    target: ParameterTarget,
) -> Result<(), CoalescedBindingPrepareError> {
    match parameter.node_type() {
        NodeType::MasterEffect => match target {
            ParameterTarget::MasterGain | ParameterTarget::MasterEffect { .. } => Ok(()),
            ParameterTarget::InstrumentEffect { .. } => {
                Err(CoalescedBindingPrepareError::TargetClassMismatch {
                    key: parameter.key(),
                    node_type: parameter.node_type(),
                })
            }
        },
        NodeType::InstrumentEffect => match target {
            ParameterTarget::InstrumentEffect { .. } => Ok(()),
            ParameterTarget::MasterGain | ParameterTarget::MasterEffect { .. } => {
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
