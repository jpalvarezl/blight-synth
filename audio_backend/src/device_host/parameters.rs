//! NRT preparation, stable-ID rebinding, and replacement lifecycle for the
//! device host's coalesced parameter generations.

use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use anyhow::anyhow;
use engine::{
    AcceptedPublication, ApplicationFailureStatus, AppliedTargetStatus,
    CoalescedParameterPublisher, CoalescedParameterStore, CoalescedStoreCounters,
    CoalescedTargetBinding, InitialNormalizedValue, ParameterSnapshotStatus,
    ParameterTableGeneration, ParameterTableGenerations, ParameterTarget,
    PreparedCoalescedBindingTable, PreparedCoalescedParameterState, PublicationRejection,
    PublicationResult, MAX_COALESCED_PARAMETER_COUNT,
};
use param_manifest::{
    builtin::{builtin_manifest, MASTER_GAIN_ID},
    ParameterId, ParameterLookup, ParameterManifest, RuntimeParamKey,
};

use crate::{Command, ParameterGenerationCommand};

#[derive(Debug)]
struct StableBinding {
    id: ParameterId,
    key: RuntimeParamKey,
}

/// Compact result of publishing by stable [`ParameterId`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StableParameterPublication {
    Accepted(AcceptedPublication),
    UnknownParameter,
    Rejected(PublicationRejection),
}

/// Compact result of a stable-ID state query.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StableParameterQuery<T> {
    Value(T),
    UnknownParameter,
    Disconnected,
    /// The ID resolved in the NRT facade but is not a valid coalesced entry in
    /// the active prepared generation. This indicates a preparation defect.
    InvalidParameter,
}

/// Cloneable NRT facade for the one active device-host generation.
///
/// Stable IDs are resolved in an immutable NRT-owned table. Publication and
/// confirmation then use the exact generation-bound dense key. The facade may
/// allocate/deallocate through `Arc` cloning/drop and must never be used or
/// finally dropped by the audio callback.
#[derive(Debug, Clone)]
pub struct DeviceHostParameterFacade {
    bindings: Arc<[StableBinding]>,
    publisher: CoalescedParameterPublisher,
    /// Engine-instance connection shared by every generation facade.
    connected: Arc<AtomicBool>,
}

impl DeviceHostParameterFacade {
    #[must_use]
    pub fn generation(&self) -> ParameterTableGeneration {
        self.publisher.generation()
    }

    pub fn publish(&self, id: &ParameterId, normalized: f32) -> StableParameterPublication {
        let Some(key) = self.key_for(id) else {
            return StableParameterPublication::UnknownParameter;
        };
        if !self.connected.load(Ordering::Acquire) {
            self.publisher.disconnect();
        }
        match self.publisher.publish(key, normalized) {
            PublicationResult::Accepted(accepted) => StableParameterPublication::Accepted(accepted),
            PublicationResult::Rejected(rejection) => {
                StableParameterPublication::Rejected(rejection)
            }
        }
    }

    #[must_use]
    pub fn latest(&self, id: &ParameterId) -> StableParameterQuery<engine::ParameterSnapshot> {
        let Some(key) = self.key_for(id) else {
            return StableParameterQuery::UnknownParameter;
        };
        if !self.connected.load(Ordering::Acquire) || self.publisher.is_disconnected() {
            return StableParameterQuery::Disconnected;
        }
        match self.publisher.latest(key) {
            ParameterSnapshotStatus::Available(snapshot) => StableParameterQuery::Value(snapshot),
            ParameterSnapshotStatus::InvalidKey | ParameterSnapshotStatus::NotControlCoalesced => {
                StableParameterQuery::InvalidParameter
            }
        }
    }

    #[must_use]
    pub fn applied(&self, id: &ParameterId) -> StableParameterQuery<AppliedTargetStatus> {
        let Some(key) = self.key_for(id) else {
            return StableParameterQuery::UnknownParameter;
        };
        if !self.connected.load(Ordering::Acquire) || self.publisher.is_disconnected() {
            return StableParameterQuery::Disconnected;
        }
        match self.publisher.applied(key) {
            status @ (AppliedTargetStatus::Pending { .. } | AppliedTargetStatus::Applied(_)) => {
                StableParameterQuery::Value(status)
            }
            AppliedTargetStatus::InvalidKey | AppliedTargetStatus::NotControlCoalesced => {
                StableParameterQuery::InvalidParameter
            }
        }
    }

    #[must_use]
    pub fn last_application_failure(
        &self,
        id: &ParameterId,
    ) -> StableParameterQuery<ApplicationFailureStatus> {
        let Some(key) = self.key_for(id) else {
            return StableParameterQuery::UnknownParameter;
        };
        if !self.connected.load(Ordering::Acquire) || self.publisher.is_disconnected() {
            return StableParameterQuery::Disconnected;
        }
        match self.publisher.last_application_failure(key) {
            status @ (ApplicationFailureStatus::None | ApplicationFailureStatus::Failed(_)) => {
                StableParameterQuery::Value(status)
            }
            ApplicationFailureStatus::InvalidKey
            | ApplicationFailureStatus::NotControlCoalesced => {
                StableParameterQuery::InvalidParameter
            }
        }
    }

    #[must_use]
    pub fn counters(&self) -> CoalescedStoreCounters {
        self.publisher.counters()
    }

    /// Close this generation. Later publication is observably rejected as
    /// [`PublicationRejection::Closed`] unless the engine is disconnected.
    pub fn close(&self) {
        self.publisher.close();
    }

    /// Disconnect only this generation's publisher. Ending the complete engine
    /// lifecycle is reserved for the NRT [`DeviceHostParameterLifecycle`] owner,
    /// so an outliving stale facade cannot disconnect a newer generation.
    pub fn disconnect(&self) {
        self.publisher.disconnect();
    }

    fn key_for(&self, id: &ParameterId) -> Option<RuntimeParamKey> {
        self.bindings
            .binary_search_by(|binding| binding.id.cmp(id))
            .ok()
            .map(|index| self.bindings[index].key)
    }
}

/// One stable target assignment used while compiling a replacement manifest.
#[derive(Debug, Clone, PartialEq)]
struct StableParameterTargetBinding {
    id: ParameterId,
    target: ParameterTarget,
}

/// Adapter-owned desired value replayed into a new generation by stable ID.
#[derive(Debug, Clone, PartialEq)]
pub struct DesiredParameterValue {
    pub id: ParameterId,
    pub normalized: f32,
}

/// Observable NRT transition. Lists are sorted by stable ID and duplicate
/// desired entries are deterministically collapsed with the last value winning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParameterGenerationTransition {
    pub previous: ParameterTableGeneration,
    pub current: ParameterTableGeneration,
    pub rebound: Vec<ParameterId>,
    /// Desired IDs absent from the replacement manifest. They remain the
    /// adapter's responsibility and are never reinterpreted as a dense key.
    pub removed_or_missing: Vec<ParameterId>,
}

/// One complete replacement owner plus its NRT facade/transition report.
pub struct PreparedParameterReplacement {
    state: Arc<PreparedCoalescedParameterState>,
    facade: DeviceHostParameterFacade,
    transition: ParameterGenerationTransition,
}

impl PreparedParameterReplacement {
    #[must_use]
    pub fn into_parts(
        self,
    ) -> (
        Command,
        DeviceHostParameterFacade,
        ParameterGenerationTransition,
    ) {
        (
            Command::ReplaceParameterGeneration(ParameterGenerationCommand::new(self.state)),
            self.facade,
            self.transition,
        )
    }
}

/// NRT owner of the monotonic generation sequence and currently published
/// facade. It never crosses into the callback.
pub struct DeviceHostParameterLifecycle {
    generations: ParameterTableGenerations,
    active: DeviceHostParameterFacade,
}

impl DeviceHostParameterLifecycle {
    #[must_use]
    pub fn facade(&self) -> DeviceHostParameterFacade {
        self.active.clone()
    }

    /// Prepare the current built-in device-host manifest with deterministic
    /// stable-ID desired replay.
    pub fn prepare_builtin_replacement(
        &mut self,
        desired: &[DesiredParameterValue],
    ) -> Result<PreparedParameterReplacement, anyhow::Error> {
        self.prepare_replacement(
            builtin_manifest(),
            &[StableParameterTargetBinding {
                id: ParameterId::from(MASTER_GAIN_ID),
                target: ParameterTarget::MasterGain,
            }],
            desired,
        )
    }

    /// Close the active generation and compile a complete replacement. Desired
    /// state is rebound by stable ID and represented directly in the initial
    /// dirty snapshot, so RT applies/confirms it before rendering the first
    /// block under the new generation.
    ///
    /// Closure is deliberately the first lifecycle step required by ADR 0005.
    /// A later preparation error is therefore fail-closed: the old physical
    /// generation remains queryable but no longer accepts writes; a caller may
    /// correct the input and prepare a later monotonic generation.
    fn prepare_replacement(
        &mut self,
        manifest: ParameterManifest,
        targets: &[StableParameterTargetBinding],
        desired: &[DesiredParameterValue],
    ) -> Result<PreparedParameterReplacement, anyhow::Error> {
        let previous = self.active.generation();
        self.active.close();

        let lookup = ParameterLookup::from_manifest(&manifest)
            .map_err(|error| anyhow!("invalid replacement parameter manifest: {error:?}"))?;
        let desired = desired.iter().fold(BTreeMap::new(), |mut values, entry| {
            values.insert(entry.id.clone(), entry.normalized);
            values
        });
        let mut initial_values = Vec::with_capacity(desired.len());
        let mut rebound = Vec::with_capacity(desired.len());
        let mut removed_or_missing = Vec::new();
        for (id, normalized) in desired {
            if let Some(key) = lookup.key_for(&id) {
                initial_values.push(InitialNormalizedValue { key, normalized });
                rebound.push(id);
            } else {
                removed_or_missing.push(id);
            }
        }

        let (state, facade) = prepare_generation(
            &mut self.generations,
            manifest,
            lookup,
            targets,
            &initial_values,
            Arc::clone(&self.active.connected),
        )?;
        let transition = ParameterGenerationTransition {
            previous,
            current: facade.generation(),
            rebound,
            removed_or_missing,
        };
        self.active = facade.clone();
        Ok(PreparedParameterReplacement {
            state: Arc::new(state),
            facade,
            transition,
        })
    }

    pub fn disconnect(&self) {
        self.active.connected.store(false, Ordering::Release);
        self.active.publisher.disconnect();
    }
}

/// NRT-prepared initial owners. Callers needing replacement retain the lifecycle;
/// static/offline tests may continue splitting out only the initial facade.
pub struct PreparedInitialParameterGeneration {
    state: PreparedCoalescedParameterState,
    lifecycle: DeviceHostParameterLifecycle,
}

impl PreparedInitialParameterGeneration {
    #[must_use]
    pub fn into_parts(self) -> (PreparedCoalescedParameterState, DeviceHostParameterFacade) {
        (self.state, self.lifecycle.active)
    }

    #[must_use]
    pub fn into_lifecycle_parts(
        self,
    ) -> (
        PreparedCoalescedParameterState,
        DeviceHostParameterLifecycle,
    ) {
        (self.state, self.lifecycle)
    }
}

/// Compile the built-in manifest/default snapshot, atomic store, concrete
/// master-gain binding, and engine-local generation allocator entirely on NRT.
pub fn prepare_initial_parameter_generation(
) -> Result<PreparedInitialParameterGeneration, anyhow::Error> {
    let manifest = builtin_manifest();
    let targets = [StableParameterTargetBinding {
        id: ParameterId::from(MASTER_GAIN_ID),
        target: ParameterTarget::MasterGain,
    }];
    let mut generations = ParameterTableGenerations::new();
    let connected = Arc::new(AtomicBool::new(true));
    let lookup = ParameterLookup::from_manifest(&manifest)
        .map_err(|error| anyhow!("invalid built-in parameter manifest: {error:?}"))?;
    let (state, active) =
        prepare_generation(&mut generations, manifest, lookup, &targets, &[], connected)?;
    Ok(PreparedInitialParameterGeneration {
        state,
        lifecycle: DeviceHostParameterLifecycle {
            generations,
            active,
        },
    })
}

fn prepare_generation(
    generations: &mut ParameterTableGenerations,
    manifest: ParameterManifest,
    lookup: ParameterLookup,
    targets: &[StableParameterTargetBinding],
    initial_values: &[InitialNormalizedValue],
    connected: Arc<AtomicBool>,
) -> Result<(PreparedCoalescedParameterState, DeviceHostParameterFacade), anyhow::Error> {
    let store = CoalescedParameterStore::prepare_with_initial_values(
        generations,
        lookup.table(),
        MAX_COALESCED_PARAMETER_COUNT,
        initial_values,
    )
    .map_err(|error| anyhow!("failed to prepare parameter store: {error:?}"))?;
    let target_bindings = targets
        .iter()
        .map(|binding| {
            lookup
                .key_for(&binding.id)
                .map(|key| CoalescedTargetBinding {
                    key,
                    target: binding.target,
                })
                .ok_or_else(|| anyhow!("target parameter `{}` is absent", binding.id))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let bindings = PreparedCoalescedBindingTable::prepare(&store, lookup.table(), &target_bindings)
        .map_err(|error| anyhow!("failed to prepare parameter bindings: {error:?}"))?;
    let publisher = store.publisher();
    let mut stable_bindings = manifest
        .parameters
        .into_iter()
        .map(|descriptor| StableBinding {
            key: lookup
                .key_for(&descriptor.id)
                .expect("validated manifest entries have runtime keys"),
            id: descriptor.id,
        })
        .collect::<Vec<_>>();
    stable_bindings.sort_unstable_by(|left, right| left.id.cmp(&right.id));
    let state = PreparedCoalescedParameterState::new(lookup.into_table(), store, bindings)
        .map_err(|error| anyhow!("failed to assemble parameter state: {error:?}"))?;
    Ok((
        state,
        DeviceHostParameterFacade {
            bindings: Arc::from(stable_bindings.into_boxed_slice()),
            publisher,
            connected,
        },
    ))
}

// Keep the synchronous NRT publication result cheap to return by value. A
// future payload expansion that crosses this bound requires an explicit review
// rather than silently boxing/allocating in control adapters.
const _: () = assert!(std::mem::size_of::<StableParameterPublication>() <= 40);
