//! Native FeOs tests. Run with `--features feos`.
#![cfg(feature = "feos")]

use kerotakis_feos::{saturation_point, Alcohol, Refusal, Saturation};

const ATM: f64 = 101.325;

fn bubble(a: Alcohol, x: f64, p: f64) -> kerotakis_feos::SaturationPoint {
    saturation_point(a, Saturation::Bubble, x, p).expect("in-domain bubble point")
}

/// Near-pure alcohol must boil near its normal boiling point. Reference:
/// NIST Chemistry WebBook normal boiling points, methanol 337.8 K,
/// ethanol 351.4 K; Gross & Sadowski (2002) report vapour-pressure AAD of
/// order 1 % for these parameters, i.e. well inside 2 K at 1 atm.
#[test]
fn near_pure_alcohols_boil_at_their_normal_boiling_points() {
    let m = bubble(Alcohol::Methanol, 0.999, ATM);
    assert!((m.t_kelvin - 337.8).abs() < 2.0, "methanol {}", m.t_kelvin);
    let e = bubble(Alcohol::Ethanol, 0.999, ATM);
    assert!((e.t_kelvin - 351.4).abs() < 2.0, "ethanol {}", e.t_kelvin);
    // Water trace end: 373.15 K.
    let w = bubble(Alcohol::Ethanol, 0.001, ATM);
    assert!((w.t_kelvin - 373.15).abs() < 2.5, "water end {}", w.t_kelvin);
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

/// Parity against the existing UNIFAC/Antoine ethanol–water path in
/// kerotakis-thermo. Two independent models, k_ij = 0 unfitted: agreement
/// is asserted within 4 K in T and 0.08 in y, which is a model-difference
/// band, not an accuracy claim. Measured values are printed for the PR.
#[test]
fn ethanol_water_parity_with_unifac() {
    for x in [0.05, 0.1, 0.2, 0.4, 0.6, 0.8] {
        let f = bubble(Alcohol::Ethanol, x, ATM);
        let u = kerotakis_thermo::vle::ethanol_water_bubble_point(x, ATM).expect("unifac");
        println!(
            "x={x}: PC-SAFT T={:.2} °C y={:.4} | UNIFAC T={:.2} °C y={:.4}",
            f.t_celsius(), f.y_alcohol, u.t_celsius, u.y[0]
        );
        assert!((f.t_celsius() - u.t_celsius).abs() < 4.0);
        assert!((f.y_alcohol - u.y[0]).abs() < 0.08);
    }
    let fd = saturation_point(Alcohol::Ethanol, Saturation::Dew, 0.5, ATM).unwrap();
    let ud = kerotakis_thermo::vle::ethanol_water_dew_point(0.5, ATM).expect("unifac dew");
    println!("dew y=0.5: PC-SAFT T={:.2} x={:.4} | UNIFAC T={:.2} x={:.4}",
        fd.t_celsius(), fd.x_alcohol, ud.t_celsius, ud.x[0]);
    assert!((fd.t_celsius() - ud.t_celsius).abs() < 4.0);
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
