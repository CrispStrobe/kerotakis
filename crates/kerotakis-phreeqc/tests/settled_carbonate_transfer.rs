//! A settled carbonate slurry must remain characterised after a partial pour.
#![cfg(feature = "engine")]

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

#[test]
fn partial_pours_of_settled_chalk_brine_conserve_inventory_and_resolve_both_sides() {
    let mut bench = Bench::new();
    let mut solvers = SolverStack::new(kerotakis_stack::standard_solvers(vec![Box::new(
        DisplacementEquilibrator::wrapping(Box::new(PhreeqcEquilibrator::new().unwrap())),
    )]));
    for line in [
        "new",
        "add v1 water 120mL",
        "add v1 NaCl 0.012mol",
        "add v1 CaCO3 2g",
        "wait 60s",
        "decant v1 v2 0.25",
        "decant v1 v2 0.3333333333333333",
    ] {
        let events = bench
            .step_with(
                script::parse_op(line).unwrap().unwrap(),
                &mut solvers,
                &PermissiveScreen,
            )
            .unwrap();
        let text = render_events_in(&events, Register(3), Locale::EN).join("\n");
        assert!(
            !text.contains("solver 'phreeqc-aqueous' failed"),
            "{line}: {text}"
        );
        if line.starts_with("decant") {
            for id in [VesselId(0), VesselId(1)] {
                let vessel = bench.vessel(id).unwrap();
                assert!(vessel.solution.is_some(), "{line}: {id:?} unresolved");
                assert!(vessel.temperature.0.is_finite());
            }
        }
    }
    let mut calcium = 0.0;
    for id in [VesselId(0), VesselId(1)] {
        let vessel = bench.vessel(id).unwrap();
        for species in ["Na+", "Cl-"] {
            let moles: f64 = vessel
                .contents
                .iter()
                .filter(|p| p.species.0 == species)
                .map(|p| p.moles.0)
                .sum();
            assert!((moles - 0.006).abs() < 1e-9, "{species}: {moles}");
        }
        calcium += vessel
            .contents
            .iter()
            .filter(|p| ["Ca+2", "CaCO3"].contains(&p.species.0.as_str()))
            .map(|p| p.moles.0)
            .sum::<f64>();
    }
    assert!((calcium - 2.0 / 100.087).abs() < 1e-8, "calcium {calcium}");
    let receiver = bench.vessel(VesselId(1)).unwrap();
    let receiver_solid: f64 = receiver
        .contents
        .iter()
        .filter(|p| p.phase == Phase::Solid)
        .map(|p| p.moles.0)
        .sum();
    assert!(receiver_solid < 1e-5, "settled chalk must remain in source");
}
