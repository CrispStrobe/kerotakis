//! A Faraday budget is conditional on the assumed electrodes and selectivity.
use kerotakis_core::{ops::NotModelledCause, *};

#[test]
fn solvent_electrolysis_exposes_selectivity_scope_for_both_anode_routes() {
    for (anion, anode) in [("Cl-", "Cl2"), ("SO4-2", "O2")] {
        let mut bench = Bench::new();
        let vessel = &mut bench.vessels[0];
        vessel.deposit(SpeciesId::new("water"), Moles(5.0), Phase::Liquid);
        vessel.deposit(SpeciesId::new("Na+"), Moles(0.1), Phase::Aqueous);
        vessel.deposit(
            SpeciesId::new(anion),
            Moles(if anion == "Cl-" { 0.1 } else { 0.05 }),
            Phase::Aqueous,
        );
        let events = bench
            .step(Operator::Electrolyse {
                vessel: VesselId(0),
                amps: 1.0,
                seconds: 60.0,
            })
            .unwrap();
        assert!(events.iter().any(|event| matches!(event, Event::Electrolysed { anode_species: Some(species), coulombs, .. } if species.0 == anode && (*coulombs-60.0).abs()<1e-12)));
        assert!(events.iter().any(|event| matches!(event, Event::NotYetModeled { cause: NotModelledCause::ModelBoundary, what, .. } if what.contains("inert electrodes") && what.contains("competing chlorine/oxygen"))));
    }
}
