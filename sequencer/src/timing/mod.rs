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
/// Prepared users should choose an explicit bound with [`TimingState::prepare`]
/// based on their maximum render slice. Since valid tick intervals are at
/// least one frame, no slice can contain more ticks than frames.
pub const DEFAULT_MAX_TICKS_PER_SLICE: u32 = 1_024;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TickBoundary {
    /// Offset in the half-open range `0..frame_count`.
    pub sample_offset: usize,
}

/// A tempo directive returned after processing an emitted tick.
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
    /// More tick boundaries existed in the slice than the prepared work bound.
    TickCapacityExceeded,
    /// Preparation or a tempo directive supplied an invalid value.
    InvalidConfiguration,
    /// The absolute frame or fixed-point tick phase was exhausted.
    PositionOverflow,
}

/// Result of advancing one exact frame slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimingAdvance {
    pub ticks_emitted: u32,
    pub status: TimingAdvanceStatus,
}

/// Prepared, allocation-free tracker tick clock.
///
/// Tick phases use unsigned Q64.64 fixed point. Adding the same prepared
/// interval once per tick makes absolute boundaries independent of render-slice
/// partitioning while retaining sub-frame phase. A BPM directive returned for
/// a tick adds the new interval to that tick's exact phase, not its rounded
/// sample offset, so an already emitted boundary never moves.
#[derive(Debug, Clone)]
pub struct TimingState {
    sample_rate: f64,
    bpm: f64,
    tick_interval: u128,
    next_tick_phase: u128,
    frame_position: u64,
    max_ticks_per_slice: u32,
    fault: Option<TimingAdvanceStatus>,
}

impl TimingState {
    /// Compatibility constructor with the default BPM and work bound.
    ///
    /// Prefer [`Self::prepare`] when invalid input must be reported before the
    /// callback starts. Invalid compatibility input creates a fail-closed clock
    /// whose advances return [`TimingAdvanceStatus::InvalidConfiguration`].
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

    /// Compatibility constructor with the default work bound.
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
            fault: None,
        })
    }

    /// Advances one half-open frame slice and exposes every accepted tick.
    ///
    /// The callback runs in strict offset order at most
    /// `max_ticks_per_slice` times and must itself obey the caller's real-time
    /// rules. Its tempo directive applies to the interval after the tick being
    /// processed. No storage is allocated or freed by this method.
    ///
    /// Capacity and position failures are fail-closed and sticky. Call
    /// [`Self::reset`] at a deliberate transport boundary after handling the
    /// status; a reset starts a full interval at the current absolute frame.
    pub fn advance_ticks(
        &mut self,
        frame_count: usize,
        on_tick: impl FnMut(TickBoundary) -> TickTempo,
    ) -> TimingAdvance {
        self.advance_internal(frame_count, false, on_tick)
    }

    fn advance_internal(
        &mut self,
        frame_count: usize,
        include_block_end: bool,
        mut on_tick: impl FnMut(TickBoundary) -> TickTempo,
    ) -> TimingAdvance {
        if let Some(status) = self.fault {
            return TimingAdvance {
                ticks_emitted: 0,
                status,
            };
        }

        let Ok(frame_count) = u64::try_from(frame_count) else {
            return self.fail(0, TimingAdvanceStatus::PositionOverflow);
        };
        let Some(block_end) = self.frame_position.checked_add(frame_count) else {
            return self.fail(0, TimingAdvanceStatus::PositionOverflow);
        };
        let block_start = self.frame_position;
        let mut ticks_emitted = 0;

        if frame_count == 0 {
            return TimingAdvance {
                ticks_emitted,
                status: TimingAdvanceStatus::Complete,
            };
        }

        loop {
            let Some(boundary_frame) = fixed_phase_ceiling(self.next_tick_phase) else {
                self.frame_position = block_end;
                return self.fail(ticks_emitted, TimingAdvanceStatus::PositionOverflow);
            };

            if boundary_frame > block_end || (!include_block_end && boundary_frame == block_end) {
                self.frame_position = block_end;
                return TimingAdvance {
                    ticks_emitted,
                    status: TimingAdvanceStatus::Complete,
                };
            }
            if boundary_frame < block_start {
                self.frame_position = block_end;
                return self.fail(ticks_emitted, TimingAdvanceStatus::PositionOverflow);
            }
            if ticks_emitted == self.max_ticks_per_slice {
                self.frame_position = block_end;
                return self.fail(ticks_emitted, TimingAdvanceStatus::TickCapacityExceeded);
            }

            let sample_offset = usize::try_from(boundary_frame - block_start)
                .expect("an offset within a usize-sized input slice fits usize");
            let tempo = on_tick(TickBoundary { sample_offset });
            ticks_emitted += 1;

            let interval = match tempo {
                TickTempo::Unchanged => self.tick_interval,
                TickTempo::SetBpm(new_bpm) => {
                    let Ok(interval) = prepare_tick_interval(self.sample_rate, new_bpm) else {
                        self.frame_position = block_end;
                        return self.fail(ticks_emitted, TimingAdvanceStatus::InvalidConfiguration);
                    };
                    self.bpm = new_bpm;
                    self.tick_interval = interval;
                    interval
                }
            };

            let Some(next_tick_phase) = self.next_tick_phase.checked_add(interval) else {
                self.frame_position = block_end;
                return self.fail(ticks_emitted, TimingAdvanceStatus::PositionOverflow);
            };
            self.next_tick_phase = next_tick_phase;
        }
    }

    /// Bounded count-only compatibility shim for the current tracker player.
    ///
    /// This deliberately discards offsets and status and must be removed when
    /// #204 migrates the tracker to offset-bearing events. It retains the old
    /// end-inclusive count behavior so existing whole-block playback references
    /// do not change before that migration; do not mix it with `advance_ticks`.
    /// Unlike the former implementation, its loop is capped by the prepared
    /// tick bound.
    pub fn advance(&mut self, frame_count: usize) -> u32 {
        self.advance_internal(frame_count, true, |_| TickTempo::Unchanged)
            .ticks_emitted
    }

    /// Latches a BPM for intervals scheduled after the next emitted tick.
    ///
    /// To change the interval immediately after a particular tick, return
    /// [`TickTempo::SetBpm`] while processing that tick in [`Self::advance_ticks`].
    /// This method never moves the already scheduled next boundary.
    pub fn set_bpm(&mut self, new_bpm: f64) -> Result<(), TimingError> {
        let interval = match prepare_tick_interval(self.sample_rate, new_bpm) {
            Ok(interval) => interval,
            Err(error) => {
                self.fault = Some(TimingAdvanceStatus::InvalidConfiguration);
                return Err(error);
            }
        };
        self.bpm = new_bpm;
        self.tick_interval = interval;
        if self.fault == Some(TimingAdvanceStatus::InvalidConfiguration) {
            self.fault = None;
        }
        Ok(())
    }

    /// Returns the currently prepared BPM.
    pub fn bpm(&self) -> f64 {
        self.bpm
    }

    /// Returns the prepared maximum number of callbacks per frame slice.
    pub fn max_ticks_per_slice(&self) -> u32 {
        self.max_ticks_per_slice
    }

    /// Reanchors the next tick one full interval after the current frame.
    ///
    /// The absolute frame counter remains monotonic; reset changes musical phase
    /// without rewinding elapsed render time.
    pub fn reset(&mut self) -> Result<(), TimingError> {
        if self.fault == Some(TimingAdvanceStatus::InvalidConfiguration) {
            return Err(TimingError::InvalidBpm);
        }
        let current_phase = u128::from(self.frame_position) << FIXED_FRACTION_BITS;
        self.next_tick_phase = current_phase
            .checked_add(self.tick_interval)
            .ok_or(TimingError::PositionOverflow)?;
        self.fault = None;
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
            fault: Some(TimingAdvanceStatus::InvalidConfiguration),
        }
    }

    fn fail(&mut self, ticks_emitted: u32, status: TimingAdvanceStatus) -> TimingAdvance {
        self.fault = Some(status);
        TimingAdvance {
            ticks_emitted,
            status,
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

        for &frame_count in partitions {
            let result = timing.advance_ticks(frame_count, |tick| {
                ticks.push(elapsed + tick.sample_offset as u64);
                TickTempo::Unchanged
            });
            assert_eq!(result.status, TimingAdvanceStatus::Complete);
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
    fn default_values_are_prepared() {
        let timing = TimingState::new(44_100.0);
        assert_eq!(timing.bpm(), INITIAL_BPM);
        assert_eq!(timing.max_ticks_per_slice(), DEFAULT_MAX_TICKS_PER_SLICE);
    }

    #[test]
    fn exact_slice_end_boundary_is_retained_for_next_slice() {
        let mut timing = TimingState::prepare(48_000.0, 125.0, 8).unwrap();
        let first = timing.advance_ticks(960, |_| TickTempo::Unchanged);
        assert_eq!(first.ticks_emitted, 0);
        assert_eq!(first.status, TimingAdvanceStatus::Complete);

        let mut offset = None;
        let second = timing.advance_ticks(1, |tick| {
            offset = Some(tick.sample_offset);
            TickTempo::Unchanged
        });
        assert_eq!(second.ticks_emitted, 1);
        assert_eq!(offset, Some(0));
    }

    #[test]
    fn zero_length_slice_retains_an_offset_zero_boundary() {
        let mut timing = TimingState::prepare(48_000.0, 125.0, 8).unwrap();
        assert_eq!(
            timing.advance_ticks(960, |_| TickTempo::Unchanged),
            TimingAdvance {
                ticks_emitted: 0,
                status: TimingAdvanceStatus::Complete,
            }
        );
        assert_eq!(
            timing.advance_ticks(0, |_| panic!("empty slice emitted a tick")),
            TimingAdvance {
                ticks_emitted: 0,
                status: TimingAdvanceStatus::Complete,
            }
        );

        let mut observed = usize::MAX;
        timing.advance_ticks(1, |tick| {
            observed = tick.sample_offset;
            TickTempo::Unchanged
        });
        assert_eq!(observed, 0);
    }

    #[test]
    fn multiple_ticks_are_emitted_in_strict_offset_order() {
        let mut timing = TimingState::prepare(48_000.0, 125.0, 16).unwrap();
        let mut offsets = [usize::MAX; 8];
        let mut len = 0;
        let result = timing.advance_ticks(5_000, |tick| {
            offsets[len] = tick.sample_offset;
            len += 1;
            TickTempo::Unchanged
        });

        assert_eq!(result.status, TimingAdvanceStatus::Complete);
        assert_eq!(&offsets[..len], &[960, 1_920, 2_880, 3_840, 4_800]);
        assert!(offsets[..len].windows(2).all(|pair| pair[0] < pair[1]));
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
            for &frame_count in partitions {
                let result = timing.advance_ticks(frame_count, |tick| {
                    ticks.push(elapsed + tick.sample_offset as u64);
                    tick_index += 1;
                    if tick_index == 3 {
                        TickTempo::SetBpm(250.0 / 11.0)
                    } else {
                        TickTempo::Unchanged
                    }
                });
                assert_eq!(result.status, TimingAdvanceStatus::Complete);
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
        let mut offsets = [usize::MAX; 4];
        let mut len = 0;
        let result = timing.advance_ticks(6, |tick| {
            offsets[len] = tick.sample_offset;
            len += 1;
            if len == 1 {
                TickTempo::SetBpm(250.0 / 11.0) // 1.1 frames/tick
            } else {
                TickTempo::Unchanged
            }
        });

        assert_eq!(result.status, TimingAdvanceStatus::Complete);
        // Exact phases are 2.5, 3.6, and 4.7. Reanchoring at the rounded first
        // boundary (frame 3) would incorrectly place the second tick at frame 5.
        assert_eq!(&offsets[..len], &[3, 4, 5]);
        assert_eq!(timing.bpm(), 250.0 / 11.0);
    }

    #[test]
    fn tpl_compatibility_argument_does_not_change_tick_spacing() {
        let mut slow_rows = TimingState::new_with_bpm_tpl(48_000.0, 125.0, 3);
        let mut fast_rows = TimingState::new_with_bpm_tpl(48_000.0, 125.0, 12);
        let mut slow = [usize::MAX; 4];
        let mut fast = [usize::MAX; 4];
        let mut slow_len = 0;
        let mut fast_len = 0;

        slow_rows.advance_ticks(4_000, |tick| {
            slow[slow_len] = tick.sample_offset;
            slow_len += 1;
            TickTempo::Unchanged
        });
        fast_rows.advance_ticks(4_000, |tick| {
            fast[fast_len] = tick.sample_offset;
            fast_len += 1;
            TickTempo::Unchanged
        });

        assert_eq!(&slow[..slow_len], &fast[..fast_len]);
    }

    #[test]
    fn invalid_inputs_are_rejected_without_callback_work() {
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
        let result = invalid.advance_ticks(usize::MAX, |_| panic!("invalid clock ran callback"));
        assert_eq!(result.ticks_emitted, 0);
        assert_eq!(result.status, TimingAdvanceStatus::InvalidConfiguration);
    }

    #[test]
    fn invalid_tempo_directive_fails_closed() {
        let mut timing = TimingState::prepare(48_000.0, 125.0, 8).unwrap();
        let result = timing.advance_ticks(2_000, |_| TickTempo::SetBpm(f64::NAN));
        assert_eq!(result.ticks_emitted, 1);
        assert_eq!(result.status, TimingAdvanceStatus::InvalidConfiguration);

        let sticky = timing.advance_ticks(2_000, |_| panic!("faulted clock ran callback"));
        assert_eq!(sticky.ticks_emitted, 0);
        assert_eq!(sticky.status, TimingAdvanceStatus::InvalidConfiguration);
    }

    #[test]
    fn capacity_overflow_is_explicit_sticky_and_resettable() {
        let mut timing = TimingState::prepare(48_000.0, 125.0, 2).unwrap();
        let overflow = timing.advance_ticks(5_000, |_| TickTempo::Unchanged);
        assert_eq!(overflow.ticks_emitted, 2);
        assert_eq!(overflow.status, TimingAdvanceStatus::TickCapacityExceeded);

        let sticky = timing.advance_ticks(1, |_| panic!("faulted clock ran callback"));
        assert_eq!(sticky.ticks_emitted, 0);
        assert_eq!(sticky.status, TimingAdvanceStatus::TickCapacityExceeded);

        timing.reset().unwrap();
        let mut offset = None;
        let resumed = timing.advance_ticks(961, |tick| {
            offset = Some(tick.sample_offset);
            TickTempo::Unchanged
        });
        assert_eq!(resumed.status, TimingAdvanceStatus::Complete);
        assert_eq!(offset, Some(960));
    }

    #[test]
    fn absolute_frame_overflow_is_explicit() {
        let mut timing = TimingState::prepare(48_000.0, 125.0, 8).unwrap();
        timing.frame_position = u64::MAX;
        let result = timing.advance_ticks(1, |_| panic!("overflowed clock ran callback"));
        assert_eq!(result.ticks_emitted, 0);
        assert_eq!(result.status, TimingAdvanceStatus::PositionOverflow);
    }

    #[test]
    fn compatibility_count_retains_legacy_end_inclusive_semantics() {
        let mut timing = TimingState::new(48_000.0);
        assert_eq!(timing.advance(959), 0);
        assert_eq!(timing.advance(1), 1);
        assert_eq!(timing.advance(1_920), 2);
    }
}
