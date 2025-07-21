/// The standard factor for calculating tick duration from BPM, in seconds.
/// The original formula is 2500 milliseconds / BPM.
const BPM_TO_TICK_DURATION_SECONDS_FACTOR: f64 = 2.5;

// 125 BPM and TPM of 6 combination recreates the Amiga's 50Hz timing.
const INITIAL_BPM: f64 = 125.0;
const INITIAL_TPL: u32 = 6;

/// Manages all timing-related state for the sequencer.
pub struct TimingState {
    sample_rate: f64,
    bpm: f64,
    tpl: u32, // Ticks Per Line

    // Internal state for the audio callback
    tick_duration_samples: f64, // milliseconds per tick in samples
    samples_until_next_tick: f64,
}

impl TimingState {
    pub fn new(sample_rate: f64) -> Self {
        Self::new_with_bpm_tpl(sample_rate, INITIAL_BPM, INITIAL_TPL)
    }

    pub fn new_with_bpm_tpl(sample_rate: f64, initial_bpm: f64, initial_tpl: u32) -> Self {
        let mut state = TimingState {
            sample_rate,
            bpm: 0.0, // Set in set_bpm
            tpl: initial_tpl,
            tick_duration_samples: 0.0, // Set in set_bpm
            samples_until_next_tick: 0.0,
        };
        state.set_bpm(initial_bpm);
        state
    }

    /// Called by the audio thread to determine how many ticks to process for a given buffer size.
    pub fn advance(&mut self, num_samples: usize) -> u32 {
        self.samples_until_next_tick -= num_samples as f64;
        let mut ticks_to_process = 0;
        while self.samples_until_next_tick <= 0.0 {
            ticks_to_process += 1;
            self.samples_until_next_tick += self.tick_duration_samples;
        }
        ticks_to_process
    }

    /// Sets a new BPM and recalculates the tick duration in samples.
    pub fn set_bpm(&mut self, new_bpm: f64) {
        self.bpm = new_bpm;
        let tick_duration_sec = BPM_TO_TICK_DURATION_SECONDS_FACTOR / self.bpm;
        self.tick_duration_samples = tick_duration_sec * self.sample_rate;
        self.samples_until_next_tick = self.tick_duration_samples;
    }

    /// Sets a new TPL.
    pub fn set_tpl(&mut self, new_tpl: u32) {
        self.tpl = new_tpl;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timing_state_default_values() {
        let sample_rate = 44100.0;
        let timing_state = TimingState::new(sample_rate);
        assert_eq!(timing_state.bpm, INITIAL_BPM);
        assert_eq!(timing_state.tpl, INITIAL_TPL);
    }

    #[test]
    fn test_timing_state_with_custom_values() {
        let sample_rate = 48000.0;
        let bpm = 120.0;
        let tpl = 4;
        let timing_state = TimingState::new_with_bpm_tpl(sample_rate, bpm, tpl);
        assert_eq!(timing_state.bpm, bpm);
        assert_eq!(timing_state.tpl, tpl);
        assert!(timing_state.tick_duration_samples > 0.0);
    }

    #[test]
    fn test_advance_tick() {
        let sample_rate = 48000.0;
        let tick_duration_in_samples =
            sample_rate * (BPM_TO_TICK_DURATION_SECONDS_FACTOR / INITIAL_BPM); // Should be 960.0

        // Assuming initial_tpl of 6, though it doesn't affect this test
        let mut timing_state = TimingState::new(sample_rate);

        // 1 fewer sample than a full tick duration should yield ZERO ticks.
        let ticks = timing_state.advance((tick_duration_in_samples - 1.0) as usize); // advance(959)
        assert_eq!(ticks, 0);

        // Advancing by the final sample should yield exactly ONE tick.
        let ticks = timing_state.advance(1);
        assert_eq!(ticks, 1);

        // Advancing by exactly two ticks worth of samples should yield TWO ticks.
        let ticks = timing_state.advance((tick_duration_in_samples * 2.0) as usize); // advance(1920)
        assert_eq!(ticks, 2);
    }

    #[test]
    fn test_advance_multiple_ticks() {
        let sample_rate = 48000.0;
        let initial_bpm = 125.0;
        let mut timing_state = TimingState::new(sample_rate);
        let tick_duration_in_samples = 960.0;

        // Advance by 2.5 ticks worth of samples, should process 2 ticks
        let ticks = timing_state.advance((tick_duration_in_samples * 2.5) as usize); // 2400 samples
        assert_eq!(ticks, 2);

        // The remainder should be 0.5 ticks (480 samples). Advancing by another 480 should trigger the 3rd tick.
        let ticks = timing_state.advance(480);
        assert_eq!(ticks, 1);
    }

    #[test]
    fn test_bpm_change_resets_counter() {
        let sample_rate = 48000.0;
        let mut timing_state = TimingState::new(sample_rate); // 960 samples/tick

        // Advance halfway through a tick
        let ticks = timing_state.advance(480);
        assert_eq!(ticks, 0); // No tick should have passed yet

        // Now, double the tempo. This should reset the tick counter.
        timing_state.set_bpm(250.0); // New duration is 480 samples/tick

        // Advancing by 479 samples should still not trigger a tick
        let ticks = timing_state.advance(479);
        assert_eq!(ticks, 0);

        // The final sample should trigger the tick
        let ticks = timing_state.advance(1);
        assert_eq!(ticks, 1);
    }

    #[test]
    fn test_small_advances_accumulate() {
        let sample_rate = 48000.0;
        let mut timing_state = TimingState::new(sample_rate); // 960 samples/tick

        // Advance by 100 samples 9 times. No ticks should pass.
        for _ in 0..9 {
            let ticks = timing_state.advance(100);
            assert_eq!(ticks, 0);
        }

        // The 10th advance of 100 samples (totaling 1000) should trigger the first tick.
        let ticks = timing_state.advance(100);
        assert_eq!(ticks, 1);
    }
}
