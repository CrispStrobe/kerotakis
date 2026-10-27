//! Reviewed dissolution reactions for every registry solid, from every
//! vendored USGS database, with the convention checked rather than assumed.
//!
//! The heat balance in [`crate::enthalpy`] prices a solid as what it
//! dissolves into, less the enthalpy of getting there. Until 2026-10-27 that
//! needed either a curated registry enthalpy of dissolution or a phase on the
//! ROUTED database whose written reaction was a plain neutral dissociation.
//! Everything else declined, so potassium nitrate (no Niter on any route),
//! gypsum and chalcanthite (hydrate water not balanced) and every hydroxide
//! written `M(OH)n + n H+ = M+n + n H2O` were refused, and quicklime was
//! priced by a one-off table row.
//!
//! This module replaces that with one path:
//!
//! 1. Every PHASES entry of `wateq4f`, `minteq.v4`, `pitzer`, `phreeqc.dat`
//!    and `llnl.dat` whose formula IS a registry solid (exact composition and
//!    hydrate count, [`derived::registry_solid_for`]) is a candidate. Per
//!    database the stablest polymorph (lowest log K) is used.
//! 2. A candidate is accepted only if its reaction balances atoms and charge
//!    with every term (H+, H2O and hydrate water included), carries no
//!    electrons or elemental redox partner, states a finite, non-placeholder
//!    `delta_h` at 25 °C, and every aqueous species it names is LINKED to the
//!    routed database: either both databases make it a basis species, or one
//!    database's defining reaction for it is (up to direction) a reaction the
//!    other database also writes, with log K within [`LOGK_TOLERANCE`] and
//!    `delta_h` within [`DELTA_H_TOLERANCE_KJ`] wherever both state one.
//!    That is what lets `llnl.dat`'s `CaCO3 + H+ = Ca+2 + HCO3-` (HCO3- is a
//!    basis species there) be used on wateq4f, where HCO3- carries −14.6 kJ.
//! 3. The solid is then `Σ h_routed(right) − Σ h_routed(left) − ΔH`.
//!
//! An anhydrous oxide with a registry hydroxide is priced through that
//! hydroxide by [`hydration`]: the difference of the two reactions in ONE
//! database must be exactly `oxide + n H2O = k hydroxide`.
//!
//! Domain: 25 °C, 1 bar reference reaction enthalpies, the same as every
//! other term of the balance. No heat-capacity extrapolation is made, and
//! the temperature range a database states for its log K fit is recorded in
//! the coverage table, not used.
//!
//! `llnl.dat` (Lawrence Livermore thermo.com.V8.R6.230 as converted by the
//! USGS for PHREEQC, public domain like the other vendored USGS files) is
//! not otherwise shipped, so only the rows this module needs are carried, as
//! [`LLNL_FIXTURE`], re-extracted and compared by
//! `tests/dissolution_enthalpy_coverage.rs` so it cannot go stale.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use kerotakis_core::species::{self, Phase};
use kerotakis_core::stoich::{self, FormulaDialect};
use kerotakis_core::SpeciesId;
use serde::{Deserialize, Serialize};

use crate::dbindex::{parse_formula, split_hydrate};
use crate::enthalpy::Unpriced;
use crate::{databases, derived};

/// log10 K agreement required for a linking reaction written by two databases.
pub const LOGK_TOLERANCE: f64 = 0.1;
/// Enthalpy agreement required for a linking reaction, kJ/mol.
pub const DELTA_H_TOLERANCE_KJ: f64 = 2.0;

/// The reviewed sources, in the order they are consulted after the routed
/// database itself.
pub const SOURCES: [&str; 5] = ["wateq4f", "minteq.v4", "pitzer", "phreeqc.dat", "llnl.dat"];

/// The extracted `llnl.dat` rows. See the module documentation.
pub const LLNL_FIXTURE: &str = include_str!("../data/llnl-dissolution-references-20261027.json");

/// Basis terms that never need linking and never widen the fixture.
const BASIS: [&str; 4] = ["H+", "H2O", "e-", "OH-"];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Reaction {
    pub lhs: Vec<(String, f64)>,
    pub rhs: Vec<(String, f64)>,
    pub log_k: Option<f64>,
    pub delta_h_kj: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhaseReaction {
    pub phase: String,
    /// The solid as written, e.g. `CuSO4:5H2O`.
    pub formula: String,
    /// Everything else on the left (H+, H2O, ...), not the solid.
    pub lhs: Vec<(String, f64)>,
    pub rhs: Vec<(String, f64)>,
    pub log_k: Option<f64>,
    pub delta_h_kj: Option<f64>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SourceTable {
    pub source: String,
    pub phases: Vec<PhaseReaction>,
    pub species: BTreeMap<String, Reaction>,
}

fn leading_number(term: &str) -> (f64, &str) {
    let end = term
        .char_indices()
        .find(|(_, c)| !(c.is_ascii_digit() || *c == '.'))
        .map_or(term.len(), |(i, _)| i);
    if end == 0 {
        return (1.0, term.trim());
    }
    let rest = term[end..].trim_start();
    match term[..end].parse::<f64>() {
        Ok(n) if !rest.is_empty() && !rest.starts_with(|c: char| c.is_ascii_digit()) => (n, rest),
        _ => (1.0, term.trim()),
    }
}

fn parse_side(side: &str) -> Option<Vec<(String, f64)>> {
    let mut out: Vec<(String, f64)> = Vec::new();
    for term in side.split(" + ") {
        let term = term.trim();
        if term.is_empty() {
            continue;
        }
        let (coefficient, species) = leading_number(term);
        if species.is_empty() || species.contains(char::is_whitespace) {
            return None;
        }
        match out.iter_mut().find(|(s, _)| s == species) {
            Some(entry) => entry.1 += coefficient,
            None => out.push((species.to_string(), coefficient)),
        }
    }
    Some(out)
}

fn parse_delta_h(tokens: &[&str]) -> Option<f64> {
    let value: f64 = tokens.get(1)?.parse().ok()?;
    let unit = tokens
        .get(2)
        .map(|u| u.trim_end_matches("/mol").to_ascii_lowercase());
    match unit.as_deref() {
        None | Some("") | Some("kj") => Some(value),
        Some("kcal") => Some(value * 4.184),
        Some("cal") => Some(value * 4.184e-3),
        Some("j") => Some(value * 1e-3),
        Some(_) => None,
    }
}

/// Every SOLUTION_SPECIES and PHASES reaction of one database file, with the
/// STATED `log_k` and `delta_h` only (an enthalpy recovered from an analytic
/// log K fit is not a reviewed dissolution enthalpy and is not read here).
/// A later definition replaces an earlier one, as in PHREEQC.
pub fn parse_source(source: &str, text: &str) -> SourceTable {
    let mut table = SourceTable {
        source: source.to_string(),
        ..SourceTable::default()
    };
    let mut phases: BTreeMap<String, PhaseReaction> = BTreeMap::new();
    let mut section = "";
    let mut pending_species: Option<String> = None;
    let mut pending_phase: Option<String> = None;
    let mut last_phase: Option<String> = None;
    for raw in text.lines() {
        let line = raw.split('#').next().unwrap_or("");
        if line.trim().is_empty() {
            continue;
        }
        let first = line.split_whitespace().next().unwrap_or("");
        if !line.starts_with([' ', '\t'])
            && first.len() > 3
            && first.chars().all(|c| c.is_ascii_uppercase() || c == '_')
        {
            section = match first {
                "SOLUTION_SPECIES" => "species",
                "PHASES" => "phases",
                _ => "",
            };
            pending_species = None;
            pending_phase = None;
            last_phase = None;
            continue;
        }
        let tokens: Vec<&str> = line.split_whitespace().collect();
        let keyword = tokens
            .first()
            .map(|t| t.trim_start_matches('-').to_ascii_lowercase());
        match section {
            "species" => {
                if let Some((lhs, rhs)) = line.split_once('=') {
                    pending_species = None;
                    let (Some(lhs), Some(rhs)) = (parse_side(lhs), parse_side(rhs)) else {
                        continue;
                    };
                    let Some(name) = rhs.first().map(|(s, _)| s.clone()) else {
                        continue;
                    };
                    table.species.insert(
                        name.clone(),
                        Reaction {
                            lhs,
                            rhs,
                            log_k: None,
                            delta_h_kj: None,
                        },
                    );
                    pending_species = Some(name);
                } else if let Some(name) = &pending_species {
                    let Some(entry) = table.species.get_mut(name) else {
                        continue;
                    };
                    match keyword.as_deref() {
                        Some("log_k") if entry.log_k.is_none() => {
                            entry.log_k = tokens.get(1).and_then(|t| t.parse().ok());
                        }
                        Some("delta_h") if entry.delta_h_kj.is_none() => {
                            entry.delta_h_kj = parse_delta_h(&tokens);
                        }
                        _ => {}
                    }
                }
            }
            "phases" => {
                if !line.starts_with([' ', '\t']) {
                    pending_phase = Some(first.to_string());
                    last_phase = None;
                } else if let Some((lhs, rhs)) = line.split_once('=') {
                    let Some(name) = pending_phase.take() else {
                        continue;
                    };
                    let (Some(lhs), Some(rhs)) = (parse_side(lhs), parse_side(rhs)) else {
                        continue;
                    };
                    let Some(((formula, _), rest)) = lhs.split_first() else {
                        continue;
                    };
                    phases.insert(
                        name.clone(),
                        PhaseReaction {
                            phase: name.clone(),
                            formula: formula.clone(),
                            lhs: rest.to_vec(),
                            rhs,
                            log_k: None,
                            delta_h_kj: None,
                        },
                    );
                    last_phase = Some(name);
                } else if let Some(name) = &last_phase {
                    let Some(entry) = phases.get_mut(name) else {
                        continue;
                    };
                    match keyword.as_deref() {
                        Some("log_k") if entry.log_k.is_none() => {
                            entry.log_k = tokens.get(1).and_then(|t| t.parse().ok());
                        }
                        Some("delta_h") if entry.delta_h_kj.is_none() => {
                            entry.delta_h_kj = parse_delta_h(&tokens);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    table.phases = phases.into_values().collect();
    table
}

/// The registry solid a database phase formula IS, if any.
pub fn registry_solid_of(formula: &str) -> Option<&'static str> {
    let (base, waters) = split_hydrate(formula);
    let composition = parse_formula(&base)?;
    if composition.is_empty() {
        return None;
    }
    derived::registry_solid_for(&composition, waters)
}

fn single_element(formula: &str) -> bool {
    let (base, _) = split_hydrate(formula);
    parse_formula(&base).is_some_and(|c| c.len() == 1)
}

/// The rows of a full table this module can use: phases that are a
/// multi-element registry solid, the species those phases name, and the
/// simple (at most two non-basis species) definitions involving them that
/// a linking check can use. This is exactly what [`LLNL_FIXTURE`] holds.
pub fn reduce_for_fixture(full: &SourceTable) -> SourceTable {
    let phases: Vec<PhaseReaction> = full
        .phases
        .iter()
        .filter(|p| !single_element(&p.formula) && registry_solid_of(&p.formula).is_some())
        .cloned()
        .collect();
    let mut named: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    for p in &phases {
        for (s, _) in p.lhs.iter().chain(&p.rhs) {
            if !BASIS.contains(&s.as_str()) {
                named.insert(s.as_str());
            }
        }
    }
    let species = full
        .species
        .iter()
        .filter(|(name, rxn)| {
            let mut others: Vec<&str> = rxn
                .lhs
                .iter()
                .chain(&rxn.rhs)
                .map(|(s, _)| s.as_str())
                .filter(|s| !BASIS.contains(s))
                .collect();
            others.sort_unstable();
            others.dedup();
            named.contains(name.as_str())
                || (others.len() <= 2 && others.iter().any(|s| named.contains(s)))
        })
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    SourceTable {
        source: full.source.clone(),
        phases,
        species,
    }
}

/// The parsed table of one reviewed source.
pub fn table(source: &str) -> Option<&'static SourceTable> {
    static WATEQ: OnceLock<SourceTable> = OnceLock::new();
    static MINTEQ: OnceLock<SourceTable> = OnceLock::new();
    static PITZER: OnceLock<SourceTable> = OnceLock::new();
    static PHREEQC: OnceLock<SourceTable> = OnceLock::new();
    static LLNL: OnceLock<SourceTable> = OnceLock::new();
    let text = |bytes: &[u8]| String::from_utf8_lossy(bytes).into_owned();
    Some(match source {
        "wateq4f" => WATEQ.get_or_init(|| parse_source(source, &text(databases::wateq4f()))),
        "minteq.v4" => MINTEQ.get_or_init(|| parse_source(source, &text(databases::minteq_v4()))),
        "pitzer" => PITZER.get_or_init(|| parse_source(source, &text(databases::pitzer()))),
        "phreeqc.dat" => PHREEQC.get_or_init(|| parse_source(source, &text(databases::PHREEQC))),
        "llnl.dat" => LLNL.get_or_init(|| {
            serde_json::from_str(LLNL_FIXTURE).expect("the llnl dissolution fixture parses")
        }),
        _ => return None,
    })
}

/// The stablest (lowest log K, then name) phase of one source that IS this
/// solid. Indexed once: the balance asks per moved species per step.
pub fn candidate(key: &str, source: &str) -> Option<&'static PhaseReaction> {
    type Index = BTreeMap<(&'static str, &'static str), &'static PhaseReaction>;
    static INDEX: OnceLock<Index> = OnceLock::new();
    let index = INDEX.get_or_init(|| {
        let mut index: Index = BTreeMap::new();
        for source in SOURCES {
            let Some(t) = table(source) else {
                continue;
            };
            for p in &t.phases {
                let Some(solid) = registry_solid_of(&p.formula) else {
                    continue;
                };
                let better = index.get(&(source, solid)).is_none_or(|q| {
                    p.log_k
                        .unwrap_or(f64::MAX)
                        .total_cmp(&q.log_k.unwrap_or(f64::MAX))
                        .then_with(|| p.phase.cmp(&q.phase))
                        .is_lt()
                });
                if better {
                    index.insert((source, solid), p);
                }
            }
        }
        index
    });
    let source: &'static str = SOURCES.iter().copied().find(|s| *s == source)?;
    let key: &'static str = species::lookup(&SpeciesId::new(key))?.key;
    index.get(&(source, key)).copied()
}

fn net(reaction_lhs: &[(String, f64)], reaction_rhs: &[(String, f64)]) -> BTreeMap<String, f64> {
    let mut map: BTreeMap<String, f64> = BTreeMap::new();
    for (s, n) in reaction_rhs {
        *map.entry(s.clone()).or_default() += n;
    }
    for (s, n) in reaction_lhs {
        *map.entry(s.clone()).or_default() -= n;
    }
    map.retain(|_, n| n.abs() > 1e-12);
    map
}

fn same_map(a: &BTreeMap<String, f64>, b: &BTreeMap<String, f64>, sign: f64) -> bool {
    a.len() == b.len()
        && a.iter()
            .all(|(k, v)| b.get(k).is_some_and(|w| (v - sign * w).abs() < 1e-9))
}

/// A species is a basis species of a table when the table defines it by
/// identity (`Ca+2 = Ca+2`).
fn is_basis(table: &SourceTable, species: &str) -> bool {
    table
        .species
        .get(species)
        .is_some_and(|r| net(&r.lhs, &r.rhs).is_empty())
}

fn agrees(a: &Reaction, b: &Reaction, sign: f64) -> bool {
    let log_k = match (a.log_k, b.log_k) {
        (Some(x), Some(y)) => (x - sign * y).abs() <= LOGK_TOLERANCE,
        _ => false,
    };
    let delta_h = match (a.delta_h_kj, b.delta_h_kj) {
        (Some(x), Some(y)) => (x - sign * y).abs() <= DELTA_H_TOLERANCE_KJ,
        _ => true,
    };
    log_k && delta_h
}

fn written_by(table: &SourceTable, reaction: &Reaction) -> bool {
    let wanted = net(&reaction.lhs, &reaction.rhs);
    table.species.values().any(|other| {
        let have = net(&other.lhs, &other.rhs);
        [1.0, -1.0]
            .into_iter()
            .any(|sign| same_map(&wanted, &have, sign) && agrees(reaction, other, sign))
    })
}

/// Whether a reviewed source's reaction enthalpy may be combined with the
/// routed database's enthalpy for `species`.
///
/// A species that is a BASIS species of the routed database is priced at
/// zero there by definition: the reaction enthalpy then refers to the
/// physical ion itself, and how the source happens to build that ion from
/// its own basis (llnl writes nitrate from NH3 and O2) is irrelevant. A
/// species the routed database prices by its own defining reaction carries
/// that database's compilation, so the source must write the same reaction
/// (in either direction) with log K within [`LOGK_TOLERANCE`] and `delta_h`
/// within [`DELTA_H_TOLERANCE_KJ`], or the two may not be mixed.
pub fn linked(species: &str, source: &SourceTable, routed: &SourceTable) -> bool {
    if BASIS[..2].contains(&species) || is_basis(routed, species) {
        return true;
    }
    if let Some(r) = routed.species.get(species) {
        if written_by(source, r) {
            return true;
        }
    }
    if !is_basis(source, species) {
        if let Some(r) = source.species.get(species) {
            if written_by(routed, r) {
                return true;
            }
        }
    }
    false
}

/// A reviewed reference for one solid, on one routed database.
#[derive(Debug, Clone, PartialEq)]
pub struct Reference {
    pub source: &'static str,
    pub phase: String,
    /// Enthalpy of the solid relative to the routed basis, kJ/mol.
    pub solid_kj: f64,
    /// The dissolution reaction's own enthalpy as written, kJ/mol.
    pub delta_h_kj: f64,
    /// H+ consumed (or OH- released) per mole: > 0 marks a base.
    pub base_equivalents: f64,
    /// Products when the reaction is a pure dissociation (only water on the
    /// left), which is an analytical aqueous equivalent of the dissolved solid.
    pub dissociative_kj: Option<f64>,
}

fn decline(key: &str, why: &'static str) -> Unpriced {
    Unpriced {
        species: key.to_string(),
        why,
    }
}

/// Elemental solids and condensed gases change state by redox
/// (`displacement`) or by phase routing, not by dissolving.
pub fn outside_dissolution(key: &str) -> bool {
    kerotakis_core::displacement::is_elemental_metal(key)
        || kerotakis_core::phase_route::is_condensed_gas(key)
        || species::lookup(&SpeciesId::new(key)).is_some_and(|d| single_element(d.formula))
}

fn atoms(species_name: &str) -> Option<(BTreeMap<String, f64>, f64)> {
    let f = stoich::parse_formula_with(species_name, FormulaDialect::PhreeqcMaster).ok()?;
    Some((f.counts, f.charge))
}

/// The source-side checks that do not depend on a routed basis: a stated,
/// finite, non-placeholder 25 °C enthalpy; no electrons or elemental redox
/// partner; and atom and charge balance over EVERY term, hydrate water
/// included. Returns the reaction enthalpy as written, kJ/mol.
pub fn validate(key: &str, phase: &PhaseReaction) -> Result<f64, Unpriced> {
    let delta_h = phase.delta_h_kj.filter(|h| h.is_finite()).ok_or_else(|| {
        decline(
            key,
            "the reviewed phase states no 25 C dissolution enthalpy",
        )
    })?;
    if delta_h == 0.0 {
        return Err(decline(
            key,
            "the reviewed phase states an exact-zero enthalpy placeholder",
        ));
    }
    let elemental = |s: &str| atoms(s).is_some_and(|(a, c)| c == 0.0 && a.len() == 1);
    if phase
        .lhs
        .iter()
        .chain(&phase.rhs)
        .any(|(s, _)| s == "e-" || elemental(s))
    {
        return Err(decline(
            key,
            "the reviewed phase dissolves by redox, not by dissociation",
        ));
    }
    let registry = species::lookup(&SpeciesId::new(key))
        .ok_or_else(|| decline(key, "no registry formula for this solid"))?;
    let (solid, solid_charge) = atoms(registry.formula)
        .ok_or_else(|| decline(key, "the registry formula cannot be balanced"))?;
    let mut balance = solid;
    let mut charge = solid_charge;
    for (sign, side) in [(1.0, &phase.lhs), (-1.0, &phase.rhs)] {
        for (s, n) in side.iter() {
            let (a, c) = atoms(s)
                .ok_or_else(|| decline(key, "a reaction species has no parseable formula"))?;
            for (e, m) in a {
                *balance.entry(e).or_default() += sign * n * m;
            }
            charge += sign * n * c;
        }
    }
    if charge.abs() > 1e-9 || balance.values().any(|v| v.abs() > 1e-9) {
        return Err(decline(
            key,
            "the reviewed reaction does not balance the registry formula",
        ));
    }
    Ok(delta_h)
}

/// Check one candidate phase against the registry solid and the routed
/// database, and price it on the routed basis.
pub fn check(
    key: &str,
    phase: &PhaseReaction,
    source: &'static str,
    routed: &str,
) -> Result<Reference, Unpriced> {
    let routed_table = table(routed)
        .ok_or_else(|| decline(key, "the routed database is not a reviewed source"))?;
    let source_table = table(source).ok_or_else(|| decline(key, "unknown reviewed source"))?;
    let delta_h = validate(key, phase)?;
    let mut right = 0.0;
    let mut left = 0.0;
    for (sign, side) in [(1.0, &phase.rhs), (-1.0, &phase.lhs)] {
        for (s, n) in side.iter() {
            let h = crate::enthalpy::aqueous_enthalpy(s, routed).ok_or_else(|| {
                decline(
                    key,
                    "a reaction species is not defined on the routed database",
                )
            })?;
            if source != routed && !linked(s, source_table, routed_table) {
                return Err(decline(
                    key,
                    "a product's convention differs between the reviewed source and the routed database",
                ));
            }
            if sign > 0.0 {
                right += n * h;
            } else {
                left += n * h;
            }
        }
    }
    let count = |side: &[(String, f64)], name: &str| -> f64 {
        side.iter().filter(|(s, _)| s == name).map(|(_, n)| n).sum()
    };
    let base_equivalents = count(&phase.lhs, "H+") - count(&phase.rhs, "H+")
        + count(&phase.rhs, "OH-")
        - count(&phase.lhs, "OH-");
    let dissociative = phase.lhs.iter().all(|(s, _)| s == "H2O");
    let solid_kj = right - left - delta_h;
    if !solid_kj.is_finite() {
        return Err(decline(key, "the reviewed reference enthalpy is nonfinite"));
    }
    Ok(Reference {
        source,
        phase: phase.phase.clone(),
        solid_kj,
        delta_h_kj: delta_h,
        base_equivalents,
        dissociative_kj: dissociative.then_some(right - left),
    })
}

/// The routed database first, then the remaining sources in [`SOURCES`] order.
pub fn search_order(routed: &str) -> Vec<&'static str> {
    let mut order: Vec<&'static str> = SOURCES.iter().copied().filter(|s| *s == routed).collect();
    order.extend(SOURCES.iter().copied().filter(|s| *s != routed));
    order
}

/// The first reviewed reference that passes every check.
pub fn reference(key: &str, routed: &str) -> Result<Reference, Unpriced> {
    if outside_dissolution(key) {
        return Err(decline(
            key,
            "an elemental or condensed-gas solid changes state by redox or phase routing, not by dissolving",
        ));
    }
    let mut first: Option<Unpriced> = None;
    for source in search_order(routed) {
        let Some(phase) = candidate(key, source) else {
            continue;
        };
        match check(key, phase, source, routed) {
            Ok(found) => return Ok(found),
            Err(e) => {
                first.get_or_insert(e);
            }
        }
    }
    Err(first.unwrap_or_else(|| {
        decline(
            key,
            "no vendored USGS database writes a dissolution reaction for this solid",
        )
    }))
}

/// A derived hydration `oxide + n H2O = k hydroxide`.
#[derive(Debug, Clone, PartialEq)]
pub struct Hydration {
    pub hydroxide: &'static str,
    pub waters: f64,
    pub hydroxides: f64,
    pub source: &'static str,
    /// kJ per mole of oxide.
    pub delta_h_kj: f64,
}

fn registry_atoms(key: &str) -> Option<BTreeMap<String, f64>> {
    let d = species::lookup(&SpeciesId::new(key))?;
    let (base, waters) = split_hydrate(d.formula);
    if waters != 0.0 || d.standard_phase != Phase::Solid {
        return None;
    }
    parse_formula(&base)
}

/// The registry hydroxide an anhydrous hydrogen-free solid hydrates to, with
/// the stoichiometry, if one exists.
pub fn hydroxide_partner(oxide: &str) -> Option<(&'static str, f64, f64)> {
    let x = registry_atoms(oxide)?;
    if x.contains_key("H") || !x.contains_key("O") || x.len() < 2 {
        return None;
    }
    for candidate in species::REGISTRY.iter().filter(|s| s.key != oxide) {
        let Some(y) = registry_atoms(candidate.key) else {
            continue;
        };
        if !y.contains_key("H") {
            continue;
        }
        for k in 1..=3 {
            let k = f64::from(k);
            let n = y.get("H").copied().unwrap_or(0.0) * k / 2.0;
            let mut expected = x.clone();
            *expected.entry("H".into()).or_default() += 2.0 * n;
            *expected.entry("O".into()).or_default() += n;
            let scaled: BTreeMap<String, f64> = y.iter().map(|(e, c)| (e.clone(), c * k)).collect();
            if n > 0.0
                && expected.len() == scaled.len()
                && expected
                    .iter()
                    .all(|(e, c)| scaled.get(e).is_some_and(|d| (c - d).abs() < 1e-9))
            {
                return Some((candidate.key, n, k));
            }
        }
    }
    None
}

/// The reviewed hydration enthalpy of an oxide: the first source in
/// [`SOURCES`] order (route-independent) whose two stablest phases both pass
/// the balance check with finite, non-placeholder enthalpies and whose
/// reactions differ by exactly `n H2O`.
pub fn hydration(oxide: &str) -> Option<Hydration> {
    let (hydroxide, waters, hydroxides) = hydroxide_partner(oxide)?;
    for source in SOURCES {
        let (Some(x), Some(y)) = (candidate(oxide, source), candidate(hydroxide, source)) else {
            continue;
        };
        // Balance and enthalpy checks are the reference checks, on the
        // source's own basis.
        let (Ok(hx), Ok(hy)) = (validate(oxide, x), validate(hydroxide, y)) else {
            continue;
        };
        let nx = net(&x.lhs, &x.rhs);
        let ny = net(&y.lhs, &y.rhs);
        let mut diff = nx.clone();
        for (s, n) in &ny {
            *diff.entry(s.clone()).or_default() -= hydroxides * n;
        }
        diff.retain(|_, n| n.abs() > 1e-9);
        let only_water =
            diff.len() == 1 && diff.get("H2O").is_some_and(|n| (n + waters).abs() < 1e-9);
        if !only_water {
            continue;
        }
        let h = hx - hydroxides * hy;
        if h.is_finite() {
            return Some(Hydration {
                hydroxide,
                waters,
                hydroxides,
                source,
                delta_h_kj: h,
            });
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_coefficients_attached_or_spaced() {
        assert_eq!(leading_number("2 H+"), (2.0, "H+"));
        assert_eq!(leading_number("2H2O"), (2.0, "H2O"));
        assert_eq!(leading_number("0.5 O2"), (0.5, "O2"));
        assert_eq!(leading_number("H4SiO4"), (1.0, "H4SiO4"));
        let side = parse_side("Ca+2 + 2 H2O").unwrap();
        assert_eq!(
            side,
            vec![("Ca+2".to_string(), 1.0), ("H2O".to_string(), 2.0)]
        );
    }

    #[test]
    fn parses_a_phase_with_left_hand_protons_and_units() {
        let t = parse_source(
            "t",
            "SOLUTION_SPECIES\nCa+2 = Ca+2\n  log_k 0\nPHASES\nLime\n  CaO + 2 H+ = Ca+2 + H2O\n  log_k 32.6\n  -delta_H -46.0 kcal/mol\n",
        );
        let lime = &t.phases[0];
        assert_eq!(lime.formula, "CaO");
        assert_eq!(lime.lhs, vec![("H+".to_string(), 2.0)]);
        assert!((lime.delta_h_kj.unwrap() - (-46.0 * 4.184)).abs() < 1e-9);
        assert!(is_basis(&t, "Ca+2"));
    }
}
