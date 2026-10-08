//! Pure binary cuts have an analytic material and latent-energy account.
use kerotakis_thermo::vle::{ethanol_water_bubble_point, ethanol_water_still, StillTake};

#[test]
fn pure_water_and_ethanol_match_analytic_fraction_and_energy_at_all_scales() {
    for (is_ethanol, latent) in [(false, 40.657), (true, 38.58)] {
        for amount in [1e-150, 1e-12, 1.0, 1e150] {
            for stages in [1, 4, 128] {
                for fraction in [0.0, 1e-14, 0.2, 0.9, 1.0] {
                    let expected = amount * fraction;
                    for take in [
                        StillTake::Fraction(fraction),
                        StillTake::EnergyKj(expected * latent),
                    ] {
                        let (water, ethanol) = if is_ethanol {
                            (0.0, amount)
                        } else {
                            (amount, 0.0)
                        };
                        let cut =
                            ethanol_water_still(water, ethanol, take, stages, 101.325).unwrap();
                        let transferred = if is_ethanol {
                            cut.ethanol_over
                        } else {
                            cut.water_over
                        };
                        if expected == 0.0 {
                            assert_eq!(transferred, 0.0);
                            assert_eq!(cut.energy_kj, 0.0);
                        } else {
                            assert!((transferred / expected - 1.0).abs() < 2e-14);
                            assert!((cut.energy_kj / (expected * latent) - 1.0).abs() < 2e-14);
                        }
                        assert_eq!(
                            if is_ethanol {
                                cut.water_over
                            } else {
                                cut.ethanol_over
                            },
                            0.0
                        );
                        assert_eq!(cut.t_start_c, cut.t_end_c);
                        assert!(!cut.azeotrope_limited);
                    }
                }
            }
        }
    }
}

#[test]
fn pure_full_and_excess_energy_cuts_publish_exact_inventory_and_latent_demand() {
    for (water, ethanol, latent) in [(2.0, 0.0, 40.657), (0.0, 2.0, 38.58)] {
        for take in [
            StillTake::Fraction(1.0),
            StillTake::EnergyKj(200.0),
            StillTake::EnergyKj(f64::MAX),
        ] {
            let cut = ethanol_water_still(water, ethanol, take, 128, 101.325).unwrap();
            assert_eq!(cut.water_over, water);
            assert_eq!(cut.ethanol_over, ethanol);
            assert_eq!(cut.energy_kj, 2.0 * latent);
        }
    }
}

#[test]
fn overflowing_full_latent_heat_still_allows_finite_partial_energy() {
    for (water, ethanol, latent) in [(f64::MAX, 0.0, 40.657), (0.0, f64::MAX, 38.58)] {
        assert!(
            ethanol_water_still(water, ethanol, StillTake::Fraction(1.0), 1, 101.325).is_none()
        );
        let cut =
            ethanol_water_still(water, ethanol, StillTake::EnergyKj(latent), 128, 101.325).unwrap();
        assert_eq!(cut.water_over + cut.ethanol_over, 1.0);
        assert_eq!(cut.energy_kj, latent);
    }
}

#[test]
fn zero_or_analytic_take_never_bypasses_the_phase_domain() {
    for (water, ethanol, x) in [(1.0, 0.0, 0.0), (0.0, 1.0, 1.0)] {
        for pressure in [0.0, -1.0, f64::NAN, f64::INFINITY, 1e9] {
            assert!(ethanol_water_bubble_point(x, pressure).is_none());
            for take in [
                StillTake::Fraction(0.0),
                StillTake::Fraction(0.2),
                StillTake::EnergyKj(1.0),
            ] {
                assert!(ethanol_water_still(water, ethanol, take, 128, pressure).is_none());
            }
        }
    }
}

#[test]
fn energy_rounding_never_claims_more_heat_than_supplied() {
    for latent in [40.657, 38.58] {
        for energy in [1e-200, 0.03, 1.0, 3.7, 17.0, latent * 0.9] {
            let (water, ethanol) = if latent == 40.657 {
                (1.0, 0.0)
            } else {
                (0.0, 1.0)
            };
            let cut = ethanol_water_still(water, ethanol, StillTake::EnergyKj(energy), 1, 101.325)
                .unwrap();
            assert!(cut.energy_kj <= energy);
            assert!((cut.energy_kj / energy - 1.0).abs() <= 8.0 * f64::EPSILON);
        }
    }
}
