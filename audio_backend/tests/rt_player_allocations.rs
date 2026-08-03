#![cfg(feature = "device-host")]

use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::Cell,
    sync::Arc,
};

use audio_backend::{
    id::InstrumentId, AudioProcessor, Command, InstrumentCmd, InstrumentFactory, MeterState,
    PlayerProcessStatus, TransportCmd,
};
use engine::RetiredState;
use ringbuf::{
    storage::Heap,
    traits::{Producer, Split},
    SharedRb,
};
use sequencer::models::{
    Chain, EffectType, Event, Phrase, Song, SongRow, DEFAULT_CHAIN_LENGTH, EMPTY_CHAIN_SLOT,
    MAX_TRACKS,
};

const INSTRUMENT_ID: InstrumentId = InstrumentId::from_raw(1);

struct TrackingAllocator;

thread_local! {
    static TRACKING: Cell<bool> = const { Cell::new(false) };
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
    static DEALLOCATIONS: Cell<usize> = const { Cell::new(0) };
    static REALLOCATIONS: Cell<usize> = const { Cell::new(0) };
}

#[global_allocator]
static ALLOCATOR: TrackingAllocator = TrackingAllocator;

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if TRACKING.try_with(Cell::get).unwrap_or(false) {
            let _ = ALLOCATIONS.try_with(|count| count.set(count.get() + 1));
        }
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if TRACKING.try_with(Cell::get).unwrap_or(false) {
            let _ = DEALLOCATIONS.try_with(|count| count.set(count.get() + 1));
        }
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        if TRACKING.try_with(Cell::get).unwrap_or(false) {
            let _ = ALLOCATIONS.try_with(|count| count.set(count.get() + 1));
        }
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if TRACKING.try_with(Cell::get).unwrap_or(false) {
            let _ = REALLOCATIONS.try_with(|count| count.set(count.get() + 1));
        }
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Counts {
    allocations: usize,
    deallocations: usize,
    reallocations: usize,
}

fn measure(operation: impl FnOnce() -> PlayerProcessStatus) -> (Counts, PlayerProcessStatus) {
    ALLOCATIONS.with(|count| count.set(0));
    DEALLOCATIONS.with(|count| count.set(0));
    REALLOCATIONS.with(|count| count.set(0));
    TRACKING.with(|tracking| tracking.set(true));
    let status = operation();
    TRACKING.with(|tracking| tracking.set(false));
    (
        Counts {
            allocations: ALLOCATIONS.with(Cell::get),
            deallocations: DEALLOCATIONS.with(Cell::get),
            reallocations: REALLOCATIONS.with(Cell::get),
        },
        status,
    )
}

#[test]
fn queued_live_attack_release_and_stopped_render_have_zero_heap_activity() {
    let commands = SharedRb::<Heap<Command>>::new(8);
    let (mut command_tx, command_rx) = commands.split();
    let retirement = SharedRb::<Heap<RetiredState>>::new(8);
    let (retirement_tx, _retirement_rx) = retirement.split();
    let mut processor = AudioProcessor::new(
        command_rx,
        retirement_tx,
        48_000.0,
        2,
        Arc::new(MeterState::new()),
    );
    assert!(command_tx
        .try_push(
            InstrumentCmd::AddInstrument {
                instrument: InstrumentFactory::new(48_000.0)
                    .create_simple_oscillator(INSTRUMENT_ID, 0.0),
            }
            .into(),
        )
        .is_ok());
    let mut output = [0.0; 512];
    processor.process(&mut output); // install and warm the stopped render path

    assert!(command_tx
        .try_push(
            InstrumentCmd::NoteOn {
                instrument_id: INSTRUMENT_ID,
                note: 60,
                velocity: 100,
            }
            .into(),
        )
        .is_ok());
    let (attack_counts, attack_status) = measure(|| processor.process(&mut output));
    assert_eq!(
        attack_counts,
        Counts {
            allocations: 0,
            deallocations: 0,
            reallocations: 0
        }
    );
    assert!(attack_status.is_complete());
    assert!(output.iter().any(|sample| *sample != 0.0));

    assert!(command_tx
        .try_push(
            InstrumentCmd::NoteOff {
                instrument_id: INSTRUMENT_ID,
            }
            .into(),
        )
        .is_ok());
    let (release_counts, release_status) = measure(|| processor.process(&mut output));
    assert_eq!(
        release_counts,
        Counts {
            allocations: 0,
            deallocations: 0,
            reallocations: 0
        }
    );
    assert!(release_status.is_complete());

    let (tail_counts, tail_status) = measure(|| processor.process(&mut output));
    assert_eq!(
        tail_counts,
        Counts {
            allocations: 0,
            deallocations: 0,
            reallocations: 0
        }
    );
    assert!(tail_status.is_complete());
}

#[test]
fn playing_tracker_tick_event_admission_and_segmented_render_have_zero_heap_activity() {
    let event = Event {
        note: 60,
        volume: 100,
        instrument_id: 1,
        effect: EffectType::Arpeggio,
        effect_param: 0,
    };
    let phrase = Phrase::from_events(std::iter::repeat_n(event, 16));
    let chain = Chain::new([0; DEFAULT_CHAIN_LENGTH]);
    let mut chains = [EMPTY_CHAIN_SLOT; MAX_TRACKS];
    chains[0] = 0;
    let mut song = Song::new("RT tracker audit");
    song.initial_bpm = u16::MAX;
    song.initial_speed = 1;
    song.phrase_bank = vec![phrase];
    song.chain_bank = vec![chain];
    song.arrangement = vec![SongRow::new(chains)];

    let commands = SharedRb::<Heap<Command>>::new(8);
    let (mut command_tx, command_rx) = commands.split();
    let retirement = SharedRb::<Heap<RetiredState>>::new(8);
    let (retirement_tx, _retirement_rx) = retirement.split();
    let mut processor = AudioProcessor::new_with_song(
        Arc::new(song),
        command_rx,
        retirement_tx,
        48_000.0,
        2,
        Arc::new(MeterState::new()),
    );
    assert!(command_tx
        .try_push(
            InstrumentCmd::AddInstrument {
                instrument: InstrumentFactory::new(48_000.0)
                    .create_simple_oscillator(INSTRUMENT_ID, 0.0),
            }
            .into(),
        )
        .is_ok());
    let mut output = [0.0; 512];
    processor.process(&mut output); // install and warm stopped DSP
    assert!(command_tx
        .try_push(TransportCmd::PlayLastSong.into())
        .is_ok());

    let (counts, status) = measure(|| processor.process(&mut output));

    assert_eq!(
        counts,
        Counts {
            allocations: 0,
            deallocations: 0,
            reallocations: 0
        }
    );
    assert!(status.is_complete());
    assert!(output.iter().any(|sample| *sample != 0.0));
}
