//! The stable product and the one you actually get.
//!
//! A Gibbs-minimising engine returns tenorite, because tenorite is the more
//! stable copper solid. A beaker returns pale blue Cu(OH)2. Both facts are
//! true, and the engine has to be told which question it is answering.

#![cfg(feature = "engine")]

use kerotakis_core::*;

fn stack() -> SolverStack {
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(kerotakis_phreeqc::PhreeqcEquilibrator::new().expect("engine")),
        Box::new(StateEquilibrator),
        Box::new(HonestyEquilibrator),
    ])
}

fn copper_and_lye(heat_joules: f64) -> Bench {
    let mut bench = Bench::new();
    let mut s = stack();
    let v = VesselId(0);
    for (key, moles) in [("water", 5.5343), ("CuSO4", 0.01), ("NaOH", 0.02)] {
        bench
            .step_with(
                Operator::Add {
                    vessel: v,
                    species: SpeciesId::new(key),
                    moles: Moles(moles),
                    at: None,
                },
                &mut s,
                &PermissiveScreen,
            )
            .expect("add");
    }
    if heat_joules > 0.0 {
        bench
            .step_with(
                Operator::Heat {
                    vessel: v,
                    energy: Joules(heat_joules),
                    source: None,
                },
                &mut s,
                &PermissiveScreen,
            )
            .expect("heat");
    }
    bench
}

fn solid(bench: &Bench, key: &str) -> f64 {
    bench
        .vessel(VesselId(0))
        .unwrap()
        .contents
        .iter()
        .filter(|p| p.species == SpeciesId::new(key) && p.phase == Phase::Solid)
        .map(|p| p.moles.0)
        .sum()
}

#[test]
fn cold_gives_the_blue_hydroxide_not_the_black_oxide() {
    let bench = copper_and_lye(0.0);
    assert!(
        solid(&bench, "Cu(OH)2") > 0.009,
        "the beaker's answer: {:?}",
        bench.vessel(VesselId(0)).unwrap().contents
    );
    assert!(
        solid(&bench, "CuO") < 1e-9,
        "tenorite is the stable phase and must still be withheld at room temperature"
    );
}

#[test]
fn heating_converts_it_to_the_stable_oxide() {
    // The demonstration: warm the blue gel and it turns black.
    let bench = copper_and_lye(20_000.0);
    let t = bench.vessel(VesselId(0)).unwrap().temperature.to_celsius();
    assert!(t > 67.0, "the threshold must actually be crossed: {t} C");
    assert!(
        solid(&bench, "CuO") > 0.009,
        "above the threshold the engine is free to find tenorite: {:?}",
        bench.vessel(VesselId(0)).unwrap().contents
    );
    assert!(solid(&bench, "Cu(OH)2") < 1e-9, "and the hydroxide is gone");
}

#[test]
fn the_withheld_phase_is_declared_rather_than_hidden() {
    let mut bench = Bench::new();
    let mut s = stack();
    let v = VesselId(0);
    let mut all = Vec::new();
    for (key, moles) in [("water", 5.5343), ("CuSO4", 0.01), ("NaOH", 0.02)] {
        all.extend(
            bench
                .step_with(
                    Operator::Add {
                        vessel: v,
                        species: SpeciesId::new(key),
                        moles: Moles(moles),
                        at: None,
                    },
                    &mut s,
                    &PermissiveScreen,
                )
                .expect("add"),
        );
    }
    let said = all.iter().any(|e| {
        matches!(e, Event::NotYetModeled { what, .. }
            if what.contains("Tenorite") && what.contains("holding back"))
    });
    assert!(
        said,
        "a curated kinetic claim must be visible, not silent: {all:?}"
    );
}
