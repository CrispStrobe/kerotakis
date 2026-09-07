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

fn zinc_adsorption_at_acid_dose(acid_moles: f64) -> (f64, f64, f64) {
    let mut solver = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    bench.vessels[0].surfaces.push(hfo("oxide grains", 1.0));

    add(&mut bench, &mut solver, "water", 55.51);
    if acid_moles > 0.0 {
        add(&mut bench, &mut solver, "HCl", acid_moles);
    }
    add(&mut bench, &mut solver, "ZnSO4", 1e-4);

    let state = bench.vessel(VesselId(0)).expect("vessel");
    let ph = state.solution.as_ref().expect("solution").ph;
    let bound = state.surfaces[0].bound(SurfaceSorbate::Zinc).0;
    (ph, bound, zinc_inventory(state))
}

fn export_oracle_candidate(cases: &[(&str, (f64, f64, f64))]) {
    let Some(path) = std::env::var_os("KERO_SURFACE_ORACLE_OUTPUT") else {
        return;
    };
    let version = std::env::var("KERO_SURFACE_ORACLE_VERSION")
        .expect("set KERO_SURFACE_ORACLE_VERSION to the tested git revision when exporting");
    let retrieved = std::env::var("KERO_SURFACE_ORACLE_DATE")
        .expect("set KERO_SURFACE_ORACLE_DATE to YYYY-MM-DD when exporting");
    let cases: Vec<_> = cases
        .iter()
        .map(|(id, (ph, bound, inventory))| {
            serde_json::json!({
                "id": id,
                "ph": ph,
                "bound_zinc_mol": bound,
                "total_zinc_mol": inventory,
            })
        })
        .collect();
    let document = serde_json::json!({
        "schema": 1,
        "benchmark": "hfo-zinc-ph-edge-v1",
        "producer": {"name": "kerotakis-phreeqc", "version": version},
        "retrieved": retrieved,
        "cases": cases,
    });
    let bytes = serde_json::to_vec_pretty(&document).expect("serialize surface oracle candidate");
    std::fs::write(path, bytes).expect("write surface oracle candidate outside the repository");
}

#[test]
fn zinc_adsorption_increases_across_the_acid_side_ph_edge() {
    // Stay on the acid side of the adsorption edge: an alkaline endpoint
    // would also make zinc hydroxide a candidate phase and stop being a
    // clean test of surface affinity. The three systems differ only in HCl
    // dose; surface capacity, water and analytical zinc are identical.
    let acidic = zinc_adsorption_at_acid_dose(1e-2);
    let shoulder = zinc_adsorption_at_acid_dose(1e-4);
    let least_acidic = zinc_adsorption_at_acid_dose(0.0);
    let cases = [acidic, shoulder, least_acidic];

    export_oracle_candidate(&[
        ("hcl-1e-2-mol", acidic),
        ("hcl-1e-4-mol", shoulder),
        ("no-added-acid", least_acidic),
    ]);

    for (ph, bound, inventory) in cases {
        assert!(ph.is_finite(), "surface solve returned non-finite pH");
        assert!(
            (inventory - 1e-4).abs() < 2e-8,
            "zinc must remain conserved at pH {ph:.4}: total={inventory:.12e} mol"
        );
        assert!(
            (0.0..=2.05e-4 + 1e-12).contains(&bound),
            "bound zinc must remain within finite site capacity at pH {ph:.4}: {bound:.12e} mol"
        );
    }

    assert!(
        acidic.0 < shoulder.0 && shoulder.0 < least_acidic.0,
        "acid doses must produce an ordered pH edge: {acidic:?}, {shoulder:?}, {least_acidic:?}"
    );
    assert!(
        acidic.1 < shoulder.1 && shoulder.1 < least_acidic.1,
        "HFO-bound zinc must increase with pH: {acidic:?}, {shoulder:?}, {least_acidic:?}"
    );
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
    let first_water_release = state.surfaces[0].water_release.0;
    let first_neutral_water = state.moles_of(&SpeciesId::new("water")).0 - first_water_release;
    let stir_events = step(
        &mut bench,
        &mut solver,
        Operator::Stir {
            vessel: VesselId(0),
            rpm: 500.0,
            seconds: 10.0,
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
    let settled_water = settled.moles_of(&SpeciesId::new("water")).0;
    let settled_water_release = settled.surfaces[0].water_release.0;
    assert!(
        (settled_water - settled_water_release - first_neutral_water).abs() < 1e-10,
        "re-equilibrating must conserve the neutral surface/water reference: first neutral={first_neutral_water:.12e} mol, settled neutral={:.12e} mol, settled water={settled_water:.12e} mol, first release={first_water_release:.12e} mol, settled release={settled_water_release:.12e} mol",
        settled_water - settled_water_release,
    );
    assert!(
        (settled.surfaces[0].bound(SurfaceSorbate::Zinc).0 - first_bound).abs() < 2e-8,
        "re-equilibrating must not change the equilibrium bound inventory: first={first_bound:.12e} mol, settled={:.12e} mol, delta={:.12e} mol",
        settled.surfaces[0].bound(SurfaceSorbate::Zinc).0,
        settled.surfaces[0].bound(SurfaceSorbate::Zinc).0 - first_bound,
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
fn sequential_surface_solves_do_not_share_numbered_reactants() {
    let mut solver = PhreeqcEquilibrator::new().expect("engine");
    let solve = |solver: &mut PhreeqcEquilibrator, id: usize, zinc: f64| {
        let mut vessel = Vessel::new(VesselId(id), format!("surface cell {id}"));
        vessel.deposit(SpeciesId::new("water"), Moles(5.5509), Phase::Liquid);
        vessel.deposit(SpeciesId::new("Zn+2"), Moles(zinc), Phase::Aqueous);
        vessel.deposit(SpeciesId::new("SO4-2"), Moles(zinc), Phase::Aqueous);
        vessel.surfaces.push(hfo(&format!("oxide {id}"), 1.0));
        solver
            .equilibrate(&mut vessel)
            .expect("surface equilibrium");
        vessel
    };

    let first = solve(&mut solver, 0, 1e-4);
    let second = solve(&mut solver, 1, 1e-5);

    assert!((zinc_inventory(&first) - 1e-4).abs() < 2e-8);
    assert!(
        (zinc_inventory(&second) - 1e-5).abs() < 2e-8,
        "the second vessel must own only its own zinc: dissolved={:.12e}, bound={:.12e}",
        second.moles_of(&SpeciesId::new("Zn+2")).0,
        second.surfaces[0].bound(SurfaceSorbate::Zinc).0,
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
