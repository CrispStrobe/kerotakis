//! Analytic pure-component and independent residual-pressure controls.
use kerotakis_thermo::batch::{ideal_still, ConstantLatent};
use kerotakis_thermo::vle::StillTake;

fn model(boiling_k: f64, latent_kj_mol: f64) -> ConstantLatent {
    ConstantLatent {
        boiling_k,
        latent_kj_mol,
        valid_k: (280.0, 430.0),
    }
}

fn relative(actual: f64, expected: f64) {
    assert!(actual.is_finite() && expected > 0.0);
    assert!(
        (actual / expected - 1.0).abs() <= 2e-12,
        "actual={actual:e}, expected={expected:e}"
    );
}

#[test]
fn positive_fraction_is_completed_at_the_request_scale() {
    for amount in [1e-150, 1.0, 1e150] {
        for fraction in [1e-14, 1e-6, 0.25, 0.9, 1.0] {
            let cut = ideal_still(
                &[amount],
                &[model(350.0, 40.0)],
                StillTake::Fraction(fraction),
                4,
                101.325,
            )
            .unwrap();
            relative(cut.overhead[0], amount * fraction);
            relative(cut.energy_kj, amount * fraction * 40.0);
            assert!((cut.t_start_k - 350.0).abs() < 1e-10);
            assert!((cut.t_end_k - 350.0).abs() < 1e-10);
            if fraction == 1.0 {
                assert_eq!(cut.overhead[0], amount);
            }
        }
    }
}

#[test]
fn zero_and_microscopic_energy_are_not_absolute_tolerance_successes() {
    for amount in [1e-150, 1.0, 1e150] {
        let zero = ideal_still(
            &[amount],
            &[model(350.0, 40.0)],
            StillTake::EnergyKj(0.0),
            2,
            101.325,
        )
        .unwrap();
        assert_eq!(zero.overhead, [0.0]);
        assert_eq!(zero.energy_kj, 0.0);
        assert_eq!(zero.t_start_k, zero.t_end_k);
        for fraction in [1e-14, 1e-6, 0.25] {
            let energy = amount * fraction * 40.0;
            let cut = ideal_still(
                &[amount],
                &[model(350.0, 40.0)],
                StillTake::EnergyKj(energy),
                3,
                101.325,
            )
            .unwrap();
            relative(cut.overhead[0], energy / 40.0);
            relative(cut.energy_kj, energy);
        }
    }
}

#[test]
fn underflowed_positive_requests_refuse_without_mutating_inputs() {
    let inventory = [f64::from_bits(1)];
    let original = inventory;
    assert!(ideal_still(
        &inventory,
        &[model(350.0, 40.0)],
        StillTake::Fraction(0.5),
        1,
        101.325
    )
    .is_none());
    assert_eq!(inventory, original);
    let ordinary = [1.0];
    assert!(ideal_still(
        &ordinary,
        &[model(350.0, 40.0)],
        StillTake::EnergyKj(f64::from_bits(1)),
        1,
        101.325
    )
    .is_none());
    assert_eq!(ordinary, [1.0]);
}

#[test]
fn partial_endpoint_temperature_is_the_final_residual_bubble_point() {
    let inventory = [1.0, 2.0];
    let models = [model(330.0, 30.0), model(390.0, 45.0)];
    for take in [StillTake::Fraction(0.4), StillTake::EnergyKj(20.0)] {
        let cut = ideal_still(&inventory, &models, take, 1, 101.325).unwrap();
        let residual = [
            inventory[0] - cut.overhead[0],
            inventory[1] - cut.overhead[1],
        ];
        assert!(residual.iter().all(|n| *n > 0.0));
        let total: f64 = residual.iter().sum();
        // Independent Raoult/Clausius-Clapeyron pressure evaluation, without
        // using the implementation's private bubble solver.
        let pressure: f64 = residual
            .iter()
            .zip(models)
            .map(|(n, m)| {
                n / total
                    * 101.325
                    * (m.latent_kj_mol / 0.008_314_462_618
                        * (1.0 / m.boiling_k - 1.0 / cut.t_end_k))
                        .exp()
            })
            .sum();
        assert!((pressure / 101.325 - 1.0).abs() < 1e-12);
        assert!(cut.t_end_k > cut.t_start_k);
    }
}

#[test]
fn mixed_full_take_has_exact_overhead_and_inventory_latent_heat() {
    let inventory = [0.2, 4.0, 0.3];
    let models = [model(340.0, 30.0), model(370.0, 40.0), model(355.0, 45.0)];
    let cut = ideal_still(&inventory, &models, StillTake::Fraction(1.0), 2, 101.325).unwrap();
    assert_eq!(cut.overhead, inventory);
    assert_eq!(cut.energy_kj, 0.2 * 30.0 + 4.0 * 40.0 + 0.3 * 45.0);
    assert!(cut.t_end_k.is_finite());
    assert!(cut.t_end_k > cut.t_start_k);
}

#[test]
fn excess_energy_exhausts_inventory_instead_of_claiming_the_supplied_heat() {
    for energy in [80.0, 160.0] {
        let cut = ideal_still(
            &[2.0],
            &[model(350.0, 40.0)],
            StillTake::EnergyKj(energy),
            1,
            101.325,
        )
        .unwrap();
        assert_eq!(cut.overhead, [2.0]);
        assert_eq!(cut.energy_kj, 80.0);
        assert!((cut.t_end_k - 350.0).abs() < 1e-10);
    }
}
