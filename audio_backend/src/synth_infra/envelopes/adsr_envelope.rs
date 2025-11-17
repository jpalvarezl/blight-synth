#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnvelopeState {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Debug, Clone)]
pub struct Envelope {
    state: EnvelopeState,
    sample_rate: f32,
    output: f32,
    target: f32,
    coefficient: f32,
    attack_coef: f32,
    decay_coef: f32,
    release_coef: f32,
    sustain_level: f32,
}

impl Envelope {
    pub fn new(sample_rate: f32) -> Self {
        // Default values
        let mut env = Self {
            state: EnvelopeState::Idle,
            sample_rate,
            output: 0.0,
            target: 0.0,
            coefficient: 0.0,
            attack_coef: 0.0,
            decay_coef: 0.0,
            release_coef: 0.0,
            sustain_level: 1.0,
        };
        env.set_parameters(0.1, 0.1, 0.8, 0.5); // Default ADSR values
        env
    }

    pub fn new_adsr(
        sample_rate: f32,
        attack_s: f32,
        decay_s: f32,
        sustain: f32,
        release_s: f32,
    ) -> Self {
        let mut env = Self {
            state: EnvelopeState::Idle,
            sample_rate,
            output: 0.0,
            target: 0.0,
            coefficient: 0.0,
            attack_coef: 0.0,
            decay_coef: 0.0,
            release_coef: 0.0,
            sustain_level: 1.0,
        };
        env.set_parameters(attack_s, decay_s, sustain, release_s);
        env
    }

    pub fn set_parameters(&mut self, attack_s: f32, decay_s: f32, sustain: f32, release_s: f32) {
        // Pre-calculate coefficients to avoid expensive math in the audio loop.
        // This formula creates an exponential curve.

        self.attack_coef = Self::time_to_coef(attack_s, self.sample_rate);
        self.decay_coef = Self::time_to_coef(decay_s, self.sample_rate);
        self.release_coef = Self::time_to_coef(release_s, self.sample_rate);
        self.sustain_level = sustain.clamp(0.0, 1.0);
    }

    pub fn gate(&mut self, is_on: bool) {
        if is_on {
            self.state = EnvelopeState::Attack;
            self.target = 1.0;
            self.coefficient = self.attack_coef;
        } else {
            self.state = EnvelopeState::Release;
            self.target = 0.0;
            self.coefficient = self.release_coef;
        }
    }

    pub fn process(&mut self) -> f32 {
        // State transition logic (infrequent branches)
        match self.state {
            EnvelopeState::Attack if self.output >= 0.999 => {
                self.state = EnvelopeState::Decay;
                self.target = self.sustain_level;
                self.coefficient = self.decay_coef;
            }
            EnvelopeState::Decay if self.output <= self.sustain_level + 0.001 => {
                self.state = EnvelopeState::Sustain;
                self.target = self.sustain_level;
                self.coefficient = 0.0; // Stay at sustain level
            }
            EnvelopeState::Release if self.output <= 0.001 => {
                self.state = EnvelopeState::Idle;
                self.coefficient = 0.0;
            }
            _ => {}
        }

        // The core branchless calculation
        self.output = self.target + self.coefficient * (self.output - self.target);

        if self.state == EnvelopeState::Idle {
            0.0
        } else {
            self.output
        }
    }

    /// Returns `true` if the envelope is currently in any state other than `Idle`.
    /// This is the primary indicator of whether a voice should be kept alive.
    pub fn is_active(&self) -> bool {
        self.state != EnvelopeState::Idle
    }

    /// Returns the current state of the envelope. Useful for diagnostics and tests.
    pub fn state(&self) -> EnvelopeState {
        self.state
    }

    pub fn set_attack(&mut self, attack_s: f32) {
        self.attack_coef = Self::time_to_coef(attack_s, self.sample_rate);
        if self.state == EnvelopeState::Attack {
            self.coefficient = self.attack_coef;
        }
    }

    pub fn set_decay(&mut self, decay_s: f32) {
        self.decay_coef = Self::time_to_coef(decay_s, self.sample_rate);
        if self.state == EnvelopeState::Decay {
            self.coefficient = self.decay_coef;
        }
    }

    pub fn set_sustain(&mut self, sustain: f32) {
        self.sustain_level = sustain.clamp(0.0, 1.0);
        if self.state == EnvelopeState::Decay || self.state == EnvelopeState::Sustain {
            self.target = self.sustain_level;
        }
    }

    pub fn set_release(&mut self, release_s: f32) {
        self.release_coef = Self::time_to_coef(release_s, self.sample_rate);
        if self.state == EnvelopeState::Release {
            self.coefficient = self.release_coef;
        }
    }

    fn time_to_coef(time_s: f32, sample_rate: f32) -> f32 {
        if time_s <= 0.0 {
            return 0.0;
        }

        let samples = (time_s * sample_rate).max(1.0);
        const TARGET_RATIO: f32 = 0.001; // match Idle cutoff in `process`

        TARGET_RATIO.powf(1.0 / samples)
    }
}

#[cfg(test)]
mod tests {
    use super::{Envelope, EnvelopeState};

    const SAMPLE_RATE: f32 = 48_000.0;
    const SUSTAIN_STABILITY_TOLERANCE: f32 = 0.02; // ≈ -34 dB window around sustain level

    #[test]
    fn envelope_reaches_sustain_and_finishes_in_three_seconds() {
        let mut env = Envelope::new_adsr(SAMPLE_RATE, 1.0, 1.0, 0.5, 1.0);

        env.gate(true);

        let sustain_sample_idx = advance_until_state(
            &mut env,
            EnvelopeState::Sustain,
            (SAMPLE_RATE as usize * 2) + 10,
        );
        assert!(
            sustain_sample_idx as f32 / SAMPLE_RATE <= 2.05,
            "Sustain took longer than expected"
        );

        env.gate(false);
        let release_samples = advance_until_state(
            &mut env,
            EnvelopeState::Idle,
            SAMPLE_RATE as usize * 2,
        );

        assert!(
            release_samples <= SAMPLE_RATE as usize,
            "Release exceeded 1 second (actual {:.2}s)",
            release_samples as f32 / SAMPLE_RATE
        );

        let total_duration_seconds = (sustain_sample_idx + release_samples) as f32 / SAMPLE_RATE;
        assert!(
            total_duration_seconds <= 3.05,
            "Envelope exceeded expected 3 second completion window"
        );

        assert!(
            !env.is_active(),
            "Envelope should be idle at the end of the release phase"
        );
    }

    #[test]
    fn envelope_fast_release_matches_requested_duration() {
        let mut env = Envelope::new_adsr(SAMPLE_RATE, 0.01, 0.01, 0.2, 0.01);

        env.gate(true);

        advance_until_state(
            &mut env,
            EnvelopeState::Sustain,
            (SAMPLE_RATE as usize / 5).max(1),
        );

        env.gate(false);
        let release_samples = advance_until_state(
            &mut env,
            EnvelopeState::Idle,
            (SAMPLE_RATE * 0.05) as usize,
        );

        let release_seconds = release_samples as f32 / SAMPLE_RATE;
        assert!(release_seconds <= 0.011, "Release took longer than requested 0.01s");
    }

    #[test]
    fn envelope_sustain_level_stays_stable_over_time() {
        let sustain_level = 0.7;
        let mut env = Envelope::new_adsr(SAMPLE_RATE, 0.05, 0.05, sustain_level, 0.2);

        env.gate(true);

        advance_until_state(
            &mut env,
            EnvelopeState::Sustain,
            (SAMPLE_RATE as usize * 2).max(1),
        );

        advance_samples(&mut env, (SAMPLE_RATE as usize / 20).max(1));

        // Measure sustain stability over 0.5s
        let mut max_deviation = 0.0f32;
        for _ in 0..(SAMPLE_RATE as usize / 2) {
            let value = env.process();
            max_deviation = max_deviation.max((value - sustain_level).abs());
        }

        assert!(
            max_deviation <= SUSTAIN_STABILITY_TOLERANCE,
            "Sustain level drifted {:.4} beyond tolerance {:.4}",
            max_deviation,
            SUSTAIN_STABILITY_TOLERANCE
        );
        assert!(env.is_active(), "Envelope should remain active while sustaining");

        env.gate(false);
        advance_until_state(&mut env, EnvelopeState::Idle, (SAMPLE_RATE as usize * 2).max(1));
    }

    #[test]
    fn envelope_instant_attack_behaves_percussively() {
        let mut env = Envelope::new_adsr(SAMPLE_RATE, 0.0, 0.02, 0.0, 0.05);

        env.gate(true);

        // First process call should immediately jump to peak treated as percussive
        let first_sample = env.process();
        assert!(first_sample >= 0.99, "Instant attack should start near full amplitude");

        let decay_samples = advance_until_state(
            &mut env,
            EnvelopeState::Sustain,
            (SAMPLE_RATE * 0.05) as usize,
        );
        assert!(decay_samples > 0, "Decay should still take some samples even with zero sustain");
        assert_eq!(env.state(), EnvelopeState::Sustain);

        env.gate(false);
        advance_until_state(&mut env, EnvelopeState::Idle, (SAMPLE_RATE * 0.1) as usize);
    }

    #[test]
    fn envelope_retrigger_during_release_returns_to_attack() {
        let mut env = Envelope::new_adsr(SAMPLE_RATE, 0.05, 0.05, 0.3, 0.4);

        env.gate(true);
        advance_until_state(&mut env, EnvelopeState::Sustain, (SAMPLE_RATE as usize) * 2);

        env.gate(false);
        advance_samples(&mut env, (SAMPLE_RATE * 0.1) as usize);
        assert_eq!(env.state(), EnvelopeState::Release);
        let value_during_release = env.process();

        env.gate(true);
        assert_eq!(env.state(), EnvelopeState::Attack, "Retrigger should restart at attack");
        let mut attack_value = value_during_release;
        for _ in 0..32 {
            attack_value = env.process();
            if attack_value > value_during_release + 0.05 {
                break;
            }
        }
        assert!(
            attack_value > value_during_release + 0.05,
            "Retriggered attack should ramp upward quickly (release value {:.3}, attack {:.3})",
            value_during_release,
            attack_value
        );
    }

    #[test]
    fn envelope_dynamic_sustain_changes_immediately() {
        let mut env = Envelope::new_adsr(SAMPLE_RATE, 0.05, 0.05, 0.4, 0.2);

        env.gate(true);
        advance_until_state(&mut env, EnvelopeState::Sustain, (SAMPLE_RATE as usize) * 2);

        env.set_sustain(0.7);
        let after_increase = env.process();
        assert!(after_increase >= 0.65 && after_increase <= 0.75);

        env.set_sustain(0.2);
        let after_decrease = env.process();
        assert!(after_decrease <= 0.25);

        env.gate(false);
        advance_until_state(&mut env, EnvelopeState::Idle, (SAMPLE_RATE as usize) * 2);
    }

    #[test]
    fn envelope_long_release_respects_duration() {
        let release_s = 2.0;
        let mut env = Envelope::new_adsr(SAMPLE_RATE, 0.1, 0.1, 0.8, release_s);

        env.gate(true);
        advance_until_state(&mut env, EnvelopeState::Sustain, (SAMPLE_RATE as usize) * 2);

        env.gate(false);
        let release_samples = advance_until_state(
            &mut env,
            EnvelopeState::Idle,
            (SAMPLE_RATE * (release_s + 0.5)) as usize,
        );

        let release_duration = release_samples as f32 / SAMPLE_RATE;
        assert!(
            (release_duration - release_s).abs() < 0.1,
            "Release lasted {:.2}s but expected ~{:.2}s",
            release_duration,
            release_s
        );
    }

    #[test]
    fn envelope_zero_release_behaves_like_gate() {
        let mut env = Envelope::new_adsr(SAMPLE_RATE, 0.0, 0.0, 1.0, 0.0);

        env.gate(true);

        let sustain_samples = advance_until_state(
            &mut env,
            EnvelopeState::Sustain,
            8,
        );
        assert!(
            sustain_samples <= 4,
            "Zero attack/decay should reach sustain immediately (took {} samples)",
            sustain_samples
        );

        let sustain_value = env.process();
        assert!(sustain_value >= 0.99, "Sustain should hold full level without decay");

        env.gate(false);
        for _ in 0..4 {
            let value = env.process();
            if env.state() == EnvelopeState::Idle {
                assert!(value <= 0.001, "Zero release should drop signal immediately");
                return;
            }
        }
        panic!("Envelope failed to return to Idle with zero release");
    }

    #[test]
    fn envelope_durations_are_consistent_across_sample_rates() {
        let attack_s = 0.1;
        let decay_s = 0.2;
        let sustain = 0.6;
        let release_s = 0.4;
        let sample_rates = [44_100.0, 48_000.0, 96_000.0];

        let mut baseline: Option<(f32, f32, f32)> = None;

        for sample_rate in sample_rates {
            let mut env = Envelope::new_adsr(sample_rate, attack_s, decay_s, sustain, release_s);

            env.gate(true);
            let attack_samples = advance_until_state(
                &mut env,
                EnvelopeState::Decay,
                (sample_rate * (attack_s + 0.1)) as usize,
            );
            let decay_samples = advance_until_state(
                &mut env,
                EnvelopeState::Sustain,
                (sample_rate * (decay_s + 0.1)) as usize,
            );

            env.gate(false);
            let release_samples = advance_until_state(
                &mut env,
                EnvelopeState::Idle,
                (sample_rate * (release_s + 0.5)) as usize,
            );

            let attack_duration = attack_samples as f32 / sample_rate;
            let decay_duration = decay_samples as f32 / sample_rate;
            let release_duration = release_samples as f32 / sample_rate;

            if let Some((base_attack, base_decay, base_release)) = baseline {
                assert!(
                    (attack_duration - base_attack).abs() < 0.01,
                    "Attack duration drifted beyond tolerance ({} Hz)",
                    sample_rate
                );
                assert!(
                    (decay_duration - base_decay).abs() < 0.01,
                    "Decay duration drifted beyond tolerance ({} Hz)",
                    sample_rate
                );
                assert!(
                    (release_duration - base_release).abs() < 0.02,
                    "Release duration drifted beyond tolerance ({} Hz)",
                    sample_rate
                );
            } else {
                baseline = Some((attack_duration, decay_duration, release_duration));
            }
        }
    }

    fn advance_until_state(
        env: &mut Envelope,
        target: EnvelopeState,
        max_samples: usize,
    ) -> usize {
        for n in 0..max_samples.max(1) {
            env.process();
            if env.state() == target {
                return n + 1;
            }
        }
        panic!(
            "Envelope did not reach {:?} within {} samples (last state: {:?})",
            target,
            max_samples,
            env.state()
        );
    }

    fn advance_samples(env: &mut Envelope, count: usize) {
        for _ in 0..count {
            env.process();
        }
    }
}
