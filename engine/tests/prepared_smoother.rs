use engine::{PreparedSmoother, SmootherPrepareError, SmootherValueError};
use param_manifest::{SmoothingCurve, SmoothingPolicy};

fn smoothed(curve: SmoothingCurve, duration_ms: f32) -> SmoothingPolicy {
    SmoothingPolicy::Smoothed { duration_ms, curve }
}

fn assert_near(actual: f32, expected: f32, tolerance: f32) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "expected {actual} to be within {tolerance} of {expected}"
    );
}

#[test]
fn prepare_rejects_every_invalid_input_class() {
    for sample_rate in [0.0, -1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert_eq!(
            PreparedSmoother::prepare(SmoothingPolicy::None, sample_rate, 0.0).unwrap_err(),
            SmootherPrepareError::InvalidSampleRate
        );
    }

    for seed in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert_eq!(
            PreparedSmoother::prepare(SmoothingPolicy::None, 48_000.0, seed).unwrap_err(),
            SmootherPrepareError::InvalidSeed
        );
    }

    for duration_ms in [-0.001, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert_eq!(
            PreparedSmoother::prepare(
                smoothed(SmoothingCurve::Linear, duration_ms),
                48_000.0,
                0.0,
            )
            .unwrap_err(),
            SmootherPrepareError::InvalidDuration
        );
    }

    assert_eq!(
        PreparedSmoother::prepare(
            smoothed(SmoothingCurve::Exponential, f32::MAX),
            f32::MAX,
            0.0,
        )
        .unwrap_err(),
        SmootherPrepareError::DurationFrameCountUnrepresentable
    );
}

#[test]
fn positive_duration_uses_ceil_and_checks_the_u32_frame_range() {
    let sub_frame =
        PreparedSmoother::prepare(smoothed(SmoothingCurve::Linear, 0.000_1), 1_000.0, 0.0).unwrap();
    let mut sub_frame = sub_frame;
    sub_frame.latch_target(1.0).unwrap();
    assert!(!sub_frame.is_settled());
    assert_eq!(sub_frame.advance(1), 1.0);
    assert!(sub_frame.is_settled());

    let mut fractional =
        PreparedSmoother::prepare(smoothed(SmoothingCurve::Linear, 1.001), 1_000.0, 0.0).unwrap();
    fractional.latch_target(2.0).unwrap();
    assert_eq!(fractional.advance(1), 1.0);
    assert!(!fractional.is_settled());
    assert_eq!(fractional.advance(1), 2.0);

    let fifteen_ms_48k =
        PreparedSmoother::prepare(smoothed(SmoothingCurve::Linear, 15.0), 48_000.0, 0.0).unwrap();
    let mut fifteen_ms_48k = fifteen_ms_48k;
    fifteen_ms_48k.latch_target(1.0).unwrap();
    assert_eq!(fifteen_ms_48k.value_at(719), 719.0 / 720.0);
    assert_eq!(fifteen_ms_48k.value_at(720), 1.0);

    let mut fifteen_ms_44k1 =
        PreparedSmoother::prepare(smoothed(SmoothingCurve::Linear, 15.0), 44_100.0, 0.0).unwrap();
    fifteen_ms_44k1.latch_target(1.0).unwrap();
    assert_eq!(fifteen_ms_44k1.value_at(661), 661.0 / 662.0);
    assert_eq!(fifteen_ms_44k1.value_at(662), 1.0);

    // At 1,000 Hz, duration milliseconds and frame count have the same
    // numeric value. The immediately preceding f32 below 2^32 is representable.
    let two_to_32 = 4_294_967_296.0_f32;
    let largest_f32_below = f32::from_bits(two_to_32.to_bits() - 1);
    let boundary = PreparedSmoother::prepare(
        smoothed(SmoothingCurve::Exponential, largest_f32_below),
        1_000.0,
        -1.0,
    )
    .unwrap();
    assert_eq!(boundary.value_at(largest_f32_below as u32), -1.0);

    assert_eq!(
        PreparedSmoother::prepare(
            smoothed(SmoothingCurve::Exponential, two_to_32),
            1_000.0,
            -1.0,
        )
        .unwrap_err(),
        SmootherPrepareError::DurationFrameCountUnrepresentable
    );
}

#[test]
fn none_and_zero_duration_jump_at_the_latch_cursor() {
    for policy in [
        SmoothingPolicy::None,
        smoothed(SmoothingCurve::Linear, 0.0),
        smoothed(SmoothingCurve::Exponential, -0.0),
    ] {
        let mut smoother = PreparedSmoother::prepare(policy, 48_000.0, -3.0).unwrap();
        assert_eq!(smoother.current(), -3.0);
        assert_eq!(smoother.target(), -3.0);
        assert!(smoother.is_settled());

        smoother.latch_target(7.5).unwrap();
        assert_eq!(smoother.current(), 7.5);
        assert_eq!(smoother.target(), 7.5);
        assert!(smoother.is_settled());
        assert_eq!(smoother.value_at(0), 7.5);
        assert_eq!(smoother.advance(u32::MAX), 7.5);
    }
}

#[test]
fn linear_uses_the_full_n_frames_and_snaps_exactly_at_n() {
    let mut smoother =
        PreparedSmoother::prepare(smoothed(SmoothingCurve::Linear, 10.0), 1_000.0, 0.0).unwrap();
    smoother.latch_target(10.0).unwrap();

    assert_eq!(smoother.value_at(0), 0.0);
    assert_eq!(smoother.value_at(1), 1.0);
    assert_eq!(smoother.value_at(9), 9.0);
    assert_eq!(smoother.current(), 0.0, "value_at must not move the cursor");
    assert_eq!(smoother.advance(0), 0.0);
    assert!(!smoother.is_settled());

    assert_eq!(smoother.advance(9), 9.0);
    assert!(!smoother.is_settled());
    assert_eq!(smoother.advance(1), 10.0);
    assert_eq!(smoother.current(), smoother.target());
    assert!(smoother.is_settled());
    assert_eq!(smoother.advance(u32::MAX), 10.0);
}

#[test]
fn exponential_has_the_minus_100_db_curve_and_exact_n_snap() {
    let mut smoother =
        PreparedSmoother::prepare(smoothed(SmoothingCurve::Exponential, 10.0), 1_000.0, 2.0)
            .unwrap();
    smoother.latch_target(12.0).unwrap();

    let halfway = smoother.value_at(5);
    let halfway_relative_residual = f64::from((halfway - 12.0).abs()) / 10.0;
    assert_near(
        halfway_relative_residual as f32,
        10.0_f32.powf(-2.5),
        1.0e-7,
    );

    let last_pre_snap = smoother.value_at(9);
    let last_relative_residual = f64::from((last_pre_snap - 12.0).abs()) / 10.0;
    assert!(last_relative_residual > 1.0e-5);
    assert_near(last_relative_residual as f32, 10.0_f32.powf(-4.5), 1.0e-7);

    assert_eq!(smoother.advance(9), last_pre_snap);
    assert!(!smoother.is_settled());
    assert_eq!(smoother.advance(1), 12.0);
    assert_eq!(smoother.value_at(10), 12.0);
    assert_eq!(smoother.value_at(u32::MAX), 12.0);
    assert!(smoother.is_settled());
}

#[test]
fn same_target_republication_does_not_restart_a_running_curve() {
    let mut smoother =
        PreparedSmoother::prepare(smoothed(SmoothingCurve::Linear, 10.0), 1_000.0, 0.0).unwrap();
    smoother.latch_target(10.0).unwrap();
    assert_eq!(smoother.advance(4), 4.0);

    smoother.latch_target(10.0).unwrap();
    assert_eq!(smoother.current(), 4.0);
    assert_eq!(smoother.advance(1), 5.0);
}

#[test]
fn retarget_starts_at_current_with_a_fresh_full_duration() {
    let mut smoother =
        PreparedSmoother::prepare(smoothed(SmoothingCurve::Linear, 10.0), 1_000.0, 0.0).unwrap();
    smoother.latch_target(10.0).unwrap();
    assert_eq!(smoother.advance(4), 4.0);

    smoother.latch_target(20.0).unwrap();
    assert_eq!(smoother.current(), 4.0);
    assert_eq!(smoother.value_at(0), 4.0);
    assert_eq!(smoother.advance(5), 12.0);
    assert_eq!(smoother.advance(5), 20.0);
    assert!(smoother.is_settled());
}

#[test]
fn changed_target_equal_to_current_settles_immediately() {
    let mut smoother =
        PreparedSmoother::prepare(smoothed(SmoothingCurve::Linear, 10.0), 1_000.0, 0.0).unwrap();
    smoother.latch_target(10.0).unwrap();
    smoother.advance(4);
    smoother.latch_target(4.0).unwrap();

    assert_eq!(smoother.current(), 4.0);
    assert_eq!(smoother.target(), 4.0);
    assert!(smoother.is_settled());
}

#[test]
fn reset_reseeds_without_a_startup_ramp_and_preserves_policy() {
    let mut smoother =
        PreparedSmoother::prepare(smoothed(SmoothingCurve::Exponential, 15.0), 1_000.0, -5.0)
            .unwrap();
    smoother.latch_target(9.0).unwrap();
    smoother.advance(7);

    smoother.reset(-11.0).unwrap();
    assert_eq!(smoother.current(), -11.0);
    assert_eq!(smoother.target(), -11.0);
    assert!(smoother.is_settled());
    assert_eq!(smoother.advance(14), -11.0);

    smoother.latch_target(4.0).unwrap();
    assert!(!smoother.is_settled());
    assert_eq!(smoother.value_at(0), -11.0);
}

#[test]
fn non_finite_latch_and_reset_are_rejected_transactionally() {
    let mut smoother =
        PreparedSmoother::prepare(smoothed(SmoothingCurve::Linear, 10.0), 1_000.0, -2.0).unwrap();
    smoother.latch_target(8.0).unwrap();
    smoother.advance(3);

    let expected = (
        smoother.current(),
        smoother.target(),
        smoother.is_settled(),
        smoother.value_at(7),
    );
    for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert_eq!(
            smoother.latch_target(invalid),
            Err(SmootherValueError::NonFinite)
        );
        assert_eq!(smoother.reset(invalid), Err(SmootherValueError::NonFinite));
        assert_eq!(
            (
                smoother.current(),
                smoother.target(),
                smoother.is_settled(),
                smoother.value_at(7),
            ),
            expected
        );
    }
}

#[test]
fn finite_sign_unconstrained_extremes_stay_finite() {
    for curve in [SmoothingCurve::Linear, SmoothingCurve::Exponential] {
        let mut smoother =
            PreparedSmoother::prepare(smoothed(curve, 4.0), 1_000.0, -f32::MAX).unwrap();
        smoother.latch_target(f32::MAX).unwrap();

        for cursor in 0..=4 {
            assert!(smoother.value_at(cursor).is_finite());
        }
        for _ in 0..4 {
            assert!(smoother.advance(1).is_finite());
        }
        assert_eq!(smoother.current(), f32::MAX);
    }
}

#[test]
fn aggregate_and_partitioned_advances_are_bit_identical() {
    for curve in [SmoothingCurve::Linear, SmoothingCurve::Exponential] {
        for total in [0, 1, 2, 17, 36, 37, 38, 100] {
            let mut aggregate =
                PreparedSmoother::prepare(smoothed(curve, 37.0), 1_000.0, -17.25).unwrap();
            aggregate.latch_target(93.5).unwrap();
            let mut partitioned = aggregate;

            aggregate.advance(total);
            let mut remaining = total;
            for partition in [3, 1, 11, 2, 7, 5, 13] {
                let step = remaining.min(partition);
                partitioned.advance(step);
                remaining -= step;
            }
            partitioned.advance(remaining);

            assert_eq!(
                partitioned.current().to_bits(),
                aggregate.current().to_bits(),
                "curve={curve:?}, total={total}"
            );
            assert_eq!(partitioned.is_settled(), aggregate.is_settled());
        }
    }
}

#[test]
fn partition_equivalence_survives_an_identical_retarget_cursor() {
    let mut left =
        PreparedSmoother::prepare(smoothed(SmoothingCurve::Exponential, 41.0), 1_000.0, 0.25)
            .unwrap();
    left.latch_target(-8.0).unwrap();
    left.advance(13);
    let mut right = left;

    left.latch_target(12.0).unwrap();
    right.latch_target(12.0).unwrap();
    left.advance(29);
    for step in [1, 7, 3, 9, 9] {
        right.advance(step);
    }

    assert_eq!(left.current().to_bits(), right.current().to_bits());
    assert_eq!(left.target(), right.target());
}
