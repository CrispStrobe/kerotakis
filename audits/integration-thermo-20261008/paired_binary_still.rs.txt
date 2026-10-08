//! Pair-coordinate phase checks and independent infinite-dilution Rayleigh laws.
use kerotakis_thermo::vle::*;

#[test]
fn water_trace_survives_even_when_ethanol_fraction_rounds_to_one() {
    let pure = ethanol_water_bubble_point(1.0, ATMOSPHERE_KPA).unwrap();
    let (_, gamma_water) = ethanol_water_activity(1.0, pure.t_celsius + KELVIN_OFFSET);
    for water in [1e-14, 1e-20, 1e-100] {
        let bp = ethanol_water_bubble_point_from_moles(1.0, water, ATMOSPHERE_KPA).unwrap();
        let expected =
            water * gamma_water * WATER.pressure_kpa(bp.t_celsius).unwrap() / ATMOSPHERE_KPA;
        assert!(bp.y[1] > 0.0 && (bp.y[1] / expected - 1.0).abs() < 1e-8);
        assert!((bp.t_celsius - pure.t_celsius).abs() < 1e-8);
    }
}

#[test]
fn trace_cuts_follow_an_independent_rayleigh_limit_in_both_orientations() {
    for water_trace in [true, false] {
        let x_bulk = if water_trace { 1.0 } else { 0.0 };
        let pure = ethanol_water_bubble_point(x_bulk, ATMOSPHERE_KPA).unwrap();
        let (ge, gw) = ethanol_water_activity(x_bulk, pure.t_celsius + KELVIN_OFFSET);
        let alpha = if water_trace {
            gw * WATER.pressure_kpa(pure.t_celsius).unwrap() / ATMOSPHERE_KPA
        } else {
            ge * ETHANOL.pressure_kpa(pure.t_celsius).unwrap() / ATMOSPHERE_KPA
        };
        let fraction: f64 = if water_trace { 0.01 } else { 1e-4 };
        let stages_to_check: &[u32] = if water_trace { &[1, 4] } else { &[1] };
        for trace in [1e-14, 1e-20, 1e-100] {
            let (water, ethanol) = if water_trace {
                (trace, 1.0)
            } else {
                (1.0, trace)
            };
            for stages in stages_to_check {
                let cut = ethanol_water_still(
                    water,
                    ethanol,
                    StillTake::Fraction(fraction),
                    *stages,
                    ATMOSPHERE_KPA,
                )
                .unwrap();
                let removed = if water_trace {
                    cut.water_over
                } else {
                    cut.ethanol_over
                };
                // d(trace)/d(bulk) = alpha^stages * trace/bulk at infinite
                // dilution. Integrating gives trace_residue/trace0=(1-f)^a.
                let expected = trace * -(alpha.powi(*stages as i32) * (-fraction).ln_1p()).exp_m1();
                assert!(removed > 0.0 && removed < trace);
                // First-order 1024-mesh integration need not equal the
                // continuous analytic solution; compare at its mesh scale.
                assert!((removed/expected-1.0).abs() < 1e-3,
                    "water_trace={water_trace}, stages={stages}, removed={removed:e}, expected={expected:e}");
                assert!(
                    ((cut.water_over + cut.ethanol_over) / ((water + ethanol) * fraction) - 1.0)
                        .abs()
                        < 2e-12
                );
                let latent = cut.water_over * WATER_HVAP_KJ_PER_MOL
                    + cut.ethanol_over * ETHANOL_HVAP_KJ_PER_MOL;
                assert!((cut.energy_kj / latent - 1.0).abs() < 2e-12);
                let endpoint = ethanol_water_bubble_point_from_moles(
                    ethanol - cut.ethanol_over,
                    water - cut.water_over,
                    ATMOSPHERE_KPA,
                )
                .unwrap();
                assert!((cut.t_end_c - endpoint.t_celsius).abs() < 1e-9);
            }
        }
    }
}

#[test]
fn paired_bubble_retains_scalar_compatibility_and_refuses_true_underflow() {
    for x in [0.0, 0.05, 0.5, 0.894, 0.95, 1.0] {
        let scalar = ethanol_water_bubble_point(x, ATMOSPHERE_KPA);
        let pair = ethanol_water_bubble_point_from_moles(x, 1.0 - x, ATMOSPHERE_KPA);
        assert_eq!(scalar, pair);
    }
    for (e, w) in [
        (1e300, 1e-300),
        (1e-300, 1e300),
        (f64::MAX, f64::MAX),
        (-1.0, 1.0),
        (0.0, 0.0),
    ] {
        assert!(ethanol_water_bubble_point_from_moles(e, w, ATMOSPHERE_KPA).is_none());
    }
}
