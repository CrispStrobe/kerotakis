//! L2g through the bench: heating a solid until it decomposes, and
//! burning magnesium — the two demonstrations every school does.

use kerotakis_cea::ThermalEquilibrator;
use kerotakis_core::*;

fn stack() -> SolverStack {
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(ThermalEquilibrator),
        Box::new(HonestyEquilibrator),
    ])
}

fn add(
    bench: &mut Bench,
    stack: &mut SolverStack,
    v: VesselId,
    key: &str,
    moles: f64,
) -> Vec<Event> {
    bench
        .step_with(
            Operator::Add {
                vessel: v,
                species: SpeciesId::new(key),
                moles: Moles(moles),
                at: None,
            },
            stack,
            &PermissiveScreen,
        )
        .expect("step")
}

fn heat(bench: &mut Bench, stack: &mut SolverStack, v: VesselId, kj: f64) -> Vec<Event> {
    bench
        .step_with(
            Operator::Heat {
                vessel: v,
                energy: Joules(kj * 1000.0),
            },
            stack,
            &PermissiveScreen,
        )
        .expect("step")
}

#[test]
fn species_map_into_the_nasa_data_by_formula() {
    // Nothing lists these pairs; they are matched by composition.
    assert_eq!(kerotakis_cea::cea_name("CaCO3"), Some("CaCO3(cr)"));
    assert_eq!(kerotakis_cea::cea_name("CaO"), Some("CaO(cr)"));
    assert_eq!(kerotakis_cea::cea_name("MgO"), Some("MgO(cr)"));
    assert_eq!(kerotakis_cea::cea_name("O2"), Some("O2"));
    assert_eq!(kerotakis_cea::cea_name("CO2"), Some("CO2"));
}

#[test]
fn chalk_in_a_cold_crucible_just_sits_there() {
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    add(&mut bench, &mut stack, v, "CaCO3", 0.1);

    let vessel = bench.vessel(v).unwrap();
    assert!(
        (vessel.moles_of(&SpeciesId::new("CaCO3")).0 - 0.1).abs() < 1e-3,
        "chalk is stable at room temperature: {:?}",
        vessel.contents
    );
}

#[test]
fn heating_chalk_hard_enough_calcines_it() {
    // The lime kiln, computed: heat until the carbonate gives way, and
    // quicklime is left with the CO2 gone.
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    add(&mut bench, &mut stack, v, "CaCO3", 0.1);
    // ~0.1 mol of chalk needs a few hundred kJ to reach kiln temperature.
    let events = heat(&mut bench, &mut stack, v, 40.0);

    let vessel = bench.vessel(v).unwrap();
    assert!(
        vessel.temperature.0 > 1100.0,
        "the crucible got hot: {:.0} K",
        vessel.temperature.0
    );
    assert!(
        vessel.moles_of(&SpeciesId::new("CaO")).0 > 0.05,
        "quicklime formed: {:?}",
        vessel.contents
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::GasEvolved { species, .. } if species.0 == "CO2")),
        "and the carbon dioxide left: {events:?}"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::ThermalEquilibrium { .. })),
        "the thermal solver reported itself"
    );
}

#[test]
fn burning_magnesium_makes_white_oxide_and_a_great_deal_of_heat() {
    // Mg + ½O₂ → MgO, ΔH ≈ −601 kJ/mol: the ribbon that burns white in every
    // school lab. Oxygen comes from the vessel's atmosphere, not from the
    // inventory.
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    add(&mut bench, &mut stack, v, "Mg", 0.05);
    let before = bench.vessel(v).unwrap().temperature.0;
    // A spark: enough to start it.
    let events = heat(&mut bench, &mut stack, v, 1.0);

    let vessel = bench.vessel(v).unwrap();
    assert!(
        vessel.moles_of(&SpeciesId::new("MgO")).0 > 0.04,
        "the ribbon becomes magnesium oxide: {:?}",
        vessel.contents
    );
    assert!(
        vessel.moles_of(&SpeciesId::new("Mg")).0 < 0.01,
        "and little metal survives"
    );
    assert!(
        vessel.temperature.0 > before + 200.0,
        "burning magnesium releases a great deal of heat: {before:.0} K → {:.0} K",
        vessel.temperature.0
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::Consumed { species, .. } if species.0 == "Mg")),
        "the metal is reported consumed: {events:?}"
    );
}

#[test]
fn the_thermal_answer_carries_its_provenance() {
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    add(&mut bench, &mut stack, v, "Mg", 0.05);
    let events = heat(&mut bench, &mut stack, v, 1.0);

    let p = events
        .iter()
        .find_map(|e| match e {
            Event::ThermalEquilibrium { provenance, .. } => Some(provenance.clone()),
            _ => None,
        })
        .expect("thermal equilibrium reported");
    assert!(p.dataset.contains("CEA"));
    assert!(p.model.contains("NASA-9"));
    assert!(
        !p.dataset_sources.is_empty(),
        "the species' own citations travel with the answer"
    );
}

#[test]
fn a_solution_is_left_to_the_aqueous_engine() {
    // Liquid water present and cool: the thermal solver must decline, so
    // the honesty pass speaks instead of a wrong answer being computed.
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    add(&mut bench, &mut stack, v, "water", 5.0);
    let events = add(&mut bench, &mut stack, v, "CaCO3", 0.01);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::NotYetModeled { .. })),
        "no aqueous engine in this stack, so the gap is stated: {events:?}"
    );
}
