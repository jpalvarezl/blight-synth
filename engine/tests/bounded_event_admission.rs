//! Executable contract for #203's bounded current-block admission and merge.

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use dsp::{
    id::{EffectId, InstrumentId, NoteEvent, NoteId},
    EffectInstallError, EffectInstallErrorKind, InstrumentTrait, MonoEffect, SynthCmd,
};
use engine::{
    BoundedEventAdmission, Engine, EngineEvent, EventAdmissionErrorKind,
    EventAdmissionPrepareError, EventProducerId, OrdinaryEventBlockStatus, ProducerAdmissionStatus,
    RecoveryAdmissionError, RecoveryAdmissionStatus, TimestampedEvent,
};

const PRODUCER_A: EventProducerId = EventProducerId::new(10);
const PRODUCER_B: EventProducerId = EventProducerId::new(20);
const RECOVERY: EventProducerId = EventProducerId::new(99);
const FRAMES: usize = 64;

fn note_on(
    offset: usize,
    producer: EventProducerId,
    sequence: u64,
    note_id: u64,
) -> TimestampedEvent {
    TimestampedEvent::new(
        offset,
        producer,
        sequence,
        EngineEvent::NoteOn {
            instrument_id: 1,
            note: NoteEvent {
                id: NoteId(note_id),
                pitch: 60,
                velocity: 100,
            },
        },
    )
}

fn note_off(
    offset: usize,
    producer: EventProducerId,
    sequence: u64,
    note_id: u64,
) -> TimestampedEvent {
    TimestampedEvent::new(
        offset,
        producer,
        sequence,
        EngineEvent::NoteOff {
            instrument_id: 1,
            note_id: NoteId(note_id),
        },
    )
}

fn rejected_kind(status: ProducerAdmissionStatus) -> EventAdmissionErrorKind {
    match status {
        ProducerAdmissionStatus::Rejected(error) => error.kind(),
        ProducerAdmissionStatus::Staged => panic!("expected producer rejection"),
    }
}

struct RecoveryProbe {
    all_notes_offs: Arc<AtomicUsize>,
}

impl InstrumentTrait for RecoveryProbe {
    fn id(&self) -> InstrumentId {
        1
    }

    fn note_on(&mut self, _event: NoteEvent) {}

    fn note_off(&mut self, _note_id: NoteId) {}

    fn all_notes_off(&mut self) {
        self.all_notes_offs.fetch_add(1, Ordering::Relaxed);
    }

    fn process(&mut self, _left: &mut [f32], _right: &mut [f32], _sample_rate: f32) {}

    fn set_pan(&mut self, _pan: f32) {}

    fn add_effect(&mut self, effect: Box<dyn MonoEffect>) -> Result<(), EffectInstallError> {
        Err(EffectInstallError::new(
            EffectInstallErrorKind::ChainFull,
            effect,
        ))
    }

    fn set_effect_parameter(&mut self, _effect_id: EffectId, _param_index: u32, _value: f32) {}

    fn try_handle_command(&mut self, _command: &SynthCmd) -> bool {
        false
    }
}

#[test]
fn preparation_validates_stable_producer_configuration_and_exposes_bounds() {
    assert_eq!(
        BoundedEventAdmission::prepare(4, &[PRODUCER_A, PRODUCER_A], RECOVERY).err(),
        Some(EventAdmissionPrepareError::DuplicateProducer)
    );
    assert_eq!(
        BoundedEventAdmission::prepare(4, &[PRODUCER_A, RECOVERY], RECOVERY).err(),
        Some(EventAdmissionPrepareError::RecoveryProducerConflict)
    );
    assert_eq!(
        BoundedEventAdmission::prepare(usize::MAX, &[PRODUCER_A], RECOVERY).err(),
        Some(EventAdmissionPrepareError::CapacityOverflow)
    );

    // Configuration iteration order does not define producer identity or order.
    let admission = BoundedEventAdmission::prepare(4, &[PRODUCER_B, PRODUCER_A], RECOVERY).unwrap();
    assert_eq!(admission.ordinary_capacity(), 4);
    assert_eq!(admission.producer_count(), 2);
    assert_eq!(admission.recovery_producer(), RECOVERY);
}

#[test]
fn exact_ordinary_capacity_is_accepted() {
    let mut admission =
        BoundedEventAdmission::prepare(3, &[PRODUCER_A, PRODUCER_B], RECOVERY).unwrap();
    admission.begin_block(FRAMES);
    assert_eq!(
        admission.submit_producer(
            PRODUCER_A,
            &[note_on(0, PRODUCER_A, 1, 1), note_off(8, PRODUCER_A, 2, 1),],
        ),
        ProducerAdmissionStatus::Staged
    );
    assert_eq!(
        admission.submit_producer(PRODUCER_B, &[note_on(4, PRODUCER_B, 50, 2)]),
        ProducerAdmissionStatus::Staged
    );

    let block = admission.finish_block();
    assert_eq!(
        block.ordinary_status(),
        OrdinaryEventBlockStatus::Accepted { event_count: 3 }
    );
    assert_eq!(block.events().len(), 3);
    assert!(block
        .events()
        .windows(2)
        .all(|pair| pair[0].order_key() < pair[1].order_key()));
    let again = admission.finish_block();
    assert_eq!(
        again.ordinary_status(),
        OrdinaryEventBlockStatus::Accepted { event_count: 3 }
    );
    assert_eq!(again.events().len(), 3);
}

#[test]
fn over_capacity_rejects_the_whole_ordinary_block_without_carrying_a_prefix() {
    let mut admission =
        BoundedEventAdmission::prepare(2, &[PRODUCER_A, PRODUCER_B], RECOVERY).unwrap();
    admission.begin_block(FRAMES);
    assert_eq!(
        admission.submit_producer(PRODUCER_A, &[note_on(0, PRODUCER_A, 1, 1)]),
        ProducerAdmissionStatus::Staged
    );
    let overflow = admission.submit_producer(
        PRODUCER_B,
        &[note_on(1, PRODUCER_B, 1, 2), note_off(2, PRODUCER_B, 2, 2)],
    );
    assert_eq!(
        rejected_kind(overflow),
        EventAdmissionErrorKind::OrdinaryCapacityExceeded
    );

    let block = admission.finish_block();
    let error = match block.ordinary_status() {
        OrdinaryEventBlockStatus::Rejected(error) => error,
        status => panic!("expected rejected block, got {status:?}"),
    };
    assert_eq!(error.producer(), PRODUCER_B);
    assert_eq!(
        error.kind(),
        EventAdmissionErrorKind::OrdinaryCapacityExceeded
    );
    assert!(block.events().is_empty(), "no accepted prefix may escape");

    // Beginning another block explicitly clears rejected data; no event is
    // silently delayed. Silent producers need not submit empty slices.
    admission.begin_block(FRAMES);
    assert_eq!(
        admission.submit_producer(PRODUCER_B, &[note_on(0, PRODUCER_B, 1, 3)]),
        ProducerAdmissionStatus::Staged
    );
    let next = admission.finish_block();
    assert_eq!(
        next.ordinary_status(),
        OrdinaryEventBlockStatus::Accepted { event_count: 1 }
    );
    assert_eq!(next.events(), &[note_on(0, PRODUCER_B, 1, 3)]);
}

#[test]
fn canonical_output_is_independent_of_submission_and_config_iteration_order() {
    fn run(configured: &[EventProducerId], submit_b_first: bool) -> Vec<TimestampedEvent> {
        let mut admission = BoundedEventAdmission::prepare(4, configured, RECOVERY).unwrap();
        admission.begin_block(FRAMES);
        // Source sequence is emission order. At one offset canonical semantic
        // precedence intentionally moves the release before the attack.
        let a = [
            note_on(8, PRODUCER_A, 10, 1),
            note_off(8, PRODUCER_A, 11, 1),
        ];
        let b = [
            note_on(8, PRODUCER_B, 20, 2),
            note_on(16, PRODUCER_B, 21, 3),
        ];
        if submit_b_first {
            assert_eq!(
                admission.submit_producer(PRODUCER_B, &b),
                ProducerAdmissionStatus::Staged
            );
            assert_eq!(
                admission.submit_producer(PRODUCER_A, &a),
                ProducerAdmissionStatus::Staged
            );
        } else {
            assert_eq!(
                admission.submit_producer(PRODUCER_A, &a),
                ProducerAdmissionStatus::Staged
            );
            assert_eq!(
                admission.submit_producer(PRODUCER_B, &b),
                ProducerAdmissionStatus::Staged
            );
        }
        admission.finish_block().events().to_vec()
    }

    let first = run(&[PRODUCER_A, PRODUCER_B], false);
    let second = run(&[PRODUCER_B, PRODUCER_A], true);
    assert_eq!(first, second);
    assert_eq!(
        first,
        vec![
            note_off(8, PRODUCER_A, 11, 1),
            note_on(8, PRODUCER_A, 10, 1),
            note_on(8, PRODUCER_B, 20, 2),
            note_on(16, PRODUCER_B, 21, 3),
        ]
    );
}

#[test]
fn malformed_identity_sequence_source_order_and_duplicate_submission_are_observable() {
    let cases = [
        (
            EventProducerId::new(404),
            vec![note_on(0, EventProducerId::new(404), 1, 1)],
            EventAdmissionErrorKind::UnknownProducer,
        ),
        (
            PRODUCER_A,
            vec![note_on(0, PRODUCER_B, 1, 1)],
            EventAdmissionErrorKind::EventProducerMismatch,
        ),
        (
            PRODUCER_A,
            vec![note_on(0, PRODUCER_A, 2, 1), note_on(1, PRODUCER_A, 2, 2)],
            EventAdmissionErrorKind::SequenceNotIncreasing,
        ),
        (
            PRODUCER_A,
            vec![note_on(4, PRODUCER_A, 1, 1), note_on(3, PRODUCER_A, 2, 2)],
            EventAdmissionErrorKind::SourceOffsetsNotOrdered,
        ),
        (
            PRODUCER_A,
            vec![note_on(FRAMES, PRODUCER_A, 1, 1)],
            EventAdmissionErrorKind::OffsetOutOfRange,
        ),
        (
            PRODUCER_A,
            vec![TimestampedEvent::new(
                0,
                PRODUCER_A,
                1,
                EngineEvent::AllNotesOff,
            )],
            EventAdmissionErrorKind::RecoveryInOrdinaryLane,
        ),
    ];

    for (producer, events, expected) in cases {
        let mut admission =
            BoundedEventAdmission::prepare(4, &[PRODUCER_A, PRODUCER_B], RECOVERY).unwrap();
        admission.begin_block(FRAMES);
        assert_eq!(
            rejected_kind(admission.submit_producer(producer, &events)),
            expected
        );
        assert!(matches!(
            admission.finish_block().ordinary_status(),
            OrdinaryEventBlockStatus::Rejected(_)
        ));
    }

    let mut admission = BoundedEventAdmission::prepare(4, &[PRODUCER_A], RECOVERY).unwrap();
    admission.begin_block(FRAMES);
    assert_eq!(
        admission.submit_producer(PRODUCER_A, &[]),
        ProducerAdmissionStatus::Staged
    );
    assert_eq!(
        rejected_kind(admission.submit_producer(PRODUCER_A, &[])),
        EventAdmissionErrorKind::ProducerAlreadySubmitted
    );
}

#[test]
fn sequence_baselines_commit_only_for_successful_blocks_and_reset_explicitly() {
    let mut admission = BoundedEventAdmission::prepare(1, &[PRODUCER_A], RECOVERY).unwrap();
    admission.begin_block(FRAMES);
    assert_eq!(
        admission.submit_producer(PRODUCER_A, &[note_on(0, PRODUCER_A, 7, 1)]),
        ProducerAdmissionStatus::Staged
    );
    assert!(matches!(
        admission.finish_block().ordinary_status(),
        OrdinaryEventBlockStatus::Accepted { .. }
    ));

    admission.begin_block(FRAMES);
    assert_eq!(
        admission.submit_producer(PRODUCER_A, &[]),
        ProducerAdmissionStatus::Staged
    );
    assert_eq!(
        admission.finish_block().ordinary_status(),
        OrdinaryEventBlockStatus::Accepted { event_count: 0 }
    );

    admission.begin_block(FRAMES);
    assert_eq!(
        rejected_kind(admission.submit_producer(PRODUCER_A, &[note_on(0, PRODUCER_A, 7, 2)])),
        EventAdmissionErrorKind::SequenceNotIncreasing,
        "an empty block must not erase the committed source baseline"
    );

    admission.reset();
    assert!(matches!(
        admission.finish_block().ordinary_status(),
        OrdinaryEventBlockStatus::NotStarted
    ));
    admission.begin_block(FRAMES);
    assert_eq!(
        admission.submit_producer(PRODUCER_A, &[note_on(0, PRODUCER_A, 7, 3)]),
        ProducerAdmissionStatus::Staged
    );
    assert_eq!(
        admission.finish_block().ordinary_status(),
        OrdinaryEventBlockStatus::Accepted { event_count: 1 }
    );
}

#[test]
fn phase_and_recovery_failures_are_compact_and_recovery_sequence_is_validated() {
    let mut admission = BoundedEventAdmission::prepare(1, &[PRODUCER_A], RECOVERY).unwrap();
    assert_eq!(
        rejected_kind(admission.submit_producer(PRODUCER_A, &[])),
        EventAdmissionErrorKind::NotCollecting
    );
    assert_eq!(
        admission.request_all_notes_off(0, 1),
        RecoveryAdmissionStatus::Rejected(RecoveryAdmissionError::NotCollecting)
    );

    admission.begin_block(FRAMES);
    assert_eq!(
        admission.request_all_notes_off(FRAMES, 1),
        RecoveryAdmissionStatus::Rejected(RecoveryAdmissionError::OffsetOutOfRange)
    );
    assert_eq!(
        admission.request_all_notes_off(0, 5),
        RecoveryAdmissionStatus::Staged
    );
    assert_eq!(
        admission.request_all_notes_off(1, 6),
        RecoveryAdmissionStatus::Rejected(RecoveryAdmissionError::AlreadyRequested)
    );
    admission.finish_block();

    admission.begin_block(FRAMES);
    assert_eq!(
        admission.request_all_notes_off(0, 5),
        RecoveryAdmissionStatus::Rejected(RecoveryAdmissionError::SequenceNotIncreasing)
    );
    assert_eq!(
        admission.request_all_notes_off(0, 6),
        RecoveryAdmissionStatus::Staged
    );
}

#[test]
fn recovery_uses_reserved_capacity_when_ordinary_storage_is_full() {
    let mut admission = BoundedEventAdmission::prepare(2, &[PRODUCER_A], RECOVERY).unwrap();
    admission.begin_block(FRAMES);
    assert_eq!(
        admission.submit_producer(
            PRODUCER_A,
            &[note_on(0, PRODUCER_A, 1, 1), note_on(8, PRODUCER_A, 2, 2),],
        ),
        ProducerAdmissionStatus::Staged
    );
    assert_eq!(
        admission.request_all_notes_off(0, 100),
        RecoveryAdmissionStatus::Staged
    );

    let block = admission.finish_block();
    assert_eq!(
        block.ordinary_status(),
        OrdinaryEventBlockStatus::Accepted { event_count: 2 }
    );
    assert_eq!(block.events().len(), 3);
    assert!(matches!(block.events()[0].event, EngineEvent::AllNotesOff));
    assert_eq!(block.events()[0].producer, RECOVERY);

    let all_notes_offs = Arc::new(AtomicUsize::new(0));
    let mut engine = Engine::new();
    engine.add_instrument(Box::new(RecoveryProbe {
        all_notes_offs: all_notes_offs.clone(),
    }));
    let mut left = [0.0; FRAMES];
    let mut right = [0.0; FRAMES];
    assert_eq!(
        engine.process_with_events(&mut left, &mut right, 48_000.0, block.events()),
        Ok(())
    );
    assert_eq!(all_notes_offs.load(Ordering::Relaxed), 1);
}

#[test]
fn recovery_remains_executable_after_ordinary_overflow() {
    let mut admission = BoundedEventAdmission::prepare(1, &[PRODUCER_A], RECOVERY).unwrap();
    admission.begin_block(FRAMES);
    assert_eq!(
        rejected_kind(admission.submit_producer(
            PRODUCER_A,
            &[note_on(0, PRODUCER_A, 1, 1), note_on(1, PRODUCER_A, 2, 2),]
        )),
        EventAdmissionErrorKind::OrdinaryCapacityExceeded
    );
    assert_eq!(
        admission.request_all_notes_off(0, 1),
        RecoveryAdmissionStatus::Staged
    );

    let block = admission.finish_block();
    assert!(matches!(
        block.ordinary_status(),
        OrdinaryEventBlockStatus::Rejected(_)
    ));
    assert_eq!(block.events().len(), 1);
    assert!(matches!(block.events()[0].event, EngineEvent::AllNotesOff));

    // Re-finalizing is idempotent and does not duplicate the reserved event.
    let again = admission.finish_block();
    assert_eq!(again.events().len(), 1);
}
