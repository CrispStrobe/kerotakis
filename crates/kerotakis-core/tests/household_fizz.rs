use kerotakis_core::*;

fn stack() -> SolverStack {
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(CuratedEquilibrator),
        Box::new(HonestyEquilibrator),
    ])
}

fn add(bench: &mut Bench, stack: &mut SolverStack, key: &str, moles: f64) -> Vec<Event> {
    bench
        .step_with(
            Operator::Add {
                vessel: VesselId(0),
                species: SpeciesId::new(key),
                moles: Moles(moles),
                at: None,
            },
            stack,
            &PermissiveScreen,
        )
        .expect("step")
}

fn moles(bench: &Bench, key: &str) -> f64 {
    bench
        .vessel(VesselId(0))
        .expect("vessel")
        .moles_of(&SpeciesId::new(key))
        .0
}

#[test]
fn household_fizz_equation_is_element_balanced() {
    let equation = stoich::parse_equation("NaHCO3 + CH3COOH -> CH3COONa + H2O + CO2")
        .expect("parse household fizz equation");
    assert!(
        equation.is_balanced(),
        "imbalance: {:?}",
        equation.element_imbalance()
    );
}

#[test]
fn vinegar_and_baking_soda_make_stoichiometric_carbon_dioxide() {
    let mut bench = Bench::new();
    let mut solvers = stack();
    add(&mut bench, &mut solvers, "NaHCO3", 0.050);
    let events = add(&mut bench, &mut solvers, "CH3COOH", 0.030);

    assert!(events.iter().any(|event| matches!(
        event,
        Event::ReactionOccurred { equation, .. } if equation.contains("NaHCO")
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        Event::GasEvolved { species, moles, .. }
            if species.0 == "CO2" && (moles.0 - 0.030).abs() < 1e-12
    )));
    assert!((moles(&bench, "NaHCO3") - 0.020).abs() < 1e-12);
    assert!(moles(&bench, "CH3COOH").abs() < 1e-12);
    assert!((moles(&bench, "NaOAc") - 0.030).abs() < 1e-12);
}

#[test]
fn sealed_fizz_keeps_the_carbon_dioxide_in_the_headspace() {
    let mut bench = Bench::new();
    let mut solvers = stack();
    bench
        .step_with(
            Operator::Seal {
                vessel: VesselId(0),
                headspace_volume: Liters(10.0),
            },
            &mut solvers,
            &PermissiveScreen,
        )
        .expect("seal");
    add(&mut bench, &mut solvers, "NaHCO3", 0.010);
    let events = add(&mut bench, &mut solvers, "CH3COOH", 0.010);

    assert!(events.iter().any(|event| matches!(
        event,
        Event::GasContained { species, moles, .. }
            if species.0 == "CO2" && (moles.0 - 0.010).abs() < 1e-12
    )));
    assert!((moles(&bench, "CO2") - 0.010).abs() < 1e-12);
}
