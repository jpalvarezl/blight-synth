use crate::{events::EventValidationError, EngineEvent, EventProducerId, TimestampedEvent};

/// Why NRT preparation of a bounded event-admission lane failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventAdmissionPrepareError {
    /// The configured producer list contains the same stable identity twice.
    DuplicateProducer,
    /// The reserved recovery identity also appears in the ordinary producer set.
    RecoveryProducerConflict,
    /// Ordinary capacity cannot be combined with the one reserved recovery slot.
    CapacityOverflow,
    /// Fixed producer/event storage could not be allocated on NRT.
    AllocationFailed,
}

/// Compact reason that an ordinary producer submission rejected the whole block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventAdmissionErrorKind {
    /// [`BoundedEventAdmission::begin_block`] has not started a collecting block.
    NotCollecting,
    /// The identity was not part of the NRT-prepared producer set.
    UnknownProducer,
    /// A producer may stage at most one slice (including an empty slice) per block.
    ProducerAlreadySubmitted,
    /// An event's embedded producer identity did not match the submitting producer.
    EventProducerMismatch,
    /// Source sequence did not increase strictly within the producer stream.
    SequenceNotIncreasing,
    /// Source offsets moved backwards in sequence/emission order.
    SourceOffsetsNotOrdered,
    /// An event offset was outside the current half-open block.
    OffsetOutOfRange,
    /// A sample-event parameter value was invalid for its prepared binding.
    InvalidEvent,
    /// Recovery must use the capacity-independent recovery request, not ordinary space.
    RecoveryInOrdinaryLane,
    /// The complete producer slice did not fit in remaining ordinary capacity.
    OrdinaryCapacityExceeded,
}

/// Producer-visible, allocation-free ordinary admission error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventAdmissionError {
    producer: EventProducerId,
    kind: EventAdmissionErrorKind,
}

impl EventAdmissionError {
    #[must_use]
    pub const fn producer(self) -> EventProducerId {
        self.producer
    }

    #[must_use]
    pub const fn kind(self) -> EventAdmissionErrorKind {
        self.kind
    }
}

/// Result of staging one producer's complete current-block slice.
///
/// `Staged` remains provisional until [`BoundedEventAdmission::finish_block`]: a
/// later producer can reject the whole ordinary block.
#[must_use = "producer admission remains provisional until the finalized block is inspected"]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProducerAdmissionStatus {
    Staged,
    Rejected(EventAdmissionError),
}

/// Compact reason that an out-of-band recovery request was not staged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryAdmissionError {
    NotCollecting,
    AlreadyRequested,
    OffsetOutOfRange,
    SequenceNotIncreasing,
}

/// Result of staging the block's capacity-independent all-notes-off event.
#[must_use = "recovery admission can fail and must be inspected"]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryAdmissionStatus {
    Staged,
    Rejected(RecoveryAdmissionError),
}

/// Final ordinary-lane outcome for one block.
#[must_use = "rejected ordinary blocks must not be processed as accepted"]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrdinaryEventBlockStatus {
    /// Every staged ordinary producer slice was accepted and canonically ordered.
    Accepted { event_count: usize },
    /// One malformed/overflow submission rejected every ordinary event.
    Rejected(EventAdmissionError),
    /// No block was collecting when finalization was requested.
    NotStarted,
}

/// Borrowed finalized block ready for [`crate::Engine::process_with_events`].
///
/// If `ordinary_status` is rejected, `events` contains no ordinary events. It
/// contains only the independently staged recovery event, if any. This is the
/// fail-closed guarantee: no ordinary prefix is exposed and nothing is queued
/// for a later block.
#[must_use = "the finalized admission status and event slice must be inspected"]
#[derive(Debug, Clone, Copy)]
pub struct FinalizedEventBlock<'a> {
    ordinary_status: OrdinaryEventBlockStatus,
    events: &'a [TimestampedEvent],
}

impl<'a> FinalizedEventBlock<'a> {
    pub const fn ordinary_status(self) -> OrdinaryEventBlockStatus {
        self.ordinary_status
    }

    #[must_use]
    pub const fn events(self) -> &'a [TimestampedEvent] {
        self.events
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AdmissionPhase {
    Idle,
    Collecting,
    Finalized,
}

struct ProducerState {
    id: EventProducerId,
    committed_sequence: Option<u64>,
    pending_sequence: Option<u64>,
    submitted: bool,
}

/// NRT-prepared, host-independent admission and canonical merge storage for one
/// current audio block at a time.
///
/// Preparation allocates a fixed producer table and storage for exactly
/// `ordinary_capacity` ordinary events plus one reserved recovery event.
/// Callback-side methods never grow or shrink that storage. A configured
/// producer may submit at most one complete slice per block; accepted work is
/// therefore bounded by the prepared producer count, event capacity, and one
/// allocation-free unstable sort. The sort compares only
/// [`TimestampedEvent::order_key`].
///
/// Construct and destroy this owner on NRT. On RT, reuse it with `begin_block`,
/// `submit_producer`, `request_all_notes_off`, and `finish_block`.
pub struct BoundedEventAdmission {
    ordinary_capacity: usize,
    producers: Vec<ProducerState>,
    recovery_producer: EventProducerId,
    recovery_committed_sequence: Option<u64>,
    recovery_event: Option<TimestampedEvent>,
    frame_count: usize,
    events: Vec<TimestampedEvent>,
    phase: AdmissionPhase,
    rejection: Option<EventAdmissionError>,
    submitted_event_count: usize,
    lowest_capacity_contributor: Option<EventProducerId>,
    finalized_status: OrdinaryEventBlockStatus,
}

impl BoundedEventAdmission {
    /// Allocate and validate fixed capacities and stable producer identities on NRT.
    pub fn prepare(
        ordinary_capacity: usize,
        ordinary_producers: &[EventProducerId],
        recovery_producer: EventProducerId,
    ) -> Result<Self, EventAdmissionPrepareError> {
        let storage_capacity = ordinary_capacity
            .checked_add(1)
            .ok_or(EventAdmissionPrepareError::CapacityOverflow)?;
        let mut producers = Vec::new();
        producers
            .try_reserve_exact(ordinary_producers.len())
            .map_err(|_| EventAdmissionPrepareError::AllocationFailed)?;
        for &id in ordinary_producers {
            producers.push(ProducerState {
                id,
                committed_sequence: None,
                pending_sequence: None,
                submitted: false,
            });
        }
        producers.sort_unstable_by_key(|producer| producer.id);
        if producers.windows(2).any(|pair| pair[0].id == pair[1].id) {
            return Err(EventAdmissionPrepareError::DuplicateProducer);
        }
        if producers
            .binary_search_by_key(&recovery_producer, |producer| producer.id)
            .is_ok()
        {
            return Err(EventAdmissionPrepareError::RecoveryProducerConflict);
        }

        let mut events = Vec::new();
        events
            .try_reserve_exact(storage_capacity)
            .map_err(|_| EventAdmissionPrepareError::AllocationFailed)?;

        Ok(Self {
            ordinary_capacity,
            producers,
            recovery_producer,
            recovery_committed_sequence: None,
            recovery_event: None,
            frame_count: 0,
            events,
            phase: AdmissionPhase::Idle,
            rejection: None,
            submitted_event_count: 0,
            lowest_capacity_contributor: None,
            finalized_status: OrdinaryEventBlockStatus::NotStarted,
        })
    }

    #[must_use]
    pub const fn ordinary_capacity(&self) -> usize {
        self.ordinary_capacity
    }

    #[must_use]
    pub fn producer_count(&self) -> usize {
        self.producers.len()
    }

    #[must_use]
    pub const fn recovery_producer(&self) -> EventProducerId {
        self.recovery_producer
    }

    /// Start a fresh current block while preserving successfully committed
    /// source-sequence baselines from earlier blocks.
    ///
    /// Any unfinalized staged data is explicitly discarded rather than carried
    /// into this block. `frame_count` must be the common buffer prefix later
    /// supplied to `Engine::process_with_events`. The operation visits only the
    /// prepared producer table.
    pub fn begin_block(&mut self, frame_count: usize) {
        self.frame_count = frame_count;
        self.events.clear();
        self.recovery_event = None;
        self.rejection = None;
        self.submitted_event_count = 0;
        self.lowest_capacity_contributor = None;
        self.finalized_status = OrdinaryEventBlockStatus::NotStarted;
        for producer in &mut self.producers {
            producer.pending_sequence = None;
            producer.submitted = false;
        }
        self.phase = AdmissionPhase::Collecting;
    }

    /// Stage one producer's complete current-block slice.
    ///
    /// Events must be in source emission order: source sequence increases
    /// strictly and sample offsets do not move backwards. Same-offset semantic
    /// order need not match emission order; finalization intentionally applies
    /// #201's canonical `order_key`, whose semantic precedence can override
    /// sequence. Each call reports its own slice status, while finalization
    /// deterministically selects one aggregate failure across every submission:
    /// malformed/protocol failures before capacity, then stable producer identity.
    /// A previous producer failure never prevents validation of a later producer.
    pub fn submit_producer(
        &mut self,
        producer: EventProducerId,
        events: &[TimestampedEvent],
    ) -> ProducerAdmissionStatus {
        if self.phase != AdmissionPhase::Collecting {
            return ProducerAdmissionStatus::Rejected(EventAdmissionError {
                producer,
                kind: EventAdmissionErrorKind::NotCollecting,
            });
        }

        let index = match self
            .producers
            .binary_search_by_key(&producer, |state| state.id)
        {
            Ok(index) => index,
            Err(_) => return self.reject(producer, EventAdmissionErrorKind::UnknownProducer),
        };
        if self.producers[index].submitted {
            return self.reject(producer, EventAdmissionErrorKind::ProducerAlreadySubmitted);
        }
        self.producers[index].submitted = true;

        // A single slice larger than the complete prepared lane is intrinsically
        // over capacity. This check keeps rejected RT work bounded by the
        // prepared capacity; all slices that could fit are fully validated before
        // aggregate/remaining-capacity effects are considered.
        if events.len() > self.ordinary_capacity {
            return self.record_capacity_rejection(producer, events.len());
        }

        let committed_sequence = self.producers[index].committed_sequence;
        let mut previous_sequence = committed_sequence;
        let mut previous_offset = None;
        for event in events {
            if event.producer != producer {
                return self.reject(producer, EventAdmissionErrorKind::EventProducerMismatch);
            }
            match event.event {
                EngineEvent::AllNotesOff => {
                    return self.reject(producer, EventAdmissionErrorKind::RecoveryInOrdinaryLane);
                }
                EngineEvent::NoteOff { .. }
                | EngineEvent::SampleParameter { .. }
                | EngineEvent::NoteOn { .. } => {}
            }
            if previous_sequence.is_some_and(|sequence| event.sequence <= sequence) {
                return self.reject(producer, EventAdmissionErrorKind::SequenceNotIncreasing);
            }
            if previous_offset.is_some_and(|offset| event.sample_offset < offset) {
                return self.reject(producer, EventAdmissionErrorKind::SourceOffsetsNotOrdered);
            }
            match event.validate_for_block(self.frame_count) {
                Ok(()) => {}
                Err(EventValidationError::OffsetOutOfRange) => {
                    return self.reject(producer, EventAdmissionErrorKind::OffsetOutOfRange);
                }
                Err(EventValidationError::InvalidParameterValue) => {
                    return self.reject(producer, EventAdmissionErrorKind::InvalidEvent);
                }
            }
            previous_sequence = Some(event.sequence);
            previous_offset = Some(event.sample_offset);
        }

        self.producers[index].pending_sequence = events.last().map(|event| event.sequence);
        if events.is_empty() {
            return ProducerAdmissionStatus::Staged;
        }

        self.submitted_event_count = self.submitted_event_count.saturating_add(events.len());
        self.lowest_capacity_contributor = Some(
            self.lowest_capacity_contributor
                .map_or(producer, |current| current.min(producer)),
        );
        if self.submitted_event_count > self.ordinary_capacity {
            return self.record_capacity_rejection(producer, 0);
        }

        // If the aggregate fits, every validated slice fits regardless of call
        // order. Storage contents on a rejected block are provisional and are
        // cleared by finish_block.
        self.events.extend_from_slice(events);
        ProducerAdmissionStatus::Staged
    }

    /// Stage one all-notes-off event in the slot reserved outside ordinary capacity.
    ///
    /// This remains available after ordinary capacity overflow. The recovery
    /// source has its own stable identity and strictly increasing sequence.
    pub fn request_all_notes_off(
        &mut self,
        sample_offset: usize,
        sequence: u64,
    ) -> RecoveryAdmissionStatus {
        if self.phase != AdmissionPhase::Collecting {
            return RecoveryAdmissionStatus::Rejected(RecoveryAdmissionError::NotCollecting);
        }
        if self.recovery_event.is_some() {
            return RecoveryAdmissionStatus::Rejected(RecoveryAdmissionError::AlreadyRequested);
        }
        if sample_offset >= self.frame_count {
            return RecoveryAdmissionStatus::Rejected(RecoveryAdmissionError::OffsetOutOfRange);
        }
        if self
            .recovery_committed_sequence
            .is_some_and(|committed| sequence <= committed)
        {
            return RecoveryAdmissionStatus::Rejected(
                RecoveryAdmissionError::SequenceNotIncreasing,
            );
        }
        self.recovery_event = Some(TimestampedEvent::new(
            sample_offset,
            self.recovery_producer,
            sequence,
            EngineEvent::AllNotesOff,
        ));
        RecoveryAdmissionStatus::Staged
    }

    /// Canonically order and expose one complete block without allocation.
    ///
    /// A rejected ordinary block exposes no ordinary prefix. A valid recovery
    /// request is retained independently, including after ordinary overflow or
    /// malformed input. On ordinary success, recovery and ordinary events share
    /// the one canonical order; on failure, the finalized slice is recovery-only.
    pub fn finish_block(&mut self) -> FinalizedEventBlock<'_> {
        if self.phase == AdmissionPhase::Finalized {
            return FinalizedEventBlock {
                ordinary_status: self.finalized_status,
                events: &self.events,
            };
        }
        if self.phase != AdmissionPhase::Collecting {
            return FinalizedEventBlock {
                ordinary_status: OrdinaryEventBlockStatus::NotStarted,
                events: &[],
            };
        }

        let ordinary_event_count = self.events.len();
        let ordinary_status = if let Some(error) = self.rejection {
            self.events.clear();
            OrdinaryEventBlockStatus::Rejected(error)
        } else {
            for producer in &mut self.producers {
                if let Some(sequence) = producer.pending_sequence {
                    producer.committed_sequence = Some(sequence);
                }
            }
            OrdinaryEventBlockStatus::Accepted {
                event_count: ordinary_event_count,
            }
        };

        if let Some(recovery) = self.recovery_event {
            // Physical storage includes exactly one slot beyond ordinary capacity.
            self.events.push(recovery);
            self.recovery_committed_sequence = Some(recovery.sequence);
        }
        // This is the sole merge comparator. Keys are total because producer
        // identities are stable and each producer sequence is strictly increasing.
        self.events.sort_unstable_by_key(|event| event.order_key());

        self.finalized_status = ordinary_status;
        self.phase = AdmissionPhase::Finalized;
        FinalizedEventBlock {
            ordinary_status,
            events: &self.events,
        }
    }

    /// Clear all block state and sequence history for explicit stream reuse.
    ///
    /// This does not release prepared storage. Hosts use it for a source/transport
    /// reset where producers are intentionally allowed to restart sequence values.
    pub fn reset(&mut self) {
        self.events.clear();
        self.recovery_event = None;
        self.recovery_committed_sequence = None;
        self.frame_count = 0;
        self.rejection = None;
        self.submitted_event_count = 0;
        self.lowest_capacity_contributor = None;
        self.finalized_status = OrdinaryEventBlockStatus::NotStarted;
        self.phase = AdmissionPhase::Idle;
        for producer in &mut self.producers {
            producer.committed_sequence = None;
            producer.pending_sequence = None;
            producer.submitted = false;
        }
    }

    fn reject(
        &mut self,
        producer: EventProducerId,
        kind: EventAdmissionErrorKind,
    ) -> ProducerAdmissionStatus {
        let error = EventAdmissionError { producer, kind };
        self.record_rejection(error);
        ProducerAdmissionStatus::Rejected(error)
    }

    fn record_capacity_rejection(
        &mut self,
        submitting_producer: EventProducerId,
        event_count: usize,
    ) -> ProducerAdmissionStatus {
        if event_count != 0 {
            self.submitted_event_count = self.submitted_event_count.saturating_add(event_count);
            self.lowest_capacity_contributor = Some(
                self.lowest_capacity_contributor
                    .map_or(submitting_producer, |current| {
                        current.min(submitting_producer)
                    }),
            );
        }
        let final_error = EventAdmissionError {
            producer: self
                .lowest_capacity_contributor
                .unwrap_or(submitting_producer),
            kind: EventAdmissionErrorKind::OrdinaryCapacityExceeded,
        };
        self.record_rejection(final_error);
        ProducerAdmissionStatus::Rejected(EventAdmissionError {
            producer: submitting_producer,
            kind: EventAdmissionErrorKind::OrdinaryCapacityExceeded,
        })
    }

    fn record_rejection(&mut self, candidate: EventAdmissionError) {
        if self
            .rejection
            .is_none_or(|current| rejection_key(candidate) < rejection_key(current))
        {
            self.rejection = Some(candidate);
        }
    }
}

// This orders diagnostics, never events. Event merge ordering remains solely
// TimestampedEvent::order_key / EventOrderKey.
const fn rejection_key(error: EventAdmissionError) -> (u8, u64, u8) {
    let (class, kind) = match error.kind {
        EventAdmissionErrorKind::OrdinaryCapacityExceeded => (1, 0),
        EventAdmissionErrorKind::NotCollecting => (2, 0),
        EventAdmissionErrorKind::UnknownProducer => (0, 0),
        EventAdmissionErrorKind::ProducerAlreadySubmitted => (0, 1),
        EventAdmissionErrorKind::EventProducerMismatch => (0, 2),
        EventAdmissionErrorKind::SequenceNotIncreasing => (0, 3),
        EventAdmissionErrorKind::SourceOffsetsNotOrdered => (0, 4),
        EventAdmissionErrorKind::OffsetOutOfRange => (0, 5),
        EventAdmissionErrorKind::InvalidEvent => (0, 6),
        EventAdmissionErrorKind::RecoveryInOrdinaryLane => (0, 7),
    };
    (class, error.producer.get(), kind)
}

const _: () = assert!(std::mem::size_of::<EventAdmissionError>() <= 16);
const _: () = assert!(std::mem::size_of::<ProducerAdmissionStatus>() <= 24);
const _: () = assert!(std::mem::size_of::<RecoveryAdmissionStatus>() <= 2);
