use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::Cell,
    hint::black_box,
};

use dsp::{
    id::{EffectId, InstrumentId},
    instruments::Waveform,
    InstrumentFactory, InstrumentTrait, MonoEffect, SynthCmd,
};
use engine::{Engine, InstrumentCmd};

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
    let tracking = TRACKING.try_with(Cell::get).unwrap_or(false);
    if tracking {
        let _ = counter.try_with(|count| count.set(count.get() + 1));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AllocationCounts {
    allocations: usize,
    deallocations: usize,
    reallocations: usize,
}

struct TrackingGuard;

impl Drop for TrackingGuard {
    fn drop(&mut self) {
        TRACKING.with(|tracking| tracking.set(false));
    }
}

fn measure_allocations(operation: impl FnOnce()) -> AllocationCounts {
    ALLOCATIONS.with(|count| count.set(0));
    DEALLOCATIONS.with(|count| count.set(0));
    REALLOCATIONS.with(|count| count.set(0));
    TRACKING.with(|tracking| tracking.set(true));
    let guard = TrackingGuard;

    operation();

    drop(guard);
    AllocationCounts {
        allocations: ALLOCATIONS.with(Cell::get),
        deallocations: DEALLOCATIONS.with(Cell::get),
        reallocations: REALLOCATIONS.with(Cell::get),
    }
}

#[test]
fn prepared_engine_note_parameter_and_render_path_has_no_heap_activity() {
    const SAMPLE_RATE: f32 = 48_000.0;
    let instrument_id = 1;
    let factory = InstrumentFactory::new(SAMPLE_RATE);
    let mut engine = Engine::new();
    engine.add_instrument(factory.create_simple_oscillator(instrument_id, 0.0));
    let mut left = [0.0; 256];
    let mut right = [0.0; 256];

    // Warm paths and any lazy process-global initialization before measuring.
    engine.note_on(instrument_id, 60, 127);
    engine.process(&mut left, &mut right, SAMPLE_RATE);
    engine.note_off(instrument_id);
    left.fill(0.0);
    right.fill(0.0);

    let counts = measure_allocations(|| {
        engine.handle_command(
            InstrumentCmd::NoteOn {
                instrument_id,
                note: 64,
                velocity: 100,
            }
            .into(),
        );
        engine.handle_command(
            InstrumentCmd::PassOnSynthCmd {
                instrument_id,
                synth_cmd: SynthCmd::SetWaveform {
                    voice_id: 0,
                    waveform: Waveform::Triangle,
                },
            }
            .into(),
        );
        engine.process(&mut left, &mut right, SAMPLE_RATE);
        engine.handle_command(InstrumentCmd::NoteOff { instrument_id }.into());
    });

    assert_eq!(counts.allocations, 0, "unexpected RT allocations");
    assert_eq!(counts.deallocations, 0, "unexpected RT deallocations");
    assert_eq!(counts.reallocations, 0, "unexpected RT reallocations");
    assert!(left.iter().any(|sample| *sample != 0.0));
}

struct IntentionallyAllocatingInstrument {
    id: InstrumentId,
}

impl InstrumentTrait for IntentionallyAllocatingInstrument {
    fn id(&self) -> InstrumentId {
        self.id
    }

    fn note_on(&mut self, _note: u8, _velocity: u8) {}

    fn note_off(&mut self) {}

    fn process(&mut self, _left: &mut [f32], _right: &mut [f32], _sample_rate: f32) {
        black_box(vec![0_u8; 128]);
    }

    fn set_pan(&mut self, _pan: f32) {}

    fn add_effect(&mut self, _effect: Box<dyn MonoEffect>) {}

    fn set_effect_parameter(&mut self, _effect_id: EffectId, _param_index: u32, _value: f32) {}

    fn try_handle_command(&mut self, _command: &SynthCmd) -> bool {
        false
    }
}

#[test]
fn audit_harness_detects_an_intentional_allocation_and_drop() {
    let mut engine = Engine::new();
    engine.add_instrument(Box::new(IntentionallyAllocatingInstrument { id: 1 }));
    let mut left = [0.0; 16];
    let mut right = [0.0; 16];

    let counts = measure_allocations(|| engine.process(&mut left, &mut right, 48_000.0));

    assert!(
        counts.allocations > 0,
        "allocation fixture was not detected"
    );
    assert!(
        counts.deallocations > 0,
        "deallocation fixture was not detected"
    );
}
