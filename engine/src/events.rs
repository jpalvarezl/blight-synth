use dsp::id::{EffectId, InstrumentId, NoteEvent, NoteId};
use param_manifest::{AutomationRate, RuntimeParamKey, RuntimeParameter};

/// Stable identity assigned to one event producer by the host scheduler.
///
/// The identity participates in the canonical total order for events that share
/// a sample offset and semantic precedence. It must not be derived from hash
/// iteration, callback arrival order, or incidental runtime registration order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EventProducerId(u64);

impl EventProducerId {
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Concrete engine object targeted by a prepared sample-accurate parameter.
///
/// Stable manifest paths are resolved to this compact runtime target on NRT.
/// Additional target classes can be added as their engine application paths
/// become available without changing the timestamped-event envelope.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParameterTarget {
    MasterEffect {
        effect_id: EffectId,
    },
    InstrumentEffect {
        instrument_id: InstrumentId,
        effect_id: EffectId,
    },
}

/// Why a manifest parameter cannot be bound to the timestamped event lane.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParameterBindingError {
    /// Only parameters classified as `AutomationRate::SampleEvent` may enter
    /// this lane. Coalesced and structural values have different overload and
    /// ownership semantics.
    NotSampleEvent,
}

/// String-free parameter identity and concrete target prepared for RT use.
///
/// Construction requires a validated [`RuntimeParameter`], proving that the
/// parameter came from the canonical manifest's RT projection. The binding
/// stores only the fields needed while applying an already-mapped engine value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PreparedParameterBinding {
    // Retained as the stable prepared-table identity for scheduler diagnostics
    // and #203 admission; event application itself uses the resolved target.
    key: RuntimeParamKey,
    target: ParameterTarget,
    engine_param_index: u32,
    min_engine: f32,
    max_engine: f32,
}

impl PreparedParameterBinding {
    pub fn new(
        parameter: RuntimeParameter,
        target: ParameterTarget,
    ) -> Result<Self, ParameterBindingError> {
        if parameter.automation_rate() != AutomationRate::SampleEvent {
            return Err(ParameterBindingError::NotSampleEvent);
        }
        Ok(Self {
            key: parameter.key(),
            target,
            engine_param_index: parameter.engine_param_index(),
            min_engine: parameter.min_engine(),
            max_engine: parameter.max_engine(),
        })
    }

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

    /// Whether an already-mapped engine value is finite and remains inside the
    /// validated descriptor range captured during NRT preparation.
    #[must_use]
    pub fn accepts_engine_value(self, value: f32) -> bool {
        value.is_finite() && (self.min_engine..=self.max_engine).contains(&value)
    }
}

/// Engine-ready event payload applied at one current-block sample offset.
///
/// This type contains no composition document, host clock, strings, owning heap
/// values, or transport-specific data. The timestamp and ordering metadata live
/// in [`TimestampedEvent`].
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EngineEvent {
    /// Host-level fail-closed recovery. This is intentionally engine-global
    /// until producer-owned voice recovery has its own accepted contract.
    AllNotesOff,
    NoteOff {
        instrument_id: InstrumentId,
        note_id: NoteId,
    },
    /// Apply an already normalized-to-engine value through an NRT-prepared
    /// sample-event binding.
    SampleParameter {
        binding: PreparedParameterBinding,
        engine_value: f32,
    },
    NoteOn {
        instrument_id: InstrumentId,
        note: NoteEvent,
    },
}

impl EngineEvent {
    /// Semantic precedence within one sample offset.
    ///
    /// Recovery and releases happen before parameter changes; parameters happen
    /// before attacks so a newly started note observes the new value. Producer
    /// identity and source-local sequence break ties after this precedence.
    #[must_use]
    pub const fn semantic_precedence(self) -> u8 {
        match self {
            Self::AllNotesOff => 0,
            Self::NoteOff { .. } => 1,
            Self::SampleParameter { .. } => 2,
            Self::NoteOn { .. } => 3,
        }
    }
}

/// Canonical total-order key shared by event producers, schedulers, and Engine
/// validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct EventOrderKey {
    sample_offset: usize,
    semantic_precedence: u8,
    producer: EventProducerId,
    sequence: u64,
}

impl EventOrderKey {
    #[must_use]
    pub const fn sample_offset(self) -> usize {
        self.sample_offset
    }

    #[must_use]
    pub const fn semantic_precedence(self) -> u8 {
        self.semantic_precedence
    }

    #[must_use]
    pub const fn producer(self) -> EventProducerId {
        self.producer
    }

    #[must_use]
    pub const fn sequence(self) -> u64 {
        self.sequence
    }
}

/// One already-offset event for the current half-open render block.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimestampedEvent {
    pub sample_offset: usize,
    pub producer: EventProducerId,
    pub sequence: u64,
    pub event: EngineEvent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EventValidationError {
    OffsetOutOfRange,
    InvalidParameterValue,
}

impl TimestampedEvent {
    #[must_use]
    pub const fn new(
        sample_offset: usize,
        producer: EventProducerId,
        sequence: u64,
        event: EngineEvent,
    ) -> Self {
        Self {
            sample_offset,
            producer,
            sequence,
            event,
        }
    }

    /// Return the one canonical ordering key that #203's scheduler must also
    /// use when merging producers.
    #[must_use]
    pub const fn order_key(self) -> EventOrderKey {
        EventOrderKey {
            sample_offset: self.sample_offset,
            semantic_precedence: self.event.semantic_precedence(),
            producer: self.producer,
            sequence: self.sequence,
        }
    }

    pub(crate) fn validate_for_block(self, frame_count: usize) -> Result<(), EventValidationError> {
        if self.sample_offset >= frame_count {
            return Err(EventValidationError::OffsetOutOfRange);
        }
        if let EngineEvent::SampleParameter {
            binding,
            engine_value,
        } = self.event
        {
            if !binding.accepts_engine_value(engine_value) {
                return Err(EventValidationError::InvalidParameterValue);
            }
        }
        Ok(())
    }
}

/// Compact callback-safe reason that a complete event slice was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventProcessError {
    /// At least one event did not belong to the half-open `0..frame_count`
    /// interval of the supplied render buffers.
    OffsetOutOfRange,
    /// Event keys were not strictly increasing according to [`EventOrderKey`].
    EventsNotOrdered,
    /// A sample-parameter value was non-finite or outside its prepared range.
    InvalidParameterValue,
}

const fn require_copy<T: Copy>() {}
const _: () = require_copy::<TimestampedEvent>();
const _: () = assert!(std::mem::size_of::<TimestampedEvent>() <= 64);
