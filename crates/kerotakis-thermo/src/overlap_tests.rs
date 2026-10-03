use super::*;

#[test]
fn overlap_join_matches_independent_formula_and_solver_evaluator() {
    assert_eq!(ETHANOL.valid_range(), Some((-57.0, 151.95)));
    for i in 0..=100 {
        let t = 79.65 + 0.35 * i as f64 / 100.0;
        let s = ((t - 79.65) / (80.0 - 79.65)).clamp(0.0, 1.0);
        let w = s * s * (3.0 - 2.0 * s);
        let low = ETHANOL_LOW.pressure_kpa(t).unwrap();
        let high = ETHANOL_HIGH.pressure_kpa(t).unwrap();
        let expected = ((1.0 - w) * low.ln() + w * high.ln()).exp();
        let actual = ETHANOL.pressure_kpa(t).unwrap();
        assert!((actual / expected - 1.0).abs() < 2e-14);
        assert_eq!(actual, ETHANOL.pressure_kpa_unchecked(t));
        assert!(actual >= low.min(high) * (1.0 - 2e-14));
        assert!(actual <= low.max(high) * (1.0 + 2e-14));
        if i > 0 {
            assert!(
                actual
                    > ETHANOL
                        .pressure_kpa(79.65 + 0.35 * (i - 1) as f64 / 100.0)
                        .unwrap()
            );
        }
    }
    for (t, fit) in [
        (-57.0, ETHANOL_LOW),
        (20.0, ETHANOL_LOW),
        (79.65, ETHANOL_LOW),
        (80.0, ETHANOL_HIGH),
        (120.0, ETHANOL_HIGH),
        (151.95, ETHANOL_HIGH),
    ] {
        assert_eq!(ETHANOL.pressure_kpa(t), fit.pressure_kpa(t));
    }
    for t in [-57.01, 151.96, f64::NAN, f64::INFINITY] {
        assert!(ETHANOL.pressure_kpa(t).is_none());
    }
    let legacy = VapourPressure::Piecewise(ETHANOL_SEGMENTS);
    let pressure =
        0.5 * (ETHANOL_LOW.pressure_kpa(80.0).unwrap() + ETHANOL_HIGH.pressure_kpa(80.0).unwrap());
    assert!(bubble_point_with(&[legacy], &[1.0], pressure, |_| vec![1.0]).is_none());
}

#[test]
fn overlap_join_refuses_nonmonotone_interior_disagreement_and_invalid_domains() {
    const NARROW: &[Antoine] = &[
        Antoine {
            a: 1.0,
            b: 100.0,
            c: 100.0,
            valid_c: (0.0, 10.0),
            source: "synthetic",
        },
        Antoine {
            a: 0.999,
            b: 100.0,
            c: 100.0,
            valid_c: (9.99, 20.0),
            source: "synthetic",
        },
    ];
    const INTERIOR: &[Antoine] = &[
        Antoine {
            a: 1.0,
            b: 4.0,
            c: 200.0,
            valid_c: (-20.0, 10.0),
            source: "synthetic",
        },
        Antoine {
            a: 0.99438,
            b: 1.0,
            c: 100.0,
            valid_c: (-10.0, 20.0),
            source: "synthetic",
        },
    ];
    for t in [-10.0, 10.0] {
        let (p, q) = (
            INTERIOR[0].pressure_kpa(t).unwrap(),
            INTERIOR[1].pressure_kpa(t).unwrap(),
        );
        assert!((p - q).abs() / p.max(q) < 0.01);
    }
    let (p, q) = (
        INTERIOR[0].pressure_kpa(0.0).unwrap(),
        INTERIOR[1].pressure_kpa(0.0).unwrap(),
    );
    assert!((p - q).abs() / p.max(q) > 0.01);
    for fits in [NARROW, INTERIOR] {
        let model = VapourPressure::Blended(fits);
        assert!(model.valid_range().is_none());
        assert!(model.pressure_kpa(10.0).is_none());
        assert!(bubble_point_with(&[model], &[1.0], 1.0, |_| vec![1.0]).is_none());
    }
    let template = NARROW[0];
    for fits in [
        vec![],
        vec![Antoine { b: 0.0, ..template }],
        vec![Antoine {
            c: -100.0,
            ..template
        }],
        vec![Antoine {
            a: f64::NAN,
            ..template
        }],
        vec![
            template,
            Antoine {
                valid_c: (10.0, 20.0),
                ..template
            },
        ],
        vec![
            template,
            Antoine {
                valid_c: (11.0, 20.0),
                ..template
            },
        ],
        vec![
            template,
            Antoine {
                valid_c: (9.0, 20.0),
                ..template
            },
            Antoine {
                valid_c: (9.5, 30.0),
                ..template
            },
        ],
    ] {
        assert!(
            blended_valid_range(&fits).is_none(),
            "invalid fits={fits:?}"
        );
    }
    // A broad, gently downward disagreement is still safely increasing.
    assert!(blended_valid_range(&[
        template,
        Antoine {
            a: 0.999,
            valid_c: (5.0, 20.0),
            ..template
        }
    ])
    .is_some());
}

#[test]
fn legacy_e110_phase_failure_is_inside_the_ethanol_switch_gap() {
    let legacy = VapourPressure::Piecewise(ETHANOL_SEGMENTS);
    let mut failed_x = None;
    let result = ethanol_water_still_with_phase(
        5.534276991396059,
        0.34252968373526665,
        StillTake::Fraction(0.1),
        4,
        ATMOSPHERE_KPA,
        |x, pressure| {
            let answer = bubble_point_with(&[legacy, WATER], &x, pressure, |tk| {
                let (ge, gw) = ethanol_water_activity_pair(x, tk);
                vec![ge, gw]
            });
            if answer.is_none() && failed_x.is_none() {
                failed_x = Some(x);
            }
            answer
        },
    );
    assert_eq!(result, Err(StillError::PhaseEvaluation));
    let x = failed_x.expect("capture the first unavailable phase root");
    let (ge, gw) = ethanol_water_activity_pair(x, 80.0 + KELVIN_OFFSET);
    let water = x[1] * gw * WATER.pressure_kpa(80.0).unwrap();
    let below = water + x[0] * ge * ETHANOL_LOW.pressure_kpa(80.0).unwrap();
    let above = water + x[0] * ge * ETHANOL_HIGH.pressure_kpa(80.0).unwrap();
    assert!(
        below < ATMOSPHERE_KPA && ATMOSPHERE_KPA < above,
        "first failure x={x:?}, switch pressures={below},{above}"
    );
    assert!((below / ATMOSPHERE_KPA - 1.0).abs() > 1e-8);
    assert!((above / ATMOSPHERE_KPA - 1.0).abs() > 1e-8);
    eprintln!("E110 legacy first failed x={x:?}; switch pressures={below},{above} kPa");
    let repaired = bubble_point_with(&[ETHANOL, WATER], &x, ATMOSPHERE_KPA, |tk| {
        let (ge, gw) = ethanol_water_activity_pair(x, tk);
        vec![ge, gw]
    })
    .expect("joined correlation must supply the captured missing E110 root");
    assert!((79.65..=80.0).contains(&repaired.t_celsius));
}

#[test]
fn ordinary_binary_cuts_cross_overlap_but_real_domain_exit_still_refuses() {
    for stages in [1, 2] {
        let cut =
            ethanol_water_still_checked(0.5, 0.5, StillTake::Fraction(0.1), stages, ATMOSPHERE_KPA)
                .expect("ordinary complete cut must cross the numerical join");
        assert!(((cut.water_over + cut.ethanol_over) / 0.1 - 1.0).abs() < 1e-8);
        assert!(cut.water_over > 0.0 && cut.ethanol_over > 0.0);
        assert!(ethanol_water_bubble_point(0.5, 150.0).is_some());
        assert_eq!(
            ethanol_water_still_checked(0.5, 0.5, StillTake::Fraction(0.99), stages, 150.0),
            Err(StillError::PhaseEvaluation)
        );
    }
    let e110 = ethanol_water_still_checked(
        5.534276991396059,
        0.34252968373526665,
        StillTake::Fraction(0.1),
        4,
        ATMOSPHERE_KPA,
    );
    eprintln!("E110 with joined correlation: {e110:?}");
    // Exploratory downstream owner; no complete-cut prediction is made here.
    assert_ne!(e110, Err(StillError::PhaseEvaluation));
}
