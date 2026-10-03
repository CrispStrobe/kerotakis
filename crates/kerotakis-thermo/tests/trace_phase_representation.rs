//! Positive volatiles must survive every phase normalization and cut.
use kerotakis_thermo::batch::{ideal_still, ConstantLatent};
use kerotakis_thermo::vle::*;

fn model() -> ConstantLatent {
    ConstantLatent {
        boiling_k: 350.0,
        latent_kj_mol: 40.0,
        valid_k: (300.0, 400.0),
    }
}

fn constant_pressure(a: f64) -> VapourPressure {
    VapourPressure::Antoine(Antoine {
        a,
        b: 0.0,
        c: 1.0,
        valid_c: (1.0, 100.0),
        source: "synthetic arithmetic control; no physical property claim",
    })
}

#[test]
fn partial_pressure_and_final_vapor_underflow_are_separate_refusals() {
    assert!(bubble_point_with(
        &[constant_pressure(-300.0), constant_pressure(0.0)],
        &[1e-100, 1.0],
        1.0,
        |_| vec![1.0, 1.0]
    )
    .is_none());
    assert!(bubble_point_with(
        &[constant_pressure(0.0), constant_pressure(300.0)],
        &[0.5, 0.5],
        5e299,
        |_| vec![1.0, 1.0]
    )
    .is_none());
}

#[test]
fn dew_and_flash_refuse_lost_feed_coordinates() {
    for feed in [[f64::MAX, f64::MAX], [1e300, 1e-300], [1e-300, 1e300]] {
        assert!(
            dew_point_with(&[ETHANOL, WATER], &feed, ATMOSPHERE_KPA, &mut |_, _| vec![
                1.0, 1.0
            ])
            .is_none()
        );
        assert!(tp_flash_with(
            &[ETHANOL, WATER],
            &feed,
            ATMOSPHERE_KPA,
            80.0,
            &mut |_, _| vec![1.0, 1.0]
        )
        .is_none());
    }
}

#[test]
fn unrepresentable_flash_coefficients_refuse_instead_of_publishing_nan() {
    for a in [-400.0, 400.0] {
        assert!(tp_flash_with(
            &[constant_pressure(a)],
            &[1.0],
            1.0,
            50.0,
            &mut |_, _| vec![1.0]
        )
        .is_none());
    }
    assert!(tp_flash_with(
        &[constant_pressure(-300.0), constant_pressure(0.0)],
        &[1e-100, 1.0],
        2.0,
        50.0,
        &mut |_, _| vec![1.0, 1.0]
    )
    .is_none());
    assert!(tp_flash_with(
        &[constant_pressure(300.0), constant_pressure(0.0)],
        &[1e-100, 1.0],
        0.5,
        50.0,
        &mut |_, _| vec![1.0, 1.0]
    )
    .is_none());
}

#[test]
fn dew_and_flash_keep_representable_traces_in_both_orientations() {
    for feed in [[1.0, 1e-100], [1e-100, 1.0]] {
        let dew = dew_point_with(&[ETHANOL, WATER], &feed, ATMOSPHERE_KPA, &mut |_, _| {
            vec![1.0, 1.0]
        })
        .unwrap();
        for (i, correlation) in [ETHANOL, WATER].iter().enumerate() {
            let expected = feed[i] / feed.iter().sum::<f64>() * ATMOSPHERE_KPA
                / correlation.pressure_kpa(dew.t_celsius).unwrap();
            assert!(dew.x[i] > 0.0 && (dew.x[i] / expected - 1.0).abs() < 1e-8);
        }
        let flash = tp_flash_with(
            &[ETHANOL, WATER],
            &feed,
            ATMOSPHERE_KPA,
            80.0,
            &mut |_, _| vec![1.0, 1.0],
        )
        .unwrap();
        assert!(flash
            .x
            .iter()
            .chain(&flash.y)
            .all(|v| v.is_finite() && *v > 0.0));
        for i in 0..2 {
            let expected = feed[i] / feed.iter().sum::<f64>();
            let restored =
                (1.0 - flash.vapour_fraction) * flash.x[i] + flash.vapour_fraction * flash.y[i];
            assert!((restored / expected - 1.0).abs() < 1e-8);
        }
    }
}

#[test]
fn overflowing_or_underflowing_bubble_composition_refuses() {
    for x in [[f64::MAX, f64::MAX], [1e300, 1e-300], [1e-300, 1e300]] {
        assert!(
            bubble_point_with(&[ETHANOL, WATER], &x, ATMOSPHERE_KPA, |_| vec![1.0, 1.0]).is_none(),
            "{x:?}"
        );
    }
}

#[test]
fn positive_activity_weight_that_underflows_refuses() {
    assert!(
        bubble_point_with(&[ETHANOL, WATER], &[1e-100, 1.0], ATMOSPHERE_KPA, |_| vec![
            f64::from_bits(1),
            1.0
        ])
        .is_none()
    );
}

#[test]
fn independent_trace_fractions_remain_positive_in_bubble_results() {
    for x in [[1.0, 1e-100], [1e-100, 1.0]] {
        let result =
            bubble_point_with(&[ETHANOL, WATER], &x, ATMOSPHERE_KPA, |_| vec![1.0, 1.0]).unwrap();
        assert!(result.y.iter().all(|v| v.is_finite() && *v > 0.0));
        let total = x.iter().sum::<f64>();
        for (i, correlation) in [ETHANOL, WATER].iter().enumerate() {
            let expected =
                x[i] / total * correlation.pressure_kpa(result.t_celsius).unwrap() / ATMOSPHERE_KPA;
            assert!((result.y[i] / expected - 1.0).abs() < 1e-8);
        }
        assert!(!result.azeotropic);
    }
}

#[test]
fn ideal_inventory_normalization_cannot_silently_drop_a_trace() {
    for n in [[1e300, 1e-300], [1e-300, 1e300]] {
        for stages in [1, 4] {
            assert!(ideal_still(
                &n,
                &[model(), model()],
                StillTake::Fraction(0.01),
                stages,
                ATMOSPHERE_KPA
            )
            .is_none());
        }
    }
}

#[test]
fn equal_volatility_traces_follow_an_independent_linear_cut_law() {
    for n in [[1.0, 1e-100], [1e-100, 1.0]] {
        for stages in [1, 4] {
            let cut = ideal_still(
                &n,
                &[model(), model()],
                StillTake::Fraction(0.01),
                stages,
                ATMOSPHERE_KPA,
            )
            .unwrap();
            for (initial, removed) in n.iter().zip(&cut.overhead) {
                assert!((removed / (initial * 0.01) - 1.0).abs() < 2e-12);
            }
            assert!((cut.energy_kj / 0.4 - 1.0).abs() < 2e-12);
        }
    }
}

#[test]
fn binary_scalar_composition_loss_refuses_instead_of_claiming_purity() {
    for (water, ethanol) in [(1e-20, 1.0), (1e300, 1e-300)] {
        for stages in [1, 4] {
            for take in [StillTake::Fraction(0.01), StillTake::EnergyKj(0.1)] {
                assert!(
                    ethanol_water_still(water, ethanol, take, stages, ATMOSPHERE_KPA).is_none()
                );
            }
        }
    }
}

#[test]
fn representable_binary_trace_still_transfers_both_components() {
    let cut =
        ethanol_water_still(1.0, 1e-14, StillTake::Fraction(0.01), 1, ATMOSPHERE_KPA).unwrap();
    assert!(cut.water_over > 0.0 && cut.ethanol_over > 0.0);
    assert!(cut.water_over < 1.0 && cut.ethanol_over < 1e-14);
    assert!(((cut.water_over + cut.ethanol_over) / (0.01 * (1.0 + 1e-14)) - 1.0).abs() < 2e-12);
}
