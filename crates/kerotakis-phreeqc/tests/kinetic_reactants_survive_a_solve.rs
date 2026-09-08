//! The same guard `curated_reactants_survive_a_solve` gives curated
//! reactions, for RATE LAWS, which have never had one.
//!
//! This file exists because the identical mistake has now been made and
//! fixed by hand twice in three days, both times found by reading rather
//! than by a test:
//!
//! - #536 borrowed thiosulfate's acid constant, and `thiosulfate-acid`
//!   named `Na2S2O3` in its stoichiometry and its rate orders. A solved
//!   beaker holds `Na+` and `S2O3-2` and no `Na2S2O3`.
//! - this change borrows sulfite's, and `iodate-bisulfite-clock` named
//!   `NaHSO3` in both. A solved beaker holds `Na+`, `HSO3-` and `SO3-2`.
//!
//! Both would have matched in a dry vessel, failed in every wet one, and
//! SAID NOTHING EITHER WAY - no error, no refusal, no honesty event, just
//! a clock that never ticks. `curated::REACTIONS` has been guarded against
//! exactly this since the vinegar-and-baking-soda bug;
//! `kinetics::REGISTRY` has not, and the whole of the protection has been
//! that somebody remembered.
//!
//! Nothing here is specific to sulfite. The next borrow gets this for free.

#![cfg(feature = "engine")]

use kerotakis_core::kinetics;
use kerotakis_phreeqc::derived::{self, DerivedRole};

/// The registry keys a vessel holds after a solve, for a species entered
/// under `key`. Deliberately the same arithmetic as the curated guard's
/// helper of the same name: if the two ever disagree, one of them is
/// describing a readback that does not happen.
fn keys_after_a_solve(key: &str) -> Option<Vec<String>> {
    match derived::role(key)? {
        DerivedRole::Solvent | DerivedRole::Mineral { .. } => None,
        DerivedRole::Dissolves(elements) => {
            let mut out = Vec::new();
            for (element, _) in elements {
                match derived::protonation_split(element) {
                    Some(split) => out.extend(split.iter().map(|(_, key)| (*key).to_string())),
                    None => out.extend(derived::booking_ion(element).map(str::to_string)),
                }
            }
            Some(out)
        }
    }
}

/// A reactant survives if the solve does not touch it, or if it is still
/// there under the name the rate law uses.
fn survives(key: &str) -> bool {
    keys_after_a_solve(key).is_none_or(|after| after.iter().any(|k| k == key))
}

/// Every aqueous rate law must be able to find its own reactants in a
/// vessel that has been through the aqueous readback.
///
/// Only the CONSUMED terms and the rate ORDERS are checked, and that is
/// the whole of the property: a product may be written under a bottle name
/// and simply speciate on the next solve, but a reactant that has been
/// renamed is a reaction that cannot fire.
///
/// Non-aqueous laws are exempt because the readback never sees them - a
/// solid rusting in air is not a solute.
#[test]
fn every_aqueous_rate_law_can_find_its_own_reactants_after_a_solve() {
    let mut broken: Vec<String> = Vec::new();
    for reaction in kinetics::REGISTRY.iter() {
        for term in reaction.stoichiometry.iter().filter(|t| t.coefficient < 0.0) {
            if term.phase != kerotakis_core::Phase::Aqueous {
                continue;
            }
            if !survives(term.species) {
                broken.push(format!(
                    "{}: consumes {:?}, which a solve renames to {:?}",
                    reaction.id,
                    term.species,
                    keys_after_a_solve(term.species).unwrap_or_default()
                ));
            }
        }
        for order in reaction.forward.orders.iter() {
            if order.phase != Some(kerotakis_core::Phase::Aqueous) {
                continue;
            }
            if !survives(order.species) {
                broken.push(format!(
                    "{}: rate order in {:?}, which a solve renames to {:?}",
                    reaction.id,
                    order.species,
                    keys_after_a_solve(order.species).unwrap_or_default()
                ));
            }
        }
    }
    assert!(
        broken.is_empty(),
        "these rate laws name reactants a solved vessel no longer holds, so they \
         would match in a dry beaker and go silent in a wet one:\n  {}",
        broken.join("\n  ")
    );
}
