//! Thermal decomposition of dry metal nitrates, table-driven by cation class.
//!
//! The 2026-11-28 sweep found two faces of one gap (S2, S3 in
//! `docs/reaction-sweep-gaps-20261120.md`). Sodium nitrate heated in a
//! crucible went to sodium nitrite and then to *liquid sodium metal*, with no
//! oxygen booked for either step; copper(II) nitrate held at 1500 °C did not
//! decompose at all. Neither was a missing number for one salt. The bench had
//! no rule for what a hot nitrate turns into, and the Gibbs solver filled the
//! hole with whatever its pool could reach.
//!
//! The rule a chemist uses is a classification by cation, and that is what
//! this module stores: [`CLASSES`] has one row per class, naming the cations
//! it covers, the solid it leaves and the gas it gives per nitrate group. A
//! salt is decomposed only when it also has a row in [`ONSETS`], which holds
//! the cited temperature where the decomposition begins and the cited
//! reaction enthalpy. A salt with no onset row is not decomposed and nothing
//! is claimed for it; a salt whose class product is not a registry species is
//! refused with a typed note rather than given an invented product.
//!
//! Sources (both United States Government works in the public domain):
//! K. H. Stern, *High Temperature Properties and Decomposition of Inorganic
//! Salts. Part 3. Nitrates and Nitrites*, J. Phys. Chem. Ref. Data 1 (1972)
//! 747–772, doi:10.1063/1.3253104, NIST reprint
//! <https://srd.nist.gov/jpcrdreprint/1.3253104.pdf> (read 2026-11-28, sha256
//! ff6961a2ca6434af08401337c0471f0d80c1e618114622c2bb0056d0a6e5fcf2); and the
//! NBS tables (D. D. Wagman et al., J. Phys. Chem. Ref. Data 11 (1982)
//! Suppl. 2), where a value is quoted from the registry's own citation.
//!
//! What is claimed: the products and the stoichiometry of the first
//! decomposition step of each class, beginning at the cited onset, paid for
//! with the cited standard reaction enthalpy out of the heat the vessel holds
//! above that onset. What is NOT claimed: rates (the decomposition runs as far
//! as the heat allows, at once), the intermediate basic nitrates of lead, the
//! nitrite's own further decomposition to oxide and nitrogen oxides (Stern
//! p. 768: above ~800 °C, and dependent on the atmosphere), the temperature
//! dependence of the enthalpy, and the sensible heat the gases carry away.
//!
//! No row may leave an alkali metal: Group 1 nitrates stop at the nitrite.
//! The same guard on the Gibbs route is `kerotakis_cea`'s alkali-metal
//! withholding; `no_class_leaves_an_alkali_metal` holds this table to it.

use crate::ops::{Event, NotModelledCause};
use crate::phrase::{Phrase, Slot};
use crate::species::{self, Phase};
use crate::units::{Kelvin, Moles};
use crate::vessel::Vessel;
use crate::SpeciesId;

const TRACE: f64 = 1e-12;

/// The alkali metals. Their nitrates stop at the nitrite, and no thermal
/// route on the bench may leave the metal itself.
pub const ALKALI_METALS: &[&str] = &["Li", "Na", "K", "Rb", "Cs"];

/// What a class of nitrate leaves behind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Residue {
    /// The metal nitrite, `MNO2`.
    Nitrite,
    /// The metal oxide at the cation's own charge (`M2O`, `MO`, `M2O3`).
    Oxide,
    /// The metal itself.
    Metal,
}

/// One cation class: which cations it covers and what one nitrate group
/// gives.
#[derive(Clone, Copy, Debug)]
pub struct ClassRow {
    pub name: &'static str,
    /// Cation elements covered. An empty list means "every other metal".
    pub cations: &'static [&'static str],
    pub residue: Residue,
    /// Moles of NO2 released per nitrate group.
    pub no2_per_nitrate: f64,
    /// Moles of O2 released per nitrate group.
    pub o2_per_nitrate: f64,
    pub source: &'static str,
}

/// One row per cation class. Order matters: the first row whose cation list
/// names the salt's metal wins, and the row with the empty list is the
/// fallback for every other metal.
pub const CLASSES: &[ClassRow] = &[
    ClassRow {
        name: "group 1 except lithium",
        cations: &["Na", "K", "Rb", "Cs"],
        residue: Residue::Nitrite,
        no2_per_nitrate: 0.0,
        o2_per_nitrate: 0.5,
        source: "Stern 1972: NaNO3 (p. 768) and KNO3 (p. 763) melt without decomposition and lose oxygen to the nitrite, MNO3 = MNO2 + 1/2 O2; the nitrite decomposes further only to the oxide and nitrogen oxides, never to the metal",
    },
    ClassRow {
        name: "noble metals",
        cations: &["Ag", "Au", "Hg", "Pt", "Pd"],
        residue: Residue::Metal,
        no2_per_nitrate: 1.0,
        o2_per_nitrate: 0.5,
        source: "Stern 1972, p. 766-767: silver oxide is unstable above 400 K, so the decomposition of AgNO3 is 2 AgNO3 = 2 Ag + 2 NO2 + O2",
    },
    ClassRow {
        name: "lithium, group 2, transition and post-transition metals",
        cations: &[],
        residue: Residue::Oxide,
        no2_per_nitrate: 1.0,
        o2_per_nitrate: 0.25,
        source: "Stern 1972: the general decomposition M(NO3)n = MO(n/2) + n NO2 + n/4 O2 (e.g. copper, p. 757-758: above 500 K the solid decomposes to NO2 + O2 leaving CuO; lead, p. 758-759: overall products PbO, NO2 and O2; lithium, p. 759)",
    },
];

/// Where a salt's onset temperature comes from.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Onset {
    /// A temperature cited in this row's `source`, K.
    Cited(f64),
    /// The registry's reviewed `decomposition_k` for the salt.
    Registry,
}

/// The cited onset and reaction enthalpy of one salt.
#[derive(Clone, Copy, Debug)]
pub struct OnsetRow {
    pub nitrate: &'static str,
    pub onset: Onset,
    /// Standard reaction enthalpy at 298.15 K per mole of the nitrate, J
    /// (positive: endothermic, heat taken from the vessel).
    pub heat_j_per_mol: f64,
    pub source: &'static str,
}

/// Cited onsets and reaction enthalpies. A nitrate absent here is not
/// decomposed by this route.
pub const ONSETS: &[OnsetRow] = &[
    OnsetRow {
        nitrate: "NaNO3",
        onset: Onset::Cited(873.15),
        heat_j_per_mol: 109_330.0,
        source: "Stern 1972, p. 768: NaNO3 'is stable in air at least to 500 °C and begins to decompose slowly at 600°' (873 K). Enthalpy: Stern p. 769, table 'Decomposition of NaNO3', NaNO3 = NaNO2 + 1/2 O2, ΔH°298.15 = 109.33 kJ",
    },
    OnsetRow {
        nitrate: "KNO3",
        onset: Onset::Cited(923.15),
        heat_j_per_mol: 124_391.0,
        source: "Stern 1972, p. 763: KNO3 melts without decomposition, 'is stable in air at least to 530 °C' and 'begins to decompose near 650°' (923 K) in air. Enthalpy from Stern's ΔHf° table (p. 763, ref. NBS TN 270): KNO3 -494.633, KNO2 -370.242 kJ/mol, so KNO3 = KNO2 + 1/2 O2 is +124.391 kJ",
    },
    OnsetRow {
        nitrate: "Cu(NO3)2",
        onset: Onset::Cited(500.0),
        heat_j_per_mol: 215_641.0,
        source: "Stern 1972, p. 757-758: below 500 K the anhydrous salt mainly sublimes; 'above 500 K both the solid and vapor phase decompose to NO2 + O2'. Enthalpy from formation enthalpies: Cu(NO3)2 cr -302.9 kJ/mol (NBS/Wagman 1982, Table 36:Cu, as cited by the registry row), CuO(cr) -155.6448 and NO2(g) +34.193 kJ/mol (NASA CEA thermo.inp, King 1973 and Gurvich 1989), so Cu(NO3)2 = CuO + 2 NO2 + 1/2 O2 is +215.641 kJ",
    },
    OnsetRow {
        nitrate: "AgNO3",
        onset: Onset::Registry,
        heat_j_per_mol: 157_485.0,
        source: "Onset: the registry's reviewed decomposition point (513 K; Stern 1972, p. 766: decomposition 'becomes appreciable 30-40° above the melting point', 483 K). Enthalpy: Stern p. 767, 2 AgNO3(c) = 2 Ag(c) + 2 NO2(g) + O2(g), ΔH°298.15 = 314.97 kJ, i.e. 157.485 kJ per mole of AgNO3",
    },
    OnsetRow {
        nitrate: "Pb(NO3)2",
        onset: Onset::Registry,
        heat_j_per_mol: 300_938.0,
        source: "Onset: the registry's reviewed decomposition point (539 K; Stern 1972, p. 758-759, the lower end of the measured first-step equilibrium range). Enthalpy from Stern's ΔHf° tables: Pb(NO3)2 -451.9, PbO (yellow) -217.32 (p. 758, ref. NBS TN 270) and NO2(g) +33.179 kJ/mol (7.93 kcal, p. 748, JANAF), so Pb(NO3)2 = PbO + 2 NO2 + 1/2 O2 is +300.938 kJ",
    },
];

/// The decomposition this route would run for one salt.
#[derive(Clone, Debug, PartialEq)]
pub struct Decomposition {
    pub nitrate: &'static str,
    pub class: &'static str,
    /// The residue's registry key (it may be absent from the registry; then
    /// the route refuses).
    pub residue: String,
    /// Moles of residue per mole of nitrate.
    pub residue_per_mol: f64,
    pub no2_per_mol: f64,
    pub o2_per_mol: f64,
    pub onset_k: f64,
    pub heat_j_per_mol: f64,
}

/// The metal and the number of nitrate groups in a nitrate formula: one
/// metal element plus nitrogen and oxygen in the ratio 1:3.
fn metal_and_groups(formula: &str) -> Option<(String, f64, f64)> {
    let f = crate::stoich::parse_formula(formula).ok()?;
    if f.charge != 0.0 {
        return None;
    }
    let n = *f.counts.get("N")?;
    let o = *f.counts.get("O")?;
    if (o - 3.0 * n).abs() > 1e-9 || f.counts.len() != 3 {
        return None;
    }
    let (metal, count) = f.counts.iter().find(|(el, _)| *el != "N" && *el != "O")?;
    Some((metal.clone(), *count, n))
}

fn class_of(metal: &str) -> &'static ClassRow {
    CLASSES
        .iter()
        .find(|row| row.cations.contains(&metal))
        .or_else(|| CLASSES.iter().find(|row| row.cations.is_empty()))
        .expect("CLASSES has a fallback row")
}

/// The registry key of `metal` written with `count` atoms and an oxygen
/// count, in the registry's formula spelling (`CuO`, `Li2O`, `Fe2O3`).
fn oxide_key(metal: &str, charge: f64) -> Option<(String, f64)> {
    // MaOb with a*charge = 2b, smallest integers.
    let z = charge.round() as i64;
    if z <= 0 || (charge - z as f64).abs() > 1e-9 {
        return None;
    }
    let (a, b) = if z % 2 == 0 { (1, z / 2) } else { (2, z) };
    let sub = |n: i64| if n == 1 { String::new() } else { n.to_string() };
    Some((format!("{metal}{}O{}", sub(a), sub(b)), a as f64))
}

/// The decomposition `nitrate` has on this table, if it has a class and an
/// onset row.
pub fn decomposition(nitrate: &str) -> Option<Decomposition> {
    let row = ONSETS.iter().find(|r| r.nitrate == nitrate)?;
    let data = species::lookup_key(nitrate)?;
    let (metal, metal_count, groups) = metal_and_groups(data.formula)?;
    let class = class_of(&metal);
    // Nitrate groups per metal atom: the cation's charge.
    let charge = groups / metal_count;
    let (residue, residue_per_mol) = match class.residue {
        Residue::Nitrite => {
            if (charge - 1.0).abs() > 1e-9 {
                return None;
            }
            (format!("{metal}NO2"), metal_count)
        }
        Residue::Metal => (metal.clone(), metal_count),
        Residue::Oxide => {
            let (key, atoms) = oxide_key(&metal, charge)?;
            (key, metal_count / atoms)
        }
    };
    let onset_k = match row.onset {
        Onset::Cited(k) => k,
        Onset::Registry => data.transitions.as_ref()?.decomposition_k?,
    };
    Some(Decomposition {
        nitrate: row.nitrate,
        class: class.name,
        residue,
        residue_per_mol,
        no2_per_mol: class.no2_per_nitrate * groups,
        o2_per_mol: class.o2_per_nitrate * groups,
        onset_k,
        heat_j_per_mol: row.heat_j_per_mol,
    })
}

/// The written equation, with the smallest whole-number coefficients.
pub fn equation(d: &Decomposition) -> String {
    let terms = [1.0, d.residue_per_mol, d.no2_per_mol, d.o2_per_mol];
    let k = (1..=8)
        .map(f64::from)
        .find(|k| terms.iter().all(|t| ((t * k) - (t * k).round()).abs() < 1e-9))
        .unwrap_or(1.0);
    let term = |c: f64, key: &str, gas: bool| -> Option<String> {
        let n = (c * k).round();
        (n > 0.0).then(|| {
            let lead = if n == 1.0 { String::new() } else { format!("{n} ") };
            format!("{lead}{}{}", subscript(key), if gas { "↑" } else { "" })
        })
    };
    let rhs: Vec<String> = [
        term(d.residue_per_mol, &d.residue, false),
        term(d.no2_per_mol, "NO2", true),
        term(d.o2_per_mol, "O2", true),
    ]
    .into_iter()
    .flatten()
    .collect();
    format!(
        "{} →Δ {}",
        term(1.0, d.nitrate, false).unwrap_or_default(),
        rhs.join(" + ")
    )
}

fn subscript(formula: &str) -> String {
    formula
        .chars()
        .map(|c| match c {
            '0'..='9' => char::from_u32(0x2080 + c as u32 - '0' as u32).unwrap_or(c),
            other => other,
        })
        .collect()
}

/// Whether the vessel holds liquid water: then the salt is in solution and
/// the aqueous engine owns it.
fn wet(vessel: &Vessel) -> bool {
    vessel
        .contents
        .iter()
        .any(|p| p.species.0 == "water" && p.phase == Phase::Liquid && p.moles.0 > TRACE)
}

/// Undissolved nitrate: the solid, or its own melt.
fn dry_moles(vessel: &Vessel, key: &str) -> f64 {
    let id = SpeciesId::new(key);
    let id = id.canonical();
    vessel
        .contents
        .iter()
        .filter(|p| &p.species == id.as_ref() && matches!(p.phase, Phase::Solid | Phase::Liquid))
        .map(|p| p.moles.0)
        .sum()
}

/// The salts that are hot enough and present, each with its decomposition.
fn candidates(vessel: &Vessel) -> Vec<(Decomposition, f64)> {
    if wet(vessel) {
        return Vec::new();
    }
    ONSETS
        .iter()
        .filter_map(|row| {
            let moles = dry_moles(vessel, row.nitrate);
            if moles <= TRACE {
                return None;
            }
            let d = decomposition(row.nitrate)?;
            (vessel.temperature.0 >= d.onset_k).then_some((d, moles))
        })
        .collect()
}

/// How far one salt can go on the heat the vessel holds above its onset.
fn extent(vessel: &Vessel, d: &Decomposition, moles: f64) -> f64 {
    if d.heat_j_per_mol <= 0.0 || vessel.heat_capacity() <= 0.0 {
        return moles;
    }
    let above = vessel.energy_between(d.onset_k, vessel.temperature.0).max(0.0);
    moles.min(above / d.heat_j_per_mol)
}

fn residue_known(d: &Decomposition) -> bool {
    species::lookup_key(&d.residue).is_some()
        && !ALKALI_METALS.contains(&d.residue.as_str())
}

/// Whether any salt would decompose (or be refused) on this solve.
pub fn fires(vessel: &Vessel) -> bool {
    candidates(vessel)
        .iter()
        .any(|(d, moles)| !residue_known(d) || extent(vessel, d, *moles) > TRACE)
}

fn note_once(events: &mut Vec<Event>, vessel: &Vessel, phrase: Phrase) {
    let dup = events.iter().any(|e| match e {
        Event::NotYetModeled {
            reason: Some(r), ..
        } => r.key == phrase.key,
        _ => false,
    });
    if !dup {
        events.push(Event::not_modeled(
            vessel.id,
            NotModelledCause::ModelBoundary,
            phrase,
        ));
    }
}

fn gas(vessel: &mut Vessel, events: &mut Vec<Event>, key: &str, moles: f64) {
    if moles <= TRACE {
        return;
    }
    let species = SpeciesId::new(key);
    let n = Moles(moles);
    if vessel.retain_gas(species.clone(), n) {
        events.push(Event::GasContained {
            vessel: vessel.id,
            species,
            moles: n,
        });
    } else {
        events.push(Event::GasEvolved {
            vessel: vessel.id,
            species,
            moles: n,
        });
    }
}

/// Decompose every hot dry nitrate as far as the heat above its onset pays
/// for. Returns whether anything happened.
pub fn apply(vessel: &mut Vessel, events: &mut Vec<Event>) -> bool {
    let mut progressed = false;
    for (d, moles) in candidates(vessel) {
        if !residue_known(&d) {
            note_once(
                events,
                vessel,
                Phrase::new(
                    "not-modeled.nitrate-decomposition-residue-missing",
                    "{nitrate} is above the temperature where it decomposes ({onset} K), but what it leaves, {residue}, is not a substance this bench can name, so it is left undecomposed rather than given an invented product",
                    vec![
                        ("nitrate".to_string(), Slot::term("species", d.nitrate)),
                        ("onset".to_string(), Slot::number(format!("{:.0}", d.onset_k))),
                        ("residue".to_string(), Slot::text(d.residue.clone())),
                    ],
                ),
            );
            continue;
        }
        let x = extent(vessel, &d, moles);
        if x <= TRACE {
            continue;
        }
        let before = vessel.temperature.0;
        let mut left = x;
        for phase in [Phase::Solid, Phase::Liquid] {
            left -= vessel
                .withdraw_phase(&SpeciesId::new(d.nitrate), Moles(left), phase)
                .0;
        }
        let taken = x - left;
        vessel.deposit(
            SpeciesId::new(&d.residue),
            Moles(taken * d.residue_per_mol),
            Phase::Solid,
        );
        events.push(Event::ReactionOccurred {
            vessel: vessel.id,
            equation: equation(&d),
        });
        events.push(Event::Consumed {
            vessel: vessel.id,
            species: SpeciesId::new(d.nitrate),
            moles: Moles(taken),
            remaining: Some(Moles(dry_moles(vessel, d.nitrate))),
        });
        gas(vessel, events, "NO2", taken * d.no2_per_mol);
        gas(vessel, events, "O2", taken * d.o2_per_mol);
        // The reaction is paid out of the heat the vessel holds above the
        // onset: the contents fall back toward it, and a burner still on
        // brings them up again on its next pass.
        let to = vessel.temperature_after(-taken * d.heat_j_per_mol);
        if to.is_finite() && to != before {
            vessel.temperature = Kelvin(to);
            events.push(Event::TemperatureChanged {
                vessel: vessel.id,
                from: Kelvin(before),
                to: Kelvin(to),
            });
        }
        progressed = true;
    }
    progressed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_onset_row_has_a_class_and_balances() {
        for row in ONSETS {
            let d = decomposition(row.nitrate)
                .unwrap_or_else(|| panic!("{} has no decomposition", row.nitrate));
            let salt = crate::stoich::parse_formula(
                species::lookup_key(row.nitrate).unwrap().formula,
            )
            .unwrap();
            let residue = crate::stoich::parse_formula(&d.residue).unwrap();
            let mut lhs = salt.counts.clone();
            let mut rhs: std::collections::BTreeMap<String, f64> = Default::default();
            for (el, n) in &residue.counts {
                *rhs.entry(el.clone()).or_insert(0.0) += n * d.residue_per_mol;
            }
            *rhs.entry("N".into()).or_insert(0.0) += d.no2_per_mol;
            *rhs.entry("O".into()).or_insert(0.0) += 2.0 * d.no2_per_mol + 2.0 * d.o2_per_mol;
            lhs.retain(|_, n| *n != 0.0);
            rhs.retain(|_, n| n.abs() > 1e-12);
            assert_eq!(lhs.len(), rhs.len(), "{}: {lhs:?} vs {rhs:?}", row.nitrate);
            for (el, n) in &lhs {
                assert!((rhs[el] - n).abs() < 1e-12, "{} {el}: {lhs:?} vs {rhs:?}", row.nitrate);
            }
            assert!(d.onset_k > 298.15 && d.heat_j_per_mol > 0.0, "{d:?}");
        }
    }

    #[test]
    fn classes_give_the_textbook_products() {
        let na = decomposition("NaNO3").unwrap();
        assert_eq!((na.residue.as_str(), na.no2_per_mol, na.o2_per_mol), ("NaNO2", 0.0, 0.5));
        assert_eq!(equation(&na), "2 NaNO₃ →Δ 2 NaNO₂ + O₂↑");
        let k = decomposition("KNO3").unwrap();
        assert_eq!(k.residue, "KNO2");
        let cu = decomposition("Cu(NO3)2").unwrap();
        assert_eq!((cu.residue.as_str(), cu.residue_per_mol), ("CuO", 1.0));
        assert_eq!(equation(&cu), "2 Cu(NO₃)₂ →Δ 2 CuO + 4 NO₂↑ + O₂↑");
        let ag = decomposition("AgNO3").unwrap();
        assert_eq!(equation(&ag), "2 AgNO₃ →Δ 2 Ag + 2 NO₂↑ + O₂↑");
        assert_eq!(ag.onset_k, 513.0);
        let pb = decomposition("Pb(NO3)2").unwrap();
        assert_eq!((pb.residue.as_str(), pb.onset_k), ("PbO", 539.0));
    }

    #[test]
    fn no_class_leaves_an_alkali_metal() {
        for row in CLASSES {
            if row.residue == Residue::Metal {
                for metal in ALKALI_METALS {
                    assert!(!row.cations.contains(metal), "{} leaves {metal}", row.name);
                }
            }
        }
        // The fallback (oxide) row covers lithium; the nitrite row names
        // every other alkali metal explicitly.
        for metal in ALKALI_METALS {
            assert_ne!(class_of(metal).residue, Residue::Metal, "{metal}");
        }
        assert_eq!(class_of("Li").residue, Residue::Oxide);
        assert!(!residue_known(&Decomposition {
            nitrate: "NaNO3",
            class: "",
            residue: "Na".into(),
            residue_per_mol: 1.0,
            no2_per_mol: 1.0,
            o2_per_mol: 0.5,
            onset_k: 873.15,
            heat_j_per_mol: 1.0,
        }));
    }

    #[test]
    fn oxide_keys_follow_the_cation_charge() {
        assert_eq!(oxide_key("Cu", 2.0), Some(("CuO".into(), 1.0)));
        assert_eq!(oxide_key("Li", 1.0), Some(("Li2O".into(), 2.0)));
        assert_eq!(oxide_key("Fe", 3.0), Some(("Fe2O3".into(), 2.0)));
    }
}
