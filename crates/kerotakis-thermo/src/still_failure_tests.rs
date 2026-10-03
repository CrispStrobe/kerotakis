//! Source-informed, independent Rayleigh controls for failure ownership.
use super::*;

fn constant_volatility(x: [f64; 2], trace_is_water: bool) -> Option<BubblePoint> {
    let weights = if trace_is_water {
        [x[0], 4.0 * x[1]]
    } else {
        [4.0 * x[0], x[1]]
    };
    let sum = weights[0] + weights[1];
    Some(BubblePoint {
        t_celsius: 90.0,
        y: vec![weights[0] / sum, weights[1] / sum],
        azeotropic: false,
    })
}

// A monotone solve of the continuous Rayleigh invariant and amount budget.
// Independent of the production mesh, phase callback and depletion limiter.
fn rayleigh_residue(trace: f64, fraction: f64, stages: u32) -> (f64, f64) {
    let target = (1.0 + trace) * (1.0 - fraction);
    let alpha = 4.0f64.powi(stages as i32);
    let (mut lo, mut hi) = (0.0f64, 1.0f64);
    for _ in 0..100 {
        let bulk = 0.5 * (lo + hi);
        if bulk + trace * bulk.powf(alpha) < target {
            lo = bulk;
        } else {
            hi = bulk;
        }
    }
    let bulk = 0.5 * (lo + hi);
    (bulk, trace * bulk.powf(alpha))
}

#[test]
fn independent_rayleigh_matrix_separates_supported_cuts_from_condensate_precision() {
    for trace_is_water in [false, true] {
        for scale in [2.0f64.powi(-100), 1.0, 2.0f64.powi(100)] {
            for (stages, fraction, supported) in [(1, 0.2, true), (4, 0.01, true), (4, 0.2, false)]
            {
                let (bulk_r, trace_r) = rayleigh_residue(1e-6, fraction, stages);
                assert!(trace_r > 0.0 && trace_r.is_finite());
                let (water, ethanol, expected_w, expected_e) = if trace_is_water {
                    (
                        1e-6 * scale,
                        scale,
                        (1e-6 - trace_r) * scale,
                        (1.0 - bulk_r) * scale,
                    )
                } else {
                    (
                        scale,
                        1e-6 * scale,
                        (1.0 - bulk_r) * scale,
                        (1e-6 - trace_r) * scale,
                    )
                };
                let target_energy =
                    expected_w * WATER_HVAP_KJ_PER_MOL + expected_e * ETHANOL_HVAP_KJ_PER_MOL;
                for take in [
                    StillTake::Fraction(fraction),
                    StillTake::EnergyKj(target_energy),
                ] {
                    let result = ethanol_water_still_with_phase(
                        water,
                        ethanol,
                        take,
                        stages,
                        ATMOSPHERE_KPA,
                        |x, _| constant_volatility(x, trace_is_water),
                    );
                    if !supported {
                        // The physical residue is positive and representable,
                        // but smaller than the spacing of the overhead amount.
                        let initial_trace = 1e-6 * scale;
                        let spacing = initial_trace - f64::from_bits(initial_trace.to_bits() - 1);
                        assert!(trace_r * scale < spacing);
                        assert_eq!(result, Err(StillError::UnrepresentableCondensate));
                        continue;
                    }
                    let cut = result.unwrap();
                    assert!((cut.water_over / expected_w - 1.0).abs() < 1e-3);
                    assert!((cut.ethanol_over / expected_e - 1.0).abs() < 1e-3);
                    assert!(water - cut.water_over > 0.0 && ethanol - cut.ethanol_over > 0.0);
                    let accounted = cut.water_over * WATER_HVAP_KJ_PER_MOL
                        + cut.ethanol_over * ETHANOL_HVAP_KJ_PER_MOL;
                    assert!((cut.energy_kj / accounted - 1.0).abs() < 2e-12);
                    match take {
                        StillTake::Fraction(f) => assert!(
                            ((cut.water_over + cut.ethanol_over) / ((water + ethanol) * f) - 1.0)
                                .abs()
                                < 2e-12
                        ),
                        StillTake::EnergyKj(q) => assert!((cut.energy_kj / q - 1.0).abs() < 2e-12),
                    }
                }
            }
        }
    }
}

#[test]
fn phase_and_coordinate_failures_have_distinct_owners_at_each_stage() {
    for stages in [1, 4] {
        for failed_call in 0..stages {
            for lost_coordinate in [None, Some(0), Some(1)] {
                let mut calls = 0;
                let result = cascade([0.25, 0.75], stages, ATMOSPHERE_KPA, &mut |x, _| {
                    let failed = calls == failed_call;
                    calls += 1;
                    if failed && lost_coordinate.is_none() {
                        return None;
                    }
                    let mut y = x;
                    if failed {
                        y[lost_coordinate.unwrap()] = 0.0;
                    }
                    Some(BubblePoint {
                        t_celsius: 90.0,
                        y: y.to_vec(),
                        azeotropic: false,
                    })
                });
                let expected = if lost_coordinate.is_none() {
                    StillError::PhaseEvaluation
                } else {
                    StillError::UnrepresentableComposition
                };
                assert_eq!(result, Err(expected));
                assert_eq!(calls, failed_call + 1);
            }
        }
    }
    assert_eq!(
        cascade([1e-300, 1e100], 4, ATMOSPHERE_KPA, &mut |_, _| panic!(
            "normalization must refuse before phase evaluation"
        )),
        Err(StillError::UnrepresentableComposition)
    );
}

#[test]
fn a_deliberately_exhausted_iteration_budget_is_not_a_phase_failure() {
    let phase = |x: [f64; 2], _: f64| {
        Some(BubblePoint {
            t_celsius: 90.0,
            y: x.to_vec(),
            azeotropic: false,
        })
    };
    assert_eq!(
        ethanol_water_still_with_limit(
            1.0,
            1.0,
            StillTake::Fraction(0.1),
            1,
            ATMOSPHERE_KPA,
            phase,
            2
        ),
        Err(StillError::IntegrationLimit)
    );
    let full = ethanol_water_still_with_limit(
        1.0,
        1.0,
        StillTake::Fraction(0.1),
        1,
        ATMOSPHERE_KPA,
        phase,
        2048,
    )
    .unwrap();
    assert!(((full.water_over + full.ethanol_over) / 0.2 - 1.0).abs() < 2e-12);
}

#[test]
fn request_residue_and_energy_boundaries_are_distinct() {
    let tiny = f64::from_bits(2);
    let nearly_one = f64::from_bits(1.0f64.to_bits() - 1);
    assert_eq!(
        pure_still_amount_checked(tiny, 40.0, StillTake::Fraction(nearly_one)),
        Err(StillError::UnrepresentableResidue)
    );
    assert_eq!(
        pure_still_amount_checked(1.0, 40.0, StillTake::EnergyKj(f64::from_bits(1))),
        Err(StillError::UnrepresentableRequest)
    );
    assert_eq!(
        pure_still_amount_checked(1e308, 40.0, StillTake::Fraction(0.5)),
        Err(StillError::UnrepresentableEnergy)
    );
    let result = ethanol_water_still_with_phase(
        tiny,
        tiny,
        StillTake::Fraction(0.5),
        1,
        ATMOSPHERE_KPA,
        |x, _| {
            Some(BubblePoint {
                t_celsius: 90.0,
                y: x.to_vec(),
                azeotropic: false,
            })
        },
    );
    assert_eq!(result, Err(StillError::UnrepresentableRequest));
}

#[test]
fn excessive_stage_counts_refuse_before_any_phase_work() {
    for stages in [MAX_STILL_STAGES + 1, u32::MAX] {
        for (water, ethanol) in [(1.0, 0.0), (1.0, 1.0)] {
            assert_eq!(
                ethanol_water_still_with_phase(
                    water,
                    ethanol,
                    StillTake::Fraction(0.1),
                    stages,
                    ATMOSPHERE_KPA,
                    |_, _| panic!("invalid stage counts must refuse before phase evaluation")
                ),
                Err(StillError::InvalidInput)
            );
        }
    }
    assert_eq!(
        ethanol_water_still_checked(1.0, 0.0, StillTake::Fraction(0.1), 0, ATMOSPHERE_KPA),
        ethanol_water_still_checked(1.0, 0.0, StillTake::Fraction(0.1), 1, ATMOSPHERE_KPA)
    );
    assert!(ethanol_water_still_checked(
        1.0,
        0.0,
        StillTake::Fraction(0.1),
        MAX_STILL_STAGES,
        ATMOSPHERE_KPA
    )
    .is_ok());
}
