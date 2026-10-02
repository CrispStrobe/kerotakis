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
