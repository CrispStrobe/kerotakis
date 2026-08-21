//! Live finite-capacity adsorption through PHREEQC `SURFACE`.

#![cfg(feature = "engine")]

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn hfo(label: &str, scale: f64) -> SurfaceSites {
    SurfaceSites {
        label: label.to_string(),
        model: SurfaceModel::HydrousFerricOxide,
        mass: Grams(0.09 * scale),
        specific_area_m2_per_g: 600.0,
        strong_capacity: Moles(5e-6 * scale),
        weak_capacity: Moles(2e-4 * scale),
        occupancy: Vec::new(),
        water_release: Moles(0.0),
    }
}

fn step(bench: &mut Bench, solver: &mut PhreeqcEquilibrator, operator: Operator) -> Vec<Event> {
    bench
        .step_with(operator, solver, &PermissiveScreen)
        .expect("bench step")
}

fn add(
    bench: &mut Bench,
    solver: &mut PhreeqcEquilibrator,
    species: &str,
    moles: f64,
) -> Vec<Event> {
    step(
        bench,
        solver,
        Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new(species),
            moles: Moles(moles),
            at: None,
        },
    )
}

fn zinc_inventory(vessel: &Vessel) -> f64 {
    vessel.moles_of(&SpeciesId::new("Zn+2")).0
        + vessel
            .surfaces
            .iter()
            .map(|surface| surface.bound(SurfaceSorbate::Zinc).0)
            .sum::<f64>()
}

fn sulfate_inventory(vessel: &Vessel) -> f64 {
    vessel.moles_of(&SpeciesId::new("SO4-2")).0
        + vessel
            .surfaces
            .iter()
            .map(|surface| surface.bound(SurfaceSorbate::Sulfate).0)
            .sum::<f64>()
}

#[test]
fn hydrous_ferric_oxide_retains_finite_zinc_occupancy() {
    let mut solver = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    bench.vessels[0].surfaces.push(hfo("oxide grains", 1.0));

    add(&mut bench, &mut solver, "water", 55.51);
    let before_zinc = bench.vessel(VesselId(0)).unwrap();
    let mass_before_zinc = before_zinc.mass().0;
    let water_before_zinc = before_zinc.moles_of(&SpeciesId::new("water")).0;
    add(&mut bench, &mut solver, "ZnSO4", 1e-4);

    let state = bench.vessel(VesselId(0)).unwrap();
    let surface = &state.surfaces[0];
    let bound = surface.bound(SurfaceSorbate::Zinc).0;
    assert!(bound > 0.0, "the oxide should bind some dissolved zinc");
    assert!(
        surface.bound(SurfaceSorbate::Sulfate).0 > 0.0,
        "the oxide should retain its adsorbed sulfate counterion"
    );
    assert!(bound <= surface.strong_capacity.0 + surface.weak_capacity.0 + 1e-12);
    assert!(surface.occupied(SurfaceSiteKind::Strong).0 <= surface.strong_capacity.0 + 1e-12);
    assert!(surface.occupied(SurfaceSiteKind::Weak).0 <= surface.weak_capacity.0 + 1e-12);
    assert!(
        (zinc_inventory(state) - 1e-4).abs() < 2e-8,
        "first zinc inventory: total={:.12e}, dissolved={:.12e}, bound={bound:.12e}",
        zinc_inventory(state),
        state.moles_of(&SpeciesId::new("Zn+2")).0,
    );
    assert!((sulfate_inventory(state) - 1e-4).abs() < 2e-8);
    assert!(
        (state.moles_of(&SpeciesId::new("water")).0 - water_before_zinc - surface.water_release.0)
            .abs()
            < 1e-10,
        "the interface ledger must own water released by ligand exchange"
    );
    let added_mass = species::lookup_key("ZnSO4").unwrap().molar_mass * 1e-4;
    let mass_error = state.mass().0 - mass_before_zinc - added_mass;
    assert!(
        mass_error.abs() < 2e-5,
        "surface mass ledger: before={mass_before_zinc:.12e} g, after={:.12e} g, added={added_mass:.12e} g, error={mass_error:.12e} g; water={:.12e} mol, surface water release={:.12e} mol, dissolved Zn={:.12e} mol, bound Zn={bound:.12e} mol, dissolved sulfate={:.12e} mol, bound sulfate={:.12e} mol",
        state.mass().0,
        state.moles_of(&SpeciesId::new("water")).0,
        surface.water_release.0,
        state.moles_of(&SpeciesId::new("Zn+2")).0,
        state.moles_of(&SpeciesId::new("SO4-2")).0,
        surface.bound(SurfaceSorbate::Sulfate).0,
    );

    let first_bound = bound;
    let first_mass = state.mass().0;
    let first_water = state.moles_of(&SpeciesId::new("water")).0;
    let stir_events = step(
        &mut bench,
        &mut solver,
        Operator::Stir {
            vessel: VesselId(0),
        },
    );
    let settled = bench.vessel(VesselId(0)).unwrap();
    assert!(
        (zinc_inventory(settled) - 1e-4).abs() < 2e-8,
        "repeat zinc inventory: total={:.12e}, dissolved={:.12e}, bound={:.12e}; events={stir_events:?}",
        zinc_inventory(settled),
        settled.moles_of(&SpeciesId::new("Zn+2")).0,
        settled.surfaces[0].bound(SurfaceSorbate::Zinc).0,
    );
    assert!(
        (sulfate_inventory(settled) - 1e-4).abs() < 2e-8,
        "repeat sulfate inventory: total={:.12e}, dissolved={:.12e}, bound={:.12e}; events={stir_events:?}",
        sulfate_inventory(settled),
        settled.moles_of(&SpeciesId::new("SO4-2")).0,
        settled.surfaces[0].bound(SurfaceSorbate::Sulfate).0,
    );
    assert!((settled.mass().0 - first_mass).abs() < 2e-5);
    assert!(
        (settled.moles_of(&SpeciesId::new("water")).0 - first_water).abs() < 1e-10,
        "re-equilibrating must not release the same ligand-exchange water twice"
    );
    assert!(
        (settled.surfaces[0].bound(SurfaceSorbate::Zinc).0 - first_bound).abs() < 2e-8,
        "re-equilibrating must not lose the previously bound inventory"
    );
}

#[test]
fn pooled_solver_result_returns_to_each_named_interface() {
    let mut solver = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    let vessel = &mut bench.vessels[0];
    vessel.surfaces.push(hfo("small bed", 1.0));
    vessel.surfaces.push(hfo("large bed", 3.0));

    add(&mut bench, &mut solver, "water", 55.51);
    add(&mut bench, &mut solver, "ZnSO4", 2e-4);

    let state = bench.vessel(VesselId(0)).unwrap();
    let small = state.surfaces[0].bound(SurfaceSorbate::Zinc).0;
    let large = state.surfaces[1].bound(SurfaceSorbate::Zinc).0;
    assert!(small > 0.0);
    assert!((large / small - 3.0).abs() < 1e-9);
    assert!(
        (zinc_inventory(state) - 2e-4).abs() < 2e-8,
        "pooled zinc inventory: total={:.12e}, dissolved={:.12e}, bound={:.12e}",
        zinc_inventory(state),
        state.moles_of(&SpeciesId::new("Zn+2")).0,
        small + large,
    );
}

#[test]
fn untracked_surface_complexes_fail_loudly_instead_of_losing_inventory() {
    let mut solver = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    bench.vessels[0].surfaces.push(hfo("oxide grains", 1.0));

    add(&mut bench, &mut solver, "water", 55.51);
    let events = add(&mut bench, &mut solver, "CaCl2", 1e-4);

    assert!(events.iter().any(|event| matches!(
        event,
        Event::SolverFailed { detail, .. }
            if detail.contains("can adsorb Ca") && detail.contains("typed interface ledger")
    )));
    assert_eq!(
        bench
            .vessel(VesselId(0))
            .unwrap()
            .moles_of(&SpeciesId::new("CaCl2")),
        Moles(1e-4),
        "a refused surface solve must leave the added material intact"
    );
}
