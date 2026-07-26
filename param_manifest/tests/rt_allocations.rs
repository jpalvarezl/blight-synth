use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::Cell,
    hint::black_box,
};

use param_manifest::{
    builtin::{master_gain_descriptor, MASTER_GAIN_ID},
    AutomationRate, DiscreteStep, Mapping, NodeRef, NodeType, ParameterDescriptor, ParameterId,
    ParameterKind, ParameterLookup, ParameterManifest, RuntimeParamKey, SmoothingPolicy, Unit,
    ValueRange, Visibility,
};

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

fn discrete_descriptor() -> ParameterDescriptor {
    ParameterDescriptor {
        id: ParameterId::from("delay.mode"),
        owner: NodeRef {
            node_type: NodeType::InstrumentEffect,
            path: "instrument/effect:delay".to_string(),
            engine_param_index: 1,
        },
        display_name: "Delay Mode".to_string(),
        short_name: "Mode".to_string(),
        unit: Unit::Count,
        range: ValueRange {
            min: 0.0,
            max: 2.0,
            default: 1.25,
        },
        mapping: Mapping::Linear { min: 0.0, max: 2.0 },
        kind: ParameterKind::Discrete {
            steps: vec![
                DiscreteStep {
                    label: "Off".to_string(),
                    engine_value: 0.0,
                },
                DiscreteStep {
                    label: "Slap".to_string(),
                    engine_value: 1.25,
                },
                DiscreteStep {
                    label: "Ping-Pong".to_string(),
                    engine_value: 2.0,
                },
            ],
        },
        automation_rate: AutomationRate::ControlCoalesced,
        smoothing: SmoothingPolicy::None,
        visibility: Visibility::default(),
        version_added: 1,
        deprecated: None,
    }
}

#[test]
fn prepared_runtime_get_and_conversion_paths_have_no_heap_activity() {
    let manifest = ParameterManifest::new(vec![master_gain_descriptor(), discrete_descriptor()]);
    let lookup = ParameterLookup::from_manifest(&manifest).expect("valid manifest");
    let continuous_key = lookup
        .key_for(&ParameterId::from(MASTER_GAIN_ID))
        .expect("continuous key");
    let discrete_key = lookup
        .key_for(&ParameterId::from("delay.mode"))
        .expect("discrete key");
    let table = lookup.table();
    let invalid_key = RuntimeParamKey(u32::MAX);

    // Initialize the thread-local audit state before entering the measured region.
    let _ = measure_allocations(|| {});

    let counts = measure_allocations(|| {
        for key in [continuous_key, discrete_key, invalid_key] {
            black_box(table.get(key));
        }
        for normalized in [0.25, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            black_box(table.normalized_to_engine(continuous_key, normalized));
            black_box(table.normalized_to_engine(discrete_key, normalized));
        }
        black_box(table.normalized_to_engine(invalid_key, f32::NAN));
    });

    assert_eq!(counts.allocations, 0, "unexpected RT allocations");
    assert_eq!(counts.deallocations, 0, "unexpected RT deallocations");
    assert_eq!(counts.reallocations, 0, "unexpected RT reallocations");
}
