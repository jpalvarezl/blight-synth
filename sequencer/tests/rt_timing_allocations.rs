use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::Cell,
    hint::black_box,
};

use sequencer::timing::{TickTempo, TimingAdvanceStatus, TimingState};

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
        record(&ALLOCATIONS);
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        record(&DEALLOCATIONS);
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        record(&ALLOCATIONS);
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        record(&REALLOCATIONS);
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

fn record(counter: &'static std::thread::LocalKey<Cell<usize>>) {
    if TRACKING.try_with(Cell::get).unwrap_or(false) {
        let _ = counter.try_with(|count| count.set(count.get() + 1));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AllocationCounts {
    allocations: usize,
    deallocations: usize,
    reallocations: usize,
}

fn measure_allocations(operation: impl FnOnce()) -> AllocationCounts {
    ALLOCATIONS.with(|count| count.set(0));
    DEALLOCATIONS.with(|count| count.set(0));
    REALLOCATIONS.with(|count| count.set(0));
    TRACKING.with(|tracking| tracking.set(true));

    operation();

    TRACKING.with(|tracking| tracking.set(false));
    AllocationCounts {
        allocations: ALLOCATIONS.with(Cell::get),
        deallocations: DEALLOCATIONS.with(Cell::get),
        reallocations: REALLOCATIONS.with(Cell::get),
    }
}

#[test]
fn prepared_tick_advance_has_no_heap_activity() {
    let mut timing = TimingState::prepare(48_000.0, 125.0, 32).unwrap();
    let mut offsets = [0_usize; 32];
    let mut written = 0;
    let mut tempo_changed = false;

    // Initialize the thread-local audit state before entering the measured region.
    let _ = measure_allocations(|| {});

    let counts = measure_allocations(|| {
        for frame_count in [64, 511, 2_048, 1, 4_096, 127] {
            let result = timing.advance_ticks(frame_count, |tick| {
                offsets[written] = tick.sample_offset;
                written += 1;
                if tempo_changed {
                    TickTempo::Unchanged
                } else {
                    tempo_changed = true;
                    TickTempo::SetBpm(130.0)
                }
            });
            black_box(result);
        }
    });

    assert!(written > 0);
    assert!(offsets[..written].iter().all(|offset| *offset < 4_096));
    assert_eq!(
        timing.advance_ticks(0, |_| TickTempo::Unchanged).status,
        TimingAdvanceStatus::Complete
    );
    assert_eq!(counts.allocations, 0, "unexpected RT allocations");
    assert_eq!(counts.deallocations, 0, "unexpected RT deallocations");
    assert_eq!(counts.reallocations, 0, "unexpected RT reallocations");
}
