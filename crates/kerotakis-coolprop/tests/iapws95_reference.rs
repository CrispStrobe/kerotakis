//! Reference values and parity. Needs the native library:
//!   cargo test -p kerotakis-coolprop --features engine
#![cfg(feature = "engine")]
use kerotakis_coolprop::*;

fn rel(a: f64, b: f64) -> f64 {
    ((a - b) / b).abs()
}

/// IAPWS R6-95(2018), Table 8 ("Thermodynamic property values in the
/// two-phase region for selected temperatures"), also Wagner & Pruss (2002).
/// Digits cross-checked 2026-10-27 against the independent IAPWS-95
/// implementation's test suite, github.com/jjgomera/iapws test.py
/// `test_saturation` (pressures there in kPa).
/// (T/K, p/MPa, rho'/kg m-3, rho''/kg m-3)
const TABLE8: [(f64, f64, f64, f64); 3] = [
    (275.0, 0.698451167e-3, 0.999887406e3, 0.550664919e-2),
    (450.0, 0.932203564, 0.890341250e3, 0.481200360e1),
    (625.0, 0.169082693e2, 0.567090385e3, 0.118290280e3),
];

#[test]
fn saturation_matches_iapws95_table8() {
    for (t, p_mpa, rl, rv) in TABLE8 {
        let s = saturation_at_temperature(t).unwrap();
        assert!(
            rel(s.pressure_pa, p_mpa * 1e6) < 1e-6,
            "p at {t}: {}",
            s.pressure_pa
        );
        assert!(rel(s.density_liquid_kg_m3, rl) < 1e-6, "rho' at {t}");
        assert!(rel(s.density_vapour_kg_m3, rv) < 1e-6, "rho'' at {t}");
        assert!(s.cp_liquid_j_kg_k > 0.0 && s.cp_vapour_j_kg_k > 0.0);
    }
}

/// IAPWS R6-95(2018) Table 7: T = 300 K, rho = 996.5560 kg/m3 gives
/// p = 0.0992418352 MPa (cross-checked against jjgomera/iapws test.py
/// `test_phase`; the release footnotes imprecision in p's last digits, which
/// moves the inverted liquid density far below 1e-6). Inverting (T, p) must return that density.
#[test]
fn single_phase_density_matches_iapws95_table7() {
    let s = water_state(300.0, 0.0992418352e6).unwrap();
    assert_eq!(s.phase, Phase::Liquid);
    assert!(rel(s.density_kg_m3, 996.5560) < 1e-6, "{}", s.density_kg_m3);
    // T = 500 K, rho = 0.435 kg/m3 -> p = 0.0999679423 MPa (vapour).
    let v = water_state(500.0, 0.0999679423e6).unwrap();
    assert_eq!(v.phase, Phase::Vapour);
    assert!(rel(v.density_kg_m3, 0.435) < 1e-6, "{}", v.density_kg_m3);
}

/// cp of liquid water at 25 C, 1 atm: 4.1813 kJ/(kg K) (NIST Chemistry
/// WebBook SRD 69 fluid properties, which evaluate IAPWS-95).
#[test]
fn liquid_cp_at_25c() {
    let s = water_state(298.15, 101_325.0).unwrap();
    assert!(rel(s.cp_j_kg_k, 4181.3) < 2e-4, "{}", s.cp_j_kg_k);
    assert!(rel(s.density_kg_m3, 997.05) < 1e-4, "{}", s.density_kg_m3);
}

/// Normal boiling point on IAPWS-95 is 373.124 K (ITS-90), not 373.15 K.
#[test]
fn normal_boiling_point() {
    let t = water_boiling_point_celsius(101.325).unwrap();
    assert!((t + 273.15 - 373.124).abs() < 2e-3, "{t}");
}

#[test]
fn on_saturation_line_is_refused() {
    let p = saturation_at_temperature(400.0).unwrap().pressure_pa;
    assert!(matches!(
        water_state(400.0, p),
        Err(CoolPropError::OnSaturationLine { .. })
    ));
}

/// Parity with Kerotakis' existing water vapour pressure (Stull 1947 Antoine,
/// 1-100 C, `kerotakis_thermo::vle::WATER`). Antoine is a fit; 1 % is the
/// agreement Kerotakis' bench claims, and the observed gap is reported.
#[test]
fn parity_with_existing_antoine_water() {
    for t_c in [5.0, 25.0, 50.0, 80.0, 100.0] {
        let antoine = kerotakis_thermo::vle::WATER.pressure_kpa(t_c).unwrap();
        let iapws = water_saturation_pressure_kpa(t_c).unwrap();
        let gap = rel(antoine, iapws);
        eprintln!("T={t_c} C antoine={antoine:.4} kPa iapws95={iapws:.4} kPa rel={gap:.2e}");
        assert!(gap < 0.01, "T={t_c}: antoine {antoine} vs iapws95 {iapws}");
    }
    // The bench's "water boils at 100 C at 1 atm" agrees to 0.05 K.
    assert!((water_boiling_point_celsius(101.325).unwrap() - 100.0).abs() < 0.05);
}

#[test]
fn linked_library_is_the_pinned_version() {
    let (version, _rev) = linked_version();
    assert_eq!(version, "6.8.0");
}
