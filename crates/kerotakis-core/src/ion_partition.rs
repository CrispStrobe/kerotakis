//! Charged solutes do not partition into hydrocarbon solvents (lane
//! 2026-11-11, SE-002).
//!
//! The defect: extracting a salt solution with hexane refused "no reviewed
//! hexane/water distribution coefficient for Cl- and Na+", so "the salt
//! stays in the water" was only ever a side effect of a missing number.
//! It is not a missing number. A bare ion carries a hydration Gibbs energy
//! of hundreds of kJ/mol (Marcus, Y. "Thermodynamics of solvation of ions.
//! Part 5. Gibbs free energy of hydration at 298.15 K", J. Chem. Soc.,
//! Faraday Trans. 1991, 87, 2995-2999, doi:10.1039/FT9918702995) that a
//! low-permittivity alkane or arene cannot repay, and an ion is moreover
//! never transferred alone: electroneutrality of each phase requires a
//! counter-ion to move with it. The standard textbook statement is that
//! inorganic ions are not extracted into an inert hydrocarbon diluent
//! unless they are first converted into a neutral, lipophilic species — an
//! ion pair with a large organic counter-ion, or an uncharged complex with
//! an extractant (Rydberg, J.; Cox, M.; Musikas, C.; Choppin, G. R., eds.
//! *Solvent Extraction Principles and Practice*, 2nd ed., Marcel Dekker,
//! New York, 2004, ISBN 978-0-8247-5063-3, the chapters on the principles
//! of extraction of ions and of ion-pair extraction).
//!
//! The rule here is exactly that statement and no stronger:
//! - the solvent is a hydrocarbon (only C and H in its registry formula);
//! - the solute carries a net charge in its registry formula;
//! - the ion is not lipophilic (fewer than [`LIPOPHILIC_ION_CARBONS`]
//!   carbons per unit charge), and
//!   the source vessel holds no lipophilic ion that could act as an
//!   ion-pairing agent.
//!
//! Where it holds, the ion is reported in the extraction event as RETAINED
//! in the aqueous phase under this rule — not as a coefficient of zero,
//! because no coefficient was measured, and not as a refusal, because the
//! answer is known. Where it does not hold (a polar solvent such as ethyl
//! acetate or 1-octanol, a surfactant or dye ion), the previous typed
//! refusal stands.

use crate::species::{self, SpeciesId};

/// An ion with at least this many carbons PER UNIT CHARGE (a long-chain
/// carboxylate, a sulfonate surfactant, a quaternary ammonium, a dye ion)
/// is treated as lipophilic enough to ion-pair; the rule declines for it and
/// for every ion beside it. Per charge, because a multiply charged small
/// anion such as citrate (C6, z = -3) is strongly hydrated, not lipophilic.
/// This is a conservative screening threshold of this lane, not a measured
/// constant: it only decides where the rule is NOT claimed.
pub const LIPOPHILIC_ION_CARBONS: f64 = 6.0;

/// The stated rule, as reported in the extraction event.
pub const RULE: &str = "charged species are not extracted into a hydrocarbon solvent without an ion-pairing agent: the ion's hydration Gibbs energy is not repaid by an alkane or arene and each phase must stay electroneutral, so the ion stays in the aqueous phase";

/// Where the rule comes from, as reported in the extraction event.
pub const PROVENANCE: &str = "Rydberg, Cox, Musikas & Choppin (eds.), Solvent Extraction Principles and Practice, 2nd ed., Marcel Dekker 2004, ISBN 978-0-8247-5063-3; Marcus, J. Chem. Soc. Faraday Trans. 87, 2995-2999 (1991), doi:10.1039/FT9918702995";

/// The ion classes the rule covers, for the coverage table.
pub const ION_CLASSES: &[&str] = &[
    "monatomic cation",
    "monatomic anion",
    "polyatomic inorganic ion",
    "small organic ion",
];

fn formula_of(species: &SpeciesId) -> Option<crate::stoich::Formula> {
    let text = species::lookup(species)
        .map(|data| data.formula)
        .unwrap_or(species.0.as_str());
    crate::stoich::parse_formula(text).ok()
}

/// Net charge of a solute from its registry formula (0 where unknown).
pub fn charge(species: &SpeciesId) -> f64 {
    formula_of(species).map(|f| f.charge).unwrap_or(0.0)
}

/// Whether `solvent` is a hydrocarbon: its registry formula holds only
/// carbon and hydrogen.
pub fn is_hydrocarbon_solvent(solvent: &SpeciesId) -> bool {
    let Some(data) = species::lookup(solvent) else {
        return false;
    };
    let Ok(parsed) = crate::stoich::parse_formula(data.formula) else {
        return false;
    };
    parsed.charge == 0.0
        && parsed.counts.contains_key("C")
        && parsed.counts.contains_key("H")
        && parsed.counts.len() == 2
}

/// Whether an ion is lipophilic enough to act as an ion-pairing partner.
pub fn is_lipophilic_ion(species: &SpeciesId) -> bool {
    formula_of(species).is_some_and(|f| {
        f.charge != 0.0
            && f.counts.get("C").copied().unwrap_or(0.0) / f.charge.abs() >= LIPOPHILIC_ION_CARBONS
    })
}

/// The class of a charged solute, for reporting and coverage.
pub fn ion_class(species: &SpeciesId) -> Option<&'static str> {
    let f = formula_of(species)?;
    if f.charge == 0.0 {
        return None;
    }
    // Organic: two or more carbons, or one carbon carrying at least two
    // hydrogens. Carbonate, bicarbonate, cyanide and thiocyanate stay
    // inorganic; formate (HCOO-) is reported with them, a naming
    // simplification only — the rule treats both classes the same.
    let carbons = f.counts.get("C").copied().unwrap_or(0.0);
    let hydrogens = f.counts.get("H").copied().unwrap_or(0.0);
    if carbons >= 2.0 || (carbons >= 1.0 && hydrogens >= 2.0) {
        return Some(ION_CLASSES[3]);
    }
    let atoms: f64 = f.counts.values().sum();
    if f.counts.len() == 1 && atoms <= 1.0 {
        Some(if f.charge > 0.0 {
            ION_CLASSES[0]
        } else {
            ION_CLASSES[1]
        })
    } else {
        Some(ION_CLASSES[2])
    }
}

/// Whether the rule keeps `solute` in the water when `solvent` extracts a
/// source whose dissolved solutes are `present`.
pub fn retained_in_aqueous<'a>(
    solute: &SpeciesId,
    solvent: &SpeciesId,
    mut present: impl Iterator<Item = &'a SpeciesId>,
) -> bool {
    charge(solute) != 0.0
        && is_hydrocarbon_solvent(solvent)
        && !is_lipophilic_ion(solute)
        && !present.any(is_lipophilic_ion)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(key: &str) -> SpeciesId {
        SpeciesId::new(key)
    }

    #[test]
    fn small_ions_stay_out_of_hexane_but_not_out_of_a_polar_solvent() {
        let none: [SpeciesId; 0] = [];
        assert!(retained_in_aqueous(&id("Na+"), &id("hexane"), none.iter()));
        assert!(retained_in_aqueous(&id("Cl-"), &id("hexane"), none.iter()));
        assert!(!retained_in_aqueous(
            &id("naphthalene"),
            &id("hexane"),
            none.iter()
        ));
        assert!(!retained_in_aqueous(
            &id("Na+"),
            &id("ethyl_acetate"),
            none.iter()
        ));
    }

    #[test]
    fn a_lipophilic_ion_beside_it_withdraws_the_rule() {
        let pair = [id("C12H25SO4-")];
        assert!(!retained_in_aqueous(&id("Na+"), &id("hexane"), pair.iter()));
    }

    #[test]
    fn citrate_is_not_lipophilic_per_charge() {
        assert!(!is_lipophilic_ion(&id("C6H5O7-3")));
        assert!(is_lipophilic_ion(&id("C12H25SO4-")));
    }

    #[test]
    fn ion_classes_are_named() {
        assert_eq!(ion_class(&id("Na+")), Some("monatomic cation"));
        assert_eq!(ion_class(&id("Cl-")), Some("monatomic anion"));
        assert_eq!(ion_class(&id("SO4-2")), Some("polyatomic inorganic ion"));
        assert_eq!(ion_class(&id("CH3COO-")), Some("small organic ion"));
        assert_eq!(ion_class(&id("HCO3-")), Some("polyatomic inorganic ion"));
        assert_eq!(ion_class(&id("naphthalene")), None);
    }
}
