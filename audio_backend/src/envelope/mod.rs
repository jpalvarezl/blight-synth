const MIN_LEVEL: f32 = 0.0001; // Threshold to consider the envelope finished in release state

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum EnvelopeState {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Debug, Clone)]
pub struct AdsrEnvelope {
    state: EnvelopeState,
    attack_rate: f32,
    decay_rate: f32,
    release_rate: f32,
    sustain_level: f32,
    current_level: f32,
    sample_rate: f32,  // Added sample_rate back
    attack_secs: f32,  // Store original time parameter
    decay_secs: f32,   // Store original time parameter
    release_secs: f32, // Store original time parameter
}

impl AdsrEnvelope {
    /// Creates a new ADSR envelope generator.
    ///
    /// # Arguments
    ///
    /// * `sample_rate` - The audio sample rate in Hz.
    /// * `attack_secs` - The attack time in seconds.
    /// * `decay_secs` - The decay time in seconds.
    /// * `sustain_level` - The sustain level (0.0 to 1.0).
    /// * `release_secs` - The release time in seconds.
    pub fn new(
        sample_rate: f32,
        attack_secs: f32,
        decay_secs: f32,
        sustain_level: f32,
        release_secs: f32,
    ) -> Self {
        // Clamp sustain level
        let sustain_level = sustain_level.max(0.0).min(1.0);

        // Attack goes from 0 to 1
        let attack_rate = Self::calculate_rate_internal(attack_secs, 1.0, sample_rate);
        // Decay goes from 1 to sustain_level
        let decay_rate =
            Self::calculate_rate_internal(decay_secs, 1.0 - sustain_level, sample_rate);
        // Release rate is initially calculated based on sustain_level.
        // It will be recalculated in release() based on current_level at the time of release.
        let release_rate =
            Self::calculate_rate_internal(release_secs, sustain_level.max(MIN_LEVEL), sample_rate);

        AdsrEnvelope {
            state: EnvelopeState::Idle,
            attack_rate,
            decay_rate,
            release_rate,
            sustain_level,
            current_level: 0.0,
            sample_rate,  // Store sample_rate
            attack_secs,  // Store original time
            decay_secs,   // Store original time
            release_secs, // Store original time
        }
    }

    // Helper function to recalculate rates, used by individual setters
    fn calculate_rate_internal(time_secs: f32, delta_level: f32, sample_rate: f32) -> f32 {
        if time_secs <= 0.0 {
            f32::INFINITY
        } else {
            delta_level / (time_secs * sample_rate)
        }
    }

    pub fn set_attack(&mut self, attack_secs: f32) {
        self.attack_secs = attack_secs; // Store new time
        self.attack_rate = Self::calculate_rate_internal(self.attack_secs, 1.0, self.sample_rate);
    }

    pub fn set_decay(&mut self, decay_secs: f32) {
        self.decay_secs = decay_secs; // Store new time
                                      // Decay rate depends on the sustain level
        self.decay_rate = Self::calculate_rate_internal(
            self.decay_secs,
            1.0 - self.sustain_level,
            self.sample_rate,
        );
    }

    pub fn set_sustain(&mut self, sustain_level: f32) {
        self.sustain_level = sustain_level.max(0.0).min(1.0);
        // Recalculate decay_rate: the stored decay_secs now targets the new self.sustain_level.
        self.decay_rate = Self::calculate_rate_internal(
            self.decay_secs,
            1.0 - self.sustain_level,
            self.sample_rate,
        );
        // Recalculate a "default" release_rate based on the new sustain_level and stored release_secs.
        // The actual release() method will calculate the precise rate based on current_level when called.
        self.release_rate = Self::calculate_rate_internal(
            self.release_secs,
            self.sustain_level.max(MIN_LEVEL),
            self.sample_rate,
        );
    }

    pub fn set_release(&mut self, release_secs: f32) {
        self.release_secs = release_secs; // Store new time
                                          // Recalculate a "default" release_rate based on the current sustain_level and new release_secs.
                                          // The actual release() method will calculate the precise rate based on current_level when called.
        self.release_rate = Self::calculate_rate_internal(
            self.release_secs,
            self.sustain_level.max(MIN_LEVEL),
            self.sample_rate,
        );
    }

    /// Processes one sample of the envelope, updating its state and level.
    /// Returns the current envelope level *before* the update for this sample.
    pub fn process(&mut self) -> f32 {
        let output_level = self.current_level;

        match self.state {
            EnvelopeState::Idle => {
                // Do nothing, level stays at 0 (or near-zero after release)
            }
            EnvelopeState::Attack => {
                if self.attack_rate.is_infinite() {
                    self.current_level = 1.0; // Instant attack
                } else {
                    self.current_level += self.attack_rate;
                }

                if self.current_level >= 1.0 {
                    self.current_level = 1.0;
                    self.state = EnvelopeState::Decay;
                }
            }
            EnvelopeState::Decay => {
                if self.decay_rate.is_infinite() || self.sustain_level >= 1.0 {
                    self.current_level = self.sustain_level; // Instant decay or sustain is at max
                } else {
                    self.current_level -= self.decay_rate;
                }

                if self.current_level <= self.sustain_level {
                    self.current_level = self.sustain_level;
                    // Only transition to Sustain if sustain level is positive
                    if self.sustain_level > MIN_LEVEL {
                        self.state = EnvelopeState::Sustain;
                    } else {
                        // If sustain is zero, go directly to idle after decay?
                        // Or should it wait for release? Let's assume it waits.
                        // If sustain is effectively zero, decay finishes, stay here until release.
                        // Alternative: self.state = EnvelopeState::Idle;
                        self.state = EnvelopeState::Sustain; // Treat as sustain phase even at zero level
                    }
                }
            }
            EnvelopeState::Sustain => {
                // Level remains at sustain_level
                // State change happens on release() call
                self.current_level = self.sustain_level;
            }
            EnvelopeState::Release => {
                if self.release_rate.is_infinite() {
                    self.current_level = 0.0; // Instant release
                } else {
                    // self.release_rate is calculated in the release() method (or by setters as a default)
                    // to make the envelope reach 0 from the level at which release was triggered,
                    // over self.release_secs.
                    self.current_level -= self.release_rate;
                    // Alternative exponential decay (more natural):
                    // self.current_level *= self.release_multiplier; // requires precalculating multiplier
                }

                if self.current_level <= MIN_LEVEL {
                    self.current_level = 0.0; // Clamp to zero
                    self.state = EnvelopeState::Idle;
                }
            }
        }
        output_level
    }

    /// Triggers the envelope, starting the Attack phase.
    pub fn trigger(&mut self) {
        if self.state == EnvelopeState::Idle || self.state == EnvelopeState::Release {
            self.current_level = 0.0; // Reset level only if starting from idle/release
        }
        // Allow re-triggering from any state? Yes, common behavior.
        self.state = EnvelopeState::Attack;
    }

    /// Starts the Release phase of the envelope.
    pub fn release(&mut self) {
        // Can only release if not already idle
        if self.state != EnvelopeState::Idle {
            // Recalculate release_rate based on the current level and stored release_secs.
            // This ensures the release time is consistent regardless of the level at which release is triggered.
            // If current_level is already very low, rate will be small or zero, and process() will quickly go to Idle.
            self.release_rate = Self::calculate_rate_internal(
                self.release_secs,
                self.current_level.max(0.0),
                self.sample_rate,
            );
            self.state = EnvelopeState::Release;
        }
    }

    /// Returns `true` if the envelope is currently active (not Idle).
    pub fn is_active(&self) -> bool {
        self.state != EnvelopeState::Idle
    }

    /// Gets the current state of the envelope.
    pub fn get_state(&self) -> EnvelopeState {
        self.state
    }

    /// Gets the current output level of the envelope.
    pub fn get_level(&self) -> f32 {
        self.current_level
    }

    // --- Getters for testing/inspection ---
    pub fn get_attack_rate(&self) -> f32 {
        self.attack_rate
    }

    pub fn get_decay_rate(&self) -> f32 {
        self.decay_rate
    }

    pub fn get_sustain_level_param(&self) -> f32 {
        self.sustain_level
    }

    pub fn get_release_rate(&self) -> f32 {
        self.release_rate
    }

    // --- Getters for original time parameters (for testing/inspection) ---
    pub fn get_attack_secs(&self) -> f32 {
        self.attack_secs
    }

    pub fn get_decay_secs(&self) -> f32 {
        self.decay_secs
    }

    pub fn get_release_secs(&self) -> f32 {
        self.release_secs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RATE: f32 = 44100.0;
    const TOLERANCE: f32 = 0.001; // Tolerance for floating point comparisons

    #[test]
    fn test_adsr_creation_and_initial_state() {
        let adsr = AdsrEnvelope::new(SAMPLE_RATE, 0.1, 0.2, 0.5, 0.3);
        assert_eq!(adsr.get_state(), EnvelopeState::Idle);
        assert_eq!(adsr.get_level(), 0.0);
        assert!(!adsr.is_active());
        assert!(adsr.attack_rate > 0.0 && adsr.attack_rate.is_finite());
        assert!(adsr.decay_rate > 0.0 && adsr.decay_rate.is_finite());
        assert!(adsr.release_rate > 0.0 && adsr.release_rate.is_finite());
        assert_eq!(adsr.sustain_level, 0.5);
        assert_eq!(adsr.sample_rate, SAMPLE_RATE); // Test sample_rate storage
    }

    #[test]
    fn test_instant_attack() {
        let mut adsr = AdsrEnvelope::new(SAMPLE_RATE, 0.0, 0.1, 0.5, 0.1);
        adsr.trigger();
        assert_eq!(adsr.get_state(), EnvelopeState::Attack);
        let level = adsr.process(); // Process first sample
        assert_eq!(level, 0.0); // Returns level *before* update
        assert_eq!(adsr.get_level(), 1.0); // Level should be 1.0 immediately
        assert_eq!(adsr.get_state(), EnvelopeState::Decay); // Should transition to Decay
    }

    #[test]
    fn test_trigger_attack_decay_sustain() {
        let attack_secs = 0.1;
        let decay_secs = 0.2;
        let sustain_level = 0.5;
        let mut adsr = AdsrEnvelope::new(SAMPLE_RATE, attack_secs, decay_secs, sustain_level, 0.3);

        adsr.trigger();
        assert_eq!(adsr.get_state(), EnvelopeState::Attack);
        assert!(adsr.is_active());

        // Simulate time passing through attack phase
        let attack_samples = (attack_secs * SAMPLE_RATE).ceil() as usize;
        let mut last_level = 0.0;
        for i in 0..=attack_samples {
            // Use <= to ensure transition
            let current_output = adsr.process();
            if i > 0 {
                // Skip check on first sample where output is 0 but level might jump
                assert!(
                    current_output >= last_level - TOLERANCE,
                    "Attack level decreased"
                );
            }
            last_level = adsr.get_level(); // Check internal level after process
            if adsr.get_state() == EnvelopeState::Decay {
                break; // Transitioned
            }
        }
        assert_eq!(
            adsr.get_state(),
            EnvelopeState::Decay,
            "Should be in Decay state"
        );
        assert!(
            (adsr.get_level() - 1.0).abs() < TOLERANCE,
            "Level should be near 1.0 at end of attack"
        );

        // Simulate time passing through decay phase
        let decay_samples = (decay_secs * SAMPLE_RATE).ceil() as usize;
        last_level = adsr.get_level();
        for i in 0..=decay_samples {
            // Use <= to ensure transition
            let current_output = adsr.process();
            if i > 0 && adsr.get_state() == EnvelopeState::Decay {
                // Only check decrease during decay
                assert!(
                    current_output <= last_level + TOLERANCE,
                    "Decay level increased"
                );
            }
            last_level = adsr.get_level();
            if adsr.get_state() == EnvelopeState::Sustain {
                break; // Transitioned
            }
        }
        assert_eq!(
            adsr.get_state(),
            EnvelopeState::Sustain,
            "Should be in Sustain state"
        );
        assert!(
            (adsr.get_level() - sustain_level).abs() < TOLERANCE,
            "Level should be near sustain_level"
        );

        // Check sustain phase
        let sustain_output1 = adsr.process();
        let sustain_output2 = adsr.process();
        assert_eq!(adsr.get_state(), EnvelopeState::Sustain);
        assert!((sustain_output1 - sustain_level).abs() < TOLERANCE);
        assert!((sustain_output2 - sustain_level).abs() < TOLERANCE);
        assert!((adsr.get_level() - sustain_level).abs() < TOLERANCE);
    }

    #[test]
    fn test_release_and_idle() {
        let sustain_level = 0.6;
        let release_secs = 0.1;
        let mut adsr = AdsrEnvelope::new(SAMPLE_RATE, 0.01, 0.02, sustain_level, release_secs);

        // Get to sustain state quickly
        adsr.trigger();
        // Simulate enough time to reach sustain
        for _ in 0..((0.01 + 0.02) * SAMPLE_RATE).ceil() as usize + 10 {
            adsr.process();
            if adsr.get_state() == EnvelopeState::Sustain {
                break;
            }
        }
        assert_eq!(adsr.get_state(), EnvelopeState::Sustain);
        assert!((adsr.get_level() - sustain_level).abs() < TOLERANCE);

        // Release
        adsr.release();
        assert_eq!(adsr.get_state(), EnvelopeState::Release);
        assert!(adsr.is_active());

        // Simulate time passing through release phase
        let release_samples = (release_secs * SAMPLE_RATE).ceil() as usize;
        let mut last_level = adsr.get_level();
        for i in 0..=release_samples + 1 {
            // Add extra sample to ensure transition if needed
            let current_output = adsr.process();
            if i > 0 && adsr.get_state() == EnvelopeState::Release {
                assert!(
                    current_output <= last_level + TOLERANCE,
                    "Release level increased"
                );
            }
            last_level = adsr.get_level();
            if !adsr.is_active() {
                // Check using is_active which means state is Idle
                break;
            }
        }

        assert_eq!(
            adsr.get_state(),
            EnvelopeState::Idle,
            "Should be in Idle state after release"
        );
        assert!(!adsr.is_active());
        assert!(adsr.get_level() < MIN_LEVEL, "Level should be near zero"); // Check internal level is near zero
    }

    #[test]
    fn test_retrigger() {
        let mut adsr = AdsrEnvelope::new(SAMPLE_RATE, 0.1, 0.1, 0.5, 0.1);
        adsr.trigger();
        // Process part way through attack
        for _ in 0..((0.1 * SAMPLE_RATE) / 2.0) as usize {
            adsr.process();
        }
        let level_before_retrigger = adsr.get_level();
        assert!(level_before_retrigger > 0.0 && level_before_retrigger < 1.0);
        assert_eq!(adsr.get_state(), EnvelopeState::Attack);

        // Retrigger
        adsr.trigger();
        assert_eq!(adsr.get_state(), EnvelopeState::Attack);
        // Level should NOT reset to 0 when retriggering from attack/decay/sustain
        assert!(
            (adsr.get_level() - level_before_retrigger).abs() < TOLERANCE,
            "Level should not reset on retrigger from attack"
        );

        // Let it decay a bit
        for _ in 0..((0.1 + 0.1) * SAMPLE_RATE) as usize {
            // Pass attack + decay time
            adsr.process();
            if adsr.get_state() == EnvelopeState::Sustain {
                break;
            }
        }
        assert_eq!(adsr.get_state(), EnvelopeState::Sustain);
        let level_in_sustain = adsr.get_level();

        // Retrigger from sustain
        adsr.trigger();
        assert_eq!(adsr.get_state(), EnvelopeState::Attack);
        assert!(
            (adsr.get_level() - level_in_sustain).abs() < TOLERANCE,
            "Level should not reset on retrigger from sustain"
        );
    }

    #[test]
    fn test_release_from_attack() {
        let mut adsr = AdsrEnvelope::new(SAMPLE_RATE, 0.1, 0.1, 0.5, 0.1);
        adsr.trigger();
        // Process part way through attack
        for _ in 0..((0.1 * SAMPLE_RATE) / 3.0) as usize {
            adsr.process();
        }
        assert_eq!(adsr.get_state(), EnvelopeState::Attack);
        let level_before_release = adsr.get_level();
        assert!(level_before_release > 0.0 && level_before_release < 1.0);

        adsr.release();
        assert_eq!(adsr.get_state(), EnvelopeState::Release);
        assert!(
            (adsr.get_level() - level_before_release).abs() < TOLERANCE,
            "Level should not change immediately on release call"
        );

        // Process one step
        let output = adsr.process();
        assert!((output - level_before_release).abs() < TOLERANCE); // Output is level *before* update
        assert!(adsr.get_level() < level_before_release); // Level should decrease
    }

    #[test]
    fn test_zero_sustain() {
        let mut adsr = AdsrEnvelope::new(SAMPLE_RATE, 0.01, 0.02, 0.0, 0.03);
        adsr.trigger();

        // Process through attack and decay
        let total_samples = ((0.01 + 0.02) * SAMPLE_RATE).ceil() as usize + 5;
        for _ in 0..total_samples {
            adsr.process();
            // It should end up in Sustain state, even if level is 0
            if adsr.get_state() == EnvelopeState::Sustain {
                break;
            }
        }
        assert_eq!(adsr.get_state(), EnvelopeState::Sustain);
        assert!(adsr.get_level() < MIN_LEVEL); // Level should be near zero

        // Stay in sustain (at zero level)
        adsr.process();
        assert_eq!(adsr.get_state(), EnvelopeState::Sustain);
        assert!(adsr.get_level() < MIN_LEVEL);

        // Release should transition to release and then quickly to idle
        adsr.release();
        assert_eq!(adsr.get_state(), EnvelopeState::Release);
        adsr.process(); // Process one step in release
                        // Since level is already ~0, it should immediately go idle
        assert_eq!(adsr.get_state(), EnvelopeState::Idle);
        assert!(adsr.get_level() < MIN_LEVEL);
    }

    // --- Tests for individual ADSR parameter setters ---

    #[test]
    fn test_set_attack() {
        let mut adsr = AdsrEnvelope::new(SAMPLE_RATE, 0.1, 0.2, 0.5, 0.3);
        adsr.set_attack(0.05); // Faster attack
                               // Verify the rate changed (it should be roughly double)
                               // Original attack_rate for 0.1s: 1.0 / (0.1 * 44100) = 1.0 / 4410 = 0.0002267
                               // New attack_rate for 0.05s: 1.0 / (0.05 * 44100) = 1.0 / 2205 = 0.0004535
        assert!((adsr.attack_rate - 1.0 / (0.05 * SAMPLE_RATE)).abs() < TOLERANCE / SAMPLE_RATE);

        adsr.set_attack(0.0); // Instant attack
        assert!(adsr.attack_rate.is_infinite());
    }

    #[test]
    fn test_set_decay() {
        let mut adsr = AdsrEnvelope::new(SAMPLE_RATE, 0.1, 0.2, 0.5, 0.3);
        // Original decay_rate for 0.2s to reach 0.5 from 1.0 (delta 0.5):
        // (1.0 - 0.5) / (0.2 * 44100) = 0.5 / 8820 = 0.000056689
        adsr.set_decay(0.1); // Faster decay
                             // New decay_rate for 0.1s: (1.0 - 0.5) / (0.1 * 44100) = 0.5 / 4410 = 0.00011337
        assert!(
            (adsr.decay_rate - (1.0 - 0.5) / (0.1 * SAMPLE_RATE)).abs() < TOLERANCE / SAMPLE_RATE
        );

        adsr.set_decay(0.0); // Instant decay
        assert!(adsr.decay_rate.is_infinite());
    }

    #[test]
    fn test_set_sustain() {
        let mut adsr = AdsrEnvelope::new(SAMPLE_RATE, 0.1, 0.2, 0.5, 0.3);
        let original_decay_rate = adsr.decay_rate;

        adsr.set_sustain(0.8);
        assert_eq!(adsr.sustain_level, 0.8);
        // Decay rate should be re-calculated if we want to maintain the original decay *time*
        // to the *new* sustain. The current implementation of set_sustain does not re-calculate decay_rate.
        // It only updates sustain_level. This means the time it takes to decay will change if decay_rate is not updated.
        // For this test, we'll assert that decay_rate is NOT changed by set_sustain,
        // and then we'll test that set_decay correctly recalculates based on the new sustain.
        assert_eq!(adsr.decay_rate, original_decay_rate);

        // Now, if we call set_decay, it should use the new sustain_level (0.8)
        adsr.set_decay(0.1); // Decay time of 0.1s
                             // New decay_rate for 0.1s to reach 0.8 from 1.0 (delta 0.2):
                             // (1.0 - 0.8) / (0.1 * 44100) = 0.2 / 4410 = 0.00004535
        assert!(
            (adsr.decay_rate - (1.0 - 0.8) / (0.1 * SAMPLE_RATE)).abs() < TOLERANCE / SAMPLE_RATE
        );

        adsr.set_sustain(1.0);
        assert_eq!(adsr.sustain_level, 1.0);
        adsr.set_decay(0.1); // Decay time of 0.1s, delta is 0 (1.0 - 1.0)
                             // If sustain is 1.0, decay delta is 0. Rate should be 0 or effectively infinite if time is also 0.
                             // Our calculate_rate_internal gives delta / (time * sr). If delta is 0, rate is 0.
                             // This means it will instantly go to sustain level if sustain is 1.0 during decay phase.
        assert!(
            (adsr.decay_rate - (1.0 - 1.0) / (0.1 * SAMPLE_RATE)).abs() < TOLERANCE / SAMPLE_RATE
        );

        adsr.set_sustain(0.0);
        assert_eq!(adsr.sustain_level, 0.0);
        adsr.set_decay(0.1); // Decay time of 0.1s, delta is 1.0 (1.0 - 0.0)
                             // (1.0 - 0.0) / (0.1 * 44100) = 1.0 / 4410 = 0.00022675
        assert!(
            (adsr.decay_rate - (1.0 - 0.0) / (0.1 * SAMPLE_RATE)).abs() < TOLERANCE / SAMPLE_RATE
        );
    }

    #[test]
    fn test_set_release() {
        let mut adsr = AdsrEnvelope::new(SAMPLE_RATE, 0.1, 0.2, 0.5, 0.3);
        // Original release_rate for 0.3s from sustain 0.5:
        // 0.5 / (0.3 * 44100) = 0.5 / 13230 = 0.00003779
        adsr.set_release(0.15); // Faster release
                                // New release_rate for 0.15s: 0.5 / (0.15 * 44100) = 0.5 / 6615 = 0.00007558
        assert!(
            (adsr.release_rate - adsr.sustain_level / (0.15 * SAMPLE_RATE)).abs()
                < TOLERANCE / SAMPLE_RATE
        );

        adsr.set_release(0.0); // Instant release
        assert!(adsr.release_rate.is_infinite());

        // Test if release rate changes correctly if sustain level was changed
        adsr.set_sustain(0.8);
        adsr.set_release(0.1); // Release time 0.1s from new sustain 0.8
                               // New release_rate: 0.8 / (0.1 * 44100) = 0.8 / 4410 = 0.0001814
        assert!(
            (adsr.release_rate - adsr.sustain_level / (0.1 * SAMPLE_RATE)).abs()
                < TOLERANCE / SAMPLE_RATE
        );
    }

    #[test]
    fn test_adsr_full_cycle_with_setters() {
        let mut adsr = AdsrEnvelope::new(SAMPLE_RATE, 0.01, 0.02, 0.5, 0.03); // Initial quick values

        adsr.set_attack(0.005); // 5ms
        adsr.set_decay(0.01); // 10ms
        adsr.set_sustain(0.6);
        adsr.set_release(0.015); // 15ms

        assert_eq!(adsr.sustain_level, 0.6);

        adsr.trigger();
        assert_eq!(adsr.get_state(), EnvelopeState::Attack);

        // Process through attack (5ms = 0.005 * 44100 = 220.5 samples, round to 221)
        for _ in 0..((0.005 * SAMPLE_RATE).ceil() as usize) {
            adsr.process();
        }
        // Should be at or very near peak, transitioning to Decay
        assert!(
            adsr.get_state() == EnvelopeState::Decay || adsr.get_level() >= 1.0 - TOLERANCE,
            "State: {:?}, Level: {}",
            adsr.get_state(),
            adsr.get_level()
        );
        adsr.process(); // One more sample to ensure transition if exactly at peak
        assert_eq!(adsr.get_state(), EnvelopeState::Decay);
        assert!((adsr.get_level() - 1.0).abs() < TOLERANCE);

        // Process through decay (10ms = 0.01 * 44100 = 441 samples)
        for _ in 0..((0.01 * SAMPLE_RATE).ceil() as usize) {
            adsr.process();
        }
        // Should be at or very near sustain level, transitioning to Sustain
        assert!(
            adsr.get_state() == EnvelopeState::Sustain
                || (adsr.get_level() - adsr.sustain_level).abs() < TOLERANCE,
            "State: {:?}, Level: {}, Sustain: {}",
            adsr.get_state(),
            adsr.get_level(),
            adsr.sustain_level
        );
        adsr.process(); // One more sample
        assert_eq!(adsr.get_state(), EnvelopeState::Sustain);
        assert!((adsr.get_level() - adsr.sustain_level).abs() < TOLERANCE);

        // Stay in sustain for a bit
        for _ in 0..100 {
            adsr.process();
        }
        assert_eq!(adsr.get_state(), EnvelopeState::Sustain);
        assert!((adsr.get_level() - adsr.sustain_level).abs() < TOLERANCE);

        adsr.release();
        assert_eq!(adsr.get_state(), EnvelopeState::Release);

        // Process through release (15ms = 0.015 * 44100 = 661.5 samples, round to 662)
        for _ in 0..((0.015 * SAMPLE_RATE).ceil() as usize) {
            adsr.process();
        }
        // Should be at or very near zero, transitioning to Idle
        assert!(
            adsr.get_state() == EnvelopeState::Idle || adsr.get_level() < MIN_LEVEL + TOLERANCE,
            "State: {:?}, Level: {}",
            adsr.get_state(),
            adsr.get_level()
        );
        adsr.process(); // One more sample
        assert_eq!(adsr.get_state(), EnvelopeState::Idle);
        assert!(adsr.get_level() < MIN_LEVEL + TOLERANCE);
    }
}
