//! Generation-bound, normalized coalesced parameter publication.
//!
//! This module implements ADR 0005's host-independent MPSC publication primitive.
//! Preparation and publication are NRT operations. [`CoalescedParameterStore::drain`]
//! is the bounded, allocation-free single-consumer operation intended for RT.

use std::{
    num::{NonZeroU32, NonZeroU64},
    sync::{
        atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
        Arc,
    },
};

use param_manifest::{
    AutomationRate, RuntimeParamKey, RuntimeParameterTable, RuntimeParameterTableIdentity,
};

use crate::parameter_atomic_protocol::{
    APPLIED_OBSERVE, APPLIED_PUBLISH, DIRTY_CONSUME, DIRTY_PUBLISH, GENERATION_CLOSE,
    GENERATION_OBSERVE, SLOT_CAS_FAILURE, SLOT_CAS_SUCCESS, SLOT_CONSUME,
};

/// Hard maximum number of active coalesced parameters in one generation.
pub const MAX_COALESCED_PARAMETER_COUNT: usize = 1_024;
/// Fixed dirty bitmap size scanned at every control boundary.
pub const COALESCED_DIRTY_WORD_COUNT: usize = 16;
const BITS_PER_DIRTY_WORD: usize = u64::BITS as usize;

const fn pack(revision: u32, payload: u32) -> u64 {
    ((revision as u64) << 32) | payload as u64
}

const fn unpack_revision(word: u64) -> u32 {
    (word >> 32) as u32
}

const fn unpack_payload(word: u64) -> u32 {
    word as u32
}

fn canonical_normalized(value: f32) -> Option<f32> {
    if !value.is_finite() {
        return None;
    }
    let clamped = value.clamp(0.0, 1.0);
    // Give zero one host-visible representation rather than preserving -0.0.
    Some(if clamped == 0.0 { 0.0 } else { clamped })
}

/// Nonzero generation identity. Values are created monotonically by
/// [`ParameterTableGenerations`] and are never wrapped or reused by it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ParameterTableGeneration(NonZeroU64);

impl ParameterTableGeneration {
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0.get()
    }
}

/// Exhaustion of the non-reused engine-local generation identity space.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GenerationExhausted;

/// Engine-instance-local monotonic generation allocator.
///
/// Keep one allocator for the lifetime of an engine instance. Exhaustion is
/// terminal and requires constructing a new engine instance; the sequence never
/// wraps to generation one.
#[derive(Debug)]
pub struct ParameterTableGenerations {
    next: u64,
    exhausted: bool,
}

impl Default for ParameterTableGenerations {
    fn default() -> Self {
        Self::new()
    }
}

impl ParameterTableGenerations {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            next: 1,
            exhausted: false,
        }
    }

    pub fn next_generation(&mut self) -> Result<ParameterTableGeneration, GenerationExhausted> {
        if self.exhausted {
            return Err(GenerationExhausted);
        }
        let generation =
            ParameterTableGeneration(NonZeroU64::new(self.next).ok_or(GenerationExhausted)?);
        if self.next == u64::MAX {
            self.exhausted = true;
        } else {
            self.next += 1;
        }
        Ok(generation)
    }

    #[cfg(test)]
    const fn starting_at(next: u64) -> Self {
        Self {
            next,
            exhausted: false,
        }
    }
}

/// One authoritative normalized seed override for generation preparation.
/// Coalesced keys not listed use their descriptor defaults.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InitialNormalizedValue {
    pub key: RuntimeParamKey,
    pub normalized: f32,
}

/// NRT preparation failure. No partially prepared store is returned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoalescedStorePrepareError {
    GenerationExhausted,
    CoalescedLimitTooHigh { requested: usize, maximum: usize },
    CoalescedCapacityExceeded { count: usize, limit: usize },
    InvalidInitialKey(RuntimeParamKey),
    InitialKeyNotCoalesced(RuntimeParamKey),
    DuplicateInitialKey(RuntimeParamKey),
    InvalidInitialValue(RuntimeParamKey),
}

/// Failure while closing an old generation and preparing its semantic reset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoalescedStoreResetError {
    GenerationExhausted,
    Preparation(CoalescedStorePrepareError),
}

/// Nonzero generation-local publication revision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PublicationRevision(NonZeroU32);

impl PublicationRevision {
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0.get()
    }

    fn from_word(word: u64) -> Option<Self> {
        NonZeroU32::new(unpack_revision(word)).map(Self)
    }
}

/// Successful publication metadata, all in the normalized host-visible domain.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AcceptedPublication {
    pub generation: ParameterTableGeneration,
    pub key: RuntimeParamKey,
    pub revision: PublicationRevision,
    pub canonical_normalized: f32,
    pub replaced_pending: bool,
}

/// Compact reason a publication did not become an accepted active-generation
/// write. Validation failures before CAS never set a dirty bit.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublicationRejection {
    InvalidKey,
    NotControlCoalesced,
    ReadOnly,
    InvalidValue,
    Closed,
    StaleGeneration,
    RevisionExhausted,
    Disconnected,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PublicationResult {
    Accepted(AcceptedPublication),
    Rejected(PublicationRejection),
}

/// Bounded, saturating producer-side diagnostics.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CoalescedStoreCounters {
    pub invalid_writes: u32,
    pub stale_writes: u32,
    pub disconnected_writes: u32,
    pub coalesced_writes: u32,
    pub revision_exhausted_writes: u32,
}

#[derive(Debug)]
struct Counters {
    invalid_writes: AtomicU32,
    stale_writes: AtomicU32,
    disconnected_writes: AtomicU32,
    coalesced_writes: AtomicU32,
    revision_exhausted_writes: AtomicU32,
}

impl Counters {
    fn new() -> Self {
        Self {
            invalid_writes: AtomicU32::new(0),
            stale_writes: AtomicU32::new(0),
            disconnected_writes: AtomicU32::new(0),
            coalesced_writes: AtomicU32::new(0),
            revision_exhausted_writes: AtomicU32::new(0),
        }
    }

    fn snapshot(&self) -> CoalescedStoreCounters {
        CoalescedStoreCounters {
            invalid_writes: self.invalid_writes.load(Ordering::Relaxed),
            stale_writes: self.stale_writes.load(Ordering::Relaxed),
            disconnected_writes: self.disconnected_writes.load(Ordering::Relaxed),
            coalesced_writes: self.coalesced_writes.load(Ordering::Relaxed),
            revision_exhausted_writes: self.revision_exhausted_writes.load(Ordering::Relaxed),
        }
    }
}

// Producer-side only. CAS retry is permitted on NRT and gives counters genuinely
// saturating semantics under concurrent writers.
fn saturating_increment(counter: &AtomicU32) {
    let mut current = counter.load(Ordering::Relaxed);
    while current != u32::MAX {
        match counter.compare_exchange_weak(
            current,
            current + 1,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => return,
            Err(observed) => current = observed,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum KeyBinding {
    NotCoalesced,
    Coalesced { slot: u16, read_only: bool },
}

#[derive(Debug)]
struct Slot {
    key: RuntimeParamKey,
    publication_word: AtomicU64,
    applied_word: AtomicU64,
    // Packed (failed revision, nonzero compact error code). It is historical
    // diagnostic state; a later success does not erase the last defect.
    failure_word: AtomicU64,
}

#[derive(Debug)]
struct Shared {
    generation: ParameterTableGeneration,
    table_identity: RuntimeParameterTableIdentity,
    accepting: AtomicBool,
    disconnected: AtomicBool,
    bindings: Box<[KeyBinding]>,
    slots: Box<[Slot]>,
    dirty: [AtomicU64; COALESCED_DIRTY_WORD_COUNT],
    counters: Counters,
}

impl Shared {
    fn binding(&self, key: RuntimeParamKey) -> Option<KeyBinding> {
        usize::try_from(key.0)
            .ok()
            .and_then(|index| self.bindings.get(index))
            .copied()
    }

    fn reject(&self, rejection: PublicationRejection) -> PublicationResult {
        match rejection {
            PublicationRejection::InvalidKey
            | PublicationRejection::NotControlCoalesced
            | PublicationRejection::ReadOnly
            | PublicationRejection::InvalidValue => {
                saturating_increment(&self.counters.invalid_writes);
            }
            PublicationRejection::Closed | PublicationRejection::StaleGeneration => {
                saturating_increment(&self.counters.stale_writes);
            }
            PublicationRejection::RevisionExhausted => {
                saturating_increment(&self.counters.revision_exhausted_writes);
            }
            PublicationRejection::Disconnected => {
                saturating_increment(&self.counters.disconnected_writes);
            }
        }
        PublicationResult::Rejected(rejection)
    }

    fn publish(&self, key: RuntimeParamKey, normalized: f32) -> PublicationResult {
        let Some(binding) = self.binding(key) else {
            return self.reject(PublicationRejection::InvalidKey);
        };
        let KeyBinding::Coalesced { slot, read_only } = binding else {
            return self.reject(PublicationRejection::NotControlCoalesced);
        };
        if read_only {
            return self.reject(PublicationRejection::ReadOnly);
        }
        let Some(canonical) = canonical_normalized(normalized) else {
            return self.reject(PublicationRejection::InvalidValue);
        };
        if self.disconnected.load(GENERATION_OBSERVE) {
            return self.reject(PublicationRejection::Disconnected);
        }
        if !self.accepting.load(GENERATION_OBSERVE) {
            return self.reject(PublicationRejection::Closed);
        }

        let slot_index = usize::from(slot);
        let publication = &self.slots[slot_index].publication_word;
        let mut old_word = publication.load(SLOT_CONSUME);
        let new_word = loop {
            let old_revision = unpack_revision(old_word);
            if old_revision == u32::MAX {
                return self.reject(PublicationRejection::RevisionExhausted);
            }
            let new_word = pack(old_revision + 1, canonical.to_bits());
            match publication.compare_exchange_weak(
                old_word,
                new_word,
                SLOT_CAS_SUCCESS,
                SLOT_CAS_FAILURE,
            ) {
                Ok(_) => break new_word,
                Err(observed) => old_word = observed,
            }
        };

        let dirty_word = slot_index / BITS_PER_DIRTY_WORD;
        let dirty_mask = 1_u64 << (slot_index % BITS_PER_DIRTY_WORD);
        let previous_dirty = self.dirty[dirty_word].fetch_or(dirty_mask, DIRTY_PUBLISH);

        if self.disconnected.load(GENERATION_OBSERVE) {
            return self.reject(PublicationRejection::Disconnected);
        }
        if !self.accepting.load(GENERATION_OBSERVE) {
            // The write and bit belong only to this physically separate old
            // generation. Conservatively reject the closure-racing call.
            return self.reject(PublicationRejection::StaleGeneration);
        }

        let replaced_pending = previous_dirty & dirty_mask != 0;
        if replaced_pending {
            saturating_increment(&self.counters.coalesced_writes);
        }
        PublicationResult::Accepted(AcceptedPublication {
            generation: self.generation,
            key,
            revision: PublicationRevision::from_word(new_word)
                .expect("successful publication revisions are nonzero"),
            canonical_normalized: canonical,
            replaced_pending,
        })
    }

    fn latest(&self, key: RuntimeParamKey) -> ParameterSnapshotStatus {
        let Some(binding) = self.binding(key) else {
            return ParameterSnapshotStatus::InvalidKey;
        };
        let KeyBinding::Coalesced { slot, .. } = binding else {
            return ParameterSnapshotStatus::NotControlCoalesced;
        };
        let slot = &self.slots[usize::from(slot)];
        // Best-effort NRT desired-state peek. Publication coherence comes from
        // the packed atomic itself; this does not synchronize other state.
        ParameterSnapshotStatus::Available(snapshot_from_word(
            self.generation,
            slot.key,
            slot.publication_word.load(SLOT_CONSUME),
        ))
    }

    fn applied(&self, key: RuntimeParamKey) -> AppliedTargetStatus {
        let Some(binding) = self.binding(key) else {
            return AppliedTargetStatus::InvalidKey;
        };
        let KeyBinding::Coalesced { slot, .. } = binding else {
            return AppliedTargetStatus::NotControlCoalesced;
        };
        let slot = &self.slots[usize::from(slot)];
        let word = slot.applied_word.load(APPLIED_OBSERVE);
        if unpack_revision(word) == 0 {
            AppliedTargetStatus::Pending {
                generation: self.generation,
                key,
            }
        } else {
            AppliedTargetStatus::Applied(snapshot_from_word(self.generation, key, word))
        }
    }

    fn last_failure(&self, key: RuntimeParamKey) -> ApplicationFailureStatus {
        let Some(binding) = self.binding(key) else {
            return ApplicationFailureStatus::InvalidKey;
        };
        let KeyBinding::Coalesced { slot, .. } = binding else {
            return ApplicationFailureStatus::NotControlCoalesced;
        };
        let word = self.slots[usize::from(slot)]
            .failure_word
            .load(APPLIED_OBSERVE);
        let Some(revision) = PublicationRevision::from_word(word) else {
            return ApplicationFailureStatus::None;
        };
        let Some(code) = NonZeroU32::new(unpack_payload(word)) else {
            return ApplicationFailureStatus::None;
        };
        ApplicationFailureStatus::Failed(ApplicationFailure {
            generation: self.generation,
            key,
            revision,
            code: ApplicationFailureCode(code),
        })
    }
}

fn snapshot_from_word(
    generation: ParameterTableGeneration,
    key: RuntimeParamKey,
    word: u64,
) -> ParameterSnapshot {
    ParameterSnapshot {
        generation,
        key,
        revision: PublicationRevision::from_word(word)
            .expect("prepared coalesced publication revisions are nonzero"),
        normalized: f32::from_bits(unpack_payload(word)),
    }
}

/// Cloneable, generation-bound NRT publisher. It can mutate only the physically
/// separate generation it owns; it accepts no caller-supplied generation or slot.
/// Its final release must occur on NRT because dropping the last shared owner can
/// deallocate prepared state.
#[derive(Debug, Clone)]
pub struct CoalescedParameterPublisher {
    shared: Arc<Shared>,
}

impl CoalescedParameterPublisher {
    #[must_use]
    pub fn generation(&self) -> ParameterTableGeneration {
        self.shared.generation
    }

    pub fn publish(&self, key: RuntimeParamKey, normalized: f32) -> PublicationResult {
        self.shared.publish(key, normalized)
    }

    #[must_use]
    pub fn latest(&self, key: RuntimeParamKey) -> ParameterSnapshotStatus {
        self.shared.latest(key)
    }

    #[must_use]
    pub fn applied(&self, key: RuntimeParamKey) -> AppliedTargetStatus {
        self.shared.applied(key)
    }

    #[must_use]
    pub fn last_application_failure(&self, key: RuntimeParamKey) -> ApplicationFailureStatus {
        self.shared.last_failure(key)
    }

    #[must_use]
    pub fn counters(&self) -> CoalescedStoreCounters {
        self.shared.counters.snapshot()
    }
}

/// Packed normalized publication/applied snapshot.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParameterSnapshot {
    pub generation: ParameterTableGeneration,
    pub key: RuntimeParamKey,
    pub revision: PublicationRevision,
    pub normalized: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParameterSnapshotStatus {
    InvalidKey,
    NotControlCoalesced,
    Available(ParameterSnapshot),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppliedTargetStatus {
    InvalidKey,
    NotControlCoalesced,
    Pending {
        generation: ParameterTableGeneration,
        key: RuntimeParamKey,
    },
    Applied(ParameterSnapshot),
}

/// Publication yielded to the RT application callback.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrainedPublication {
    pub generation: ParameterTableGeneration,
    pub key: RuntimeParamKey,
    pub revision: PublicationRevision,
    pub normalized: f32,
}

/// Nonzero compact engine-application defect code. Rich diagnostics remain NRT.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApplicationFailureCode(NonZeroU32);

impl ApplicationFailureCode {
    #[must_use]
    pub const fn new(code: NonZeroU32) -> Self {
        Self(code)
    }

    #[must_use]
    pub const fn get(self) -> u32 {
        self.0.get()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParameterApplicationResult {
    Applied,
    Failed(ApplicationFailureCode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApplicationFailure {
    pub generation: ParameterTableGeneration,
    pub key: RuntimeParamKey,
    pub revision: PublicationRevision,
    pub code: ApplicationFailureCode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplicationFailureStatus {
    InvalidKey,
    NotControlCoalesced,
    None,
    Failed(ApplicationFailure),
}

/// Fixed-work drain outcome. `scanned_dirty_words` is always 16.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CoalescedDrainSummary {
    pub scanned_dirty_words: u8,
    pub dirty_slots: u16,
    pub applied: u16,
    pub failed: u16,
}

/// Prepared store owner and bounded RT consumer handle.
///
/// Preparation allocates; `drain` does not. This owner and every publisher clone
/// must be retired/dropped on NRT. Only one consumer may call `drain` for a store.
#[derive(Debug)]
pub struct CoalescedParameterStore {
    shared: Arc<Shared>,
}

impl CoalescedParameterStore {
    pub fn prepare(
        generations: &mut ParameterTableGenerations,
        table: &RuntimeParameterTable,
        coalesced_limit: usize,
    ) -> Result<Self, CoalescedStorePrepareError> {
        Self::prepare_with_initial_values(generations, table, coalesced_limit, &[])
    }

    pub fn prepare_with_initial_values(
        generations: &mut ParameterTableGenerations,
        table: &RuntimeParameterTable,
        coalesced_limit: usize,
        initial_values: &[InitialNormalizedValue],
    ) -> Result<Self, CoalescedStorePrepareError> {
        let generation = generations
            .next_generation()
            .map_err(|_| CoalescedStorePrepareError::GenerationExhausted)?;
        Self::prepare_generation(generation, table, coalesced_limit, initial_values)
    }

    fn prepare_generation(
        generation: ParameterTableGeneration,
        table: &RuntimeParameterTable,
        coalesced_limit: usize,
        initial_values: &[InitialNormalizedValue],
    ) -> Result<Self, CoalescedStorePrepareError> {
        if coalesced_limit > MAX_COALESCED_PARAMETER_COUNT {
            return Err(CoalescedStorePrepareError::CoalescedLimitTooHigh {
                requested: coalesced_limit,
                maximum: MAX_COALESCED_PARAMETER_COUNT,
            });
        }

        for (index, initial) in initial_values.iter().enumerate() {
            let Some(parameter) = table.get(initial.key) else {
                return Err(CoalescedStorePrepareError::InvalidInitialKey(initial.key));
            };
            if parameter.automation_rate() != AutomationRate::ControlCoalesced {
                return Err(CoalescedStorePrepareError::InitialKeyNotCoalesced(
                    initial.key,
                ));
            }
            if canonical_normalized(initial.normalized).is_none() {
                return Err(CoalescedStorePrepareError::InvalidInitialValue(initial.key));
            }
            if initial_values[..index]
                .iter()
                .any(|previous| previous.key == initial.key)
            {
                return Err(CoalescedStorePrepareError::DuplicateInitialKey(initial.key));
            }
        }

        let coalesced_count = table
            .entries()
            .iter()
            .filter(|parameter| parameter.automation_rate() == AutomationRate::ControlCoalesced)
            .count();
        if coalesced_count > coalesced_limit {
            return Err(CoalescedStorePrepareError::CoalescedCapacityExceeded {
                count: coalesced_count,
                limit: coalesced_limit,
            });
        }

        let mut bindings = Vec::with_capacity(table.len());
        let mut slots = Vec::with_capacity(coalesced_count);
        let mut initial_dirty_words = [0_u64; COALESCED_DIRTY_WORD_COUNT];

        for parameter in table.entries() {
            if parameter.automation_rate() != AutomationRate::ControlCoalesced {
                bindings.push(KeyBinding::NotCoalesced);
                continue;
            }

            let slot_index = slots.len();
            let slot =
                u16::try_from(slot_index).expect("coalesced hard capacity is representable by u16");
            let normalized = initial_values
                .iter()
                .find(|initial| initial.key == parameter.key())
                .map_or_else(
                    || {
                        canonical_normalized(
                            table
                                .default_normalized(parameter.key())
                                .expect("parameter key belongs to the prepared table"),
                        )
                        .expect("validated descriptor defaults are finite and normalized")
                    },
                    |initial| {
                        canonical_normalized(initial.normalized)
                            .expect("initial values were validated above")
                    },
                );
            let initial_word = pack(1, normalized.to_bits());
            slots.push(Slot {
                key: parameter.key(),
                publication_word: AtomicU64::new(initial_word),
                applied_word: AtomicU64::new(0),
                failure_word: AtomicU64::new(0),
            });
            bindings.push(KeyBinding::Coalesced {
                slot,
                read_only: parameter.read_only(),
            });
            initial_dirty_words[slot_index / BITS_PER_DIRTY_WORD] |=
                1_u64 << (slot_index % BITS_PER_DIRTY_WORD);
        }

        Ok(Self {
            shared: Arc::new(Shared {
                generation,
                table_identity: table.identity(),
                accepting: AtomicBool::new(true),
                disconnected: AtomicBool::new(false),
                bindings: bindings.into_boxed_slice(),
                slots: slots.into_boxed_slice(),
                dirty: initial_dirty_words.map(AtomicU64::new),
                counters: Counters::new(),
            }),
        })
    }

    /// Observable semantic reset: reserve a non-reused generation, close this
    /// generation with Release ordering, and prepare physically separate dirty
    /// slots for the replacement. There is no concurrent in-place dirty clear.
    pub fn prepare_reset(
        &self,
        generations: &mut ParameterTableGenerations,
        table: &RuntimeParameterTable,
        coalesced_limit: usize,
        initial_values: &[InitialNormalizedValue],
    ) -> Result<Self, CoalescedStoreResetError> {
        let generation = generations
            .next_generation()
            .map_err(|_| CoalescedStoreResetError::GenerationExhausted)?;
        self.close();
        Self::prepare_generation(generation, table, coalesced_limit, initial_values)
            .map_err(CoalescedStoreResetError::Preparation)
    }

    #[must_use]
    pub fn generation(&self) -> ParameterTableGeneration {
        self.shared.generation
    }

    /// Whether `table` is the exact runtime table used to prepare this store,
    /// rather than a structurally equal table from another generation.
    #[must_use]
    pub fn is_for_table(&self, table: &RuntimeParameterTable) -> bool {
        table.has_identity(&self.shared.table_identity)
    }

    #[must_use]
    pub fn publisher(&self) -> CoalescedParameterPublisher {
        CoalescedParameterPublisher {
            shared: Arc::clone(&self.shared),
        }
    }

    /// Stop accepting this generation. This is the ADR's exact Release close;
    /// publishers check with Acquire before CAS and after dirty publication.
    pub fn close(&self) {
        self.shared.accepting.store(false, GENERATION_CLOSE);
    }

    /// Mark the engine instance disconnected and close publication.
    pub fn disconnect(&self) {
        self.shared.disconnected.store(true, GENERATION_CLOSE);
        self.shared.accepting.store(false, GENERATION_CLOSE);
    }

    #[must_use]
    pub fn is_accepting(&self) -> bool {
        self.shared.accepting.load(GENERATION_OBSERVE)
            && !self.shared.disconnected.load(GENERATION_OBSERVE)
    }

    /// Scan all 16 dirty words exactly once and apply at most one publication per
    /// active slot. The callback must be bounded, nonallocating, nonblocking, and
    /// non-panicking. There is no RT retry loop.
    pub fn drain(
        &self,
        mut apply: impl FnMut(DrainedPublication) -> ParameterApplicationResult,
    ) -> CoalescedDrainSummary {
        let mut summary = CoalescedDrainSummary {
            scanned_dirty_words: COALESCED_DIRTY_WORD_COUNT as u8,
            ..CoalescedDrainSummary::default()
        };

        for (word_index, dirty_word) in self.shared.dirty.iter().enumerate() {
            // Contractual acquire RMW clear. Do not replace with load/store.
            let dirty = dirty_word.swap(0, DIRTY_CONSUME);
            for bit_index in 0..BITS_PER_DIRTY_WORD {
                let mask = 1_u64 << bit_index;
                if dirty & mask == 0 {
                    continue;
                }
                let slot_index = word_index * BITS_PER_DIRTY_WORD + bit_index;
                let Some(slot) = self.shared.slots.get(slot_index) else {
                    continue;
                };
                // Contractual relaxed slot load after acquiring the dirty RMW.
                let word = slot.publication_word.load(SLOT_CONSUME);
                let Some(revision) = PublicationRevision::from_word(word) else {
                    continue;
                };
                summary.dirty_slots += 1;
                let publication = DrainedPublication {
                    generation: self.shared.generation,
                    key: slot.key,
                    revision,
                    normalized: f32::from_bits(unpack_payload(word)),
                };
                match apply(publication) {
                    ParameterApplicationResult::Applied => {
                        // Confirm the exact coherent revision/value only after
                        // the engine target was successfully latched.
                        slot.applied_word.store(word, APPLIED_PUBLISH);
                        summary.applied += 1;
                    }
                    ParameterApplicationResult::Failed(code) => {
                        slot.failure_word
                            .store(pack(revision.get(), code.get()), APPLIED_PUBLISH);
                        summary.failed += 1;
                    }
                }
            }
        }
        summary
    }

    #[must_use]
    pub fn latest(&self, key: RuntimeParamKey) -> ParameterSnapshotStatus {
        self.shared.latest(key)
    }

    #[must_use]
    pub fn applied(&self, key: RuntimeParamKey) -> AppliedTargetStatus {
        self.shared.applied(key)
    }

    #[must_use]
    pub fn last_application_failure(&self, key: RuntimeParamKey) -> ApplicationFailureStatus {
        self.shared.last_failure(key)
    }

    #[must_use]
    pub fn counters(&self) -> CoalescedStoreCounters {
        self.shared.counters.snapshot()
    }

    #[cfg(test)]
    fn force_publication_revision(&self, key: RuntimeParamKey, revision: u32, normalized: f32) {
        let KeyBinding::Coalesced { slot, .. } = self.shared.binding(key).unwrap() else {
            panic!("test key must be coalesced");
        };
        self.shared.slots[usize::from(slot)]
            .publication_word
            .store(pack(revision, normalized.to_bits()), Ordering::Relaxed);
    }
}

// Keep synchronous result/status values cheap for transport adapters.
const _: () = assert!(std::mem::size_of::<PublicationRejection>() == 1);
const _: () = assert!(std::mem::size_of::<CoalescedDrainSummary>() <= 8);

#[cfg(test)]
mod tests {
    use super::*;
    use param_manifest::{builtin::builtin_manifest, ParameterLookup};

    fn fixture() -> (
        ParameterLookup,
        CoalescedParameterStore,
        RuntimeParamKey,
        ParameterTableGenerations,
    ) {
        let lookup = ParameterLookup::from_manifest(&builtin_manifest()).unwrap();
        let key = lookup.entries()[0].key();
        let mut generations = ParameterTableGenerations::new();
        let store = CoalescedParameterStore::prepare(&mut generations, lookup.table(), 1).unwrap();
        (lookup, store, key, generations)
    }

    #[test]
    fn revisions_reject_exhaustion_without_redirtying() {
        let (_lookup, store, key, _generations) = fixture();
        store.drain(|_| ParameterApplicationResult::Applied);
        store.force_publication_revision(key, u32::MAX, 0.25);

        assert_eq!(
            store.publisher().publish(key, 0.5),
            PublicationResult::Rejected(PublicationRejection::RevisionExhausted)
        );
        assert_eq!(
            store
                .drain(|_| ParameterApplicationResult::Applied)
                .dirty_slots,
            0
        );
        assert_eq!(store.counters().revision_exhausted_writes, 1);
    }

    #[test]
    fn generation_sequence_never_wraps_or_reuses() {
        let mut generations = ParameterTableGenerations::starting_at(u64::MAX);
        assert_eq!(generations.next_generation().unwrap().get(), u64::MAX);
        assert_eq!(generations.next_generation(), Err(GenerationExhausted));
        assert_eq!(generations.next_generation(), Err(GenerationExhausted));
    }

    #[test]
    fn producer_counters_saturate() {
        let (_lookup, store, _key, _generations) = fixture();
        store
            .shared
            .counters
            .invalid_writes
            .store(u32::MAX, Ordering::Relaxed);
        let result = store.publisher().publish(RuntimeParamKey(u32::MAX), 0.5);
        assert_eq!(
            result,
            PublicationResult::Rejected(PublicationRejection::InvalidKey)
        );
        assert_eq!(store.counters().invalid_writes, u32::MAX);
    }
}
