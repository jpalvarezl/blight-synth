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
    }

    /// Sets a new TPL.
    pub fn set_tpl(&mut self, new_tpl: u32) {
        self.tpl = new_tpl;
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Mul;

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
    fn test_advance_tickk() {
        let sample_rate = 48000.0;
        let tick_duration_in_samples = sample_rate * (BPM_TO_TICK_DURATION_SECONDS_FACTOR / INITIAL_BPM); // 125 BPM

        let mut timing_state = TimingState::new(sample_rate);

        // 1 fewer sample than a full tick duration
        let ticks = timing_state.advance((tick_duration_in_samples - 1.0) as usize);
        assert_eq!(ticks, 1);

        // Advance the needed sample to complete the tick
        let ticks = timing_state.advance(1);
        assert_eq!(ticks, 1);

        // Advance 2 ticks worth of samples
        let ticks = timing_state.advance(tick_duration_in_samples.mul(2.0) as usize);
        assert_eq!(ticks, 2);
    }
}