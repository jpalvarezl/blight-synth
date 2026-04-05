use std::sync::{Arc, RwLock};

/// Shared state between the audio thread and OSC server.
/// All fields accessed from the audio callback must be lock-free (atomics).
#[derive(Clone)]
pub struct AppState(Arc<Inner>);

struct Inner {
    // TODO: use atomics for real-time-safe parameter access
    // e.g. gain: AtomicF32
    // e.g. filter_cutoff: AtomicF32
    // TODO: transport state (playing, bpm, position)
    // TODO: preset name / metadata
}

impl AppState {
    pub fn new() -> Self {
        Self(Arc::new(Inner {
            // TODO: initialise defaults
        }))
    }

    pub fn get_gain(&self) -> f32 {
        // TODO: return atomic read
        1.0
    }

    pub fn set_gain(&self, value: f32) {
        // TODO: atomic write, clamp to valid range
    }

    pub fn get_filter_cutoff(&self) -> f32 {
        // TODO: return atomic read
        1000.0
    }

    pub fn set_filter_cutoff(&self, value: f32) {
        // TODO: atomic write, clamp to valid range
    }

    pub fn is_playing(&self) -> bool {
        // TODO: atomic read
        false
    }

    pub fn set_playing(&self, playing: bool) {
        // TODO: atomic write
    }
}
