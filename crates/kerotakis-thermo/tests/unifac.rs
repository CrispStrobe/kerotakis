//! UNIFAC against the oracle: the γ grid first, because if those ten
//! numbers match, the algorithm and the curated parameters are both right
//! and everything downstream is flash arithmetic.

use kerotakis_thermo::unifac::{Component, Mixture};

const FIXTURE: &str = include_str!("../../../tools/fixtures/vle-ethanol-water.json");

fn fixture() -> serde_json::Value {
    serde_json::from_str(FIXTURE).expect("fixture parses")
}

/// The oracle's method is stated in the fixture's provenance block:
/// original UNIFAC with the open-literature parameter tables. Ours are
/// transcribed from the same publication by a different hand, and the
/// implementations share no code — so agreement to the fixture's six
/// printed decimals is two independent routes to one number.
#[test]
fn the_gamma_grid_matches_the_oracle_to_its_printed_precision() {
    let fx = fixture();
    assert!(
        fx["_provenance"]["activity_model"]
            .as_str()
            .unwrap()
            .contains("original UNIFAC"),
        "apples to apples: the oracle must be original UNIFAC"
    );
    let mut worst: f64 = 0.0;
    for p in fx["unifac_gammas"].as_array().unwrap() {
        let x = p["x_ethanol"].as_f64().unwrap();
        let t = p["T_K"].as_f64().unwrap();
        let m = Mixture::new(&[(Component::ethanol(), x), (Component::water(), 1.0 - x)]).unwrap();
        let g = m.activity_coefficients(t);
        for (ours, theirs) in [
            (g[0], p["gamma_ethanol"].as_f64().unwrap()),
            (g[1], p["gamma_water"].as_f64().unwrap()),
        ] {
            let rel = (ours / theirs - 1.0).abs();
            worst = worst.max(rel);
            assert!(
                rel < 2e-6,
                "T={t} x={x}: ours {ours:.6} vs oracle {theirs:.6} ({rel:.1e})"
            );
        }
    }
    // Measured 2026-08-20: 4.5e-7, which is the six-decimal print floor.
    assert!(worst < 2e-6, "worst {worst:.1e}");
}

/// Infinite-dilution behaviour has the right shape: a little ethanol in
/// water is strongly non-ideal, a little water in ethanol less so — the
/// asymmetry the azeotrope is built on. The numbers are the *model's*:
/// original UNIFAC gives γ∞(ethanol in water) ≈ 7.5 at 25 °C where
/// measurement says about 4 — a known overprediction of the 1975 model
/// at infinite dilution, and one of the reasons modified UNIFAC exists.
/// Asserted as what the model computes, not as truth about ethanol; the
/// codex is where that boundary gets taught.
#[test]
fn dilute_ethanol_in_water_is_the_more_non_ideal_end() {
    let m = Mixture::new(&[(Component::ethanol(), 0.001), (Component::water(), 0.999)]).unwrap();
    let g_e = m.activity_coefficients(298.15)[0];
    let m = Mixture::new(&[(Component::ethanol(), 0.999), (Component::water(), 0.001)]).unwrap();
    let g_w = m.activity_coefficients(298.15)[1];
    assert!(g_e > 6.5 && g_e < 8.5, "γ∞(ethanol in water) {g_e}");
    assert!(g_w > 2.0 && g_w < 3.5, "γ∞(water in ethanol) {g_w}");
    assert!(g_e > g_w);
}

/// Acetic acid is in the table too, and a three-component mixture runs
/// through the same code path.
#[test]
fn a_ternary_with_acetic_acid_computes() {
    let m = Mixture::new(&[
        (Component::ethanol(), 0.3),
        (Component::water(), 0.6),
        (Component::acetic_acid(), 0.1),
    ])
    .unwrap();
    let g = m.activity_coefficients(323.15);
    assert_eq!(g.len(), 3);
    assert!(g.iter().all(|v| v.is_finite() && *v > 0.0), "{g:?}");
}
