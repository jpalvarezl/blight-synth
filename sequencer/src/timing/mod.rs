//! Allocation-free tracker tick timing.
//!
//! A tick has an exact fixed-point phase and is exposed at the first integer
//! render frame at or after that phase. Render slices are half-open: a boundary
//! equal to `frame_count` is retained and appears at offset zero of the next
//! non-empty slice. Ticks-per-line (TPL) is deliberately not part of this
//! clock; TPL advances tracker rows after emitted ticks and does not change tick
//! spacing.

/// The standard factor for calculating tick duration from BPM, in seconds.
/// The original formula is 2500 milliseconds / BPM.
const BPM_TO_TICK_DURATION_SECONDS_FACTOR: f64 = 2.5;
const INITIAL_BPM: f64 = 125.0;
const FIXED_FRACTION_BITS: u32 = 64;
const ONE_FRAME: u128 = 1_u128 << FIXED_FRACTION_BITS;
const FRACTION_MASK: u128 = ONE_FRAME - 1;
const MAX_TICK_INTERVAL_FRAMES: f64 = u32::MAX as f64;

/// Default callback work bound used by the compatibility constructors.
///
/// This covers every tick possible in the project's prepared 4096-frame render
/// slice because valid intervals are at least one frame. Callers with another
/// maximum slice size should use [`TimingState::prepare`] and provide a bound
/// of at least one tick per frame.
pub const DEFAULT_MAX_TICKS_PER_SLICE: u32 = 4_096;

/// A timing configuration rejected during non-real-time preparation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimingError {
    InvalidSampleRate,
    InvalidBpm,
    TickIntervalOutOfRange,
    ZeroTickCapacity,
    PositionOverflow,
}

/// A tick boundary relative to the exact frame slice being advanced.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TickBoundary {
    /// Offset in the half-open range `0..frame_count`.
    pub sample_offset: usize,
}

/// A tempo directive used while planning an emitted tick.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TickTempo {
    /// Keep the currently prepared interval.
    Unchanged,
    /// Use this BPM for the interval immediately after this tick.
    SetBpm(f64),
}

/// Compact outcome of one bounded timing advance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TimingAdvanceStatus {
    Complete,
    /// More tick boundaries existed in the slice than the prepared work or
    /// caller-provided output capacity.
    TickCapacityExceeded,
    /// Preparation or a tempo directive supplied an invalid value.
    InvalidConfiguration,
    /// The absolute frame or fixed-point tick phase was exhausted.
    PositionOverflow,
}

/// Result of advancing one exact frame slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimingAdvance {
    /// Number of valid entries written to the caller's output on success.
    /// This is always zero when `status` is not `Complete`.
    pub ticks_emitted: u32,
    pub status: TimingAdvanceStatus,
}

/// Prepared, allocation-free tracker tick clock.
///
/// Tick phases use unsigned Q64.64 fixed point. Adding the same prepared
/// interval once per tick makes absolute boundaries independent of render-slice
/// partitioning while retaining sub-frame phase. A BPM directive planned for a
/// tick adds the new interval to that tick's exact phase, not its rounded
/// sample offset, so an already emitted boundary never moves.
#[derive(Debug, Clone)]
pub struct TimingState {
    sample_rate: f64,
    bpm: f64,
    tick_interval: u128,
    next_tick_phase: u128,
    frame_position: u64,
    max_ticks_per_slice: u32,
    configuration_valid: bool,
    fault: Option<TimingAdvanceStatus>,
}

impl TimingState {
    /// Compatibility constructor with the default BPM and work bound.
    ///
    /// Prefer [`Self::prepare`] when invalid input must be reported before the
    /// callback starts. Invalid compatibility input creates a fail-closed clock
    /// whose status is [`TimingAdvanceStatus::InvalidConfiguration`].
    pub fn new(sample_rate: f64) -> Self {
        Self::new_with_bpm(sample_rate, INITIAL_BPM)
    }

    /// Compatibility constructor retained until tracker integration issue #204.
    ///
    /// `initial_tpl` is intentionally ignored: TPL is row-progression state,
    /// not a tick-spacing input. New callers should use [`Self::prepare`].
    pub fn new_with_bpm_tpl(sample_rate: f64, initial_bpm: f64, _initial_tpl: u32) -> Self {
        Self::new_with_bpm(sample_rate, initial_bpm)
    }

    /// Compatibility constructor with the default 4096-tick work bound.
    pub fn new_with_bpm(sample_rate: f64, initial_bpm: f64) -> Self {
        Self::prepare(sample_rate, initial_bpm, DEFAULT_MAX_TICKS_PER_SLICE)
            .unwrap_or_else(|_| Self::invalid(sample_rate, initial_bpm))
    }

    /// Prepares a valid clock and its hard per-slice callback work bound.
    pub fn prepare(
        sample_rate: f64,
        initial_bpm: f64,
        max_ticks_per_slice: u32,
    ) -> Result<Self, TimingError> {
        if max_ticks_per_slice == 0 {
            return Err(TimingError::ZeroTickCapacity);
        }

        let tick_interval = prepare_tick_interval(sample_rate, initial_bpm)?;
        Ok(Self {
            sample_rate,
            bpm: initial_bpm,
            tick_interval,
            next_tick_phase: tick_interval,
            frame_position: 0,
            max_ticks_per_slice,
            configuration_valid: true,
            fault: None,
        })
    }

    /// Plans one half-open frame slice into caller-owned prepared storage.
    ///
    /// `plan_tempo` is a planning callback only: it may inspect immutable
    /// producer/document state to select the interval after a tick, but it must
    /// not commit producer side effects. Timing is staged locally and committed
    /// only after the complete slice, every tempo directive, and every output
    /// entry have been validated. On a non-[`TimingAdvanceStatus::Complete`]
    /// result, `ticks_emitted` is zero and the caller must ignore all output.
    /// This lets the caller apply producer mutations only from a complete
    /// result, so invalid tempo or capacity cannot leave producer state partly
    /// advanced.
    ///
    /// The planner and timing path allocate and deallocate nothing. Both work
    /// and output are bounded by the lesser of the prepared tick capacity and
    /// `output.len()`.
    pub fn advance_ticks(
        &mut self,
        frame_count: usize,
        output: &mut [TickBoundary],
        plan_tempo: impl FnMut(TickBoundary) -> TickTempo,
    ) -> TimingAdvance {
        let output_capacity = output.len().min(self.max_ticks_per_slice as usize);
        self.advance_transaction(
            frame_count,
            false,
            output_capacity,
            plan_tempo,
            |index, boundary| output[index] = boundary,
        )
    }

    /// Bounded count-only compatibility shim for the current tracker player.
    ///
    /// This returns the full status instead of silently discarding it. It keeps
    /// the historical end-inclusive count behavior so existing whole-block
    /// playback references do not change before #204; do not mix it with
    /// [`Self::advance_ticks`]. The compatibility caller must handle every
    /// non-complete status before applying the returned count.
    pub fn advance(&mut self, frame_count: usize) -> TimingAdvance {
        self.advance_transaction(
            frame_count,
            true,
            self.max_ticks_per_slice as usize,
            |_| TickTempo::Unchanged,
            |_, _| {},
        )
    }

    fn advance_transaction(
        &mut self,
        frame_count: usize,
        include_block_end: bool,
        capacity: usize,
        mut plan_tempo: impl FnMut(TickBoundary) -> TickTempo,
        mut write_boundary: impl FnMut(usize, TickBoundary),
    ) -> TimingAdvance {
        if let Some(status) = self.fault {
            return TimingAdvance {
                ticks_emitted: 0,
                status,
            };
        }

        let mut staged = self.clone();
        match staged.plan_slice(
            frame_count,
            include_block_end,
            capacity,
            &mut plan_tempo,
            &mut write_boundary,
        ) {
            Ok(ticks_emitted) => {
                *self = staged;
                TimingAdvance {
                    ticks_emitted,
                    status: TimingAdvanceStatus::Complete,
                }
            }
            Err(status) => {
                // A representable rejected slice still consumes its host frames.
                // Recovery reanchors from that boundary rather than replaying time.
                self.frame_position = staged.frame_position;
                self.fault = Some(status);
                TimingAdvance {
                    ticks_emitted: 0,
                    status,
                }
            }
        }
    }

    fn plan_slice(
        &mut self,
        frame_count: usize,
        include_block_end: bool,
        capacity: usize,
        plan_tempo: &mut impl FnMut(TickBoundary) -> TickTempo,
        write_boundary: &mut impl FnMut(usize, TickBoundary),
    ) -> Result<u32, TimingAdvanceStatus> {
        let frame_count =
            u64::try_from(frame_count).map_err(|_| TimingAdvanceStatus::PositionOverflow)?;
        let block_start = self.frame_position;
        let block_end = block_start
            .checked_add(frame_count)
            .ok_or(TimingAdvanceStatus::PositionOverflow)?;
        // Even a rejected slice consumes the host-provided frame interval. The
        // transaction commits no ticks/tempo, but recovery must reanchor after
        // this slice rather than replaying its elapsed time.
        self.frame_position = block_end;

        if frame_count == 0 {
            return Ok(0);
        }

        let hard_capacity = capacity.min(self.max_ticks_per_slice as usize);
        let mut ticks_emitted = 0_usize;
        loop {
            let boundary_frame = fixed_phase_ceiling(self.next_tick_phase)
                .ok_or(TimingAdvanceStatus::PositionOverflow)?;

            if boundary_frame > block_end || (!include_block_end && boundary_frame == block_end) {
                self.frame_position = block_end;
                return u32::try_from(ticks_emitted)
                    .map_err(|_| TimingAdvanceStatus::TickCapacityExceeded);
            }
            if boundary_frame < block_start {
                self.frame_position = block_end;
                return Err(TimingAdvanceStatus::PositionOverflow);
            }
            if ticks_emitted == hard_capacity {
                self.frame_position = block_end;
                return Err(TimingAdvanceStatus::TickCapacityExceeded);
            }

            let boundary = TickBoundary {
                sample_offset: usize::try_from(boundary_frame - block_start)
                    .map_err(|_| TimingAdvanceStatus::PositionOverflow)?,
            };
            let (bpm, interval) = match plan_tempo(boundary) {
                TickTempo::Unchanged => (self.bpm, self.tick_interval),
                TickTempo::SetBpm(new_bpm) => (
                    new_bpm,
                    prepare_tick_interval(self.sample_rate, new_bpm)
                        .map_err(|_| TimingAdvanceStatus::InvalidConfiguration)?,
                ),
            };
            let next_tick_phase = self
                .next_tick_phase
                .checked_add(interval)
                .ok_or(TimingAdvanceStatus::PositionOverflow)?;

            write_boundary(ticks_emitted, boundary);
            ticks_emitted += 1;
            self.bpm = bpm;
            self.tick_interval = interval;
            self.next_tick_phase = next_tick_phase;
        }
    }

    /// Latches a BPM for intervals scheduled after the next emitted tick.
    ///
    /// This never moves the already scheduled next boundary. An invalid value
    /// is rejected without changing or faulting an otherwise valid clock. If an
    /// invalid tempo directive previously faulted the clock, a valid value
    /// reanchors a full interval at the current host-frame boundary and resumes
    /// it. To change tempo while planning a particular tick inside a slice, use
    /// [`TickTempo::SetBpm`] in [`Self::advance_ticks`].
    pub fn set_bpm(&mut self, new_bpm: f64) -> Result<(), TimingError> {
        let interval = prepare_tick_interval(self.sample_rate, new_bpm)?;
        let recovering_invalid_configuration = !self.configuration_valid
            || self.fault == Some(TimingAdvanceStatus::InvalidConfiguration);

        self.bpm = new_bpm;
        self.tick_interval = interval;
        self.configuration_valid = true;
        if recovering_invalid_configuration {
            self.reanchor_at_current_frame()?;
            self.fault = None;
        }
        Ok(())
    }

    /// Returns the currently prepared BPM.
    pub fn bpm(&self) -> f64 {
        self.bpm
    }

    /// Returns the prepared maximum number of ticks per frame slice.
    pub fn max_ticks_per_slice(&self) -> u32 {
        self.max_ticks_per_slice
    }

    /// Returns the current sticky callback status.
    pub fn status(&self) -> TimingAdvanceStatus {
        self.fault.unwrap_or(TimingAdvanceStatus::Complete)
    }

    /// Starts a new transport epoch with the next tick one full interval away.
    ///
    /// Reset discards prior absolute frame position, clears capacity/position or
    /// directive faults, and makes position-overflow recovery reachable. It
    /// cannot repair an invalid sample rate or initial BPM; prepare a valid BPM
    /// first with [`Self::set_bpm`] (an invalid sample rate requires a newly
    /// prepared clock).
    pub fn reset(&mut self) -> Result<(), TimingError> {
        let interval = prepare_tick_interval(self.sample_rate, self.bpm)?;
        self.tick_interval = interval;
        self.frame_position = 0;
        self.next_tick_phase = interval;
        self.configuration_valid = true;
        self.fault = None;
        Ok(())
    }

    fn reanchor_at_current_frame(&mut self) -> Result<(), TimingError> {
        let current_phase = u128::from(self.frame_position) << FIXED_FRACTION_BITS;
        self.next_tick_phase = current_phase
            .checked_add(self.tick_interval)
            .ok_or(TimingError::PositionOverflow)?;
        Ok(())
    }

    fn invalid(sample_rate: f64, bpm: f64) -> Self {
        Self {
            sample_rate,
            bpm,
            tick_interval: ONE_FRAME,
            next_tick_phase: ONE_FRAME,
            frame_position: 0,
            max_ticks_per_slice: DEFAULT_MAX_TICKS_PER_SLICE,
            configuration_valid: false,
            fault: Some(TimingAdvanceStatus::InvalidConfiguration),
        }
    }
}

fn prepare_tick_interval(sample_rate: f64, bpm: f64) -> Result<u128, TimingError> {
    if !sample_rate.is_finite() || sample_rate <= 0.0 {
        return Err(TimingError::InvalidSampleRate);
    }
    if !bpm.is_finite() || bpm <= 0.0 {
        return Err(TimingError::InvalidBpm);
    }

    let interval_frames = BPM_TO_TICK_DURATION_SECONDS_FACTOR * sample_rate / bpm;
    if !interval_frames.is_finite() || !(1.0..=MAX_TICK_INTERVAL_FRAMES).contains(&interval_frames)
    {
        return Err(TimingError::TickIntervalOutOfRange);
    }

    let whole_frames = interval_frames.floor() as u128;
    let fractional_frames = interval_frames - whole_frames as f64;
    let fractional_phase = (fractional_frames * ONE_FRAME as f64).round() as u128;
    let interval = (whole_frames << FIXED_FRACTION_BITS)
        .checked_add(fractional_phase)
        .ok_or(TimingError::TickIntervalOutOfRange)?;

    if interval < ONE_FRAME {
        return Err(TimingError::TickIntervalOutOfRange);
    }
    Ok(interval)
}

fn fixed_phase_ceiling(phase: u128) -> Option<u64> {
    let whole_frames = phase >> FIXED_FRACTION_BITS;
    let rounded_frames = whole_frames.checked_add(u128::from(phase & FRACTION_MASK != 0))?;
    u64::try_from(rounded_frames).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn absolute_ticks(partitions: &[usize], sample_rate: f64, bpm: f64) -> Vec<u64> {
        let mut timing = TimingState::prepare(sample_rate, bpm, 4_096).unwrap();
        let mut elapsed = 0_u64;
        let mut ticks = Vec::new();
        let mut output = [TickBoundary::default(); 4_096];

        for &frame_count in partitions {
            let result = timing.advance_ticks(frame_count, &mut output, |_| TickTempo::Unchanged);
            assert_eq!(result.status, TimingAdvanceStatus::Complete);
            ticks.extend(
                output[..result.ticks_emitted as usize]
                    .iter()
                    .map(|tick| elapsed + tick.sample_offset as u64),
            );
            elapsed += frame_count as u64;
        }
        ticks
    }

    fn patterned_partitions(total: usize, pattern: &[usize]) -> Vec<usize> {
        let mut result = Vec::new();
        let mut remaining = total;
        let mut index = 0;
        while remaining != 0 {
            let frame_count = pattern[index % pattern.len()].min(remaining);
            result.push(frame_count);
            remaining -= frame_count;
            index += 1;
        }
        result
    }

    #[test]
    fn default_values_cover_the_maximum_render_slice() {
        let timing = TimingState::new(44_100.0);
        assert_eq!(timing.bpm(), INITIAL_BPM);
        assert_eq!(timing.max_ticks_per_slice(), 4_096);
        assert_eq!(timing.status(), TimingAdvanceStatus::Complete);
    }

    #[test]
    fn exact_slice_end_boundary_is_retained_for_next_slice() {
        let mut timing = TimingState::prepare(48_000.0, 125.0, 8).unwrap();
        let mut output = [TickBoundary::default(); 8];
        let first = timing.advance_ticks(960, &mut output, |_| TickTempo::Unchanged);
        assert_eq!(first.ticks_emitted, 0);
        assert_eq!(first.status, TimingAdvanceStatus::Complete);

        let second = timing.advance_ticks(1, &mut output, |_| TickTempo::Unchanged);
        assert_eq!(second.ticks_emitted, 1);
        assert_eq!(output[0].sample_offset, 0);
    }

    #[test]
    fn zero_length_slice_retains_an_offset_zero_boundary() {
        let mut timing = TimingState::prepare(48_000.0, 125.0, 8).unwrap();
        let mut output = [TickBoundary::default(); 8];
        assert_eq!(
            timing.advance_ticks(960, &mut output, |_| TickTempo::Unchanged),
            TimingAdvance {
                ticks_emitted: 0,
                status: TimingAdvanceStatus::Complete,
            }
        );
        assert_eq!(
            timing.advance_ticks(0, &mut output, |_| panic!("empty slice planned a tick")),
            TimingAdvance {
                ticks_emitted: 0,
                status: TimingAdvanceStatus::Complete,
            }
        );
        assert_eq!(
            timing.advance_ticks(1, &mut output, |_| TickTempo::Unchanged),
            TimingAdvance {
                ticks_emitted: 1,
                status: TimingAdvanceStatus::Complete,
            }
        );
        assert_eq!(output[0].sample_offset, 0);
    }

    #[test]
    fn multiple_ticks_are_emitted_in_strict_offset_order() {
        let mut timing = TimingState::prepare(48_000.0, 125.0, 16).unwrap();
        let mut output = [TickBoundary::default(); 16];
        let result = timing.advance_ticks(5_000, &mut output, |_| TickTempo::Unchanged);

        assert_eq!(result.status, TimingAdvanceStatus::Complete);
        let offsets: Vec<_> = output[..result.ticks_emitted as usize]
            .iter()
            .map(|tick| tick.sample_offset)
            .collect();
        assert_eq!(offsets, [960, 1_920, 2_880, 3_840, 4_800]);
        assert!(offsets.windows(2).all(|pair| pair[0] < pair[1]));
    }

    #[test]
    fn absolute_ticks_are_partition_invariant() {
        let total = 20_000;
        let oversized = vec![total];
        let fixed = patterned_partitions(total, &[512]);
        let alternating = patterned_partitions(total, &[1, 2_047, 17, 333]);
        let single_sample = vec![1; total];
        let expected = absolute_ticks(&oversized, 48_000.0, 125.0);

        assert_eq!(absolute_ticks(&fixed, 48_000.0, 125.0), expected);
        assert_eq!(absolute_ticks(&alternating, 48_000.0, 125.0), expected);
        assert_eq!(absolute_ticks(&single_sample, 48_000.0, 125.0), expected);
    }

    #[test]
    fn tempo_change_trace_is_partition_invariant() {
        fn trace(partitions: &[usize]) -> Vec<u64> {
            let mut timing = TimingState::prepare(10.0, 10.0, 128).unwrap();
            let mut elapsed = 0_u64;
            let mut tick_index = 0;
            let mut ticks = Vec::new();
            let mut output = [TickBoundary::default(); 128];
            for &frame_count in partitions {
                let result = timing.advance_ticks(frame_count, &mut output, |_| {
                    tick_index += 1;
                    if tick_index == 3 {
                        TickTempo::SetBpm(250.0 / 11.0)
                    } else {
                        TickTempo::Unchanged
                    }
                });
                assert_eq!(result.status, TimingAdvanceStatus::Complete);
                ticks.extend(
                    output[..result.ticks_emitted as usize]
                        .iter()
                        .map(|tick| elapsed + tick.sample_offset as u64),
                );
                elapsed += frame_count as u64;
            }
            ticks
        }

        let total = 100;
        let expected = trace(&[total]);
        assert_eq!(
            trace(&patterned_partitions(total, &[7, 1, 13, 2])),
            expected
        );
        assert_eq!(trace(&vec![1; total]), expected);
    }

    #[test]
    fn fractional_phase_has_no_partition_dependent_drift() {
        let total = 1_000_000;
        let oversized = vec![total];
        let alternating = patterned_partitions(total, &[127, 2_048, 3, 511, 1]);
        let expected = absolute_ticks(&oversized, 44_100.0, 120.0);
        let actual = absolute_ticks(&alternating, 44_100.0, 120.0);

        assert_eq!(actual, expected);
        assert_eq!(&expected[..4], &[919, 1_838, 2_757, 3_675]);
        assert!(expected.len() > 1_000);
    }

    #[test]
    fn bpm_change_at_tick_uses_new_next_interval_and_exact_phase() {
        let mut timing = TimingState::prepare(10.0, 10.0, 8).unwrap(); // 2.5 frames/tick
        let mut output = [TickBoundary::default(); 8];
        let mut planned = 0;
        let result = timing.advance_ticks(6, &mut output, |_| {
            planned += 1;
            if planned == 1 {
                TickTempo::SetBpm(250.0 / 11.0) // 1.1 frames/tick
            } else {
                TickTempo::Unchanged
            }
        });

        assert_eq!(result.status, TimingAdvanceStatus::Complete);
        let offsets: Vec<_> = output[..result.ticks_emitted as usize]
            .iter()
            .map(|tick| tick.sample_offset)
            .collect();
        // Exact phases are 2.5, 3.6, and 4.7. Reanchoring at the rounded first
        // boundary (frame 3) would incorrectly place the second tick at frame 5.
        assert_eq!(offsets, [3, 4, 5]);
        assert_eq!(timing.bpm(), 250.0 / 11.0);
    }

    #[test]
    fn public_set_bpm_between_ticks_keeps_next_boundary_then_uses_new_interval() {
        let mut timing = TimingState::prepare(10.0, 10.0, 8).unwrap(); // 2.5 frames/tick
        let mut output = [TickBoundary::default(); 8];
        assert_eq!(
            timing.advance_ticks(2, &mut output, |_| TickTempo::Unchanged),
            TimingAdvance {
                ticks_emitted: 0,
                status: TimingAdvanceStatus::Complete,
            }
        );

        timing.set_bpm(250.0 / 11.0).unwrap(); // 1.1 frames/tick
        let result = timing.advance_ticks(3, &mut output, |_| TickTempo::Unchanged);
        assert_eq!(result.status, TimingAdvanceStatus::Complete);
        assert_eq!(result.ticks_emitted, 2);
        // The old exact boundary remains 2.5 -> exposed frame 3 (offset 1).
        // The new interval then yields exact phase 3.6 -> frame 4 (offset 2).
        assert_eq!(output[0].sample_offset, 1);
        assert_eq!(output[1].sample_offset, 2);
    }

    #[test]
    fn tpl_compatibility_argument_does_not_change_tick_spacing() {
        let mut slow_rows = TimingState::new_with_bpm_tpl(48_000.0, 125.0, 3);
        let mut fast_rows = TimingState::new_with_bpm_tpl(48_000.0, 125.0, 12);
        let mut slow = [TickBoundary::default(); 8];
        let mut fast = [TickBoundary::default(); 8];

        let slow_result = slow_rows.advance_ticks(4_000, &mut slow, |_| TickTempo::Unchanged);
        let fast_result = fast_rows.advance_ticks(4_000, &mut fast, |_| TickTempo::Unchanged);

        assert_eq!(slow_result, fast_result);
        assert_eq!(
            &slow[..slow_result.ticks_emitted as usize],
            &fast[..fast_result.ticks_emitted as usize]
        );
    }

    #[test]
    fn invalid_inputs_are_rejected_without_planner_work() {
        assert_eq!(
            TimingState::prepare(0.0, 125.0, 8).unwrap_err(),
            TimingError::InvalidSampleRate
        );
        assert_eq!(
            TimingState::prepare(f64::NAN, 125.0, 8).unwrap_err(),
            TimingError::InvalidSampleRate
        );
        assert_eq!(
            TimingState::prepare(48_000.0, 0.0, 8).unwrap_err(),
            TimingError::InvalidBpm
        );
        assert_eq!(
            TimingState::prepare(48_000.0, f64::INFINITY, 8).unwrap_err(),
            TimingError::InvalidBpm
        );
        assert_eq!(
            TimingState::prepare(48_000.0, 120_001.0, 8).unwrap_err(),
            TimingError::TickIntervalOutOfRange
        );
        assert_eq!(
            TimingState::prepare(48_000.0, 125.0, 0).unwrap_err(),
            TimingError::ZeroTickCapacity
        );

        let mut invalid = TimingState::new(f64::NAN);
        let mut output = [TickBoundary::default(); 8];
        let result = invalid.advance_ticks(usize::MAX, &mut output, |_| {
            panic!("invalid clock ran planner")
        });
        assert_eq!(result.ticks_emitted, 0);
        assert_eq!(result.status, TimingAdvanceStatus::InvalidConfiguration);
    }

    #[test]
    fn invalid_public_bpm_is_rejected_without_faulting_valid_playback() {
        let mut timing = TimingState::prepare(48_000.0, 125.0, 8).unwrap();
        assert_eq!(timing.set_bpm(0.0), Err(TimingError::InvalidBpm));
        assert_eq!(timing.bpm(), 125.0);
        assert_eq!(timing.status(), TimingAdvanceStatus::Complete);
        assert_eq!(timing.advance(960).status, TimingAdvanceStatus::Complete);
    }

    #[test]
    fn invalid_tempo_plan_commits_no_output_and_reset_recovers_old_tempo() {
        let mut timing = TimingState::prepare(48_000.0, 125.0, 8).unwrap();
        let mut output = [TickBoundary::default(); 8];
        let result = timing.advance_ticks(2_000, &mut output, |_| TickTempo::SetBpm(f64::NAN));
        assert_eq!(result.ticks_emitted, 0);
        assert_eq!(result.status, TimingAdvanceStatus::InvalidConfiguration);
        assert_eq!(timing.bpm(), 125.0);

        let sticky =
            timing.advance_ticks(2_000, &mut output, |_| panic!("faulted clock ran planner"));
        assert_eq!(sticky.ticks_emitted, 0);
        assert_eq!(sticky.status, TimingAdvanceStatus::InvalidConfiguration);

        timing.reset().unwrap();
        let resumed = timing.advance_ticks(961, &mut output, |_| TickTempo::Unchanged);
        assert_eq!(resumed.status, TimingAdvanceStatus::Complete);
        assert_eq!(resumed.ticks_emitted, 1);
        assert_eq!(output[0].sample_offset, 960);
    }

    #[test]
    fn valid_bpm_recovers_invalid_directive_after_the_rejected_slice() {
        let mut timing = TimingState::prepare(48_000.0, 125.0, 8).unwrap();
        let mut output = [TickBoundary::default(); 8];
        let failed = timing.advance_ticks(2_000, &mut output, |_| TickTempo::SetBpm(f64::NAN));
        assert_eq!(failed.status, TimingAdvanceStatus::InvalidConfiguration);

        timing.set_bpm(125.0).unwrap();
        assert_eq!(timing.status(), TimingAdvanceStatus::Complete);
        assert_eq!(
            timing.advance_ticks(960, &mut output, |_| TickTempo::Unchanged),
            TimingAdvance {
                ticks_emitted: 0,
                status: TimingAdvanceStatus::Complete,
            }
        );
        let resumed = timing.advance_ticks(1, &mut output, |_| TickTempo::Unchanged);
        assert_eq!(resumed.ticks_emitted, 1);
        assert_eq!(output[0].sample_offset, 0);
    }

    #[test]
    fn valid_bpm_recovers_an_invalid_initial_bpm() {
        let mut timing = TimingState::new_with_bpm(48_000.0, 0.0);
        assert_eq!(timing.status(), TimingAdvanceStatus::InvalidConfiguration);
        assert_eq!(timing.reset(), Err(TimingError::InvalidBpm));

        timing.set_bpm(125.0).unwrap();
        assert_eq!(timing.status(), TimingAdvanceStatus::Complete);
        assert_eq!(timing.advance(960).ticks_emitted, 1);
    }

    #[test]
    fn capacity_overflow_commits_no_ticks_is_sticky_and_resettable() {
        let mut timing = TimingState::prepare(48_000.0, 125.0, 2).unwrap();
        let mut output = [TickBoundary::default(); 8];
        let overflow = timing.advance_ticks(5_000, &mut output, |_| TickTempo::Unchanged);
        assert_eq!(overflow.ticks_emitted, 0);
        assert_eq!(overflow.status, TimingAdvanceStatus::TickCapacityExceeded);

        let sticky = timing.advance_ticks(1, &mut output, |_| panic!("faulted clock ran planner"));
        assert_eq!(sticky.ticks_emitted, 0);
        assert_eq!(sticky.status, TimingAdvanceStatus::TickCapacityExceeded);

        timing.reset().unwrap();
        let resumed = timing.advance_ticks(961, &mut output, |_| TickTempo::Unchanged);
        assert_eq!(resumed.status, TimingAdvanceStatus::Complete);
        assert_eq!(resumed.ticks_emitted, 1);
        assert_eq!(output[0].sample_offset, 960);
    }

    #[test]
    fn output_capacity_failure_is_transactional_and_reusable_after_reset() {
        let mut timing = TimingState::prepare(48_000.0, 125.0, 8).unwrap();
        let mut output = [TickBoundary::default(); 1];
        let overflow = timing.advance_ticks(2_000, &mut output, |_| TickTempo::Unchanged);
        assert_eq!(overflow.ticks_emitted, 0);
        assert_eq!(overflow.status, TimingAdvanceStatus::TickCapacityExceeded);

        timing.reset().unwrap();
        let complete = timing.advance_ticks(961, &mut output, |_| TickTempo::Unchanged);
        assert_eq!(complete.ticks_emitted, 1);
        assert_eq!(complete.status, TimingAdvanceStatus::Complete);
    }

    #[test]
    fn absolute_frame_overflow_is_explicit_and_transport_reset_recovers() {
        let mut timing = TimingState::prepare(48_000.0, 125.0, 8).unwrap();
        timing.frame_position = u64::MAX;
        let mut output = [TickBoundary::default(); 8];
        let result =
            timing.advance_ticks(1, &mut output, |_| panic!("overflowed clock ran planner"));
        assert_eq!(result.ticks_emitted, 0);
        assert_eq!(result.status, TimingAdvanceStatus::PositionOverflow);

        timing.reset().unwrap();
        assert_eq!(timing.status(), TimingAdvanceStatus::Complete);
        assert_eq!(
            timing.advance_ticks(961, &mut output, |_| TickTempo::Unchanged),
            TimingAdvance {
                ticks_emitted: 1,
                status: TimingAdvanceStatus::Complete,
            }
        );
    }

    #[test]
    fn maximum_host_chunk_at_high_valid_bpm_stays_within_default_bound() {
        let mut timing = TimingState::new_with_bpm(48_000.0, u16::MAX as f64);
        let result = timing.advance(4_096);
        assert_eq!(result.status, TimingAdvanceStatus::Complete);
        assert_eq!(result.ticks_emitted, 2_236);
    }

    #[test]
    fn one_frame_interval_covers_the_complete_default_host_chunk() {
        let mut timing = TimingState::new_with_bpm(48_000.0, 120_000.0);
        let result = timing.advance(4_096);
        assert_eq!(result.status, TimingAdvanceStatus::Complete);
        assert_eq!(result.ticks_emitted, 4_096);
    }

    #[test]
    fn compatibility_count_retains_legacy_end_inclusive_semantics_and_status() {
        let mut timing = TimingState::new(48_000.0);
        assert_eq!(timing.advance(959).ticks_emitted, 0);
        assert_eq!(timing.advance(1).ticks_emitted, 1);
        assert_eq!(
            timing.advance(1_920),
            TimingAdvance {
                ticks_emitted: 2,
                status: TimingAdvanceStatus::Complete,
            }
        );
    }
}
