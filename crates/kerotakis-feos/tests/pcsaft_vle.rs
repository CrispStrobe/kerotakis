//! Native FeOs tests. Run with `--features feos`.
#![cfg(feature = "feos")]

use kerotakis_feos::{
    pure_boiling_point, pure_vapour_pressure, saturation_point, Alcohol, Refusal, Saturation,
};

const ATM: f64 = 101.325;

fn bubble(a: Alcohol, x: f64, p: f64) -> kerotakis_feos::SaturationPoint {
    saturation_point(a, Saturation::Bubble, x, p).expect("in-domain bubble point")
}

/// Near-pure alcohol must boil near its normal boiling point. The reference
/// temperatures (methanol 337.8 K, ethanol 351.4 K, water 373.15 K) are
/// LOOSE SANITY BOUNDS only, not shipped numbers. They agree with the
/// already-reviewed, DOI-cited Antoine constants in
/// `crates/kerotakis-thermo/src/vle.rs` (methanol and water: Stull 1947,
/// doi:10.1021/ie50448a022; ethanol: Susial Badajoz et al., ACS Omega,
/// doi:10.1021/acsomega.6c04827). (Water boils at about 373.12 K on ITS-90.)
#[test]
fn near_pure_alcohols_boil_at_their_normal_boiling_points() {
    let m = bubble(Alcohol::Methanol, 0.999, ATM);
    assert!((m.t_kelvin - 337.8).abs() < 2.0, "methanol {}", m.t_kelvin);
    let e = bubble(Alcohol::Ethanol, 0.999, ATM);
    assert!((e.t_kelvin - 351.4).abs() < 2.0, "ethanol {}", e.t_kelvin);
    // Water trace end: 373.15 K.
    let w = bubble(Alcohol::Ethanol, 0.001, ATM);
    assert!(
        (w.t_kelvin - 373.15).abs() < 2.5,
        "water end {}",
        w.t_kelvin
    );
}

/// Physics shape: alcohol enriches in the vapour, bubble T falls with
/// alcohol content, and dew lies above bubble at the same composition.
#[test]
fn alcohol_enriches_in_vapour_and_dew_lies_above_bubble() {
    for a in [Alcohol::Methanol, Alcohol::Ethanol] {
        let lo = bubble(a, 0.1, ATM);
        let hi = bubble(a, 0.4, ATM);
        assert!(lo.y_alcohol > lo.x_alcohol);
        assert!(hi.t_kelvin < lo.t_kelvin);
        let dew = saturation_point(a, Saturation::Dew, 0.4, ATM).unwrap();
        assert!(dew.t_kelvin >= hi.t_kelvin - 1e-6);
        assert!(dew.x_alcohol < 0.4);
    }
}

/// PC-SAFT (Esper et al. 2023 pure + Rehner et al. 2023 binary records,
/// i.e. fitted ethanol-water cross-association) vs the existing UNIFAC path
/// vs experiment. History: with the Gross & Sadowski (2002) set and
/// k_ij = 0, hosted run d3d1a79 sat 5-6 K below UNIFAC at x = 0.05-0.1
/// while reproducing the azeotrope (78.15 °C); that set was replaced by
/// user decision. The sweep and per-point deltas are printed. Asserted:
///
/// * Azeotrope at 101.325 kPa: 78.2 °C at x = 0.894 (95.6 wt%) ethanol.
///   Source in this repository: the reviewed control
///   `crates/kerotakis-thermo/tests/vle.rs::unifac_finds_the_ethanol_water_azeotrope`
///   (and `tests/still.rs`), which asserts the same 78.2 °C / x = 0.894.
///   That control carries no primary citation itself; the candidate open
///   primary measurement is Kamihama et al., J. Chem. Eng. Data 57 (2012)
///   339, doi:10.1021/je2008704 (binary ethanol + water at 101.3 kPa), whose
///   azeotrope values have NOT yet been checked against 78.2 / 0.894. The
///   PC-SAFT bubble point at that composition must lie within 3 K.
/// * TODO(water-rich bound): assert the x = 0.05-0.1 bubble points against
///   an openly citable primary measurement (candidate Kamihama et al. 2012,
///   doi:10.1021/je2008704) once its tables are confirmed; until then the
///   water-rich side is printed, not asserted.
/// * Every bubble point lies between the PC-SAFT pure boiling points of
///   ethanol and water (with 0.5 K slack), i.e. no unphysical overshoot.
#[test]
fn ethanol_water_against_experiment_and_unifac() {
    let tb_e = pure_boiling_point(Some(Alcohol::Ethanol), ATM)
        .unwrap()
        .t_kelvin;
    let tb_w = pure_boiling_point(None, ATM).unwrap().t_kelvin;
    let mut rows = Vec::new();
    for x in [0.05, 0.1, 0.2, 0.4, 0.6, 0.8, 0.894] {
        let f = bubble(Alcohol::Ethanol, x, ATM);
        let u = kerotakis_thermo::vle::ethanol_water_bubble_point(x, ATM).expect("unifac");
        println!(
            "x={x}: PC-SAFT T={:.2} °C y={:.4} | UNIFAC T={:.2} °C y={:.4} | dT={:+.2} K",
            f.t_celsius(),
            f.y_alcohol,
            u.t_celsius,
            u.y[0],
            f.t_celsius() - u.t_celsius
        );
        rows.push((x, f));
    }
    let fd = saturation_point(Alcohol::Ethanol, Saturation::Dew, 0.5, ATM).unwrap();
    let ud = kerotakis_thermo::vle::ethanol_water_dew_point(0.5, ATM).expect("unifac dew");
    println!(
        "dew y=0.5: PC-SAFT T={:.2} x={:.4} | UNIFAC T={:.2} x={:.4}",
        fd.t_celsius(),
        fd.x_alcohol,
        ud.t_celsius,
        ud.x[0]
    );
    for (x, f) in &rows {
        assert!(
            f.t_kelvin > tb_e - 0.5 && f.t_kelvin < tb_w + 0.5,
            "x={x}: {} K outside [{tb_e}, {tb_w}]",
            f.t_kelvin
        );
    }
    let (_, az) = rows.last().unwrap();
    assert!(
        (az.t_celsius() - 78.2).abs() < 3.0,
        "azeotrope-composition bubble point {} °C vs 78.2 °C (kerotakis-thermo azeotrope control)",
        az.t_celsius()
    );
}

/// Methanol-water: printed for the record (no UNIFAC entry point exists
/// in kerotakis-thermo for it); asserted only physically bounded.
#[test]
fn methanol_water_bubble_curve_is_bounded() {
    let tb_m = pure_boiling_point(Some(Alcohol::Methanol), ATM)
        .unwrap()
        .t_kelvin;
    let tb_w = pure_boiling_point(None, ATM).unwrap().t_kelvin;
    for x in [0.05, 0.2, 0.5, 0.8] {
        let f = bubble(Alcohol::Methanol, x, ATM);
        println!(
            "methanol x={x}: T={:.2} °C y={:.4}",
            f.t_celsius(),
            f.y_alcohol
        );
        assert!(f.t_kelvin > tb_m - 0.5 && f.t_kelvin < tb_w + 0.5);
        assert!(f.y_alcohol > x);
    }
}

/// Solved temperature outside the window refuses: 10 kPa ethanol-rich
/// bubble lies near 300 K (admitted), but nothing reaches above 523 K in
/// this pressure box, so check the low end: a low-pressure alcohol-rich
/// liquid is admitted only while T >= 273.15 K.
#[test]
fn low_pressure_result_is_admitted_or_typed_refusal() {
    match saturation_point(Alcohol::Methanol, Saturation::Bubble, 0.99, 10.0) {
        Ok(p) => assert!(p.t_kelvin >= 273.15),
        Err(e) => assert!(matches!(e, Refusal::TemperatureOutOfDomain(_))),
    }
}

/// Diagnostic: pure-component PC-SAFT from the vendored Esper 2023 file.
/// Loose sanity references only (not shipped numbers): normal boiling
/// points as in `near_pure_alcohols_boil_at_their_normal_boiling_points`,
/// and 330 K vapour pressures (methanol ~68.5, ethanol ~40.6, water
/// ~17.2 kPa) evaluated from Antoine forms of the same DOI-cited fits
/// carried in `crates/kerotakis-thermo/src/vle.rs`.
/// Bands are initial and loose for the Esper 2023 set; actual values are
/// printed for the PR and the bands re-tightened from them.
#[test]
fn pure_component_saturation_diagnostic() {
    let cases = [
        // Parameter set changed to Esper et al. (2023): the tolerances
        // tightened from the Gross 2002 run (670ee975) do not carry over.
        // Initial bands again; re-tighten from the printed values of the
        // first hosted run on the Esper set.
        ("methanol", Some(Alcohol::Methanol), 337.8, 68.5, 2.0, 0.08),
        ("ethanol", Some(Alcohol::Ethanol), 351.4, 40.6, 2.0, 0.08),
        ("water", None, 373.15, 17.2, 2.0, 0.08),
    ];
    for (name, c, tb, p330, t_tol, p_rel) in cases {
        let b = pure_boiling_point(c, ATM).expect("pure boiling point");
        let v = pure_vapour_pressure(c, 330.0).expect("pure vapour pressure");
        println!(
            "{name}: T_b(1 atm) = {:.2} K (ref {tb}); p_sat(330 K) = {:.2} kPa (ref {p330})",
            b.t_kelvin, v.pressure_kpa
        );
        assert!((b.t_kelvin - tb).abs() < t_tol, "{name} T_b {}", b.t_kelvin);
        assert!(
            (v.pressure_kpa / p330 - 1.0).abs() < p_rel,
            "{name} p_sat {}",
            v.pressure_kpa
        );
    }
}
