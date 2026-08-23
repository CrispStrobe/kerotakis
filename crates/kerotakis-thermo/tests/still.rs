//! The batch still: Rayleigh drift, the column, and the burner's meter.

use kerotakis_thermo::vle::*;

/// Distilling deep into the pot depletes it: the residue is leaner than
/// the charge, the distillate richer, and the pot's boiling point climbs
/// as the light component leaves — none of which a frozen-composition
/// one-shot can show.
#[test]
fn rayleigh_drift_depletes_the_pot_and_raises_the_boil() {
    let cut = ethanol_water_still(9.4, 0.6, StillTake::Fraction(0.5), 1, ATMOSPHERE_KPA)
        .expect("wine distils");
    let x0 = 0.6 / 10.0;
    let x_dist = cut.ethanol_over / (cut.ethanol_over + cut.water_over);
    let residue_e = 0.6 - cut.ethanol_over;
    let residue_w = 9.4 - cut.water_over;
    let x_res = residue_e / (residue_e + residue_w);
    assert!(x_dist > x0, "distillate must be richer than the charge");
    assert!(x_res < x0, "residue must be leaner than the charge");
    assert!(
        cut.t_end_c > cut.t_start_c + 0.5,
        "the boil must climb as ethanol leaves: {:.2} -> {:.2}",
        cut.t_start_c,
        cut.t_end_c
    );
    assert!(cut.energy_kj > 0.0);
}

/// More stages, more separation — up to the azeotrope and never past it.
#[test]
fn stages_climb_to_the_azeotrope_and_stop() {
    let one = ethanol_water_still(9.4, 0.6, StillTake::Fraction(0.05), 1, ATMOSPHERE_KPA)
        .expect("one stage");
    let five = ethanol_water_still(9.4, 0.6, StillTake::Fraction(0.05), 5, ATMOSPHERE_KPA)
        .expect("five stages");
    let x1 = one.ethanol_over / (one.ethanol_over + one.water_over);
    let x5 = five.ethanol_over / (five.ethanol_over + five.water_over);
    assert!(
        x5 > x1 + 0.1,
        "five stages must separate much harder: {x1:.3} vs {x5:.3}"
    );
    assert!(
        x5 < 0.90,
        "no column passes the azeotrope at x = 0.894, got {x5:.3}"
    );

    let tall = ethanol_water_still(9.4, 0.6, StillTake::Fraction(0.05), 40, ATMOSPHERE_KPA)
        .expect("forty stages");
    let x40 = tall.ethanol_over / (tall.ethanol_over + tall.water_over);
    assert!(
        tall.azeotrope_limited,
        "a forty-stage column from wine must report the azeotrope wall"
    );
    assert!(
        (x40 - 0.894).abs() < 0.02,
        "forty stages land on the azeotrope, got {x40:.3}"
    );
}

/// The burner's meter: energy in, moles over, and the arithmetic agrees.
#[test]
fn the_energy_budget_is_a_real_meter() {
    let cut = ethanol_water_still(5.0, 0.0, StillTake::EnergyKj(40.657), 1, ATMOSPHERE_KPA)
        .expect("water boils");
    assert!(
        (cut.water_over - 1.0).abs() < 0.01,
        "40.657 kJ lifts one mole of water, got {:.4}",
        cut.water_over
    );
    assert!((cut.energy_kj - 40.657).abs() < 1e-6);

    let small = ethanol_water_still(9.4, 0.6, StillTake::EnergyKj(8.0), 1, ATMOSPHERE_KPA)
        .expect("a short burn");
    let latent =
        small.ethanol_over * ETHANOL_HVAP_KJ_PER_MOL + small.water_over * WATER_HVAP_KJ_PER_MOL;
    assert!(
        (latent - small.energy_kj).abs() < 1e-9 && small.energy_kj <= 8.0 + 1e-9,
        "the meter must equal the latent heat of what came over"
    );
}
