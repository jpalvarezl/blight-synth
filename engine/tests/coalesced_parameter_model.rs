//! Deterministic model of ADR 0005's load-bearing slot/dirty protocol.
//!
//! This intentionally spells out the same Relaxed CAS, Release dirty RMW,
//! Acquire swap, and Relaxed slot load used by `CoalescedParameterStore` so Loom
//! explores publication/consumption interleavings that ordinary stress may miss.

use loom::{
    sync::{atomic::AtomicU64, Arc},
    thread,
};

#[allow(dead_code)]
#[path = "../src/parameter_atomic_protocol.rs"]
mod parameter_atomic_protocol;
use parameter_atomic_protocol::{
    APPLIED_OBSERVE, APPLIED_PUBLISH, DIRTY_CONSUME, DIRTY_PUBLISH, SLOT_CAS_FAILURE,
    SLOT_CAS_SUCCESS, SLOT_CONSUME,
};

fn publish(slot: &AtomicU64, dirty: &AtomicU64, value: u32) -> u64 {
    let mut old = slot.load(SLOT_CONSUME);
    let new = loop {
        let revision = (old >> 32) as u32;
        let new = (u64::from(revision + 1) << 32) | u64::from(value);
        match slot.compare_exchange_weak(old, new, SLOT_CAS_SUCCESS, SLOT_CAS_FAILURE) {
            Ok(_) => break new,
            Err(observed) => old = observed,
        }
    };
    dirty.fetch_or(1, DIRTY_PUBLISH);
    new
}

fn drain_once(slot: &AtomicU64, dirty: &AtomicU64, applied: &AtomicU64) {
    if dirty.swap(0, DIRTY_CONSUME) & 1 != 0 {
        let publication = slot.load(SLOT_CONSUME);
        applied.store(publication, APPLIED_PUBLISH);
    }
}

#[test]
fn release_sequence_has_no_lost_final_wakeup() {
    loom::model(|| {
        let slot = Arc::new(AtomicU64::new(1_u64 << 32));
        let dirty = Arc::new(AtomicU64::new(0));
        let applied = Arc::new(AtomicU64::new(0));

        let producer_slot = Arc::clone(&slot);
        let producer_dirty = Arc::clone(&dirty);
        let producer = thread::spawn(move || publish(&producer_slot, &producer_dirty, 7));

        let consumer_slot = Arc::clone(&slot);
        let consumer_dirty = Arc::clone(&dirty);
        let consumer_applied = Arc::clone(&applied);
        let consumer =
            thread::spawn(move || drain_once(&consumer_slot, &consumer_dirty, &consumer_applied));

        let published = producer.join().unwrap();
        consumer.join().unwrap();
        // The first boundary may have raced before the Release fetch_or. Once the
        // producer is quiescent, this next fixed boundary must observe the final
        // publication (or redundantly re-apply one it already observed).
        drain_once(&slot, &dirty, &applied);
        assert_eq!(applied.load(APPLIED_OBSERVE), published);
    });
}

#[test]
fn already_dirty_multi_producer_release_sequence_exposes_a_coherent_winner() {
    loom::model(|| {
        let slot = Arc::new(AtomicU64::new(1_u64 << 32));
        let dirty = Arc::new(AtomicU64::new(1));
        let applied = Arc::new(AtomicU64::new(0));

        let first_slot = Arc::clone(&slot);
        let first_dirty = Arc::clone(&dirty);
        let first = thread::spawn(move || publish(&first_slot, &first_dirty, 11));
        let second_slot = Arc::clone(&slot);
        let second_dirty = Arc::clone(&dirty);
        let second = thread::spawn(move || publish(&second_slot, &second_dirty, 22));

        let first_word = first.join().unwrap();
        let second_word = second.join().unwrap();
        let winner = if first_word >> 32 > second_word >> 32 {
            first_word
        } else {
            second_word
        };
        drain_once(&slot, &dirty, &applied);
        assert_eq!(applied.load(APPLIED_OBSERVE), winner);
    });
}
