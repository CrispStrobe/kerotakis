//! The aqueous heat balance, as one state-function inventory.
//!
//! Three separate heats used to be charged here, each from its own proxy:
//! neutralisation (from how much the solutes' net charge cancelled),
//! dissolution (from a registry enthalpy, per `Dissolved` event), and the
//! carbonate route (nothing at all, because the neutralisation enthalpy was
//! the wrong number for it and there was no other). They disagreed with each
//! other and with the beaker: baking soda and vinegar came out 0.17 K WARMER
//! in one pouring order and unmoved in the other, for a reaction a child can
//! feel go cold.
//!
//! The cause was never the enthalpies. It was that charge cancellation is
//! not the extent of the acid–carbonate reaction — a proton moves from
//! acetic acid to bicarbonate and the counter-ions stay put, so almost no
//! charge cancels while the whole reaction runs.
//!
//! Enthalpy is a state function, so the fix is not a better proxy but the
//! absence of one. Give every species an enthalpy relative to a single
//! basis, add up what the vessel holds before and after, and the difference
//! is the heat. Order independence is then not a property to test for and
//! hope: it is what a state function *is*.
//!
//! # The basis
//!
//! PHREEQC's master species, at the database's own standard state, are
//! zero. Everything else is measured from there:
//!
//! * an **aqueous species** carries the `SOLUTION_SPECIES` reaction
//!   enthalpy that defines it (`H+ + CO3-2 = HCO3-` → −14.6 kJ/mol);
//! * a **gas** carries the negative of its `PHASES` dissolution enthalpy,
//!   because that reaction is written dissolving *into* the basis;
//! * a **solid** carries the sum of its dissolution products' enthalpies
//!   less the registry's curated enthalpy of dissolution — the products
//!   matter, and getting that wrong is the subtlety this module exists to
//!   not repeat (see [`DISSOCIATION`]);
//! * **free acid and base** are carried by charge, not by portions: the
//!   readback never writes `H⁺` or `OH⁻` back (it would double-count
//!   water's own hydrogen and oxygen), so a solute charge of +q means q
//!   moles of unwritten hydroxide, and −q means that much free acid. `H⁺`
//!   is a master species and contributes nothing; `OH⁻` carries the
//!   database's own +55.81 kJ/mol.
//!
//! # What falls out
//!
//! `H⁺ + OH⁻ → H₂O` is not a special case in here. Neutralising a strong
//! acid with a strong base moves one mole of hydroxide (h = +55.81) to
//! water (h = 0), and the sum returns −55.81 kJ/mol without being asked —
//! the same figure `neutralisation_enthalpy` used to read off the engine.
//! Dissolving the alkali first is the other term of the same sum, so
//! `NaOH(s) + HCl → NaCl + H₂O` comes back as −100.31 kJ/mol, being −44.5
//! of dissolution and −55.81 of neutralisation, without either being
//! computed separately.
//!
//! The volcano comes back at +26.8 kJ/mol endothermic against a literature
//! +25 to +30, and by construction it returns the same number whichever way
//! round the beaker was filled.
//!
//! # Where it declines
//!
//! A species whose amount CHANGED and whose enthalpy this lab does not hold
//! stops the whole balance for that step, by name. A species that merely
//! sits there unpriced does not: it appears identically on both sides and
//! cancels, so an exotic spectator cannot silence the heat of a reaction it
//! took no part in.
//!
//! Declining is the deliberate behaviour and not a gap to be filled with
//! the nearest available number. Charging a plausible enthalpy for the
//! wrong substance is exactly how this went wrong before.

use kerotakis_core::{species, Phase, Portion, SpeciesId};
use std::collections::BTreeMap;

use crate::derived;

/// What each solid with a curated enthalpy of dissolution actually
/// dissolves INTO, in the database's own spelling.
///
/// This table is stoichiometry only — every enthalpy in the calculation
/// still comes from the database or the registry. It exists because
/// `h(solid) = Σ h(products) − ΔH_dissolution` needs the products, and
/// getting that wrong is quiet and plausible: taking `h(NaHCO₃) = −16.7`
/// (the enthalpy alone, as though the products were all masters) rather
/// than `(0 + −14.6) − 16.7 = −31.3` puts the volcano at +12.2 kJ/mol
/// instead of +26.8. Both look like reasonable answers. Only one of them
/// is within a mile of what the world measures.
///
/// A solid that is not in here is not priced, and its step declines by
/// name. Deriving these by splitting formulae was considered and dropped:
/// it would have to guess for exactly the awkward ones, and a wrong guess
/// here is indistinguishable from a right one until someone puts a
/// thermometer in the beaker.
pub const DISSOCIATION: &[(&str, &[(&str, f64)])] = &[
    ("NaCl", &[("Na+", 1.0), ("Cl-", 1.0)]),
    ("KCl", &[("K+", 1.0), ("Cl-", 1.0)]),
    ("CaCl2", &[("Ca+2", 1.0), ("Cl-", 2.0)]),
    ("NaOH", &[("Na+", 1.0), ("OH-", 1.0)]),
    ("KOH", &[("K+", 1.0), ("OH-", 1.0)]),
    ("Ca(OH)2", &[("Ca+2", 1.0), ("OH-", 2.0)]),
    // Also flagged: +16.7 matches neither the formation-enthalpy
    // difference (~+18.7) nor a tabulated ~+17.5. The volcano test below
    // asserts the LITERATURE BAND rather than a point precisely so a
    // correction here moves the answer without breaking the test — at
    // +18.7 the reaction comes out near +28.8, still inside 25–30.
    ("NaHCO3", &[("Na+", 1.0), ("HCO3-", 1.0)]),
    ("NaOAc", &[("Na+", 1.0), ("Acetate-", 1.0)]),
    ("Na2SO4", &[("Na+", 2.0), ("SO4-2", 1.0)]),
    ("CuSO4", &[("Cu+2", 1.0), ("SO4-2", 1.0)]),
    ("ZnSO4", &[("Zn+2", 1.0), ("SO4-2", 1.0)]),
    ("AgNO3", &[("Ag+", 1.0), ("NO3-", 1.0)]),
    ("AgCl", &[("Ag+", 1.0), ("Cl-", 1.0)]),
    ("Pb(NO3)2", &[("Pb+2", 1.0), ("NO3-", 2.0)]),
    // SUSPECT VALUE, kept rather than dropped. kerotakis-59's registry
    // agent could not reproduce the registry's +16.2 kJ/mol against a
    // commonly tabulated ~+43.6, and the gap is not a sign, a unit or a
    // hydrate. It is left in because the `Dissolved` loop this replaces
    // charges the same +16.2 today, so keeping it is the status quo and
    // dropping it would silently remove a heat permanganate vessels
    // already get. When the registry row is corrected this follows it
    // without a change here — the number is read, never copied.
    ("KMnO4", &[("K+", 1.0), ("MnO4-", 1.0)]),
    ("NH4Cl", &[("NH4+", 1.0), ("Cl-", 1.0)]),
    ("NH4NO3", &[("NH4+", 1.0), ("NO3-", 1.0)]),
    // Below here the registry names a substance that is ALREADY in
    // solution and that the database spells only as its ions. These carry
    // no enthalpy of dissolution — there is no dissolution left to do —
    // so they are priced as the sum of what they are. `NaOAc` appears in
    // both halves on purpose: as a solid it is these ions less its
    // dissolution enthalpy, and as the aqueous portion a curated reaction
    // deposits it is just the ions.
    ("HCl", &[("H+", 1.0), ("Cl-", 1.0)]),
    ("HNO3", &[("H+", 1.0), ("NO3-", 1.0)]),
    ("H2SO4", &[("H+", 2.0), ("SO4-2", 1.0)]),
];

/// A species the balance cannot price, named so the refusal can say which.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unpriced {
    pub species: String,
    pub why: &'static str,
}

/// The enthalpy of one aqueous species relative to the master basis.
fn aqueous_enthalpy(key: &str, db_tag: &str) -> Option<f64> {
    let idx = derived::index_for(db_tag);
    if key == "water" || key == "H2O" {
        return Some(0.0);
    }
    if let Some(dh) = idx.species_delta_h_kj.get(key) {
        return Some(*dh);
    }
    // The registry and the database do not always spell the same substance
    // the same way — `CH3COOH` is `H(Acetate)` in minteq.v4.
    for (_, pairs) in derived::PROTONATION_SPLITS {
        for (db_name, registry_name) in *pairs {
            if *registry_name == key {
                if let Some(dh) = idx.species_delta_h_kj.get(*db_name) {
                    return Some(*dh);
                }
                if idx.species_element.contains_key(*db_name) {
                    return Some(0.0);
                }
            }
        }
    }
    // A master species is the basis.
    idx.species_element.contains_key(key).then_some(0.0)
}

/// The enthalpy of one portion's species, kJ/mol, relative to the basis.
pub fn species_enthalpy(key: &str, phase: Phase, db_tag: &str) -> Result<f64, Unpriced> {
    let idx = derived::index_for(db_tag);
    match phase {
        // A gas is priced by where it goes, less what the trip costs —
        // the same rule as a solid, and for the same reason.
        //
        // `delta_h` is the enthalpy of the dissolution reaction AS THE
        // DATABASE WRITES IT, and the databases do not write the same
        // reaction. minteq.v4 dissolves CO2(g) to `2 H+ + CO3-2`, which are
        // master species and cost nothing, so the gas is just the reaction
        // reversed. wateq4f dissolves it to an aqueous `CO2` that carries
        // about -24 kJ/mol of its own, and ignoring that put the same gas
        // leaving the same beaker at +19.98 kJ/mol on one route and -4.06
        // on the other. The volcano then cooled by 1.70 K poured one way
        // and 1.28 K poured the other, and the 1.28 was the correct one.
        Phase::Gas => {
            let phase = idx.phases.get(&format!("{key}(g)")).ok_or(Unpriced {
                species: key.to_string(),
                why: "the routed database defines no gas phase of this name",
            })?;
            let dissolution = phase.delta_h_kj.ok_or(Unpriced {
                species: key.to_string(),
                why: "the routed database states no enthalpy for this gas dissolving",
            })?;
            let mut sum = 0.0;
            for (product, n) in &phase.products {
                let h = aqueous_enthalpy(product, db_tag).ok_or(Unpriced {
                    species: product.clone(),
                    why: "a dissolution product the routed database does not define",
                })?;
                sum += h * n;
            }
            Ok(sum - dissolution)
        }
        // A solid is where its ions go, LESS what the trip costs.
        //
        // The enthalpy of dissolution belongs in this sum and not beside
        // it. It used to be charged separately, per `Dissolved` event, and
        // that survived as the last second path in the heat balance — with
        // exactly the failure a second path always has. Pouring baking soda
        // into water dissolves it through the aqueous tail, the event
        // fires, +16.7 kJ/mol is charged. Pouring it into vinegar lets the
        // curated row consume the solid outright: no dissolution event, no
        // dissolution heat, and the same reaction came out cooling by 0.49 K
        // one way round and 1.70 K the other.
        //
        // Here it is one term of one sum, so both routes begin from the
        // same solid in the same state and reach +26.8 kJ/mol either way.
        //
        // The subtlety, and it cost a factor of two before it was caught:
        // the products are NOT all master species. `NaHCO3 -> Na+ + HCO3-`
        // and the bicarbonate carries -14.6 of its own, so the solid is
        // `(0 + -14.6) - 16.7 = -31.3`, not `-16.7`. Both look reasonable.
        // Only one is within a mile of what the world measures.
        Phase::Solid => {
            let products = DISSOCIATION
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, p)| *p)
                .ok_or(Unpriced {
                    species: key.to_string(),
                    why: "no curated dissociation for this solid, so where it goes is unknown",
                })?;
            let dissolution = species::lookup(&SpeciesId::new(key))
                .and_then(|d| d.dissolution_enthalpy_kj)
                .ok_or(Unpriced {
                    species: key.to_string(),
                    why: "the registry holds no enthalpy of dissolution for this solid",
                })?;
            let mut sum = 0.0;
            for (product, n) in products {
                let h = aqueous_enthalpy(product, db_tag).ok_or(Unpriced {
                    species: (*product).to_string(),
                    why: "a dissociation product the routed database does not define",
                })?;
                sum += h * n;
            }
            Ok(sum - dissolution)
        }
        // Water is the solvent and the basis; any other miscible liquid is
        // priced as what it becomes in solution, which leaves its enthalpy
        // of MIXING to `hmix`, where it already lives, rather than counting
        // it twice.
        Phase::Liquid | Phase::Aqueous => {
            if let Some(h) = aqueous_enthalpy(key, db_tag) {
                return Ok(h);
            }
            // Already dissolved, and the database spells it only as its
            // ions: it is the sum of them, with no dissolution term
            // because there is no dissolution left to do.
            let products = DISSOCIATION
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, p)| *p)
                .ok_or(Unpriced {
                    species: key.to_string(),
                    why: "the routed database does not define this species in solution",
                })?;
            let mut sum = 0.0;
            for (product, n) in products {
                let h = aqueous_enthalpy(product, db_tag).ok_or(Unpriced {
                    species: (*product).to_string(),
                    why: "a dissociation product the routed database does not define",
                })?;
                sum += h * n;
            }
            Ok(sum)
        }
    }
}

/// `Phase` is not `Ord`, and this map wants a stable key rather than a
/// derive on a core type that nothing else asks for.
fn phase_key(phase: Phase) -> u8 {
    match phase {
        Phase::Solid => 0,
        Phase::Liquid => 1,
        Phase::Aqueous => 2,
        Phase::Gas => 3,
    }
}

fn phase_of(key: u8) -> Phase {
    match key {
        0 => Phase::Solid,
        1 => Phase::Liquid,
        2 => Phase::Aqueous,
        _ => Phase::Gas,
    }
}

/// Everything the vessel holds, as moles keyed by (species, phase).
fn tally(contents: &[Portion]) -> BTreeMap<(String, u8), f64> {
    let mut t = BTreeMap::new();
    for p in contents {
        *t.entry((p.species.0.clone(), phase_key(p.phase)))
            .or_insert(0.0) += p.moles.0;
    }
    t
}

/// Below this, a change is not a reaction — it is the solver's last digit.
const NEGLIGIBLE: f64 = 1e-12;

/// The heat released, in JOULES, by everything that changed between two
/// states of one vessel. Positive warms the beaker.
///
/// `gas_out` is what left the liquid during this step and is therefore no
/// longer in `after`; it is priced as gas and counted on the after side.
/// `oh_before`/`oh_after` are MOLES of free hydroxide, read from PHREEQC's
/// own species distribution. They are not inferred from the solutes' net
/// charge, and that distinction is load-bearing: a positive solute charge
/// means free base only when the vessel's inputs were charge-balanced, and
/// a beaker handed a bare cation is not. Reading 6.4 mol of `Ag+` as 6.4
/// mol of hydroxide charged 355 kJ of imaginary neutralisation and drove a
/// vessel to MINUS 198 K. Charge correlates with free base; it is not free
/// base, and the engine already reports the real thing.
///
/// Only species whose amount actually MOVED are priced, so an unpriceable
/// spectator costs nothing. If something that moved cannot be priced, the
/// whole step declines and says which species stopped it.
pub fn heat_released_j(
    before: &[Portion],
    oh_before: f64,
    after: &[Portion],
    oh_after: f64,
    gas_out: &[(String, f64)],
    db_tag: &str,
) -> Result<f64, Unpriced> {
    let mut delta = tally(after);
    for (k, n) in tally(before) {
        *delta.entry(k).or_insert(0.0) -= n;
    }
    for (species, moles) in gas_out {
        *delta
            .entry((species.clone(), phase_key(Phase::Gas)))
            .or_insert(0.0) += moles;
    }
    // The unwritten hydroxide. Free acid is H+, a master species, and
    // contributes nothing, so only the base side appears here.
    let d_oh = oh_after - oh_before;
    // Hydroxide has to come from somewhere nameable.
    //
    // A beaker handed bare `Na+` — 3.9 mol of cation with no anion — is
    // charge-balanced by the engine with 3.9 mol of hydroxide, and that
    // hydroxide is real as far as the speciation is concerned. But nothing
    // in the ledger supplied it: the sodium is priced as the master
    // species it is, and the hydroxide then appears out of the basis
    // carrying +55.81 kJ/mol of enthalpy each. 3.9 mol of that is 218 kJ
    // of invented cooling, and it drove a vessel to MINUS 27 K.
    //
    // A real alkali does not do this, because it arrives as a portion that
    // is priced with its hydroxide in it — `NaOH(s)` is Na+ plus OH-, so
    // the hydroxide appearing on one side is paid for by the solid
    // disappearing on the other, and the two cancel. So the rule is simply
    // that: if free hydroxide GREW, something that contains hydroxide must
    // have been consumed to make it. Nothing did, and the vessel was never
    // charge-coherent to begin with, so no heat may be drawn from it.
    //
    // Hydroxide SHRINKING needs no such licence — that is neutralisation,
    // and what consumed it is the acid, which is priced already.
    if d_oh.abs() > NEGLIGIBLE {
        let h = aqueous_enthalpy("OH-", db_tag).ok_or(Unpriced {
            species: "OH-".to_string(),
            why: "the routed database does not define hydroxide",
        })?;
        *delta
            .entry(("__free_hydroxide".to_string(), phase_key(Phase::Aqueous)))
            .or_insert(0.0) += d_oh;
        // Priced directly rather than through species_enthalpy, since the
        // key is ours and not a registry name.
        let _ = h;
    }

    // How much hydroxide has to appear before something must account for
    // it. The two cases this separates are six orders of magnitude apart,
    // so the line is not delicate:
    //
    // * a bicarbonate solution sits at pH 8.3 and makes about 9 µmol of
    //   hydroxide by hydrolysing — genuinely supplied, by a weak base this
    //   table does not list as a hydroxide source, and worth 0.5 J;
    // * a beaker handed 3.9 mol of bare `Na+` is charge-balanced by the
    //   engine with 3.9 MOL of hydroxide, worth 218 kJ, and reached minus
    //   27 K before this check existed.
    //
    // A millimole is comfortably above every hydrolysis this bench sees
    // and a factor of ~4000 below the pathology. What slips under it is
    // worth at most 56 J, which is 0.13 K in a 100 g beaker and is charged
    // rather than refused — declining a real reaction's heat to avoid a
    // rounding error would be the worse trade.
    const HYDROXIDE_FROM_A_REAGENT: f64 = 1e-3;
    if d_oh > HYDROXIDE_FROM_A_REAGENT {
        let supplied = delta.iter().any(|((key, _), moved)| {
            *moved < -NEGLIGIBLE
                && DISSOCIATION
                    .iter()
                    .any(|(k, products)| k == key && products.iter().any(|(p, _)| *p == "OH-"))
        });
        if !supplied {
            return Err(Unpriced {
                species: "OH-".to_string(),
                why: "free hydroxide appeared with nothing in the vessel to supply it — \
                      the charge was balanced by the engine, not by a reagent, so this \
                      beaker was never charge-coherent",
            });
        }
    }

    let mut d_h_kj = 0.0;
    for ((key, phase), moved) in &delta {
        if moved.abs() <= NEGLIGIBLE {
            continue;
        }
        let h = if key == "__free_hydroxide" {
            aqueous_enthalpy("OH-", db_tag).ok_or(Unpriced {
                species: "OH-".to_string(),
                why: "the routed database does not define hydroxide",
            })?
        } else {
            species_enthalpy(key, phase_of(*phase), db_tag)?
        };
        d_h_kj += h * moved;
        if std::env::var("KERO_BAL").is_ok() {
            eprintln!(
                "    [d] {key:<12} ph={phase} moved={moved:+.9} h={h:+.3} -> {:+.4} kJ",
                h * moved
            );
        }
    }
    // Heat released is the negative of the enthalpy the contents gained.
    Ok(-d_h_kj * 1000.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kerotakis_core::{Moles, SpeciesId};

    fn portion(key: &str, moles: f64, phase: Phase) -> Portion {
        Portion {
            species: SpeciesId::new(key),
            moles: Moles(moles),
            phase,
        }
    }

    const DB: &str = "minteq.v4";

    /// A solid is where its ions go, less what the trip costs. Both terms
    /// belong in this one number: charging the dissolution separately, per
    /// event, was the last second path in the balance and it made the
    /// answer depend on which solver happened to consume the solid.
    #[test]
    fn a_solids_enthalpy_is_its_ions_less_the_cost_of_getting_there() {
        // NaHCO3 -> Na+ (0) + HCO3- (-14.6), less +16.7 of dissolution.
        // Taking the dissolution alone gives -16.7 and puts the volcano at
        // +12.2 kJ/mol instead of +26.8. Both look reasonable; one is not.
        let h = species_enthalpy("NaHCO3", Phase::Solid, DB).expect("priced");
        assert!((h - (-31.3)).abs() < 1e-9, "{h}");
        // NaOH -> Na+ (0) + OH- (+55.81), less -44.5 of dissolution. Both
        // halves of `NaOH + HCl` are in this one number.
        let h = species_enthalpy("NaOH", Phase::Solid, DB).expect("priced");
        assert!((h - 100.31).abs() < 1e-9, "{h}");
        // NaCl -> two master species, so it is only its dissolution.
        let h = species_enthalpy("NaCl", Phase::Solid, DB).expect("priced");
        assert!((h - (-3.88)).abs() < 1e-9, "{h}");
    }

    /// The strong-acid/strong-base heat is not a special case in here; it
    /// is one hydroxide going to water, and the sum says so.
    #[test]
    fn neutralisation_falls_out_of_the_basis() {
        // A beaker holding 0.1 mol of free base, neutralised to nothing.
        let before = [portion("Na+", 0.1, Phase::Aqueous)];
        let after = [
            portion("Na+", 0.1, Phase::Aqueous),
            portion("Cl-", 0.1, Phase::Aqueous),
        ];
        let q = heat_released_j(&before, 0.1, &after, 0.0, &[], DB).expect("priced");
        // 0.1 mol x -55.81 kJ/mol of enthalpy lost = that much heat out.
        assert!(
            (q - 5581.0).abs() < 1e-6,
            "expected the database's own 55.81 kJ/mol, got {} J",
            q
        );
    }

    /// The reaction this module was written for, and the number the world
    /// already knows: +25 to +30 kJ/mol endothermic.
    #[test]
    fn the_volcano_is_endothermic_and_the_right_size() {
        let before = [
            portion("NaHCO3", 0.02, Phase::Solid),
            portion("CH3COOH", 0.02, Phase::Liquid),
        ];
        let after = [
            portion("Na+", 0.02, Phase::Aqueous),
            portion("CH3COO-", 0.02, Phase::Aqueous),
        ];
        let q = heat_released_j(&before, 0.0, &after, 0.0, &[("CO2".into(), 0.02)], DB)
            .expect("priced");
        let kj_per_mol = -q / 1000.0 / 0.02;
        // The whole reaction now, dissolution included, against the
        // literature's +25 to +30 for baking soda and vinegar. Pinned to
        // the BAND rather than to our own figure, so a correction to the
        // registry's enthalpy of dissolution (its +16.7 is flagged as not
        // reproducible) moves the answer without breaking the test.
        assert!(
            (25.0..=30.0).contains(&kj_per_mol),
            "expected +25..30 kJ/mol endothermic, got {kj_per_mol}"
        );
    }

    /// A spectator nobody can price must not silence a reaction it took no
    /// part in — it is on both sides and cancels.
    #[test]
    fn an_unpriceable_spectator_that_does_not_move_is_harmless() {
        let before = [
            portion("Na+", 0.1, Phase::Aqueous),
            portion("glitter", 1.0, Phase::Solid),
        ];
        let after = [
            portion("Na+", 0.1, Phase::Aqueous),
            portion("Cl-", 0.1, Phase::Aqueous),
            portion("glitter", 1.0, Phase::Solid),
        ];
        assert!(heat_released_j(&before, 0.1, &after, 0.0, &[], DB).is_ok());
    }

    /// But one that MOVES stops the balance, by name, rather than being
    /// charged the nearest number that happens to be to hand.
    #[test]
    fn something_unpriceable_that_moves_declines_by_name() {
        let before = [portion("glitter", 1.0, Phase::Solid)];
        let after = [portion("glitter", 0.5, Phase::Solid)];
        let err = heat_released_j(&before, 0.0, &after, 0.0, &[], DB).expect_err("declines");
        assert_eq!(err.species, "glitter");
    }

    /// The basis must not be able to collapse to all-zeros.
    ///
    /// Every fallback in `aqueous_enthalpy` ends at "it is a master
    /// species, so it is zero" — which is correct, and is also what a
    /// BROKEN PARSE looks like. If `species_delta_h_kj` came back empty,
    /// HCO3- would not be found as a species, would be found as
    /// Alkalinity's master, and would price at 0.0 instead of -14.6. Every
    /// test above would still pass except the two that pin numbers, and
    /// every heat in the bench would quietly go to zero without a single
    /// refusal, because zero is a number and the balance would sum it
    /// happily.
    ///
    /// So assert the corpus is really there before trusting anything drawn
    /// from it: a population of enthalpies, and non-zero ones at that.
    #[test]
    fn the_basis_is_populated_and_not_silently_zero() {
        let idx = derived::index_for(DB);
        assert!(
            idx.species_delta_h_kj.len() > 100,
            "only {} species enthalpies parsed — suspect the parser, not the file",
            idx.species_delta_h_kj.len()
        );
        let nonzero = idx
            .species_delta_h_kj
            .values()
            .filter(|v| **v != 0.0)
            .count();
        assert!(nonzero > 100, "only {nonzero} of them are non-zero");
        let phases = idx
            .phases
            .values()
            .filter(|p| p.delta_h_kj.is_some_and(|d| d != 0.0))
            .count();
        assert!(phases > 50, "only {phases} phase enthalpies");
        // And the three the carbonate and neutralisation answers actually
        // rest on, by name and value — a populated map is not the same as
        // a map containing what this module reads out of it.
        assert!((aqueous_enthalpy("HCO3-", DB).unwrap() + 14.6).abs() < 1e-9);
        assert!((aqueous_enthalpy("OH-", DB).unwrap() - 55.81).abs() < 1e-9);
        assert!(
            (species_enthalpy("CO2", Phase::Gas, DB).unwrap() + 4.06).abs() < 1e-9,
            "the CO2(g) phase enthalpy"
        );
    }

    /// Every row prices, and a solid prices as its ions LESS its enthalpy
    /// of dissolution — both terms, once each. If the dissolution ever
    /// leaves this number it becomes an event-driven second path, and the
    /// same reaction then costs different amounts depending on which
    /// solver consumed the solid.
    ///
    /// Compared against the products directly rather than against the
    /// same key priced as aqueous, because those are not always the same
    /// substance: minteq.v4 defines an aqueous ION PAIR called `NaHCO3`
    /// (`Na+ + H+ + CO3-2 = NaHCO3`, -28.33 kJ/mol) which is a different
    /// thing from dissolved sodium bicarbonate, and comparing the two
    /// would be comparing a salt with a complex that shares its name.
    #[test]
    fn a_solid_prices_as_its_ions_less_its_dissolution() {
        for (key, products) in DISSOCIATION {
            let expected: f64 = products
                .iter()
                .map(|(product, n)| {
                    aqueous_enthalpy(product, DB)
                        .unwrap_or_else(|| panic!("{key}: no h for product {product}"))
                        * n
                })
                .sum();
            // The rows below the solids arrive ALREADY dissolved and have
            // no dissolution to charge — they only have to price in
            // solution, as the sum of what they are.
            let Some(dissolution) =
                species::lookup(&SpeciesId::new(key)).and_then(|d| d.dissolution_enthalpy_kj)
            else {
                let aqueous = species_enthalpy(key, Phase::Aqueous, DB).unwrap_or_else(|e| {
                    panic!(
                        "{key} arrives dissolved and does not price: {} ({})",
                        e.species, e.why
                    )
                });
                assert!(
                    (aqueous - expected).abs() < 1e-9,
                    "{key} in solution: priced {aqueous}, its ions come to {expected}"
                );
                continue;
            };
            let solid = species_enthalpy(key, Phase::Solid, DB)
                .unwrap_or_else(|e| panic!("{key} as a solid: {} ({})", e.species, e.why));
            assert!(
                (solid - (expected - dissolution)).abs() < 1e-9,
                "{key}: priced {solid}, expected its ions ({expected}) less \
                 its dissolution ({dissolution})"
            );
            // And it must price in solution too, by whichever route.
            species_enthalpy(key, Phase::Aqueous, DB)
                .unwrap_or_else(|e| panic!("{key} in solution: {} ({})", e.species, e.why));
        }
    }
}
