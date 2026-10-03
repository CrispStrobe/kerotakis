//! Synthetic activity laws isolate convergence from empirical property data.
use kerotakis_thermo::vle::*;

fn constant_pressure(pressure: f64) -> VapourPressure {
    VapourPressure::Antoine(Antoine {
        a: pressure.log10(),
        b: 0.0,
        c: 1.0,
        valid_c: (1.0, 100.0),
        source: "synthetic convergence control; no physical property claim",
    })
}

fn relative(actual: f64, expected: f64) {
    assert!(actual.is_finite() && actual > 0.0);
    assert!(
        (actual / expected - 1.0).abs() < 1e-8,
        "actual={actual:e}, expected={expected:e}"
    );
}

#[test]
fn dew_trace_reaches_its_independent_activity_fixed_point() {
    for trace in [1e-12, 1e-100] {
        let mut calls = 0;
        let result = dew_point_with(
            &[WATER, WATER],
            &[trace, 1.0],
            ATMOSPHERE_KPA,
            &mut |x, _| {
                calls += 1;
                vec![8.0 * (x[0] / trace).sqrt(), 1.0]
            },
        )
        .unwrap();
        // Equal saturation pressures give x0*gamma0/x1 = y0/y1.
        // In the trace limit r=x0/y0 solves 8*r^(3/2)=1, hence r=1/4.
        relative(result.x[0], trace * 0.25);
        relative(
            result.x[0] * 8.0 * (result.x[0] / trace).sqrt() / result.x[1],
            trace,
        );
        assert!(
            calls > 100,
            "one absolute-small substitution is not convergence"
        );
    }
}

#[test]
fn vapor_only_flash_trace_reaches_its_independent_fixed_point() {
    for trace in [1e-12, 1e-100] {
        let p = constant_pressure(2.0);
        let result = tp_flash_with(&[p, p], &[trace, 1.0], 1.0, 50.0, &mut |x, _| {
            vec![4.0 * (x[0] / trace).sqrt(), 1.0]
        })
        .unwrap();
        assert_eq!(result.vapour_fraction, 1.0);
        // Dew-implied normalization gives r=1/(4*sqrt(r)).
        relative(result.x[0], trace * 4.0_f64.powf(-2.0 / 3.0));
        relative(result.k[0], 8.0 * (result.x[0] / trace).sqrt());
        relative(result.y[0], trace);
    }
}

#[test]
fn two_phase_flash_trace_reaches_its_independent_fixed_point() {
    for trace in [1e-12, 1e-100] {
        let p = constant_pressure(1.0);
        let result = tp_flash_with(&[p, p, p], &[trace, 0.5, 0.5], 1.0, 50.0, &mut |x, _| {
            vec![14.0 * (x[0] / trace).sqrt(), 0.5, 2.0]
        })
        .unwrap();
        assert!((result.vapour_fraction - 0.5).abs() < 1e-8);
        // Carrier K=(1/2,2) implies V=1/2. The trace equation is
        // r*(1/2 + 7*sqrt(r))=1, whose positive root is r=1/4.
        relative(result.x[0], trace * 0.25);
        relative(result.k[0], 14.0 * (result.x[0] / trace).sqrt());
        for (i, feed) in [trace, 0.5, 0.5].iter().enumerate() {
            relative(
                (1.0 - result.vapour_fraction) * result.x[i] + result.vapour_fraction * result.y[i],
                *feed,
            );
        }
    }
}

#[test]
fn oscillating_dew_trace_refuses_despite_absolute_smallness() {
    let trace = 1e-100;
    assert!(dew_point_with(
        &[WATER, WATER],
        &[trace, 1.0],
        ATMOSPHERE_KPA,
        &mut |x, _| vec![if x[0] / trace > 0.5 { 8.0 } else { 0.125 }, 1.0]
    )
    .is_none());
}

#[test]
fn oscillating_vapor_flash_trace_refuses_despite_absolute_smallness() {
    let trace = 1e-100;
    let p = constant_pressure(2.0);
    assert!(
        tp_flash_with(&[p, p], &[trace, 1.0], 1.0, 50.0, &mut |x, _| vec![
            if x[0] / trace > 0.5 { 8.0 } else { 0.125 },
            1.0
        ])
        .is_none()
    );
}

fn discontinuous_gamma(x: f64, trace: f64) -> f64 {
    // A candidate can move less than the relative tolerance yet cross an
    // activity-law discontinuity. Verify gamma at the returned candidate.
    if (x / trace - 1.0).abs() < 1e-12 {
        1.0 + 1e-11
    } else {
        2.0
    }
}

#[test]
fn dew_checks_activity_at_the_composition_it_returns() {
    let trace = 1e-100;
    let result = dew_point_with(
        &[WATER, WATER],
        &[trace, 1.0],
        ATMOSPHERE_KPA,
        &mut |x, _| vec![discontinuous_gamma(x[0], trace), 1.0],
    )
    .unwrap();
    relative(result.x[0], trace / 2.0);
}

#[test]
fn vapor_flash_checks_activity_at_the_composition_it_returns() {
    let trace = 1e-100;
    let p = constant_pressure(2.0);
    let result = tp_flash_with(&[p, p], &[trace, 1.0], 1.0, 50.0, &mut |x, _| {
        vec![discontinuous_gamma(x[0], trace), 1.0]
    })
    .unwrap();
    relative(result.x[0], trace / 2.0);
    relative(result.k[0], 4.0);
}

#[test]
fn two_phase_flash_checks_activity_at_the_composition_it_returns() {
    let trace = 1e-100;
    let p = constant_pressure(1.0);
    let result = tp_flash_with(&[p, p, p], &[trace, 0.5, 0.5], 1.0, 50.0, &mut |x, _| {
        vec![discontinuous_gamma(x[0], trace), 0.5, 2.0]
    })
    .unwrap();
    relative(result.x[0], trace * 2.0 / 3.0);
    relative(result.k[0], 2.0);
}
