//! NRT preparation and stable-ID access for the device host's initial parameter generation.
//!
//! This is deliberately a static lifecycle: one generation is prepared before
//! the callback starts and is never replaced. Issue #245 owns replacement,
//! rebinding, and retirement transitions.

use std::sync::Arc;

use anyhow::anyhow;
use engine::{
    AcceptedPublication, ApplicationFailureStatus, AppliedTargetStatus,
    CoalescedParameterPublisher, CoalescedParameterStore, CoalescedStoreCounters,
    CoalescedTargetBinding, InitialNormalizedValue, ParameterSnapshotStatus,
    ParameterTableGeneration, ParameterTableGenerations, ParameterTarget,
    PreparedCoalescedBindingTable, PreparedCoalescedParameterState, PublicationRejection,
    PublicationResult,
};
use param_manifest::{
    builtin::{builtin_manifest, MASTER_GAIN_ID},
    ParameterId, ParameterLookup, RuntimeParamKey,
};

const INITIAL_COALESCED_CAPACITY: usize = 1;

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
        if self.publisher.is_disconnected() {
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
        if self.publisher.is_disconnected() {
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
        if self.publisher.is_disconnected() {
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

    /// Close this static generation without replacing it. Later publication is
    /// observably rejected as [`PublicationRejection::Closed`].
    pub fn close(&self) {
        self.publisher.close();
    }

    /// End the owning engine lifecycle. Later publication is observably rejected
    /// and known-ID queries return a compact disconnected outcome.
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

/// NRT-prepared owners split between the callback Engine and stable-ID facade.
/// There is intentionally no replacement/install method after construction.
pub struct PreparedInitialParameterGeneration {
    state: PreparedCoalescedParameterState,
    facade: DeviceHostParameterFacade,
}

impl PreparedInitialParameterGeneration {
    #[must_use]
    pub fn into_parts(self) -> (PreparedCoalescedParameterState, DeviceHostParameterFacade) {
        (self.state, self.facade)
    }
}

/// Compile the current built-in manifest, authoritative initial snapshot,
/// atomic store, and concrete master-gain binding entirely on NRT.
pub fn prepare_initial_parameter_generation(
) -> Result<PreparedInitialParameterGeneration, anyhow::Error> {
    let manifest = builtin_manifest();
    let lookup = ParameterLookup::from_manifest(&manifest)
        .map_err(|error| anyhow!("invalid built-in parameter manifest: {error:?}"))?;
    let gain_id = ParameterId::from(MASTER_GAIN_ID);
    let gain_key = lookup
        .key_for(&gain_id)
        .ok_or_else(|| anyhow!("built-in manifest is missing {MASTER_GAIN_ID}"))?;
    // The manifest default is the single authority for this initial snapshot.
    // The store maps and invokes it before the first rendered sample, so the
    // concrete effect's NRT constructor value is not a second state authority.
    let initial_snapshot = [InitialNormalizedValue {
        key: gain_key,
        normalized: lookup
            .table()
            .default_normalized(gain_key)
            .ok_or_else(|| anyhow!("built-in gain has no normalized default"))?,
    }];

    let mut generations = ParameterTableGenerations::new();
    let store = CoalescedParameterStore::prepare_with_initial_values(
        &mut generations,
        lookup.table(),
        INITIAL_COALESCED_CAPACITY,
        &initial_snapshot,
    )
    .map_err(|error| anyhow!("failed to prepare initial parameter store: {error:?}"))?;
    let bindings = PreparedCoalescedBindingTable::prepare(
        &store,
        lookup.table(),
        &[CoalescedTargetBinding {
            key: gain_key,
            target: ParameterTarget::MasterGain,
        }],
    )
    .map_err(|error| anyhow!("failed to prepare initial parameter bindings: {error:?}"))?;

    let publisher = store.publisher();
    let mut stable_bindings = manifest
        .parameters
        .iter()
        .map(|descriptor| {
            let key = lookup
                .key_for(&descriptor.id)
                .expect("validated manifest entries have runtime keys");
            StableBinding {
                id: descriptor.id.clone(),
                key,
            }
        })
        .collect::<Vec<_>>();
    stable_bindings.sort_unstable_by(|left, right| left.id.cmp(&right.id));

    let state = PreparedCoalescedParameterState::new(lookup.into_table(), store, bindings)
        .map_err(|error| anyhow!("failed to assemble initial parameter state: {error:?}"))?;
    Ok(PreparedInitialParameterGeneration {
        state,
        facade: DeviceHostParameterFacade {
            bindings: Arc::from(stable_bindings.into_boxed_slice()),
            publisher,
        },
    })
}

// Keep the synchronous NRT publication result cheap to return by value. A
// future payload expansion that crosses this bound requires an explicit review
// rather than silently boxing/allocating in control adapters.
const _: () = assert!(std::mem::size_of::<StableParameterPublication>() <= 40);
