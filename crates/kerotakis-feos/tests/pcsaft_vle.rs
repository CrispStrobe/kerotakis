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
/// doi:10.1021/acsomega.6c04827). Water is defined at 373.124 K by ITS-90.
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

/// PC-SAFT (k_ij = 0) vs the existing UNIFAC path vs experiment.
///
/// No published ethanol-water k_ij exists for the Gross & Sadowski (2002)
/// pure-component set: `gross2002_binary.json` has none, and the
/// ethanol-water / methanol-water entries in FeOs `rehner2023_binary.json`
/// belong to the different Rehner et al. (2023) pure-parameter set and must
/// not be mixed with these records. So k_ij stays 0 (a prediction), and this
/// test does NOT demand UNIFAC parity: the first hosted run measured a
/// 6.08 K gap at x = 0.05 (PC-SAFT 83.39 °C vs UNIFAC 89.47 °C), recorded in
/// PROVENANCE.md. Instead it prints the full comparison and asserts against
/// experiment:
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

/// Diagnostic: pure-component PC-SAFT from the vendored FeOs parameter file.
/// Loose sanity references only (not shipped numbers): normal boiling
/// points as in `near_pure_alcohols_boil_at_their_normal_boiling_points`,
/// and 330 K vapour pressures (methanol ~68.5, ethanol ~40.6, water
/// ~17.2 kPa) evaluated from Antoine forms of the same DOI-cited fits
/// carried in `crates/kerotakis-thermo/src/vle.rs`.
/// Gross & Sadowski (2002) report vapour-pressure AAD of a few per cent for
/// the alcohols; 2B water with constant sigma is known to be worse, so its
/// band is wider. Actual values are printed for the PR.
#[test]
fn pure_component_saturation_diagnostic() {
    let cases = [
        // Tolerances set from hosted run of 670ee975 (printed: methanol
        // 338.50 K / 71.81 kPa, ethanol 351.66 K / 40.58 kPa, water
        // 373.27 K / 17.36 kPa), each about 1.5x the observed deviation.
        ("methanol", Some(Alcohol::Methanol), 337.8, 68.5, 1.0, 0.07),
        ("ethanol", Some(Alcohol::Ethanol), 351.4, 40.6, 0.5, 0.02),
        ("water", None, 373.15, 17.2, 0.5, 0.02),
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
