//! Executable contract for #201's engine-facing current-block events.
//!
//! These tests deliberately exercise only note/recovery application, ordering,
//! validation, and segmented rendering. Sample-accurate parameter bindings are a
//! separate test tranche once this core process contract is green.

use std::sync::{
    atomic::{AtomicU32, AtomicUsize, Ordering},
    Arc,
};

use dsp::{
    id::{EffectId, InstrumentId, NoteEvent, NoteId},
    EffectInstallError, EffectInstallErrorKind, InstrumentTrait, MonoEffect, SynthCmd,
};
use engine::{
    Engine, EngineEvent, EventProcessError, EventProducerId, ParameterBindingError,
    ParameterTarget, PreparedParameterBinding, TimestampedEvent,
};
use param_manifest::{
    builtin::{master_gain_descriptor, MASTER_GAIN_ID},
    AutomationRate, NodeType, ParameterId, ParameterLookup, ParameterManifest,
};

const SAMPLE_RATE: f32 = 48_000.0;
const INSTRUMENT_ID: InstrumentId = 7;
const PRODUCER: EventProducerId = EventProducerId::new(1);

/// Minimal stateful DSP fixture: silence before note-on, `1.0` while a note is
/// active, then silence after the matching note-off or global recovery.
struct GateInstrument {
    active_note: Option<NoteId>,
    note_ons: Arc<AtomicUsize>,
    note_offs: Arc<AtomicUsize>,
    all_notes_offs: Arc<AtomicUsize>,
    effect_value: Arc<AtomicU32>,
}

impl InstrumentTrait for GateInstrument {
    fn id(&self) -> InstrumentId {
        INSTRUMENT_ID
    }

    fn note_on(&mut self, event: NoteEvent) {
        self.active_note = Some(event.id);
        self.note_ons.fetch_add(1, Ordering::Relaxed);
    }

    fn note_off(&mut self, note_id: NoteId) {
        if self.active_note == Some(note_id) {
            self.active_note = None;
        }
        self.note_offs.fetch_add(1, Ordering::Relaxed);
    }

    fn all_notes_off(&mut self) {
        self.active_note = None;
        self.all_notes_offs.fetch_add(1, Ordering::Relaxed);
    }

    fn process(&mut self, left: &mut [f32], right: &mut [f32], _sample_rate: f32) {
        if self.active_note.is_some() {
            for (left, right) in left.iter_mut().zip(right) {
                *left += 1.0;
                *right += 1.0;
            }
        }
    }

    fn set_pan(&mut self, _pan: f32) {}

    fn add_effect(&mut self, effect: Box<dyn MonoEffect>) -> Result<(), EffectInstallError> {
        Err(EffectInstallError::new(
            EffectInstallErrorKind::ChainFull,
            effect,
        ))
    }

    fn set_effect_parameter(&mut self, effect_id: EffectId, param_index: u32, value: f32) {
        if effect_id == 77 && param_index == 0 {
            self.effect_value.store(value.to_bits(), Ordering::Relaxed);
        }
    }

    fn try_handle_command(&mut self, _command: &SynthCmd) -> bool {
        false
    }
}

struct Fixture {
    engine: Engine,
    note_ons: Arc<AtomicUsize>,
    note_offs: Arc<AtomicUsize>,
    all_notes_offs: Arc<AtomicUsize>,
    effect_value: Arc<AtomicU32>,
}

impl Fixture {
    fn new() -> Self {
        let note_ons = Arc::new(AtomicUsize::new(0));
        let note_offs = Arc::new(AtomicUsize::new(0));
        let all_notes_offs = Arc::new(AtomicUsize::new(0));
        let effect_value = Arc::new(AtomicU32::new(0));
        let mut engine = Engine::new();
        engine.add_instrument(Box::new(GateInstrument {
            active_note: None,
            note_ons: note_ons.clone(),
            note_offs: note_offs.clone(),
            all_notes_offs: all_notes_offs.clone(),
            effect_value: effect_value.clone(),
        }));
        Self {
            engine,
            note_ons,
            note_offs,
            all_notes_offs,
            effect_value,
        }
    }
}

fn note_on(offset: usize, producer: EventProducerId, sequence: u64, id: u64) -> TimestampedEvent {
    TimestampedEvent::new(
        offset,
        producer,
        sequence,
        EngineEvent::NoteOn {
            instrument_id: INSTRUMENT_ID,
            note: NoteEvent {
                id: NoteId(id),
                pitch: 60,
                velocity: 100,
            },
        },
    )
}

fn note_off(offset: usize, producer: EventProducerId, sequence: u64, id: u64) -> TimestampedEvent {
    TimestampedEvent::new(
        offset,
        producer,
        sequence,
        EngineEvent::NoteOff {
            instrument_id: INSTRUMENT_ID,
            note_id: NoteId(id),
        },
    )
}

fn all_notes_off(offset: usize, producer: EventProducerId, sequence: u64) -> TimestampedEvent {
    TimestampedEvent::new(offset, producer, sequence, EngineEvent::AllNotesOff)
}

#[test]
fn note_events_take_effect_at_their_exact_half_open_offsets() {
    let mut fixture = Fixture::new();
    let events = [note_on(3, PRODUCER, 0, 42), note_off(6, PRODUCER, 1, 42)];
    let mut left = [0.0; 8];
    let mut right = [0.0; 8];

    let result = fixture
        .engine
        .process_with_events(&mut left, &mut right, SAMPLE_RATE, &events);

    assert_eq!(result, Ok(()));
    assert_eq!(left, [0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0, 0.0]);
    assert_eq!(right, left);
    assert_eq!(fixture.note_ons.load(Ordering::Relaxed), 1);
    assert_eq!(fixture.note_offs.load(Ordering::Relaxed), 1);
}

#[test]
fn offset_zero_is_applied_before_the_first_frame_is_rendered() {
    let mut fixture = Fixture::new();
    let events = [note_on(0, PRODUCER, 0, 42)];
    let mut left = [0.0; 4];
    let mut right = [0.0; 4];

    let result = fixture
        .engine
        .process_with_events(&mut left, &mut right, SAMPLE_RATE, &events);

    assert_eq!(result, Ok(()));
    assert_eq!(left, [1.0; 4]);
    assert_eq!(right, [1.0; 4]);
}

#[test]
fn an_event_at_frame_count_belongs_to_the_next_block() {
    let mut fixture = Fixture::new();
    let events = [note_on(4, PRODUCER, 0, 42)];
    let mut left = [0.25; 4];
    let mut right = [0.5; 4];

    let result = fixture
        .engine
        .process_with_events(&mut left, &mut right, SAMPLE_RATE, &events);

    assert_eq!(result, Err(EventProcessError::OffsetOutOfRange));
    assert_eq!(
        left, [0.25; 4],
        "invalid input must not render or mutate output"
    );
    assert_eq!(
        right, [0.5; 4],
        "invalid input must not render or mutate output"
    );
    assert_eq!(fixture.note_ons.load(Ordering::Relaxed), 0);
}

#[test]
fn the_whole_event_slice_is_validated_before_audio_or_engine_state_changes() {
    let mut fixture = Fixture::new();
    let events = [
        note_on(0, PRODUCER, 0, 42),
        // Invalid only after a valid event: a streaming validate/apply loop would
        // incorrectly activate the note before discovering this error.
        note_off(8, PRODUCER, 1, 42),
    ];
    let mut left = [0.25; 8];
    let mut right = [0.5; 8];

    let result = fixture
        .engine
        .process_with_events(&mut left, &mut right, SAMPLE_RATE, &events);

    assert_eq!(result, Err(EventProcessError::OffsetOutOfRange));
    assert_eq!(left, [0.25; 8]);
    assert_eq!(right, [0.5; 8]);
    assert_eq!(fixture.note_ons.load(Ordering::Relaxed), 0);
    assert_eq!(fixture.note_offs.load(Ordering::Relaxed), 0);
}

#[test]
fn events_must_be_in_ascending_sample_offset_order() {
    let mut fixture = Fixture::new();
    let events = [note_on(4, PRODUCER, 0, 42), note_off(2, PRODUCER, 1, 42)];
    let mut left = [0.0; 8];
    let mut right = [0.0; 8];

    let result = fixture
        .engine
        .process_with_events(&mut left, &mut right, SAMPLE_RATE, &events);

    assert_eq!(result, Err(EventProcessError::EventsNotOrdered));
    assert_eq!(left, [0.0; 8]);
    assert_eq!(right, [0.0; 8]);
    assert_eq!(fixture.note_ons.load(Ordering::Relaxed), 0);
}

#[test]
fn same_offset_semantic_order_is_recovery_then_release_then_attack() {
    let mut fixture = Fixture::new();
    fixture
        .engine
        .note_on_with_id(INSTRUMENT_ID, NoteId(1), 60, 100);
    // Descending sequence values are intentional: semantic precedence is the
    // primary same-offset key, with producer/sequence breaking ties afterward.
    let events = [
        all_notes_off(0, PRODUCER, 9),
        note_off(0, PRODUCER, 8, 1),
        note_on(0, PRODUCER, 7, 2),
    ];
    let mut left = [0.0; 4];
    let mut right = [0.0; 4];

    let result = fixture
        .engine
        .process_with_events(&mut left, &mut right, SAMPLE_RATE, &events);

    assert_eq!(result, Ok(()));
    assert_eq!(left, [1.0; 4], "the same-offset attack happens last");
    assert_eq!(right, [1.0; 4]);
    assert_eq!(fixture.all_notes_offs.load(Ordering::Relaxed), 1);
    assert_eq!(fixture.note_offs.load(Ordering::Relaxed), 1);
    // One direct setup note-on plus the timestamped attack.
    assert_eq!(fixture.note_ons.load(Ordering::Relaxed), 2);
}

#[test]
fn reversed_same_offset_semantics_are_rejected_without_reordering() {
    let mut fixture = Fixture::new();
    let events = [note_on(0, PRODUCER, 0, 2), note_off(0, PRODUCER, 1, 2)];
    let mut left = [0.0; 4];
    let mut right = [0.0; 4];

    let result = fixture
        .engine
        .process_with_events(&mut left, &mut right, SAMPLE_RATE, &events);

    assert_eq!(result, Err(EventProcessError::EventsNotOrdered));
    assert_eq!(fixture.note_ons.load(Ordering::Relaxed), 0);
    assert_eq!(fixture.note_offs.load(Ordering::Relaxed), 0);
}

#[test]
fn producer_identity_orders_events_after_offset_and_semantics() {
    let mut fixture = Fixture::new();
    let producer_one = EventProducerId::new(1);
    let producer_two = EventProducerId::new(2);
    let events = [
        note_on(0, producer_two, 0, 2),
        note_on(0, producer_one, 0, 1),
    ];
    let mut left = [0.0; 4];
    let mut right = [0.0; 4];

    let result = fixture
        .engine
        .process_with_events(&mut left, &mut right, SAMPLE_RATE, &events);

    assert_eq!(result, Err(EventProcessError::EventsNotOrdered));
    assert_eq!(fixture.note_ons.load(Ordering::Relaxed), 0);
}

#[test]
fn source_local_sequence_orders_events_after_offset_semantics_and_producer() {
    let mut fixture = Fixture::new();
    let events = [note_on(0, PRODUCER, 2, 2), note_on(0, PRODUCER, 1, 1)];
    let mut left = [0.0; 4];
    let mut right = [0.0; 4];

    let result = fixture
        .engine
        .process_with_events(&mut left, &mut right, SAMPLE_RATE, &events);

    assert_eq!(result, Err(EventProcessError::EventsNotOrdered));
    assert_eq!(fixture.note_ons.load(Ordering::Relaxed), 0);
}

fn prepared_master_gain_binding(
    automation_rate: AutomationRate,
) -> (Result<PreparedParameterBinding, ParameterBindingError>, f32) {
    let mut descriptor = master_gain_descriptor();
    descriptor.automation_rate = automation_rate;
    let lookup = ParameterLookup::from_manifest(&ParameterManifest::new(vec![descriptor]))
        .expect("test parameter manifest is valid");
    let key = lookup
        .key_for(&ParameterId::from(MASTER_GAIN_ID))
        .expect("stable parameter id resolves on NRT");
    let parameter = *lookup.get(key).expect("runtime parameter is prepared");
    let engine_value = lookup
        .normalized_to_engine(key, 0.5)
        .expect("normalized value maps through the prepared table");
    (
        PreparedParameterBinding::new(parameter, ParameterTarget::MasterEffect { effect_id: 99 }),
        engine_value,
    )
}

#[test]
fn only_sample_event_parameters_can_build_timestamped_bindings() {
    let (sample_binding, _) = prepared_master_gain_binding(AutomationRate::SampleEvent);
    assert!(sample_binding.is_ok());

    for wrong_rate in [AutomationRate::ControlCoalesced, AutomationRate::Structural] {
        let (binding, _) = prepared_master_gain_binding(wrong_rate);
        assert_eq!(binding, Err(ParameterBindingError::NotSampleEvent));
    }
}

#[test]
fn malformed_sample_parameter_values_reject_the_whole_block() {
    for invalid_value in [f32::NAN, 1.0] {
        let mut fixture = Fixture::new();
        let (binding, _) = prepared_master_gain_binding(AutomationRate::SampleEvent);
        let binding = binding.expect("sample-event binding");
        let events = [
            note_on(0, PRODUCER, 0, 42),
            TimestampedEvent::new(
                1,
                PRODUCER,
                1,
                EngineEvent::SampleParameter {
                    binding,
                    engine_value: invalid_value,
                },
            ),
        ];
        let mut left = [0.25; 4];
        let mut right = [0.5; 4];

        assert_eq!(
            fixture
                .engine
                .process_with_events(&mut left, &mut right, SAMPLE_RATE, &events,),
            Err(EventProcessError::InvalidParameterValue),
        );
        assert_eq!(left, [0.25; 4]);
        assert_eq!(right, [0.5; 4]);
        assert_eq!(fixture.note_ons.load(Ordering::Relaxed), 0);
    }
}

#[test]
fn sample_parameter_changes_take_effect_at_their_exact_offset() {
    let mut fixture = Fixture::new();
    fixture.engine.add_master_effect(
        dsp::EffectFactory::new(SAMPLE_RATE).create_stereo_gain(99, 1.0),
        &mut engine::DropRetireSink,
    );
    let (binding, engine_value) = prepared_master_gain_binding(AutomationRate::SampleEvent);
    let binding = binding.expect("sample-event binding");
    let events = [
        note_on(0, PRODUCER, 0, 42),
        TimestampedEvent::new(
            4,
            PRODUCER,
            1,
            EngineEvent::SampleParameter {
                binding,
                engine_value,
            },
        ),
    ];
    let mut left = [0.0; 8];
    let mut right = [0.0; 8];

    let result = fixture
        .engine
        .process_with_events(&mut left, &mut right, SAMPLE_RATE, &events);

    assert_eq!(result, Ok(()));
    assert_eq!(&left[..4], &[1.0; 4]);
    for sample in &left[4..] {
        assert!((*sample - 0.5).abs() < 1.0e-6);
    }
    assert_eq!(right, left);
    assert_eq!(binding.key().0, 0);
}

#[test]
fn same_offset_parameter_changes_precede_note_attacks() {
    let mut fixture = Fixture::new();
    fixture.engine.add_master_effect(
        dsp::EffectFactory::new(SAMPLE_RATE).create_stereo_gain(99, 1.0),
        &mut engine::DropRetireSink,
    );
    let (binding, engine_value) = prepared_master_gain_binding(AutomationRate::SampleEvent);
    let binding = binding.expect("sample-event binding");
    let parameter = TimestampedEvent::new(
        0,
        PRODUCER,
        10,
        EngineEvent::SampleParameter {
            binding,
            engine_value,
        },
    );
    let attack = note_on(0, PRODUCER, 0, 42);
    let mut left = [0.0; 4];
    let mut right = [0.0; 4];

    assert_eq!(
        fixture.engine.process_with_events(
            &mut left,
            &mut right,
            SAMPLE_RATE,
            &[attack, parameter]
        ),
        Err(EventProcessError::EventsNotOrdered),
        "the engine rejects rather than silently reorders same-offset input"
    );
    assert_eq!(fixture.note_ons.load(Ordering::Relaxed), 0);

    assert_eq!(
        fixture.engine.process_with_events(
            &mut left,
            &mut right,
            SAMPLE_RATE,
            &[parameter, attack]
        ),
        Ok(())
    );
    for sample in left {
        assert!((sample - 0.5).abs() < 1.0e-6);
    }
}

#[test]
fn prepared_instrument_effect_binding_dispatches_to_its_concrete_target() {
    let mut descriptor = master_gain_descriptor();
    descriptor.owner.node_type = NodeType::InstrumentEffect;
    descriptor.owner.path = "instrument/effect:gain".to_string();
    descriptor.automation_rate = AutomationRate::SampleEvent;
    let lookup = ParameterLookup::from_manifest(&ParameterManifest::new(vec![descriptor]))
        .expect("instrument-effect descriptor is valid");
    let key = lookup
        .key_for(&ParameterId::from(MASTER_GAIN_ID))
        .expect("stable parameter id resolves");
    let runtime_parameter = *lookup.get(key).expect("runtime parameter is prepared");
    let binding = PreparedParameterBinding::new(
        runtime_parameter,
        ParameterTarget::InstrumentEffect {
            instrument_id: INSTRUMENT_ID,
            effect_id: 77,
        },
    )
    .expect("sample-event parameter binds");
    let engine_value = lookup
        .normalized_to_engine(key, 0.5)
        .expect("normalized value maps on NRT");
    let event = TimestampedEvent::new(
        0,
        PRODUCER,
        0,
        EngineEvent::SampleParameter {
            binding,
            engine_value,
        },
    );
    let mut fixture = Fixture::new();
    let mut left = [0.0; 1];
    let mut right = [0.0; 1];

    assert_eq!(
        fixture
            .engine
            .process_with_events(&mut left, &mut right, SAMPLE_RATE, &[event],),
        Ok(())
    );
    assert_eq!(
        f32::from_bits(fixture.effect_value.load(Ordering::Relaxed)),
        engine_value,
    );
}
