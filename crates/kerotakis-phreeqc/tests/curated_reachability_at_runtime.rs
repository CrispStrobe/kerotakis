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
//! the reaction. Three reactions were reachable in one order and silent in
//! the other, and every one of them was found by accident:
//!
//!   NaHCO₃ + CH₃COOH        vinegar and baking soda
//!   CaCO₃ + 2 CH₃COOH       vinegar on an eggshell
//!   NaOCl + 2 HCl           why bleach and acid are never mixed
//!
//! The last of those is a hazard demonstration, and it was silent in the
//! order where the acid is already in the beaker. Nobody would have found
//! that from a bug report: pouring one way round worked.
//!
//! THIS TEST HAS BEEN CHECKED AGAINST THE BUG IT IS FOR. Delete the
//! `HCO₃⁻ + CH₃COOH` row from `curated.rs` and it fails with
//!
//!     silent when added in the order ["NaHCO3", "CH3COOH"]
//!
//! which is the defect kerotakis-5f found by accident. A guard that passes
//! on a fixed tree has proved nothing about its own power; this one has
//! been shown to catch the thing it was written for.
//!
//! A reaction is counted reachable if ANY curated equation fires — not
//! necessarily its own. That is deliberate: the established remedy for a
//! renamed reagent is a sibling row written in the names the vessel holds
//! (`MnO₄⁻` beside `KMnO4`, `HCO₃⁻` beside `NaHCO3`), and what matters is
//! that the capability survives, not which row carries it.

#![cfg(feature = "engine")]

use kerotakis_core::curated::{CuratedReaction, REACTIONS};
use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;
use kerotakis_safety::ReactiveGroupScreen;

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
    let mut stack = stack();
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
    if let Some(catalyst) = reaction.catalyst {
        step(&mut bench, &mut stack, catalyst, 0.01);
    }
    order.iter().fold(false, |fired, (key, moles)| {
        let events = step(&mut bench, &mut stack, key, *moles);
        fired
            || events
                .iter()
                .any(|e| matches!(e, Event::ReactionOccurred { .. }))
    })
}

/// Reactions this test cannot pose, with the reason. Not a list of
/// failures — a list of things the harness above is not equipped to build.
const NOT_POSED: &[(&str, &str)] = &[(
    "NaOCl + Cl⁻ + 2 H⁺ → Cl2↑ + Na⁺ + H₂O",
    "one of its three reagents is the vessel's acidity, which is not a \
     species and so is not in `reactants` for this harness to add. Adding \
     bare `Cl-` happens to supply some — an anion with no cation IS a \
     charge imbalance, which is what this bench means by free acid — but \
     only after a solve has recorded it, so the answer depends on whether \
     the chloride went in first or last and neither answer is about the \
     reaction. Both real orders are covered directly in `acid_base.rs`, \
     along with the case that matters most: brine and bleach doing nothing.",
)];

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
            if !fires_when_added_in(&order, reaction) {
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
