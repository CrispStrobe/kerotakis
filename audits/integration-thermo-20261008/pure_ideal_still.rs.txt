//! Pure-liquid controls use amount/latent-energy algebra independently of the
//! production helper, across scales, component positions, and pressure domains.
use kerotakis_thermo::batch::{ideal_still, ConstantLatent};
use kerotakis_thermo::vle::StillTake;

fn model() -> ConstantLatent {
    ConstantLatent {
        boiling_k: 350.0,
        latent_kj_mol: 40.0,
        valid_k: (280.0, 430.0),
    }
}

fn relative(actual: f64, expected: f64) {
    assert!(actual.is_finite());
    if expected == 0.0 {
        assert_eq!(actual, 0.0);
    } else {
        assert!(
            (actual / expected - 1.0).abs() < 2e-14,
            "actual={actual:e}, expected={expected:e}"
        );
    }
}

#[test]
fn pure_fraction_and_energy_match_analytic_amount_at_every_position_and_scale() {
    for amount in [1e-150, 1e-12, 1.0, 1e150] {
        for position in 0..3 {
            let mut inventory = [0.0; 3];
            inventory[position] = amount;
            for stages in [1, 4, 128] {
                for fraction in [0.0, 1e-14, 0.25, 0.9, 1.0] {
                    let expected = amount * fraction;
                    for take in [
                        StillTake::Fraction(fraction),
                        StillTake::EnergyKj(expected * 40.0),
                    ] {
                        let cut =
                            ideal_still(&inventory, &[model(); 3], take, stages, 101.325).unwrap();
                        for (index, overhead) in cut.overhead.iter().enumerate() {
                            relative(*overhead, if index == position { expected } else { 0.0 });
                        }
                        relative(cut.energy_kj, expected * 40.0);
                        assert_eq!(cut.t_start_k, cut.t_end_k);
                        assert!((cut.t_start_k - 350.0).abs() < 1e-10);
                    }
                }
            }
        }
    }
}

#[test]
fn excess_energy_reports_only_the_inventory_heat_and_exact_full_amount() {
    for energy in [80.0, 160.0, f64::MAX] {
        let cut = ideal_still(
            &[0.0, 2.0],
            &[model(); 2],
            StillTake::EnergyKj(energy),
            128,
            101.325,
        )
        .unwrap();
        assert_eq!(cut.overhead, [0.0, 2.0]);
        assert_eq!(cut.energy_kj, 80.0);
        assert_eq!(cut.t_start_k, cut.t_end_k);
    }
}

#[test]
fn zero_take_still_requires_valid_pressure_stage_and_inactive_model() {
    for take in [StillTake::Fraction(0.0), StillTake::EnergyKj(0.0)] {
        for pressure in [0.0, -1.0, f64::NAN, f64::INFINITY, 1e9] {
            assert!(ideal_still(&[1.0], &[model()], take, 1, pressure).is_none());
        }
        for stages in [0, 129] {
            assert!(ideal_still(&[1.0], &[model()], take, stages, 101.325).is_none());
        }
        for invalid in [
            ConstantLatent {
                latent_kj_mol: f64::NAN,
                ..model()
            },
            ConstantLatent {
                latent_kj_mol: f64::INFINITY,
                ..model()
            },
            ConstantLatent {
                latent_kj_mol: 0.0,
                ..model()
            },
            ConstantLatent {
                valid_k: (430.0, 280.0),
                ..model()
            },
        ] {
            assert!(ideal_still(&[1.0, 0.0], &[model(), invalid], take, 1, 101.325).is_none());
        }
    }
}

#[test]
fn pure_boiling_pressure_is_independently_inverted_and_domain_checked() {
    for temperature in [290.0_f64, 350.0, 420.0] {
        let pressure =
            101.325 * (40.0 / 0.008_314_462_618 * (1.0 / 350.0 - 1.0 / temperature)).exp();
        let cut = ideal_still(&[1.0], &[model()], StillTake::Fraction(0.3), 128, pressure).unwrap();
        assert!((cut.t_start_k - temperature).abs() < 1e-10);
        assert_eq!(cut.t_start_k, cut.t_end_k);
    }
    let excluded = ConstantLatent {
        valid_k: (360.0, 430.0),
        ..model()
    };
    assert!(ideal_still(&[1.0], &[excluded], StillTake::Fraction(0.3), 1, 101.325).is_none());
}

#[test]
fn nonrepresentable_positive_requests_and_outputs_refuse_without_mutating_inputs() {
    let tiny = [f64::from_bits(1)];
    assert!(ideal_still(&tiny, &[model()], StillTake::Fraction(0.5), 1, 101.325).is_none());
    assert_eq!(tiny, [f64::from_bits(1)]);
    assert!(ideal_still(
        &[1.0],
        &[model()],
        StillTake::EnergyKj(f64::from_bits(1)),
        1,
        101.325
    )
    .is_none());
    let huge = [f64::MAX];
    assert!(ideal_still(&huge, &[model()], StillTake::Fraction(1.0), 1, 101.325).is_none());
    assert_eq!(huge, [f64::MAX]);
    // Overflow of the unrequested full heat does not invalidate a finite cut.
    let cut = ideal_still(&huge, &[model()], StillTake::EnergyKj(40.0), 128, 101.325).unwrap();
    assert_eq!(cut.overhead, [1.0]);
    assert_eq!(cut.energy_kj, 40.0);
    let cut = ideal_still(&huge, &[model()], StillTake::Fraction(1e-308), 128, 101.325).unwrap();
    relative(cut.overhead[0], f64::MAX * 1e-308);
    relative(cut.energy_kj, f64::MAX * 1e-308 * 40.0);
}

#[test]
fn invalid_inventory_and_take_do_not_enter_the_pure_shortcut() {
    for inventory in [
        vec![],
        vec![0.0],
        vec![-1.0],
        vec![f64::NAN],
        vec![f64::INFINITY],
    ] {
        assert!(
            ideal_still(&inventory, &[model()], StillTake::Fraction(0.5), 1, 101.325).is_none()
        );
    }
    assert!(ideal_still(&[1.0], &[], StillTake::Fraction(0.5), 1, 101.325).is_none());
    for take in [
        StillTake::Fraction(-0.1),
        StillTake::Fraction(1.1),
        StillTake::Fraction(f64::NAN),
        StillTake::EnergyKj(-1.0),
        StillTake::EnergyKj(f64::INFINITY),
    ] {
        assert!(ideal_still(&[1.0], &[model()], take, 1, 101.325).is_none());
    }
}
