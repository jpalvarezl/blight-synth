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
    // Keep sample_rate if needed for more complex calculations later,
    // but rates are pre-calculated now.
    // sample_rate: f32,
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

        // Calculate rates based on time to reach target level (1.0 for attack, sustain_level for decay/release)
        // Rate = delta_level / (time_secs * sample_rate)
        // Handle zero times to avoid division by zero - use a very small time instead for near-instantaneous change
        let calculate_rate = |time_secs: f32, delta_level: f32| {
            if time_secs <= 0.0 {
                // Effectively infinite rate for instant change
                f32::INFINITY
            } else {
                delta_level / (time_secs * sample_rate)
            }
        };

        // Attack goes from 0 to 1
        let attack_rate = calculate_rate(attack_secs, 1.0);
        // Decay goes from 1 to sustain_level
        let decay_rate = calculate_rate(decay_secs, 1.0 - sustain_level);
        // Release goes from current level (assumed sustain_level for calculation) to 0
        let release_rate = calculate_rate(release_secs, sustain_level.max(MIN_LEVEL)); // Use sustain level for rate calc

        AdsrEnvelope {
            state: EnvelopeState::Idle,
            attack_rate,
            decay_rate,
            release_rate,
            sustain_level,
            current_level: 0.0,
            // sample_rate,
        }
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
                    // Use the rate calculated based on sustain level, but apply it proportionally
                    // Rate = delta_level / samples => delta_level = Rate * samples
                    // We want to decrease by release_rate (which assumes starting from sustain_level)
                    // A simple linear decrease:
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
        for i in 0..=attack_samples { // Use <= to ensure transition
            let current_output = adsr.process();
             if i > 0 { // Skip check on first sample where output is 0 but level might jump
                 assert!(current_output >= last_level - TOLERANCE, "Attack level decreased");
             }
            last_level = adsr.get_level(); // Check internal level after process
            if adsr.get_state() == EnvelopeState::Decay {
                break; // Transitioned
            }
        }
        assert_eq!(adsr.get_state(), EnvelopeState::Decay, "Should be in Decay state");
        assert!((adsr.get_level() - 1.0).abs() < TOLERANCE, "Level should be near 1.0 at end of attack");


        // Simulate time passing through decay phase
        let decay_samples = (decay_secs * SAMPLE_RATE).ceil() as usize;
        last_level = adsr.get_level();
         for i in 0..=decay_samples { // Use <= to ensure transition
            let current_output = adsr.process();
             if i > 0 && adsr.get_state() == EnvelopeState::Decay { // Only check decrease during decay
                 assert!(current_output <= last_level + TOLERANCE, "Decay level increased");
             }
            last_level = adsr.get_level();
            if adsr.get_state() == EnvelopeState::Sustain {
                break; // Transitioned
            }
        }
        assert_eq!(adsr.get_state(), EnvelopeState::Sustain, "Should be in Sustain state");
        assert!((adsr.get_level() - sustain_level).abs() < TOLERANCE, "Level should be near sustain_level");

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
            if adsr.get_state() == EnvelopeState::Sustain { break; }
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
        for i in 0..=release_samples + 1 { // Add extra sample to ensure transition if needed
             let current_output = adsr.process();
             if i > 0 && adsr.get_state() == EnvelopeState::Release {
                 assert!(current_output <= last_level + TOLERANCE, "Release level increased");
             }
            last_level = adsr.get_level();
            if !adsr.is_active() { // Check using is_active which means state is Idle
                break;
            }
        }

        assert_eq!(adsr.get_state(), EnvelopeState::Idle, "Should be in Idle state after release");
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
        assert!((adsr.get_level() - level_before_retrigger).abs() < TOLERANCE, "Level should not reset on retrigger from attack");

        // Let it decay a bit
         for _ in 0..((0.1 + 0.1) * SAMPLE_RATE) as usize { // Pass attack + decay time
            adsr.process();
             if adsr.get_state() == EnvelopeState::Sustain { break; }
        }
         assert_eq!(adsr.get_state(), EnvelopeState::Sustain);
         let level_in_sustain = adsr.get_level();

         // Retrigger from sustain
         adsr.trigger();
         assert_eq!(adsr.get_state(), EnvelopeState::Attack);
         assert!((adsr.get_level() - level_in_sustain).abs() < TOLERANCE, "Level should not reset on retrigger from sustain");


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
        assert!((adsr.get_level() - level_before_release).abs() < TOLERANCE, "Level should not change immediately on release call");

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
}

