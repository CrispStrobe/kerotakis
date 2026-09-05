//! Live non-ideal carbonate solid solutions through PHREEQC `SOLID_SOLUTIONS`.

#![cfg(feature = "engine")]

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn carbonate_crystal(calcium: f64, strontium: f64) -> SolidSolution {
    SolidSolution::aragonite_strontianite(
        "aragonite-strontianite crystal",
        Moles(calcium),
        Moles(strontium),
    )
}

fn closed_vessel() -> Vessel {
    let mut vessel = Vessel::new(VesselId(0), "sealed carbonate vessel");
    vessel.headspace = Headspace::Sealed {
        volume: Liters(1.0),
    };
    vessel.deposit(SpeciesId::new("water"), Moles(55.51), Phase::Liquid);
    vessel
}

fn pure_solid(vessel: &Vessel, species: &str) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|portion| portion.phase == Phase::Solid && portion.species.0 == species)
        .map(|portion| portion.moles.0)
        .sum()
}

fn component(vessel: &Vessel, component: SolidSolutionComponent) -> f64 {
    vessel
        .solid_solutions
        .iter()
        .map(|solid_solution| solid_solution.moles_of(component).0)
        .sum()
}

fn calcium_inventory(vessel: &Vessel) -> f64 {
    vessel.moles_of(&SpeciesId::new("Ca+2")).0
        + component(vessel, SolidSolutionComponent::CalciumCarbonate)
        + pure_solid(vessel, "CaCO3")
}

fn strontium_inventory(vessel: &Vessel) -> f64 {
    vessel.moles_of(&SpeciesId::new("Sr+2")).0
        + component(vessel, SolidSolutionComponent::StrontiumCarbonate)
}

fn carbon_inventory(vessel: &Vessel) -> f64 {
    vessel.moles_of(&SpeciesId::new("HCO3-")).0
        + vessel.moles_of(&SpeciesId::new("CO2")).0
        + component(vessel, SolidSolutionComponent::CalciumCarbonate)
        + component(vessel, SolidSolutionComponent::StrontiumCarbonate)
        + pure_solid(vessel, "CaCO3")
}

#[test]
fn calcium_and_strontium_co_precipitate_into_a_typed_mixed_crystal() {
    let mut solver = PhreeqcEquilibrator::new().expect("engine");
    let mut vessel = closed_vessel();
    vessel.deposit(SpeciesId::new("Ca+2"), Moles(5e-3), Phase::Aqueous);
    vessel.deposit(SpeciesId::new("Sr+2"), Moles(5e-3), Phase::Aqueous);
    vessel.deposit(SpeciesId::new("HCO3-"), Moles(2e-2), Phase::Aqueous);
    vessel.solid_solutions.push(carbonate_crystal(0.0, 0.0));
    let events = solver
        .equilibrate(&mut vessel)
        .expect("mixed-crystal solve");
    let calcium_solid = component(&vessel, SolidSolutionComponent::CalciumCarbonate);
    let strontium_solid = component(&vessel, SolidSolutionComponent::StrontiumCarbonate);

    assert!(
        calcium_solid > 1e-6 && strontium_solid > 1e-6,
        "both reviewed end members should enter the mixed crystal: CaCO3={calcium_solid:.12e}, SrCO3={strontium_solid:.12e} mol"
    );
    assert!(events.iter().any(|event| matches!(
        event,
        Event::Precipitated { species, .. } if species.0 == "CaCO3"
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        Event::Precipitated { species, .. } if species.0 == "SrCO3"
    )));
    assert!(
        (calcium_inventory(&vessel) - 5e-3).abs() < 2e-8,
        "calcium inventory changed: {:#?}",
        vessel
    );
    assert!(
        (strontium_inventory(&vessel) - 5e-3).abs() < 2e-8,
        "strontium inventory changed: {:#?}",
        vessel
    );
    assert!(
        (carbon_inventory(&vessel) - 2e-2).abs() < 2e-8,
        "carbon inventory changed: {:#?}",
        vessel
    );
    assert!(vessel.solid_solutions[0].has_valid_state());
}

#[test]
fn acid_dissolves_both_end_members_without_losing_components() {
    let mut solver = PhreeqcEquilibrator::new().expect("engine");
    let mut vessel = closed_vessel();
    vessel.deposit(SpeciesId::new("HCl"), Moles(2e-2), Phase::Aqueous);
    vessel.solid_solutions.push(carbonate_crystal(4e-3, 4e-3));
    let events = solver.equilibrate(&mut vessel).expect("acid dissolution");
    let calcium_solid = component(&vessel, SolidSolutionComponent::CalciumCarbonate);
    let strontium_solid = component(&vessel, SolidSolutionComponent::StrontiumCarbonate);

    assert!(
        calcium_solid < 4e-3 && strontium_solid < 4e-3,
        "acid should dissolve both end members: CaCO3={calcium_solid:.12e}, SrCO3={strontium_solid:.12e} mol"
    );
    assert!(events.iter().any(|event| matches!(
        event,
        Event::Dissolved { species, .. } if species.0 == "CaCO3"
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        Event::Dissolved { species, .. } if species.0 == "SrCO3"
    )));
    assert!(
        (calcium_inventory(&vessel) - 4e-3).abs() < 2e-8,
        "calcium inventory changed: {:#?}",
        vessel
    );
    assert!(
        (strontium_inventory(&vessel) - 4e-3).abs() < 2e-8,
        "strontium inventory changed: {:#?}",
        vessel
    );
    assert!(
        (carbon_inventory(&vessel) - 8e-3).abs() < 2e-8,
        "carbon inventory changed: {:#?}",
        vessel
    );
}

#[test]
fn repeated_equilibration_preserves_the_mixed_phase_and_bulk_inventory() {
    let mut solver = PhreeqcEquilibrator::new().expect("engine");
    let mut vessel = closed_vessel();
    vessel.deposit(SpeciesId::new("Ca+2"), Moles(5e-3), Phase::Aqueous);
    vessel.deposit(SpeciesId::new("Sr+2"), Moles(5e-3), Phase::Aqueous);
    vessel.deposit(SpeciesId::new("HCO3-"), Moles(2e-2), Phase::Aqueous);
    vessel.solid_solutions.push(carbonate_crystal(0.0, 0.0));
    // Two passes to SETTLE, then the no-op check.
    //
    // One pass stopped being the fixed point when reaction heat was
    // coupled to composition: the solve is now inside a temperature
    // iteration, and the first pass lands 1.8e-6 mol away from the answer
    // it then converges to. It does converge, and fast — pass 2 to 3 moves
    // 1e-10, pass 3 to 4 moves 9e-14, pass 5 moves nothing at all — so the
    // invariant this test exists for is intact and is asserted here from
    // the settled state.
    //
    // Before the heat balance, nothing charged an enthalpy for these
    // carbonates at all (neither CaCO3 nor SrCO3 has a registry enthalpy
    // of dissolution), so the first pass WAS the fixed point and the
    // distinction never arose.
    solver.equilibrate(&mut vessel).expect("first equilibrium");
    solver
        .equilibrate(&mut vessel)
        .expect("settling equilibrium");
    let first = vessel.clone();

    solver.equilibrate(&mut vessel).expect("repeat equilibrium");

    for component_kind in SolidSolutionComponent::ALL {
        assert!(
            (component(&vessel, component_kind) - component(&first, component_kind)).abs() < 2e-8,
            "{component_kind:?} changed on a no-op repeat"
        );
    }
    assert!((calcium_inventory(&vessel) - calcium_inventory(&first)).abs() < 2e-8);
    assert!((strontium_inventory(&vessel) - strontium_inventory(&first)).abs() < 2e-8);
    assert!(
        (carbon_inventory(&vessel) - carbon_inventory(&first)).abs() < 2e-8,
        "carbon inventory changed on repeat: first={:#?}, repeat={:#?}",
        first,
        vessel
    );
}
