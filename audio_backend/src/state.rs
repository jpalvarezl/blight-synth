use std::sync::{
    atomic::{AtomicBool, AtomicU32, Ordering},
    Arc,
};

/// Lock-free state shared between non-real-time control paths and the audio callback.
///
/// Values read from the audio callback must be atomics or otherwise real-time safe.
#[derive(Clone)]
pub struct SharedAudioState(Arc<Inner>);

struct Inner {
    master_gain_bits: AtomicU32,
    playing: AtomicBool,
}

impl Default for SharedAudioState {
    fn default() -> Self {
        Self::new()
    }
}

impl SharedAudioState {
    pub fn new() -> Self {
        Self(Arc::new(Inner {
            master_gain_bits: AtomicU32::new(1.0_f32.to_bits()),
            playing: AtomicBool::new(false),
        }))
    }

    /// Normalised master gain, currently clamped to 0.0..=1.0.
    pub fn master_gain(&self) -> f32 {
        f32::from_bits(self.0.master_gain_bits.load(Ordering::Relaxed))
    }

    pub fn set_master_gain(&self, value: f32) {
        let value = value.clamp(0.0, 1.0);
        self.0
            .master_gain_bits
            .store(value.to_bits(), Ordering::Relaxed);
    }

    pub fn is_playing(&self) -> bool {
        self.0.playing.load(Ordering::Relaxed)
    }

    pub fn set_playing(&self, playing: bool) {
        self.0.playing.store(playing, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::SharedAudioState;

    #[test]
    fn defaults_to_unity_gain_and_stopped_transport() {
        let state = SharedAudioState::new();

        assert_eq!(state.master_gain(), 1.0);
        assert!(!state.is_playing());
    }

    #[test]
    fn stores_master_gain() {
        let state = SharedAudioState::new();

        state.set_master_gain(0.5);

        assert_eq!(state.master_gain(), 0.5);
    }

    #[test]
    fn clamps_master_gain_to_normalized_range() {
        let state = SharedAudioState::new();

        state.set_master_gain(-1.0);
        assert_eq!(state.master_gain(), 0.0);

        state.set_master_gain(2.0);
        assert_eq!(state.master_gain(), 1.0);
    }

    #[test]
    fn stores_playing_state() {
        let state = SharedAudioState::new();

        state.set_playing(true);
        assert!(state.is_playing());

        state.set_playing(false);
        assert!(!state.is_playing());
    }
}
