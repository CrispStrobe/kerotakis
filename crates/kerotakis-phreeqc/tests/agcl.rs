//! The P0 end-to-end case (PLAN.md): silver nitrate + table salt, with the
//! saturation index and the precipitated amount coming out of thermodynamic
//! data — derived, not hardcoded.

#![cfg(feature = "engine")]

use kerotakis_phreeqc::{databases, Phreeqc};

/// 10 mmol AgNO3 + 10 mmol NaCl in a kilogram of water: massively
/// supersaturated in AgCl (cerargyrite), SI far above 0.
#[test]
fn silver_chloride_is_supersaturated_before_equilibration() {
    let mut pqc = Phreeqc::with_database(databases::WATEQ4F).expect("load wateq4f");
    pqc.run(
        r#"
SOLUTION 1  Mixed AgNO3 + NaCl, 0.01 mol each
    units     mol/kgw
    temp      25
    pH        7  charge
    Na        0.01
    Cl        0.01
    Ag        0.01
    N(5)      0.01
SELECTED_OUTPUT
    -reset    false
    -si       Cerargyrite
    -ionic_strength true
END
"#,
    )
    .expect("run mixed solution");

    // Not naively log(0.01·0.01/Ksp) ≈ 5.7: a good share of the silver is
    // complexed as AgCl(aq)/AgCl2- at these concentrations, which is exactly
    // the kind of thing the engine knows and a lookup table would not.
    let si = pqc.last_value("si_Cerargyrite").expect("SI column");
    assert!(
        si > 3.0,
        "0.01 m Ag+ and Cl- should be far above AgCl saturation, got SI = {si}"
    );
    // Below the naive 0.02 m for the same reason: neutral complexes
    // (AgCl(aq), AgNO3(aq)) take ions out of the ionic-strength sum.
    let ionic_strength = pqc.last_value("mu").expect("mu column");
    assert!(
        ionic_strength > 0.008 && ionic_strength < 0.03,
        "ionic strength of the mix should be ~0.01-0.02 m, got {ionic_strength}"
    );
}

/// Let the solid form: virtually all silver leaves solution as AgCl, and the
/// equilibrated water sits exactly at saturation (SI = 0).
#[test]
fn silver_chloride_precipitates_on_equilibration() {
    let mut pqc = Phreeqc::with_database(databases::WATEQ4F).expect("load wateq4f");
    pqc.run(
        r#"
SOLUTION 1
    units     mol/kgw
    temp      25
    pH        7  charge
    Na        0.01
    Cl        0.01
    Ag        0.01
    N(5)      0.01
EQUILIBRIUM_PHASES 1
    Cerargyrite 0 0
SELECTED_OUTPUT
    -reset    false
    -si       Cerargyrite
    -equilibrium_phases Cerargyrite
    -totals   Ag
END
"#,
    )
    .expect("equilibrate with cerargyrite");

    let si = pqc.last_value("si_Cerargyrite").expect("SI column");
    assert!(
        si.abs() < 0.01,
        "equilibrated solution must sit at saturation, got SI = {si}"
    );
    let precipitated = pqc.last_value("Cerargyrite").expect("moles of phase");
    assert!(
        precipitated > 0.0099 && precipitated <= 0.01,
        "nearly all 0.01 mol Ag should precipitate as AgCl, got {precipitated} mol"
    );
    let ag_left = pqc.last_value("Ag").expect("dissolved Ag");
    assert!(
        ag_left < 1e-4,
        "dissolved silver after precipitation should be trace, got {ag_left} mol/kgw"
    );
}

/// Honest failure: nonsense input must come back as an error string, not a
/// crash.
#[test]
fn bad_input_is_an_error_not_a_crash() {
    // PHREEQC is lenient with unknown keywords (warning, not error); a
    // non-numeric value where a number is required is a hard error.
    let mut pqc = Phreeqc::with_database(databases::PHREEQC).expect("load phreeqc.dat");
    let result = pqc.run("SOLUTION 1\n    pH banana\nEND\n");
    assert!(result.is_err(), "malformed input must error honestly");
}

/// All four embedded databases load.
#[test]
fn all_embedded_databases_load() {
    for (name, db) in [
        ("phreeqc.dat", databases::PHREEQC),
        ("wateq4f.dat", databases::WATEQ4F),
        ("minteq.v4.dat", databases::MINTEQ_V4),
        ("pitzer.dat", databases::PITZER),
    ] {
        Phreeqc::with_database(db).unwrap_or_else(|e| panic!("{name} failed to load: {e}"));
    }
}
