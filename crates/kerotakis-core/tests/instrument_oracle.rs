//! CAP-22: independent second opinions for the sprint's instruments.
//!
//! The rule these tests enforce: every number an instrument reports is
//! checked against arithmetic done *outside* the implementation — a
//! hand-worked textbook example, a closed-form identity, a literature
//! constant — because self-consistency tests are exactly what let a
//! divergent UNIFAC ship. The chromatography example below is computed
//! by hand in the comments; if the code and the comments disagree, the
//! comments win until a human decides otherwise.

use kerotakis_core::instrument::{Calorimeter, ChromatographyColumn, InstrumentContract};
use kerotakis_core::*;

/// Plate-theory chromatography against a hand-worked example.
///
/// Standard relations (any instrumental-analysis text; e.g. Harris,
/// Quantitative Chemical Analysis — the relations are generic):
/// t_R = t0 (1 + k), k = K·β; w = 4 t_R / √N; Rs = 2 (t2 − t1)/(w1 + w2).
///
/// Worked by hand: N = 10_000, t0 = 60 s, β = 0.5,
///   K1 = 4.0 → k1 = 2.0 → t1 = 60·3 = 180 s, w1 = 4·180/100 = 7.2 s
///   K2 = 4.4 → k2 = 2.2 → t2 = 60·3.2 = 192 s, w2 = 4·192/100 = 7.68 s
///   Rs = 2·(192 − 180)/(7.2 + 7.68) = 24/14.88 = 1.6129…
#[test]
fn chromatography_matches_the_hand_worked_example() {
    let col = ChromatographyColumn {
        plates: 10_000,
        void_time_s: 60.0,
        phase_ratio: 0.5,
    };
    let t1 = col.retention_time(4.0);
    let t2 = col.retention_time(4.4);
    assert!((t1 - 180.0).abs() < 1e-9, "t1 = {t1}");
    assert!((t2 - 192.0).abs() < 1e-9, "t2 = {t2}");
    assert!((col.peak_width(t1) - 7.2).abs() < 1e-9);
    assert!((col.peak_width(t2) - 7.68).abs() < 1e-9);
    let rs = col.resolution(t1, t2);
    assert!(
        (rs - 24.0 / 14.88).abs() < 1e-9,
        "Rs = {rs}, hand arithmetic says {}",
        24.0 / 14.88
    );

    // Limiting identities: an unretained species elutes at the void time,
    // and resolution grows as the square root of the plate count.
    assert!((col.retention_time(0.0) - 60.0).abs() < 1e-12);
    let col4 = ChromatographyColumn {
        plates: 40_000,
        ..col
    };
    let rs4 = col4.resolution(col4.retention_time(4.0), col4.retention_time(4.4));
    assert!(
        (rs4 / rs - 2.0).abs() < 1e-9,
        "4x the plates is exactly 2x the resolution: {rs} -> {rs4}"
    );
}

/// The calorimeter against the energy ledger's closed form: heating a
/// known mass of water by a known energy must move the reading by
/// exactly that energy — the instrument may not invent or lose heat.
#[test]
fn calorimeter_reads_back_exactly_the_heat_put_in() {
    let mut bench = Bench::new();
    bench.step(Operator::NewVessel { kind: None }).unwrap();
    bench
        .step(Operator::Add {
            vessel: VesselId(1),
            species: SpeciesId::new("water"),
            moles: Moles(5.0),
            at: None,
        })
        .unwrap();
    let cal = Calorimeter;
    let before = cal.measure(&bench.vessels[1]).expect("reads").value;
    bench
        .step(Operator::Heat {
            vessel: VesselId(1),
            energy: Joules(10_000.0),
            source: None,
        })
        .unwrap();
    let after = cal.measure(&bench.vessels[1]).expect("reads").value;
    assert!(
        ((after - before) - 10.0).abs() < 1e-9,
        "10 kJ in must read as 10 kJ, got {:.6}",
        after - before
    );
}
