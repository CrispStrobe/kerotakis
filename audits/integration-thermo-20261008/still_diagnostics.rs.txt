//! Public API compatibility and independently specified boundary categories.
use kerotakis_thermo::vle::*;

#[test]
fn checked_and_option_apis_publish_identical_cuts_and_refusals() {
    for (water, ethanol, take, stages, pressure) in [
        (1.0, 0.0, StillTake::Fraction(0.4), 128, ATMOSPHERE_KPA),
        (0.0, 1.0, StillTake::EnergyKj(0.1), 4, ATMOSPHERE_KPA),
        (0.5, 0.5, StillTake::Fraction(0.01), 2, ATMOSPHERE_KPA),
        (1e-100, 1.0, StillTake::Fraction(0.01), 4, ATMOSPHERE_KPA),
        (0.5, 0.5, StillTake::Fraction(0.1), 1, ATMOSPHERE_KPA),
        (1e-300, 1e100, StillTake::Fraction(0.1), 4, ATMOSPHERE_KPA),
        (1.0, 0.0, StillTake::Fraction(0.1), 1, 1e9),
        (1.0, 1.0, StillTake::Fraction(0.0), 1, ATMOSPHERE_KPA),
    ] {
        assert_eq!(
            ethanol_water_still_checked(water, ethanol, take, stages, pressure).ok(),
            ethanol_water_still(water, ethanol, take, stages, pressure)
        );
    }
}

#[test]
fn invalid_inputs_and_numerical_limits_are_not_all_phase_failures() {
    for (water, ethanol, take, pressure, expected) in [
        (
            -1.0,
            1.0,
            StillTake::Fraction(0.1),
            ATMOSPHERE_KPA,
            StillError::InvalidInput,
        ),
        (
            1.0,
            1.0,
            StillTake::Fraction(f64::NAN),
            ATMOSPHERE_KPA,
            StillError::InvalidInput,
        ),
        (
            1.0,
            1.0,
            StillTake::Fraction(0.1),
            -1.0,
            StillError::InvalidInput,
        ),
        (
            1.0,
            1.0,
            StillTake::Fraction(0.1),
            f64::NAN,
            StillError::InvalidInput,
        ),
        (
            1e-300,
            1e100,
            StillTake::Fraction(0.1),
            ATMOSPHERE_KPA,
            StillError::UnrepresentableComposition,
        ),
        (
            1e-300,
            1e-300,
            StillTake::Fraction(1e-30),
            ATMOSPHERE_KPA,
            StillError::UnrepresentableRequest,
        ),
        (
            1.0,
            0.0,
            StillTake::EnergyKj(f64::from_bits(1)),
            ATMOSPHERE_KPA,
            StillError::UnrepresentableRequest,
        ),
        (
            1e308,
            0.0,
            StillTake::Fraction(0.5),
            ATMOSPHERE_KPA,
            StillError::UnrepresentableEnergy,
        ),
        (
            1.0,
            0.0,
            StillTake::Fraction(0.1),
            1e9,
            StillError::PhaseEvaluation,
        ),
    ] {
        let result = ethanol_water_still_checked(water, ethanol, take, 4, pressure);
        assert_eq!(result, Err(expected));
        assert!(!expected.code().is_empty());
        assert!(!expected.to_string().is_empty());
    }
}
