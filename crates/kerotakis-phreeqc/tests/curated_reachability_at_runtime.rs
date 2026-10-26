//! Can each curated reaction still be reached after the vessel has been
//! solved — and can it be reached whichever order the reagents go in?
//!
//! `curated_reactants_survive_a_solve.rs` asks the first half of that
//! statically, by walking each reactant through the renaming the readback
//! would do to it. That walk has a blind spot it cannot close: it exempts
//! minerals, on the grounds that a mineral books back as itself, and that
//! holds only while the mineral is SATURATED. `NaHCO3` dissolves completely
//! and is renamed as thoroughly as any acid. Whether a solid survives is a
//! question about a particular vessel, and a static walk has no vessel.
//!
//! So this file asks the question the only way it can actually be answered:
//! by building the vessel, solving it, and looking.
//!
//! ORDER IS THE WHOLE POINT. Every instance of this defect found so far was
//! invisible in one order and obvious in the other, because `curated` runs
//! before the aqueous tail — so on the step where a reagent is ADDED the
//! ledger still holds it as written, and the match succeeds. Add it first
//! instead, let it go through a solve, and its name has changed underneath
//! the reaction. The bicarbonate/acetic overlap now has an explicit native
//! ownership route, so that pair is checked for actual final-dose CO2 and a
//! closed native material ledger; the other curated reactions still require
//! their existing curated-event behavior.
//!
//!   NaHCO₃ + CH₃COOH        vinegar and baking soda
//!   CaCO₃ + 2 CH₃COOH       vinegar on an eggshell
//!   NaOCl + 2 HCl           why bleach and acid are never mixed
//!
//! The last of those is a hazard demonstration, and it was silent in the
//! order where the acid is already in the beaker. Nobody would have found
//! that from a bug report: pouring one way round worked.
//!
//! Historically, deleting the ionic bicarbonate sibling made this integrated
//! test fail on soda-first addition. That history remains frozen in the audit.
//! The selected native owner now handles that overlap; requiring its old
//! curated event would misidentify a working native route as silence. Direct
//! and no-owner curated fallback remain explicit core mechanism controls.
//!
//! For curated-owned reactions, reachability still means ANY curated
//! equation fires — not necessarily its own. That is deliberate: the
//! established remedy for a renamed reagent is a sibling row written in the
//! names the vessel holds (`MnO₄⁻` beside `KMnO4`). The selected
//! bicarbonate/acetic native route is the explicit exception and is checked
//! above by its final-dose physical outcome and closed material ledger.

#![cfg(feature = "engine")]

use kerotakis_core::curated::{CuratedReaction, REACTIONS};
use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;
use kerotakis_safety::ReactiveGroupScreen;
#[path = "support/native_material_checks.rs"]
mod native_material_checks;

fn stack() -> SolverStack {
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(CuratedEquilibrator),
        Box::new(PhreeqcEquilibrator::new().expect("engine")),
        Box::new(HonestyEquilibrator),
    ])
}

/// Put the reagents in a beaker of water in the given order, letting the
/// bench solve between each, and report whether a curated equation fired at
/// ANY point.
///
/// At any point, not on the last addition. The first version watched only
/// the final step and reported the starch hydrolysis as unreachable: water
/// is one of its reagents and the beaker already has 5.5 mol of it, so the
/// reaction had fired when the starch went in and there was nothing left to
/// do when the rotation added more water. The reaction was not missing; the
/// question was.
fn fires_when_added_in(order: &[(&str, f64)], reaction: &CuratedReaction) -> bool {
    let mut bench = Bench::new();
    let mut stack = if reaction.post_aqueous.is_some() {
        SolverStack::new(kerotakis_stack::standard_solvers(vec![Box::new(
            DisplacementEquilibrator::wrapping(Box::new(
                PhreeqcEquilibrator::new().expect("engine"),
            )),
        )]))
    } else {
        stack()
    };
    let v = VesselId(0);
    let step = |bench: &mut Bench, stack: &mut SolverStack, key: &str, moles: f64| {
        bench
            .step_with(
                Operator::Add {
                    vessel: v,
                    species: SpeciesId::new(key),
                    moles: Moles(moles),
                    at: None,
                },
                stack,
                &ReactiveGroupScreen,
            )
            .unwrap_or_default()
    };
    step(&mut bench, &mut stack, "water", 5.5);
    if reaction.post_aqueous.is_some() {
        // This row explicitly owns six proton equivalents per extent and a
        // sulfate medium. Its ionic reactant list alone is not a complete kit.
        step(&mut bench, &mut stack, "H2SO4", 0.08);
    }
    if let Some(catalyst) = reaction.catalyst {
        step(&mut bench, &mut stack, catalyst, 0.01);
    }
    order.iter().fold(false, |fired, (key, moles)| {
        let stock = if reaction.post_aqueous.is_some() && *key == "MnO4-" {
            "KMnO4" // complete potassium counterion, same ionic reactant
        } else {
            key
        };
        let events = step(&mut bench, &mut stack, stock, *moles);
        fired
            || events.iter().any(|e| {
                matches!(e, Event::ReactionOccurred { equation, .. }
                    if reaction.post_aqueous.is_none() || equation == reaction.equation)
            })
    })
}

fn is_bicarbonate_acetic_overlap(reaction: &CuratedReaction) -> bool {
    if reaction.reactants.len() != 2 {
        return false;
    }
    let has_acid = reaction
        .reactants
        .iter()
        .any(|(species, _)| *species == "CH3COOH");
    let has_bicarbonate = reaction
        .reactants
        .iter()
        .any(|(species, _)| *species == "NaHCO3" || *species == "HCO3-");
    has_acid && has_bicarbonate
}

fn native_bicarbonate_acetic_fizz_when_added_in(order: &[(&str, f64)]) -> bool {
    let mut bench = Bench::new();
    let mut stack = stack();
    let vessel = VesselId(0);
    let add_water = |bench: &mut Bench, stack: &mut SolverStack| {
        bench
            .step_with(
                Operator::Add {
                    vessel,
                    species: SpeciesId::new("water"),
                    moles: Moles(5.5),
                    at: None,
                },
                stack,
                &ReactiveGroupScreen,
            )
            .unwrap_or_else(|error| panic!("water setup failed: {error:?}"))
    };
    let water_events = add_water(&mut bench, &mut stack);
    assert!(
        !water_events
            .iter()
            .any(|event| matches!(event, Event::SolverFailed { .. })),
        "water setup failed: {water_events:?}"
    );

    for (index, (key, moles)) in order.iter().enumerate() {
        // A bare bicarbonate row is a reaction identity, not a complete
        // neutral stock. Pose the same dose with its actual Na counterion;
        // keep its moles unchanged and count NaHCO3 in the inlet ledger.
        let actual_stock = if *key == "HCO3-" { "NaHCO3" } else { *key };
        let events = native_material_checks::add_with_native_material_closure(
            &mut bench,
            &mut stack,
            vessel,
            actual_stock,
            *moles,
            "bicarbonate/acetic order reachability",
        );
        if index + 1 == order.len() {
            return native_material_checks::positive_co2_gas_out(&events) > 1e-4;
        }
    }
    false
}

/// Reactions this test cannot pose, with the reason. Not a list of
/// failures — a list of things the harness above is not equipped to build.
const NOT_POSED: &[(&str, &str)] = &[
    (
        "NaOCl + Cl⁻ + 2 H⁺ → Cl2↑ + Na⁺ + H₂O",
        "one of its three reagents is the vessel's acidity, which is not a \
     species and so is not in `reactants` for this harness to add. Adding \
     bare `Cl-` happens to supply some — an anion with no cation IS a \
     charge imbalance, which is what this bench means by free acid — but \
     only after a solve has recorded it, so the answer depends on whether \
     the chloride went in first or last and neither answer is about the \
     reaction. Both real orders are covered directly in `acid_base.rs`, \
     along with the case that matters most: brine and bleach doing nothing.",
    ),
    (
        "ClO⁻ + Cl⁻ + 2 H⁺ → Cl2↑ + H₂O",
        "the sibling of the row above, written on the anion a solved bleach \
         solution holds, and not posable here for exactly the same reason: \
         its third reagent is the vessel's acidity.",
    ),
    (
        "HClO + Cl⁻ + H⁺ → Cl2↑ + H₂O",
        "the same row on the couple's acid form, and unposable for the same \
         reason a third time: the proton it needs is the vessel's acidity, \
         which is a charge imbalance rather than a species this harness can \
         add.",
    ),
];

#[test]
fn every_curated_reaction_is_reachable_in_every_order() {
    let mut unreachable: Vec<String> = Vec::new();
    for reaction in REACTIONS {
        // A solvent-gated reaction wants a non-aqueous bench, which is a
        // different harness. `curated_reactants_survive_a_solve` covers
        // them: with no water there is no readback to rename anything.
        if reaction.solvent.is_some() || reaction.min_temp_k.is_some() {
            continue;
        }
        if NOT_POSED.iter().any(|(eq, _)| *eq == reaction.equation) {
            continue;
        }
        // Enough of each to be unambiguous, in its own coefficient ratio,
        // and small enough to dissolve.
        let amounts: Vec<(&str, f64)> = reaction
            .reactants
            .iter()
            .map(|(key, coeff)| (*key, 0.02 * coeff))
            .collect();

        // Every rotation, so that each reagent gets a turn at going in
        // first and being solved before the others arrive.
        for skip in 0..amounts.len() {
            let mut order = amounts.clone();
            order.rotate_left(skip);
            let reachable = if is_bicarbonate_acetic_overlap(reaction) {
                native_bicarbonate_acetic_fizz_when_added_in(&order)
            } else {
                fires_when_added_in(&order, reaction)
            };
            if !reachable {
                let names: Vec<&str> = order.iter().map(|(k, _)| *k).collect();
                unreachable.push(format!(
                    "  {}\n    silent when added in the order {:?}",
                    reaction.equation, names
                ));
            }
        }
    }
    assert!(
        unreachable.is_empty(),
        "curated reactions that cannot be reached in some order of adding \
         their own reagents. `curated` runs before the aqueous tail, so a \
         reagent added FIRST goes through a solve and is renamed before the \
         others arrive — and the reaction that names it can no longer match. \
         The remedy is a sibling row written in the names the vessel holds \
         afterwards, as `HCO₃⁻` is written beside `NaHCO3`:\n{}",
        unreachable.join("\n")
    );
}
