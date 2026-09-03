//! A curated reaction can only fire on reactants the vessel still holds
//! under the keys its reactant list names.
//!
//! That sounds like a tautology and it was not one. `NaHCO₃ + CH₃COOH →
//! CH₃COONa + H₂O + CO₂↑` and `CaCO₃ + 2 CH₃COOH → …` — vinegar and baking
//! soda, vinegar on an eggshell, the two most-performed reactions in school
//! chemistry — were curated, reviewed, carried provenance, and could not
//! fire in a beaker with water in it. The aqueous readback books an element
//! total as one ion, and Acetate's was `CH3COO-`; so pouring vinegar into
//! water handed back acetate ion, and the acid named in the reactant list
//! was no longer in the vessel by the time the second reagent arrived. They
//! would only ever have fired in a dry vessel, which is not how anybody
//! does it.
//!
//! Both were silent for as long as they existed. There was no error, no
//! refusal, no honesty event: the reagents went in, the aqueous route
//! answered on its own, and the answer was chemically reasonable. Only the
//! curiosity corpus recorded it — reason code `computed-route` on two rows
//! whose whole subject is a curated reaction — and reading that as an
//! absence claim requires knowing that the classifier checks the curated
//! route first. A fact that can only be read off a file by someone who
//! already knows it is not written down.
//!
//! So it is written down here instead, as arithmetic rather than prose.
//! Every curated reactant is walked through the same renaming the readback
//! would do to it, and the reaction is at risk exactly when the key comes
//! back as something else. Adding a curated reaction on a species the
//! solver renames now fails this test instead of quietly never firing.

use kerotakis_core::curated;
use kerotakis_phreeqc::derived::{self, DerivedRole};

/// The registry keys a vessel holds after a solve, for a species entered
/// under `key` — the readback's own arithmetic, run forwards.
///
/// `None` means the solve does not touch it: no derived role at all
/// (honestly unmappable, and left alone), or a mineral, which books back
/// as the solid it already was.
fn keys_after_a_solve(key: &str) -> Option<Vec<String>> {
    match derived::role(key)? {
        DerivedRole::Solvent | DerivedRole::Mineral { .. } => None,
        DerivedRole::Dissolves(elements) => {
            let mut out = Vec::new();
            for (element, _) in elements {
                // A state with a protonation split comes back as every
                // registry species it divides between; anything else comes
                // back as its single booking ion.
                match derived::protonation_split(element) {
                    Some(split) => out.extend(split.iter().map(|(_, key)| (*key).to_string())),
                    None => out.extend(derived::booking_ion(element).map(str::to_string)),
                }
            }
            Some(out)
        }
    }
}

/// True when every reactant is still in the vessel under the key this
/// reaction names, after the vessel has been solved.
fn reachable_after_a_solve(reaction: &curated::CuratedReaction) -> bool {
    reaction.reactants.iter().all(|(reactant, _)| {
        keys_after_a_solve(reactant).is_none_or(|after| after.iter().any(|k| k == reactant))
    })
}

/// Reactions the aqueous readback renames out of reach, and why each one
/// is allowed to stay that way.
///
/// An unreachable reaction is not automatically a defect, and the two
/// permanganate rows show the established answer: a second entry written
/// in the names the beaker actually holds — `MnO₄⁻` where the shelf bottle
/// says `KMnO4`. The original entry is then dead code, but the capability
/// is intact and the experiment works.
///
/// This is a hand-kept list rather than an inferred one on purpose. Sibling
/// coverage cannot be detected by comparing products — the two permanganate
/// pairs differ by their spectator potassium (`KOH` against `OH⁻`, `K⁺`
/// against nothing) — and any rule loose enough to match those would also
/// match reactions that are not each other's substitutes. A wrong automatic
/// answer here reads as coverage that does not exist, so the reason is
/// written out instead, and the test below makes sure the list cannot go
/// stale.
const KNOWN_UNREACHABLE: &[(&str, &str)] = &[
    (
        "4 KMnO4 + 3 C₂H₅OH → 4 MnO₂↓ + 3 CH₃COOH + 4 KOH + H₂O",
        "covered: the `4 MnO₄⁻ + 3 C₂H₅OH` row fires in its place, on the ion \
         the vessel holds after the solve. This entry is dead code and the \
         experiment works.",
    ),
    (
        "2 KMnO₄ + 5 H₂C₂O₄ → 2 Mn²⁺ + 2 K⁺ + 10 CO₂↑ + 2 H₂O + 6 OH⁻",
        "covered: the `2 MnO₄⁻ + 5 H₂C₂O₄` row fires in its place. Verified by \
         running it — permanganate and oxalic acid decolourise and evolve \
         0.02 mol of CO₂.",
    ),
    (
        "NaOCl + 2 HCl → Cl2↑ + NaCl + H2O",
        "NOT covered, and not fixable the same way. HCl is a strong acid and \
         booking it as Cl⁻ is correct chemistry — there is no undissociated \
         HCl in water to find — so a sibling keyed on (NaOCl, Cl⁻) would fire \
         on table salt stirred into bleach and evolve chlorine from a beaker \
         that is doing nothing. That is a worse failure than this one. It \
         needs an acidity precondition on CuratedReaction, which does not \
         exist. Recorded rather than papered over: this is the demonstration \
         of why bleach and acid are never mixed, and it is currently silent.",
    ),
];

#[test]
fn every_curated_reaction_can_still_fire_in_water() {
    let mut lost: Vec<String> = Vec::new();
    for reaction in curated::REACTIONS {
        // A reaction pinned to a non-aqueous solvent never meets this
        // readback at all — there is no aqueous solve to rename anything.
        if reaction.solvent.is_some() || reachable_after_a_solve(reaction) {
            continue;
        }
        if KNOWN_UNREACHABLE
            .iter()
            .any(|(equation, _)| *equation == reaction.equation)
        {
            continue;
        }
        let renamed: Vec<String> = reaction
            .reactants
            .iter()
            .filter_map(|(reactant, _)| {
                let after = keys_after_a_solve(reactant)?;
                (!after.iter().any(|k| k == reactant))
                    .then(|| format!("`{reactant}` is booked back as {after:?}"))
            })
            .collect();
        lost.push(format!(
            "  {}\n    {}",
            reaction.equation,
            renamed.join("; ")
        ));
    }
    assert!(
        lost.is_empty(),
        "curated reactions the aqueous readback renames out of reach, with no \
         recorded reason. Each one is a reviewed reaction that will silently \
         never happen in water — no error, no refusal, just the reagents going \
         in and nothing coming out:\n{}\n\n\
         Either give the species a protonation split so the reactant survives \
         (see PROTONATION_SPLITS), add a sibling written in the names the \
         vessel holds after a solve (see the two MnO₄⁻ rows), or add it to \
         KNOWN_UNREACHABLE with the reason it can be neither.",
        lost.join("\n")
    );
}

/// The recorded list stays honest in the other direction too: an entry that
/// has since been fixed must come off it, or the list becomes a place where
/// solved problems go to look unsolved.
#[test]
fn nothing_on_the_unreachable_list_has_quietly_been_fixed() {
    for (equation, _) in KNOWN_UNREACHABLE {
        let reaction = curated::REACTIONS
            .iter()
            .find(|r| r.equation == *equation)
            .unwrap_or_else(|| panic!("{equation}: no such curated reaction any more"));
        assert!(
            !reachable_after_a_solve(reaction),
            "{equation} can fire now — take it off KNOWN_UNREACHABLE"
        );
    }
}

/// The other half of the same claim: a protonation split is only useful if
/// the neutral member is a registry key something can actually look up.
///
/// The database spelling and the registry spelling are not the same word —
/// minteq.v4 writes undissociated acetic acid `H(Acetate)` and this lab
/// writes `CH3COOH` — so a row that mapped to a key the registry does not
/// carry would book matter into a name nothing can resolve, and do it
/// silently.
#[test]
fn every_protonation_split_names_registry_species() {
    for (element, split) in derived::PROTONATION_SPLITS {
        for (database_species, registry_key) in *split {
            assert!(
                kerotakis_core::species::lookup_key(registry_key).is_some(),
                "{element}: the split books {database_species} as `{registry_key}`, \
                 which is not a registry key"
            );
        }
    }
}

/// The two tables that decide whether an acid survives a solve must agree.
///
/// `PROTONATION_SPLITS` (here) decides which species the ledger CARRIES;
/// `displacement::LEDGER_ACIDS` decides whose proton the ledger COUNTS. If
/// a split adds a neutral acid and nothing tells `oxidant_available` about
/// it, that acid's protons vanish from the metal's point of view — a beaker
/// of vinegar reads its dissociated 1e-3 and magnesium stops dissolving in
/// it. That is precisely the failure this pair of changes exists to avoid,
/// so the two lists are checked against each other rather than trusted to
/// stay in step.
///
/// The check is one-directional on purpose. Every ledger acid must come
/// from a split, because an acid the readback still strips can never appear
/// in the inventory to be counted. The reverse does NOT hold: `N(-3)` is
/// split and `NH4+` is deliberately not a ledger acid, for the reason given
/// where that list is defined.
#[test]
fn every_counted_acid_is_one_the_ledger_can_actually_hold() {
    let carried: Vec<&str> = derived::PROTONATION_SPLITS
        .iter()
        .flat_map(|(_, split)| split.iter().map(|(_, key)| *key))
        .collect();
    for (acid, base) in kerotakis_core::displacement::ledger_acids() {
        assert!(
            carried.contains(acid),
            "`{acid}` is counted as a titratable acid but no protonation \
             split puts it in the ledger, so the readback will book its \
             element total to one ion and the species will never be there \
             to count"
        );
        assert!(
            carried.contains(base),
            "`{acid}` books its conjugate base as `{base}`, which no \
             protonation split carries — spending the proton would deposit \
             a species the solve then has no name for"
        );
    }
}
