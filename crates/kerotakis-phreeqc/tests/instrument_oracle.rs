//! CAP-22, engine side: the spectrophotometer against the literature and
//! against Beer–Lambert's own linearity, through the full pipeline —
//! bench, aqueous engine, speciation, registry spectrum, instrument.

#![cfg(feature = "engine")]

use kerotakis_core::instrument::{InstrumentContract, Spectrophotometer};
use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn brine_with_kmno4(eq: &mut PhreeqcEquilibrator, mmol: f64) -> Bench {
    let mut bench = Bench::new();
    bench
        .step_with(Operator::NewVessel, eq, &PermissiveScreen)
        .unwrap();
    for (key, moles) in [("water", 5.55), ("KMnO4", mmol / 1000.0)] {
        bench
            .step_with(
                Operator::Add {
                    vessel: VesselId(1),
                    species: SpeciesId::new(key),
                    moles: Moles(moles),
                    at: None,
                },
                eq,
                &PermissiveScreen,
            )
            .unwrap();
    }
    bench
}

/// The literature anchor: permanganate's molar absorptivity at 525 nm is
/// ~2455 L mol⁻¹ cm⁻¹ (the classic self-indicating titrant value quoted
/// across instrumental-analysis texts). 0.10 mmol KMnO₄ in 100 g water is
/// 1.0 mmol/kgw; through a 1 cm cell the whole pipeline — engine
/// speciation to registry spectrum to instrument — must land within 10 %
/// of ε_lit·c·l. The registry curates ε(525) = 2400; the margin covers
/// that curation plus band sampling, and a pipeline that drifts further
/// than that has lost real chemistry somewhere.
#[test]
fn permanganate_absorbance_matches_the_literature_anchor() {
    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    // 5.55 mol water = 100 g; 0.1 mmol KMnO4 -> 1.0 mmol/kgw.
    let bench = brine_with_kmno4(&mut eq, 0.1);
    let spec = Spectrophotometer { path_cm: 1.0 };
    let reading = spec.measure(&bench.vessels[1]).expect("reads");
    let expected = 2455.0 * 1.0e-3 * 1.0; // ε_lit · c · l
    assert!(
        (reading.value - expected).abs() / expected < 0.10,
        "A(peak) = {:.4} vs literature {:.4}",
        reading.value,
        expected
    );
}

/// Beer–Lambert linearity as a metamorphic invariant through the whole
/// stack: double the permanganate, double the absorbance. Nothing in the
/// pipeline — speciation, water mass, band arithmetic — may bend the
/// line at these dilutions.
#[test]
fn absorbance_is_linear_in_concentration() {
    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    let spec = Spectrophotometer { path_cm: 1.0 };
    let a1 = spec
        .measure(&brine_with_kmno4(&mut eq, 0.1).vessels[1])
        .expect("reads")
        .value;
    let a2 = spec
        .measure(&brine_with_kmno4(&mut eq, 0.2).vessels[1])
        .expect("reads")
        .value;
    assert!(
        (a2 / a1 - 2.0).abs() < 0.02,
        "doubling the permanganate must double the absorbance: {a1:.4} -> {a2:.4}"
    );
}
