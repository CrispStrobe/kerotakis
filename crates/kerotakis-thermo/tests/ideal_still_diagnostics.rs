//! Independent checked refusal categories and analytic accepted-cut controls.
use kerotakis_thermo::batch::{ideal_still, ideal_still_checked, ConstantLatent};
use kerotakis_thermo::vle::{StillError, StillTake};

fn model(boiling_k: f64) -> ConstantLatent {
    ConstantLatent {
        boiling_k,
        latent_kj_mol: 40.0,
        valid_k: (280.0, 430.0),
    }
}

#[test]
fn invalid_inputs_are_distinct_from_representability_and_phase_failures() {
    for inventory in [
        vec![],
        vec![0.0],
        vec![-1.0],
        vec![f64::NAN],
        vec![f64::INFINITY],
    ] {
        let models = vec![model(350.0); inventory.len()];
        assert!(matches!(
            ideal_still_checked(&inventory, &models, StillTake::Fraction(0.1), 1, 101.325),
            Err(StillError::InvalidInput)
        ));
    }
    for (stages, pressure) in [(0, 101.325), (129, 101.325), (1, 0.0), (1, f64::NAN)] {
        assert!(matches!(
            ideal_still_checked(
                &[1.0],
                &[model(350.0)],
                StillTake::Fraction(0.1),
                stages,
                pressure
            ),
            Err(StillError::InvalidInput)
        ));
    }
    let invalid = ConstantLatent {
        latent_kj_mol: f64::NAN,
        ..model(350.0)
    };
    assert!(matches!(
        ideal_still_checked(
            &[1.0, 0.0],
            &[model(350.0), invalid],
            StillTake::Fraction(0.1),
            1,
            101.325
        ),
        Err(StillError::InvalidInput)
    ));
}

#[test]
fn independent_precision_and_model_boundaries_have_specific_categories() {
    for (inventory, fraction, pressure, expected) in [
        (
            vec![1e100, 1e-300],
            0.1,
            101.325,
            StillError::UnrepresentableComposition,
        ),
        (
            vec![f64::MAX, f64::MAX],
            0.1,
            101.325,
            StillError::UnrepresentableComposition,
        ),
        (
            vec![f64::from_bits(1)],
            0.5,
            101.325,
            StillError::UnrepresentableRequest,
        ),
        (vec![1e307], 0.5, 101.325, StillError::UnrepresentableEnergy),
        (vec![1.0], 0.1, 1e9, StillError::PhaseEvaluation),
    ] {
        let models = vec![model(350.0); inventory.len()];
        let before = inventory.clone();
        let checked = ideal_still_checked(
            &inventory,
            &models,
            StillTake::Fraction(fraction),
            1,
            pressure,
        );
        assert!(
            matches!(checked,Err(error) if error == expected),
            "{expected:?}: {checked:?}"
        );
        assert!(ideal_still(
            &inventory,
            &models,
            StillTake::Fraction(fraction),
            1,
            pressure
        )
        .is_none());
        assert_eq!(inventory, before);
    }
}

#[test]
fn checked_and_legacy_apis_preserve_analytic_pure_and_mixture_acceptance() {
    for inventory in [vec![2.0], vec![0.5, 1.5]] {
        let models = vec![model(350.0); inventory.len()];
        for take in [
            StillTake::Fraction(0.0),
            StillTake::Fraction(0.1),
            StillTake::Fraction(1.0),
            StillTake::EnergyKj(8.0),
        ] {
            let checked = ideal_still_checked(&inventory, &models, take, 2, 101.325).unwrap();
            let legacy = ideal_still(&inventory, &models, take, 2, 101.325).unwrap();
            assert_eq!(checked.overhead, legacy.overhead);
            assert_eq!(checked.energy_kj, legacy.energy_kj);
            assert_eq!(checked.t_start_k, legacy.t_start_k);
            assert_eq!(checked.t_end_k, legacy.t_end_k);
            let fraction = match take {
                StillTake::Fraction(f) => f,
                StillTake::EnergyKj(e) => e / 80.0,
            };
            for (removed, initial) in checked.overhead.iter().zip(&inventory) {
                assert!((removed - initial * fraction).abs() <= 1e-10);
            }
            assert!((checked.energy_kj - 80.0 * fraction).abs() <= 1e-8);
        }
    }
}
