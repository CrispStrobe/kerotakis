#![cfg(feature = "engine")]
//! Milk has a pH, and the pH is the mineral buffer's.
//!
//! Before the serum minerals were resolved, `whole_milk` was
//! `[water 0.87]` with everything else conserved as unresolved solids. A
//! vessel of it held one species, that species was the solvent, and the
//! aqueous tail characterised no solution at all — so `measure ph` on a
//! beaker of milk read nothing, and the two yoghurt rows in the curiosity
//! corpus were `missing` for that reason rather than for want of a
//! fermentation. The recipe now books milk's diffusible phase: potassium,
//! sodium, the soluble share of the calcium, chloride, inorganic phosphate
//! and citrate. This file is what holds that to the number.
//!
//! What is pinned here is a WINDOW, not a digit. Fresh cow's milk is
//! pH 6.6 to 6.8 and the recipe is built to land inside it; the window is
//! about the chemistry rather than about one database revision, and the
//! failure message prints what the tail actually computed so that a
//! reviewer who moves a mineral can see where it went.
//!
//! What is NOT pinned here, and cannot be: an acidified milk. Casein is
//! not modelled, and between pH 6.6 and pH 5.0 casein and its colloidal
//! calcium phosphate carry more of milk's buffer capacity than the salts
//! do. A yoghurt pH computed from this recipe is therefore a LOWER bound
//! on the real thing at the same acid dose — the recipe's own lot
//! assumptions say so — which is why the acidification test below asserts
//! an ordering against water and not a value.

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn stack() -> SolverStack {
    SolverStack::new(kerotakis_stack::standard_solvers(vec![Box::new(
        PhreeqcEquilibrator::new().expect("engine"),
    )]))
}

/// `grams` of the whole-milk recipe in a beaker, equilibrated.
fn milk(grams: f64) -> Vessel {
    let recipe = kerotakis_core::material::lookup("whole_milk", None).expect("the milk recipe");
    let mut bench = Bench::new();
    let mut solvers = stack();
    bench
        .step_with(
            Operator::AddMaterial {
                vessel: VesselId(0),
                material: recipe.canonical_key.clone(),
                recipe_id: recipe.id.clone(),
                recipe_version: recipe.version,
                total_amount: grams,
                basis: recipe.basis,
                sample_seed: 0,
                at: None,
            },
            &mut solvers,
            &PermissiveScreen,
        )
        .expect("add milk");
    bench.vessel(VesselId(0)).expect("the beaker").clone()
}

fn solution(vessel: &Vessel) -> SolutionInfo {
    vessel
        .solution
        .clone()
        .expect("milk's mineral buffer must characterise a solution")
}

/// 100 mL of fresh milk, and the number a pH meter dipped into it reads.
#[test]
fn fresh_milk_reads_the_ph_of_fresh_milk() {
    let vessel = milk(103.0);
    let info = solution(&vessel);
    assert!(
        (6.4..=7.0).contains(&info.ph),
        "fresh milk is pH 6.6 to 6.8 and this recipe is built to land in it; \
         the tail computed pH {} at ionic strength {} mol/kgw",
        info.ph,
        info.ionic_strength
    );
    // Milk's serum is a real ionic strength, not a trace. Holt's diffusate
    // is quoted at 0.073 mol/kgw; anything an order of magnitude away means
    // a mineral did not reach the solver.
    assert!(
        (0.02..=0.2).contains(&info.ionic_strength),
        "milk serum's ionic strength should be near 0.07 mol/kgw, got {}",
        info.ionic_strength
    );
}

/// The buffer has to reach the database that can speciate it. Citrate and
/// free phosphoric acid live only in minteq.v4 among the three files this
/// lab loads, and the router picks by the elements the problem produced —
/// so this also proves the citrate and phosphate got there at all.
#[test]
fn the_milk_buffer_routes_to_the_dataset_that_carries_citrate_and_phosphate() {
    let vessel = milk(103.0);
    let info = solution(&vessel);
    let dataset = info
        .provenance
        .as_ref()
        .map(|p| p.dataset.clone())
        .unwrap_or_default();
    assert!(
        dataset.contains("minteq"),
        "citrate and free phosphate live only in minteq.v4; routed to {dataset:?}"
    );
}

/// A buffer is a thing that resists, so the test of one is a comparison
/// and not a value. The same dose of strong acid into milk and into the
/// water milk is mostly made of: the milk must end up markedly less acid.
///
/// The dose is deliberately small — 0.5 mmol into 100 mL, well inside what
/// the phosphate and citrate can absorb — because the claim being made is
/// that the buffer exists, not that it is milk's whole buffer. It is not:
/// casein is missing, and the recipe says so.
#[test]
fn the_mineral_buffer_actually_buffers() {
    let acid = 0.000_5;

    let recipe = kerotakis_core::material::lookup("whole_milk", None).expect("the milk recipe");
    let mut bench = Bench::new();
    let mut solvers = stack();
    bench
        .step_with(
            Operator::AddMaterial {
                vessel: VesselId(0),
                material: recipe.canonical_key.clone(),
                recipe_id: recipe.id.clone(),
                recipe_version: recipe.version,
                total_amount: 103.0,
                basis: recipe.basis,
                sample_seed: 0,
                at: None,
            },
            &mut solvers,
            &PermissiveScreen,
        )
        .expect("add milk");
    bench
        .step_with(
            Operator::Add {
                vessel: VesselId(0),
                species: SpeciesId::new("HCl"),
                moles: Moles(acid),
                at: None,
            },
            &mut solvers,
            &PermissiveScreen,
        )
        .expect("acidify the milk");
    let acidified_milk = solution(bench.vessel(VesselId(0)).expect("the beaker")).ph;

    // The same acid into the same mass of plain water.
    let mut plain = Bench::new();
    let mut plain_solvers = stack();
    for (key, moles) in [("water", 89.61 / 18.015), ("HCl", acid)] {
        plain
            .step_with(
                Operator::Add {
                    vessel: VesselId(0),
                    species: SpeciesId::new(key),
                    moles: Moles(moles),
                    at: None,
                },
                &mut plain_solvers,
                &PermissiveScreen,
            )
            .expect("acidify the water");
    }
    let acidified_water = plain
        .vessel(VesselId(0))
        .expect("the beaker")
        .solution
        .clone()
        .expect("acid in water is a solution")
        .ph;

    assert!(
        acidified_milk > acidified_water + 1.0,
        "0.5 mmol of HCl takes water to pH {acidified_water} and must leave buffered \
         milk far above it; milk went to pH {acidified_milk}"
    );
    assert!(
        acidified_milk > 5.5,
        "0.5 mmol of HCl is well inside what milk's phosphate and citrate absorb; \
         milk went to pH {acidified_milk}"
    );
}
