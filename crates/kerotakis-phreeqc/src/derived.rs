//! The derivation layer: how registry species participate in aqueous
//! problems, computed from their chemical formulas and the embedded
//! databases — not maintained by hand.
//!
//! What remains curated here, deliberately and documented:
//! - `OXYANION_GROUPS`: ~six rows of real chemistry (a sulfate group is
//!   S(6)) that give formulas their valence mapping;
//! - `BOOKING_OVERRIDES`: the protonation state dissolved element totals
//!   are booked as at teaching pH (bicarbonate rather than the database's
//!   carbonate master, dihydrogen phosphate rather than PO4-3);
//! - `ATMOSPHERIC`: physical constants — log10 partial pressures of the
//!   gases an open vessel vents against.
//!
//! Everything else (which compounds map to which elements, which mineral
//! phases exist where, their stoichiometry, hydrate waters, polymorph
//! choice) is derived.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use kerotakis_core::species::{self, Phase};

use crate::databases;
use crate::dbindex::{parse_formula, split_hydrate, DbIndex};

#[derive(Debug, Clone)]
pub enum DerivedRole {
    Solvent,
    /// Element contributions when dissolved (freely soluble or ionic).
    Dissolves(Vec<(String, f64)>),
    /// Enters as an amount-limited mineral phase.
    Mineral {
        phase: String,
        elements: Vec<(String, f64)>,
    },
}

#[derive(Debug, Clone)]
pub struct DerivedPhase {
    pub name: String,
    /// Registry key of the solid this phase is booked as.
    pub species: &'static str,
    /// Waters of crystallisation moved between liquid and crystal.
    pub waters: f64,
    /// Elements the dissolution reaction exchanges with solution.
    pub elements: Vec<(String, f64)>,
}

/// Gas phases an open vessel vents, with the log10 atmospheric partial
/// pressure it equilibrates against and the water co-product per mole of
/// gas (HCO3- + H+ → CO2↑ + H2O). Physical constants, not chemistry the
/// databases know.
pub const ATMOSPHERIC: &[(&str, &str, f64, f64)] = &[("CO2(g)", "CO2", -3.408, 1.0)];

/// Oxyanion groups: the valence-carrying units formulas decompose into.
/// (Element-count signatures; order matters — longest/most specific first.)
fn oxyanion_groups() -> &'static [(&'static str, &'static str)] {
    &[
        ("C2H3O2", "Acetate"), // CH3COO
        ("HCO3", "C"),
        ("CO3", "C"),
        ("NO3", "N(5)"),
        ("SO4", "S(6)"),
        ("PO4", "P"),
        ("MnO4", "Mn(7)"),
    ]
}

/// Elements allowed as bare residue after group extraction: those whose
/// aqueous identity is unambiguous (simple cations and halides). Anything
/// else left over (N, S, C, P outside a group; unbalanced O) means the
/// compound's aqueous chemistry is not derivable — honestly unmappable.
const RESIDUE_OK: &[&str] = &[
    "Na", "K", "Ca", "Mg", "Ag", "Li", "Sr", "Ba", "Cl", "Br", "F", "Cu", "Mn", "Fe", "Zn",
];

/// The subset of `RESIDUE_OK` that forms simple cations.
///
/// Oxygen left over after the hydrogen is accounted for is *oxide* oxygen
/// when the only thing beside it is a metal: CuO dissolves as Cu²⁺ + H₂O,
/// the oxide leaving as water exactly as hydroxide does, and PHREEQC's
/// `pH charge` recovers the protons it consumed. With a halogen residue the
/// same arithmetic would be wrong — the oxygen in hypochlorite is bound to
/// chlorine and is not available as water — which is what the original
/// oxygen guard was protecting against, rather more broadly than it needed
/// to: it rejected every simple metal oxide in the databases.
const CATION_RESIDUE: &[&str] = &[
    "Na", "K", "Ca", "Mg", "Ag", "Li", "Sr", "Ba", "Cu", "Mn", "Fe", "Zn",
];

/// How dissolved element totals are booked back into the vessel inventory:
/// the database's master species, unless overridden by the documented
/// protonation-state choice.
const BOOKING_OVERRIDES: &[(&str, &str)] = &[
    ("C", "HCO3-"),
    ("P", "H2PO4-"),
    ("Acetate", "CH3COO-"),
    ("Mn(7)", "MnO4-"),
    // Bare manganese books as the reduced ion, which is what the databases
    // treat as the master species and what dissolved manganese actually is
    // at bench conditions. Needed once MnO2 entered the registry: it is the
    // first substance to put Mn in solution as anything but permanganate.
    ("Mn", "Mn+2"),
    ("Cu", "Cu+2"),
];

pub struct Derived {
    pub wateq4f: DbIndex,
    pub minteq: DbIndex,
    pub pitzer: DbIndex,
    roles: BTreeMap<&'static str, DerivedRole>,
    phases: Vec<DerivedPhase>,
    bookings: BTreeMap<String, &'static str>,
}

static DERIVED: OnceLock<Derived> = OnceLock::new();

pub fn derived() -> &'static Derived {
    DERIVED.get_or_init(Derived::build)
}

pub fn role(key: &str) -> Option<&'static DerivedRole> {
    derived().roles.get(key)
}

pub fn candidate_phases() -> &'static [DerivedPhase] {
    &derived().phases
}

pub fn phase_by_name(name: &str) -> Option<&'static DerivedPhase> {
    derived().phases.iter().find(|p| p.name == name)
}

/// Registry key an element total is booked as. `None` means the element has
/// no registry ion to book into (a registry gap, surfaced by tests).
pub fn booking_ion(element: &str) -> Option<&'static str> {
    derived().bookings.get(element).copied()
}

pub fn index_for(db_tag: &str) -> &'static DbIndex {
    let d = derived();
    match db_tag {
        "minteq.v4" => &d.minteq,
        "pitzer" => &d.pitzer,
        _ => &d.wateq4f,
    }
}

impl Derived {
    fn build() -> Derived {
        let wateq4f = DbIndex::parse(databases::WATEQ4F);
        let minteq = DbIndex::parse(databases::MINTEQ_V4);
        let pitzer = DbIndex::parse(databases::PITZER);
        let indexes = [&wateq4f, &minteq, &pitzer];

        // --- Candidate phases: database phases whose formula matches a
        // solid registry species exactly (including hydrate waters), deduped
        // per species by stability (lowest log_k = least soluble polymorph).
        let mut phases: Vec<(DerivedPhase, f64)> = Vec::new();
        for idx in indexes {
            for (name, info) in &idx.phases {
                if info.is_gas || phases.iter().any(|(p, _)| p.name == *name) {
                    continue;
                }
                let Some(reg) = registry_solid_matching(&info.composition, info.waters) else {
                    continue;
                };
                // Elements from the formula's own composition through the
                // same group decomposition as registry formulas — robust
                // against per-database equation quirks (e.g. wateq4f's
                // alkalinity conventions).
                let Some(elements) = contribution_from_counts(info.composition.clone(), indexes)
                else {
                    continue;
                };
                phases.push((
                    DerivedPhase {
                        name: name.clone(),
                        species: reg,
                        waters: info.waters,
                        elements,
                    },
                    info.log_k.unwrap_or(f64::MAX),
                ));
            }
        }
        // Dedupe polymorphs: keep the stablest phase per solid species.
        phases.sort_by(|a, b| a.1.total_cmp(&b.1));
        let mut seen: Vec<&str> = Vec::new();
        let phases: Vec<DerivedPhase> = phases
            .into_iter()
            .filter_map(|(p, _)| {
                if seen.contains(&p.species) {
                    None
                } else {
                    seen.push(p.species);
                    Some(p)
                }
            })
            .collect();

        // --- Bookings: master species per element (must exist as a registry
        // key), with the documented protonation overrides.
        let mut bookings: BTreeMap<String, &'static str> = BTreeMap::new();
        for idx in indexes {
            for (element, master) in &idx.masters {
                if bookings.contains_key(element) {
                    continue;
                }
                let over = BOOKING_OVERRIDES
                    .iter()
                    .find(|(el, _)| el == element)
                    .map(|(_, ion)| *ion);
                let candidate: Option<&'static str> = over.or_else(|| {
                    species::REGISTRY
                        .iter()
                        .find(|s| s.key == master.species)
                        .map(|s| s.key)
                });
                if let Some(ion) = candidate {
                    bookings.insert(element.clone(), ion);
                }
            }
        }

        // --- Roles from formulas.
        let mut roles: BTreeMap<&'static str, DerivedRole> = BTreeMap::new();
        for s in species::REGISTRY {
            if s.key == "water" {
                roles.insert(s.key, DerivedRole::Solvent);
                continue;
            }
            if s.standard_phase == Phase::Gas {
                continue; // gases don't enter aqueous problems directly
            }
            // Mineral: a candidate phase books as this species.
            if s.standard_phase == Phase::Solid {
                if let Some(p) = phases.iter().find(|p| p.species == s.key) {
                    roles.insert(
                        s.key,
                        DerivedRole::Mineral {
                            phase: p.name.clone(),
                            elements: p.elements.clone(),
                        },
                    );
                    continue;
                }
            }
            if let Some(contrib) = derive_contribution(s.formula, indexes) {
                roles.insert(s.key, DerivedRole::Dissolves(contrib));
            }
            // else: honestly unmappable — no role, the honesty pass speaks.
        }

        Derived {
            wateq4f,
            minteq,
            pitzer,
            roles,
            phases,
            bookings,
        }
    }
}

/// A solid registry species whose formula matches this composition and
/// hydrate count exactly.
fn registry_solid_matching(
    composition: &BTreeMap<String, f64>,
    waters: f64,
) -> Option<&'static str> {
    for s in species::REGISTRY {
        if s.standard_phase != Phase::Solid {
            continue;
        }
        let (base, w) = split_hydrate(s.formula);
        if w != waters {
            continue;
        }
        if parse_formula(&base).as_ref() == Some(composition) {
            return Some(s.key);
        }
    }
    None
}

/// Decompose a formula into element contributions: greedy oxyanion-group
/// extraction, then residue rules (drop hydroxide OH pairs and acid
/// protons; anything else unaccounted → unmappable).
fn derive_contribution(formula: &str, indexes: [&DbIndex; 3]) -> Option<Vec<(String, f64)>> {
    let (base, _) = split_hydrate(formula);
    let counts = parse_formula(&base)?;
    contribution_from_counts(counts, indexes)
}

/// The charge this element books into solution as, resolved without
/// touching `derived()` — see the note at the call site.
fn cation_charge(element: &str, indexes: [&DbIndex; 3]) -> Option<f64> {
    let ion = BOOKING_OVERRIDES
        .iter()
        .find(|(el, _)| *el == element)
        .map(|(_, ion)| (*ion).to_string())
        .or_else(|| {
            indexes
                .iter()
                .find_map(|idx| idx.masters.get(element).map(|m| m.species.clone()))
        })?;
    kerotakis_core::stoich::parse_formula(&ion)
        .ok()
        .map(|f| f.charge)
}

fn contribution_from_counts(
    mut counts: BTreeMap<String, f64>,
    indexes: [&DbIndex; 3],
) -> Option<Vec<(String, f64)>> {
    let mut contrib: Vec<(String, f64)> = Vec::new();

    for (group_formula, element) in oxyanion_groups() {
        let sig = parse_formula(group_formula).expect("group formulas parse");
        let fit = sig
            .iter()
            .map(|(el, n)| (counts.get(el).copied().unwrap_or(0.0) / n).floor())
            .fold(f64::INFINITY, f64::min);
        if fit >= 1.0 {
            for (el, n) in &sig {
                let c = counts.get_mut(el).unwrap();
                *c -= n * fit;
            }
            contrib.push((element.to_string(), fit));
        }
    }
    counts.retain(|_, n| *n > 0.0);

    // Hydroxide pairs and acid protons drop out: they live in the water /
    // charge-balance domain, and PHREEQC's `pH charge` recovers them.
    let h = counts.remove("H").unwrap_or(0.0);
    let o = counts.remove("O").unwrap_or(0.0);
    if o > h {
        // Oxide oxygen may be dropped as water only when doing so conserves
        // electrons. Treating O as −2 and each O–H pair as hydroxide, the
        // metal's oxidation state in the solid is (2·O − H) per metal atom;
        // it must equal the charge on the ion the element books as.
        //
        // CuO booked as Cu²⁺ balances, so tenorite dissolves as itself.
        // MnO2 booked as Mn²⁺ does not — that is Mn(IV) + 2e⁻ with no
        // oxidant anywhere, and the orphaned oxygen leaves as alkalinity:
        // 5 g of manganese dioxide dissolving completely to give a
        // cation-only solution reading pH 13.55, which is not chemistry.
        //
        // Excluding every redox-active element instead was the first
        // attempt and was too blunt: it took copper with it, and copper's
        // oxide is not a redox problem at all.
        let balanced = counts.len() == 1
            && counts.iter().all(|(el, n)| {
                if !CATION_RESIDUE.contains(&el.as_str()) || *n <= 0.0 {
                    return false;
                }
                let implied = (2.0 * o - h) / n;
                // Resolved from the indexes in hand, never through
                // `booking_ion`: this runs inside the `derived()` one-time
                // initialiser, and asking `derived()` for anything from in
                // here deadlocks on its own lock.
                cation_charge(el, indexes).is_some_and(|q| (q - implied).abs() < 1e-9)
            });
        if !balanced {
            // Either an oxyanion (hypochlorite's oxygen is bound to
            // chlorine, not available as water) or an oxide whose
            // dissolution would be a redox step we do not model. Not
            // derivable, and said so rather than guessed at.
            return None;
        }
    }

    for (el, n) in counts {
        if !RESIDUE_OK.contains(&el.as_str()) {
            return None;
        }
        if !indexes.iter().any(|idx| idx.has_element(&el)) {
            return None;
        }
        contrib.push((el, n));
    }
    if contrib.is_empty() {
        None
    } else {
        Some(contrib)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dissolves(key: &str) -> Vec<(String, f64)> {
        match role(key) {
            Some(DerivedRole::Dissolves(c)) => c.clone(),
            other => panic!("{key}: expected Dissolves, got {other:?}"),
        }
    }

    #[test]
    fn roles_are_derived_correctly() {
        // Minerals found by formula match, stablest polymorph chosen.
        for (key, phase) in [
            ("NaCl", "Halite"),
            ("AgCl", "Cerargyrite"),
            ("CaCO3", "Calcite"),
            ("gypsum", "Gypsum"),
        ] {
            match role(key) {
                Some(DerivedRole::Mineral { phase: p, .. }) => assert_eq!(p, phase, "{key}"),
                other => panic!("{key}: expected Mineral, got {other:?}"),
            }
        }

        // Freely-soluble compounds decomposed by group + residue.
        assert_eq!(
            dissolves("AgNO3"),
            vec![("N(5)".into(), 1.0), ("Ag".into(), 1.0)]
        );
        assert_eq!(dissolves("HCl"), vec![("Cl".into(), 1.0)]);
        assert_eq!(dissolves("NaOH"), vec![("Na".into(), 1.0)]);
        assert_eq!(
            dissolves("CaCl2"),
            vec![("Ca".into(), 1.0), ("Cl".into(), 2.0)]
        );
        assert_eq!(dissolves("H3PO4"), vec![("P".into(), 1.0)]);
        assert_eq!(dissolves("CH3COOH"), vec![("Acetate".into(), 1.0)]);
        // NaHCO3 may be Nahcolite-backed (derived from the databases) or
        // freely soluble; either way its elements are Na + C.
        let mut nahco3: Vec<String> = match role("NaHCO3").expect("NaHCO3 has a role") {
            DerivedRole::Dissolves(c) | DerivedRole::Mineral { elements: c, .. } => {
                c.iter().map(|(el, _)| el.clone()).collect()
            }
            DerivedRole::Solvent => panic!("NaHCO3 is not the solvent"),
        };
        nahco3.sort();
        assert_eq!(nahco3, vec!["C".to_string(), "Na".to_string()]);

        // Ions map through their formulas.
        assert_eq!(dissolves("SO4-2"), vec![("S(6)".into(), 1.0)]);
        assert_eq!(dissolves("HCO3-"), vec![("C".into(), 1.0)]);

        // Honestly unmappable: hypochlorite (O without H), ammonia (bare N),
        // organics (residual C), gases.
        assert!(role("NaOCl").is_none());
        assert!(role("NH3").is_none());
        assert!(role("ethanol").is_none());
        assert!(role("Cl2").is_none());
        assert!(role("CO2").is_none());
    }

    #[test]
    fn bookings_cover_every_derivable_element() {
        // Every element any derived role can produce must have a booking ion
        // that exists in the registry.
        let mut elements: Vec<String> = Vec::new();
        for s in species::REGISTRY {
            match role(s.key) {
                Some(DerivedRole::Dissolves(c))
                | Some(DerivedRole::Mineral { elements: c, .. }) => {
                    for (el, _) in c {
                        if !elements.contains(el) {
                            elements.push(el.clone());
                        }
                    }
                }
                _ => {}
            }
        }
        for el in &elements {
            let ion = booking_ion(el).unwrap_or_else(|| panic!("no booking ion for {el}"));
            assert!(
                species::lookup_key(ion).is_some(),
                "booking ion {ion} for {el} missing from registry"
            );
        }
    }

    #[test]
    fn phase_table_is_derived() {
        let sylvite = phase_by_name("Sylvite").expect("Sylvite derived from pitzer");
        assert_eq!(sylvite.species, "KCl");
        let gypsum = phase_by_name("Gypsum").expect("Gypsum derived");
        assert_eq!(gypsum.waters, 2.0);
        // Aragonite lost the polymorph dedupe to calcite.
        assert!(phase_by_name("Aragonite").is_none());
        assert!(phase_by_name("Calcite").is_some());
    }
}
