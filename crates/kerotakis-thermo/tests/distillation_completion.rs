//! A fraction cut either completes its requested inventory transfer or refuses.
use kerotakis_thermo::vle::{ethanol_water_still, StillTake};

fn close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 1e-12 * expected,
        "requested cut {expected} mol, reported successful cut {actual} mol"
    );
}

#[test]
fn an_intermediate_model_boundary_cannot_be_a_successful_partial_fraction_cut() {
    // Frozen independent experiment098 exposed different silent early exits
    // for these two stage counts. Either a complete solution or an explicit
    // None is valid; an unlabelled partial success is not.
    for stages in [1, 2] {
        if let Some(cut) = ethanol_water_still(0.5, 0.5, StillTake::Fraction(0.1), stages, 101.325)
        {
            close(cut.water_over + cut.ethanol_over, 0.1);
        }
    }
}

#[test]
fn supported_small_binary_cuts_complete_and_additional_stages_enrich_ethanol() {
    let single = ethanol_water_still(0.5, 0.5, StillTake::Fraction(0.01), 1, 101.325).unwrap();
    let staged = ethanol_water_still(0.5, 0.5, StillTake::Fraction(0.01), 2, 101.325).unwrap();
    close(single.water_over + single.ethanol_over, 0.01);
    close(staged.water_over + staged.ethanol_over, 0.01);
    assert!(staged.ethanol_over > single.ethanol_over);
    for cut in [single, staged] {
        assert!(cut.water_over >= 0.0 && cut.ethanol_over <= 0.5);
        assert!(cut.energy_kj.is_finite() && cut.energy_kj > 0.0);
    }
}

#[test]
fn a_supported_pure_water_cut_reaches_the_requested_fraction() {
    let cut = ethanol_water_still(1.0, 0.0, StillTake::Fraction(0.3), 1, 101.325).unwrap();
    close(cut.water_over, 0.3);
    assert_eq!(cut.ethanol_over, 0.0);
}

#[test]
fn pure_water_fraction_cuts_preserve_requested_amount_across_inventory_scales() {
    for water in [1e-300, 1e-16, 1e-13, 1e-12, 1e-6, 1.0, 1e100] {
        let cut = ethanol_water_still(water, 0.0, StillTake::Fraction(0.5), 1, 101.325)
            .expect("representable pure-water half cut is within the same physical model");
        close(cut.water_over, water * 0.5);
        assert_eq!(cut.ethanol_over, 0.0);
        assert!(cut.energy_kj.is_finite() && cut.energy_kj > 0.0);
    }
}

#[test]
fn invalid_stocks_energy_and_overflowing_total_are_refused() {
    for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -1.0] {
        assert!(ethanol_water_still(invalid, 1.0, StillTake::Fraction(0.5), 1, 101.325).is_none());
        assert!(ethanol_water_still(1.0, invalid, StillTake::Fraction(0.5), 1, 101.325).is_none());
        assert!(ethanol_water_still(1.0, 0.0, StillTake::EnergyKj(invalid), 1, 101.325).is_none());
    }
    assert!(ethanol_water_still(1e308, 1e308, StillTake::Fraction(0.5), 1, 101.325).is_none());
    // Stock and requested amount are finite, but their latent-energy account
    // exceeds floating-point range and cannot be published as a finite cut.
    assert!(ethanol_water_still(1e308, 0.0, StillTake::Fraction(0.5), 1, 101.325).is_none());
}

#[test]
fn positive_unrepresentable_requests_refuse_while_zero_requests_succeed() {
    let smallest = f64::from_bits(1);
    // The first positive mole budget itself rounds to zero; the second
    // remains positive but its integration substeps round to zero.
    for water in [smallest, smallest * 64.0] {
        assert!(ethanol_water_still(water, 0.0, StillTake::Fraction(0.5), 1, 101.325).is_none());
    }
    assert!(ethanol_water_still(1.0, 0.0, StillTake::EnergyKj(smallest), 1, 101.325).is_none());
    let affordable = ethanol_water_still(1.0, 0.0, StillTake::EnergyKj(1.0), 1, 101.325)
        .expect("a finite affordable energy budget still gives a positive cut");
    assert!(affordable.water_over > 0.0 && affordable.water_over < 1.0);
    assert_eq!(affordable.energy_kj, 1.0);
    for request in [StillTake::Fraction(0.0), StillTake::EnergyKj(0.0)] {
        let cut = ethanol_water_still(1.0, 0.0, request, 1, 101.325).unwrap();
        assert_eq!(cut.water_over, 0.0);
        assert_eq!(cut.ethanol_over, 0.0);
        assert_eq!(cut.energy_kj, 0.0);
    }
}
