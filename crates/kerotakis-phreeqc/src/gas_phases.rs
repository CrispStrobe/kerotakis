//! Gas-phase reconciliation across the routed databases.
//!
//! Each routed database (wateq4f, minteq.v4, pitzer) ships its own set of
//! gas `PHASES`. A gas one route lacks is imported from a reviewed vendored
//! USGS database (wateq4f.dat, phreeqc.dat, minteq.v4.dat, pitzer.dat — all
//! U.S. Government public domain, see the USGS User Rights Notice in each
//! file) ONLY when every check below passes; otherwise the route keeps the
//! gap and the aqueous adapter refuses a headspace that needs it
//! (`solver-reason.aqueous.gas-phase-unavailable`).
//!
//! Checks, per (gas, route, donor):
//! 1. the donor phase states log K at 25 °C and a temperature dependence
//!    (`delta_h` or `-analytic`): a constant-K import would extrapolate
//!    silently away from 25 °C;
//! 2. every linking aqueous species (non-basis species of the dissolution
//!    reaction other than the gas itself) exists on the route;
//! 3. its formation reaction, expanded recursively to identity-defined
//!    master species, has the SAME stoichiometry on both databases (same
//!    master-species convention), log K within [`LOG_K_TOLERANCE`] and
//!    enthalpy within [`DELTA_H_TOLERANCE_KJ`] at 25 °C;
//! 4. a redox-coupled dissolution (net electrons after expansion) is only
//!    imported where the route speciates every non-H/O element of the gas
//!    in more than one oxidation state.
//!
//! The phase block is copied verbatim; nothing is fitted or hand-copied.
//! `docs/gas-phase-coverage-20261027.md` and
//! `tests/fixtures/gas_phase_coverage.json` are generated from [`coverage`]
//! and pinned by `tests/gas_phase_coverage.rs`.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

/// Maximum |Δ log K| (25 °C) between donor and route for a linking species.
/// The ammonia link that motivated this (NH4+ = NH3 + H+: wateq4f -9.2442
/// via its analytic expression, minteq.v4 -9.244) differs by 2e-4.
pub const LOG_K_TOLERANCE: f64 = 0.02;
/// Maximum |Δ ΔH| (kJ/mol, 25 °C) between donor and route linking species.
pub const DELTA_H_TOLERANCE_KJ: f64 = 4.0;
/// Routes in the order the coverage table lists them.
pub const ROUTES: [&str; 3] = ["wateq4f", "minteq.v4", "pitzer"];
/// Donors in priority order: the first passing donor is imported.
pub const DONORS: [&str; 4] = ["wateq4f", "phreeqc", "minteq.v4", "pitzer"];

const T: f64 = 298.15;
const R: f64 = 8.314_462_618_153_24;
const BASIS: [&str; 3] = ["H2O", "H+", "e-"];

#[derive(Debug, Clone, Default)]
struct Entry {
    /// Species → coefficient; products positive, reactants negative. For a
    /// phase the gas formula itself is excluded.
    reaction: BTreeMap<String, f64>,
    formula: String,
    log_k: Option<f64>,
    delta_h: Option<f64>,
    analytic: Option<(f64, Option<f64>)>,
}

impl Entry {
    fn lk(&self) -> Option<f64> {
        self.analytic.map(|(lk, _)| lk).or(self.log_k)
    }
    fn h(&self) -> Option<f64> {
        match self.analytic {
            Some((_, h)) => h,
            None => self.delta_h,
        }
    }
    fn attr(&mut self, line: &str) {
        for part in line.split(';') {
            let tokens: Vec<&str> = part.split_whitespace().collect();
            let Some(first) = tokens.first() else {
                continue;
            };
            let key = first.trim_start_matches('-').to_ascii_lowercase();
            match key.as_str() {
                "log_k" | "logk" => {
                    if let Some(v) = tokens.get(1).and_then(|t| t.parse::<f64>().ok()) {
                        self.log_k = Some(v);
                    }
                }
                "delta_h" | "deltah" => self.delta_h = delta_h(&tokens),
                k if k.starts_with("analytic") || k.starts_with("a_e") => {
                    if let Some(a) = analytic(&tokens) {
                        self.analytic = Some(a);
                    }
                }
                _ => {}
            }
        }
    }
}

fn delta_h(tokens: &[&str]) -> Option<f64> {
    let v: f64 = tokens.get(1)?.parse().ok()?;
    let unit = tokens
        .get(2)
        .map(|u| u.trim_end_matches("/mol").to_ascii_lowercase())
        .unwrap_or_default();
    match unit.as_str() {
        "" | "kj" => Some(v),
        "kcal" => Some(v * 4.184),
        "cal" => Some(v * 4.184e-3),
        "j" => Some(v * 1e-3),
        _ => None,
    }
}

/// log K and ΔH (kJ/mol) at 25 °C from a PHREEQC analytic expression.
fn analytic(tokens: &[&str]) -> Option<(f64, Option<f64>)> {
    let mut a = [0.0f64; 6];
    let mut seen = 0;
    for (i, slot) in a.iter_mut().enumerate() {
        match tokens.get(i + 1).and_then(|t| t.parse::<f64>().ok()) {
            Some(v) => {
                *slot = v;
                seen += 1;
            }
            None => break,
        }
    }
    if seen == 0 {
        return None;
    }
    let lk = a[0] + a[1] * T + a[2] / T + a[3] * T.log10() + a[4] / (T * T) + a[5] * T * T;
    let h = (seen >= 2).then(|| {
        let ln10 = std::f64::consts::LN_10;
        let d =
            a[1] - a[2] / (T * T) + a[3] / (T * ln10) - 2.0 * a[4] / (T * T * T) + 2.0 * a[5] * T;
        R * T * T * ln10 * d / 1000.0
    });
    Some((lk, h))
}

/// `2 H+ + CO3-2` → {H+: 2, CO3-2: 1}, preserving first-term order.
fn parse_terms(side: &str) -> Vec<(String, f64)> {
    let mut out: Vec<(String, f64)> = Vec::new();
    let padded = format!(" {} ", side.trim());
    let mut terms: Vec<&str> = Vec::new();
    let mut rest = padded.as_str();
    // Terms are separated by a '+' with whitespace on both sides; a bare
    // '+' belongs to a charge (Na+).
    loop {
        let bytes = rest.as_bytes();
        let mut cut = None;
        for i in 1..bytes.len().saturating_sub(1) {
            if bytes[i] == b'+'
                && bytes[i - 1].is_ascii_whitespace()
                && bytes[i + 1].is_ascii_whitespace()
            {
                cut = Some(i);
                break;
            }
        }
        match cut {
            Some(i) => {
                terms.push(&rest[..i]);
                rest = &rest[i + 1..];
            }
            None => {
                terms.push(rest);
                break;
            }
        }
    }
    for term in terms {
        let parts: Vec<&str> = term.split_whitespace().collect();
        let (c, s) = match parts.as_slice() {
            [] => continue,
            [one] => {
                let digits = one
                    .find(|ch: char| !(ch.is_ascii_digit() || ch == '.'))
                    .unwrap_or(0);
                match one[..digits].parse::<f64>() {
                    Ok(c) if digits > 0 => (c, one[digits..].to_string()),
                    _ => (1.0, one.to_string()),
                }
            }
            [c, s, ..] => match c.parse::<f64>() {
                Ok(c) => (c, s.to_string()),
                Err(_) => (1.0, c.to_string()),
            },
        };
        match out.iter_mut().find(|(name, _)| *name == s) {
            Some(e) => e.1 += c,
            None => out.push((s, c)),
        }
    }
    out
}

fn is_keyword(line: &str) -> bool {
    if line.starts_with([' ', '\t']) {
        return false;
    }
    let first = line.split_whitespace().next().unwrap_or("");
    first.len() > 3 && first.chars().all(|c| c.is_ascii_uppercase() || c == '_')
}

#[derive(Debug, Default)]
struct RawDb {
    species: BTreeMap<String, Entry>,
    phases: BTreeMap<String, Entry>,
    blocks: BTreeMap<String, Vec<String>>,
    masters: BTreeSet<String>,
}

impl RawDb {
    fn parse(db: &[u8]) -> RawDb {
        // Latin-1 decoding, as PHREEQC is encoding-agnostic.
        let text: String = db.iter().map(|&b| b as char).collect();
        let mut out = RawDb::default();
        let mut section = String::new();
        let mut current: Option<Entry> = None;
        let mut current_species: Option<String> = None;
        let mut phase: Option<String> = None;
        let flush_species =
            |out: &mut RawDb, name: &mut Option<String>, entry: &mut Option<Entry>| {
                if let (Some(n), Some(e)) = (name.take(), entry.take()) {
                    out.species.insert(n, e);
                }
            };
        for raw in text.split('\n') {
            let raw = raw.trim_end_matches('\r');
            let line = raw.split('#').next().unwrap_or("");
            if line.trim().is_empty() {
                if section == "PHASES" {
                    if let Some(name) = &phase {
                        out.blocks.get_mut(name).unwrap().push(raw.to_string());
                    }
                }
                continue;
            }
            if is_keyword(line) {
                flush_species(&mut out, &mut current_species, &mut current);
                if let (Some(name), Some(e)) = (phase.take(), current.take()) {
                    out.phases.insert(name, e);
                }
                section = line.split_whitespace().next().unwrap_or("").to_string();
                continue;
            }
            match section.as_str() {
                "SOLUTION_MASTER_SPECIES" => {
                    if let Some(el) = line.split_whitespace().next() {
                        out.masters.insert(el.to_string());
                    }
                }
                "SOLUTION_SPECIES" => {
                    if let Some((lhs, rhs)) = line.split_once('=') {
                        flush_species(&mut out, &mut current_species, &mut current);
                        let products = parse_terms(rhs);
                        let Some((first, _)) = products.first().cloned() else {
                            continue;
                        };
                        let mut reaction: BTreeMap<String, f64> = BTreeMap::new();
                        for (s, c) in products {
                            *reaction.entry(s).or_default() += c;
                        }
                        for (s, c) in parse_terms(lhs) {
                            *reaction.entry(s).or_default() -= c;
                        }
                        current = Some(Entry {
                            reaction,
                            ..Entry::default()
                        });
                        current_species = Some(first);
                    } else if let Some(e) = current.as_mut() {
                        e.attr(line);
                    }
                }
                "PHASES" => {
                    if !line.starts_with([' ', '\t']) {
                        if let (Some(name), Some(e)) = (phase.take(), current.take()) {
                            out.phases.insert(name, e);
                        }
                        let name = line.split_whitespace().next().unwrap().to_string();
                        out.blocks.insert(name.clone(), vec![raw.to_string()]);
                        phase = Some(name);
                        current = None;
                    } else if let Some(name) = &phase {
                        out.blocks.get_mut(name).unwrap().push(raw.to_string());
                        match current.as_mut() {
                            None => {
                                if let Some((lhs, rhs)) = line.split_once('=') {
                                    let left = parse_terms(lhs);
                                    let formula =
                                        left.first().map(|(s, _)| s.clone()).unwrap_or_default();
                                    let mut reaction: BTreeMap<String, f64> = BTreeMap::new();
                                    for (s, c) in parse_terms(rhs) {
                                        *reaction.entry(s).or_default() += c;
                                    }
                                    for (s, c) in left.into_iter().skip(1) {
                                        *reaction.entry(s).or_default() -= c;
                                    }
                                    current = Some(Entry {
                                        reaction,
                                        formula,
                                        ..Entry::default()
                                    });
                                }
                            }
                            Some(e) => e.attr(line),
                        }
                    }
                }
                _ => {}
            }
        }
        flush_species(&mut out, &mut current_species, &mut current);
        if let (Some(name), Some(e)) = (phase.take(), current.take()) {
            out.phases.insert(name, e);
        }
        for block in out.blocks.values_mut() {
            while block.last().is_some_and(|l| l.trim().is_empty()) {
                block.pop();
            }
        }
        out
    }

    /// Formation of `species` from identity-defined species: (stoichiometry,
    /// log K, ΔH kJ/mol) at 25 °C.
    fn expand(
        &self,
        species: &str,
        depth: usize,
    ) -> Option<(BTreeMap<String, f64>, f64, Option<f64>)> {
        if BASIS.contains(&species) {
            return Some((BTreeMap::from([(species.to_string(), 1.0)]), 0.0, Some(0.0)));
        }
        let e = self.species.get(species)?;
        if depth > 12 {
            return None;
        }
        let coef = e.reaction.get(species).copied().unwrap_or(0.0);
        if coef.abs() < 1e-12 {
            // `X = X`: a master species.
            return Some((BTreeMap::from([(species.to_string(), 1.0)]), 0.0, Some(0.0)));
        }
        let mut num = e.lk()?;
        let mut h = e.h();
        let mut total: BTreeMap<String, f64> = BTreeMap::new();
        for (s, c) in &e.reaction {
            if s == species {
                continue;
            }
            let (m, slk, sh) = self.expand(s, depth + 1)?;
            for (k, v) in m {
                *total.entry(k).or_default() -= c * v / coef;
            }
            num -= c * slk;
            h = match (h, sh) {
                (Some(h), Some(sh)) => Some(h - c * sh),
                _ => None,
            };
        }
        total.retain(|_, v| v.abs() > 1e-12);
        Some((total, num / coef, h.map(|h| h / coef)))
    }

    fn redox_elements(&self) -> BTreeSet<String> {
        let mut n: BTreeMap<String, usize> = BTreeMap::new();
        for el in &self.masters {
            if el.contains('(') && el.contains(')') {
                *n.entry(el.split('(').next().unwrap().to_string())
                    .or_default() += 1;
            }
        }
        n.into_iter()
            .filter(|(_, c)| *c >= 2)
            .map(|(b, _)| b)
            .collect()
    }
}

/// Why a gas could not be imported from a donor onto a route.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GasRefusal {
    NoReviewedDonorDefinition,
    DonorPhaseWithoutLogK,
    DonorPhaseWithoutTemperatureDependence,
    DonorLinkingSpeciesUnresolved,
    NoLinkingAqueousSpecies,
    LinkingSpeciesAbsent,
    LinkingSpeciesConventionDiffers,
    LinkingLogKOutsideTolerance,
    LinkingEnthalpyUnstated,
    LinkingEnthalpyOutsideTolerance,
    RedoxStateNotModelledOnRoute,
}

impl GasRefusal {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NoReviewedDonorDefinition => "no-reviewed-donor-definition",
            Self::DonorPhaseWithoutLogK => "donor-phase-without-log-k",
            Self::DonorPhaseWithoutTemperatureDependence => {
                "donor-phase-without-temperature-dependence"
            }
            Self::DonorLinkingSpeciesUnresolved => "donor-linking-species-unresolved",
            Self::NoLinkingAqueousSpecies => "no-linking-aqueous-species",
            Self::LinkingSpeciesAbsent => "linking-species-absent",
            Self::LinkingSpeciesConventionDiffers => "linking-species-convention-differs",
            Self::LinkingLogKOutsideTolerance => "linking-log-k-outside-tolerance",
            Self::LinkingEnthalpyUnstated => "linking-enthalpy-unstated",
            Self::LinkingEnthalpyOutsideTolerance => "linking-enthalpy-outside-tolerance",
            Self::RedoxStateNotModelledOnRoute => "redox-state-not-modelled-on-route",
        }
    }
}

/// One linking aqueous species compared between donor and route.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Link {
    pub species: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub log_k_delta: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delta_h_delta_kj: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Attempt {
    pub donor: String,
    pub refusal: Option<GasRefusal>,
    pub links: Vec<Link>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GasStatus {
    Native,
    Imported,
    Refused,
}

/// One cell of the coverage table.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CoverageRow {
    pub gas: String,
    pub route: String,
    pub status: GasStatus,
    pub attempts: Vec<Attempt>,
}

fn check(donor: &RawDb, target: &RawDb, gas: &str) -> (Option<GasRefusal>, Vec<Link>) {
    let p = &donor.phases[gas];
    let mut links = Vec::new();
    if p.lk().is_none() {
        return (Some(GasRefusal::DonorPhaseWithoutLogK), links);
    }
    if p.h().is_none() {
        return (
            Some(GasRefusal::DonorPhaseWithoutTemperatureDependence),
            links,
        );
    }
    let mut electrons = p.reaction.get("e-").copied().unwrap_or(0.0);
    for (s, c) in &p.reaction {
        if BASIS.contains(&s.as_str()) {
            continue;
        }
        let Some(d) = donor.expand(s, 0) else {
            return (Some(GasRefusal::DonorLinkingSpeciesUnresolved), links);
        };
        electrons += c * d.0.get("e-").copied().unwrap_or(0.0);
        let Some(t) = target.expand(s, 0) else {
            links.push(Link {
                species: s.clone(),
                log_k_delta: None,
                delta_h_delta_kj: None,
            });
            return (Some(GasRefusal::LinkingSpeciesAbsent), links);
        };
        let keys: BTreeSet<&String> = d.0.keys().chain(t.0.keys()).collect();
        let same = keys.iter().all(|k| {
            (d.0.get(*k).copied().unwrap_or(0.0) - t.0.get(*k).copied().unwrap_or(0.0)).abs() < 1e-9
        });
        if !same {
            links.push(Link {
                species: s.clone(),
                log_k_delta: None,
                delta_h_delta_kj: None,
            });
            return (Some(GasRefusal::LinkingSpeciesConventionDiffers), links);
        }
        let dlk = (d.1 - t.1).abs();
        links.push(Link {
            species: s.clone(),
            log_k_delta: Some(dlk),
            delta_h_delta_kj: None,
        });
        if dlk > LOG_K_TOLERANCE {
            return (Some(GasRefusal::LinkingLogKOutsideTolerance), links);
        }
        let (Some(dh), Some(th)) = (d.2, t.2) else {
            return (Some(GasRefusal::LinkingEnthalpyUnstated), links);
        };
        let dh = (dh - th).abs();
        links.last_mut().unwrap().delta_h_delta_kj = Some(dh);
        if dh > DELTA_H_TOLERANCE_KJ {
            return (Some(GasRefusal::LinkingEnthalpyOutsideTolerance), links);
        }
    }
    if links.is_empty() {
        return (Some(GasRefusal::NoLinkingAqueousSpecies), links);
    }
    if electrons.abs() > 1e-9 {
        let redox = target.redox_elements();
        let unmodelled = crate::dbindex::parse_formula(&p.formula)
            .map(|f| {
                f.keys()
                    .filter(|el| *el != "H" && *el != "O")
                    .any(|el| !redox.contains(el.as_str()))
            })
            .unwrap_or(true);
        if unmodelled {
            return (Some(GasRefusal::RedoxStateNotModelledOnRoute), links);
        }
    }
    (None, links)
}

fn donor_bytes(name: &str) -> &'static [u8] {
    use crate::databases;
    match name {
        "wateq4f" => databases::WATEQ4F,
        "phreeqc" => databases::PHREEQC,
        "minteq.v4" => databases::MINTEQ_V4,
        _ => databases::PITZER,
    }
}

fn donors() -> &'static [(&'static str, RawDb)] {
    static D: OnceLock<Vec<(&'static str, RawDb)>> = OnceLock::new();
    D.get_or_init(|| {
        DONORS
            .iter()
            .map(|n| (*n, RawDb::parse(donor_bytes(n))))
            .collect()
    })
}

/// Bench gases: every registry gas species (by formula), the reviewed
/// exchange capabilities and NH3(g), whose aqueous form is a solute.
fn bench_gases() -> BTreeSet<String> {
    let mut out: BTreeSet<String> = kerotakis_core::species::REGISTRY
        .iter()
        .filter(|s| s.standard_phase == kerotakis_core::species::Phase::Gas)
        .map(|s| format!("{}(g)", s.formula))
        .collect();
    out.extend(
        crate::derived::EQUILIBRIUM_GASES
            .iter()
            .map(|(p, _)| p.to_string()),
    );
    out.insert("NH3(g)".to_string());
    out
}

fn decide(route: &str, target: &RawDb, gas: &str) -> CoverageRow {
    if target.phases.contains_key(gas) {
        return CoverageRow {
            gas: gas.to_string(),
            route: route.to_string(),
            status: GasStatus::Native,
            attempts: vec![],
        };
    }
    let mut attempts = Vec::new();
    for (name, donor) in donors() {
        if *name == route || !donor.phases.contains_key(gas) {
            continue;
        }
        let (refusal, links) = check(donor, target, gas);
        attempts.push(Attempt {
            donor: name.to_string(),
            refusal,
            links,
        });
        if refusal.is_none() {
            return CoverageRow {
                gas: gas.to_string(),
                route: route.to_string(),
                status: GasStatus::Imported,
                attempts,
            };
        }
    }
    if attempts.is_empty() {
        attempts.push(Attempt {
            donor: String::new(),
            refusal: Some(GasRefusal::NoReviewedDonorDefinition),
            links: vec![],
        });
    }
    CoverageRow {
        gas: gas.to_string(),
        route: route.to_string(),
        status: GasStatus::Refused,
        attempts,
    }
}

fn candidates(targets: &[&RawDb]) -> BTreeSet<String> {
    let mut all = bench_gases();
    for (_, d) in donors() {
        all.extend(d.phases.keys().filter(|k| k.ends_with("(g)")).cloned());
    }
    for t in targets {
        all.extend(t.phases.keys().filter(|k| k.ends_with("(g)")).cloned());
    }
    all
}

/// The route database with every passing gas import inserted before its
/// final `END`. `base` is the route as built without gas reconciliation.
pub fn reconcile(route: &str, base: &[u8]) -> Vec<u8> {
    let target = RawDb::parse(base);
    let mut block = String::new();
    for gas in candidates(&[&target]) {
        let row = decide(route, &target, &gas);
        if row.status != GasStatus::Imported {
            continue;
        }
        let donor = &row.attempts.last().unwrap().donor;
        let (_, d) = donors().iter().find(|(n, _)| n == donor).unwrap();
        block.push_str(&format!(
            "# {gas}: imported verbatim from vendored USGS {donor}.dat by gas_phases::reconcile\n"
        ));
        for line in &d.blocks[&gas] {
            block.push_str(line);
            block.push('\n');
        }
    }
    if block.is_empty() {
        return base.to_vec();
    }
    let at = crate::databases::find_last_end(base).unwrap_or(base.len());
    let mut bytes = Vec::with_capacity(base.len() + block.len() + 16);
    bytes.extend_from_slice(&base[..at]);
    bytes.extend_from_slice(b"\nPHASES\n");
    bytes.extend_from_slice(block.as_bytes());
    bytes.extend_from_slice(&base[at..]);
    bytes
}

/// The full (gas × route) table: every gas defined by any routed or donor
/// database and every gas the bench can hold.
pub fn coverage() -> Vec<CoverageRow> {
    let targets: Vec<(&str, RawDb)> = ROUTES
        .iter()
        .map(|r| (*r, RawDb::parse(crate::databases::route_base(r))))
        .collect();
    let refs: Vec<&RawDb> = targets.iter().map(|(_, t)| t).collect();
    let mut rows = Vec::new();
    for gas in candidates(&refs) {
        for (route, t) in &targets {
            rows.push(decide(route, t, &gas));
        }
    }
    rows
}

/// Finite headspace ↔ aqueous acid/base coupling: which supplied gas the
/// aqueous adapter poses to the native solve (and so equilibrates with
/// protonation in the same solve) versus carries as inert inventory.
pub fn headspace_coupled(phase: &str) -> bool {
    phase == "NH3(g)"
        || crate::derived::EQUILIBRIUM_GASES
            .iter()
            .any(|(p, _)| *p == phase)
}

/// The volatile acids/bases audited for headspace coupling (#834 siblings).
pub const VOLATILE_ACID_BASE: [&str; 7] = [
    "NH3(g)", "CO2(g)", "HBr(g)", "H2S(g)", "SO2(g)", "HCl(g)", "Cl2(g)",
];

/// Markdown table for docs/gas-phase-coverage-20261027.md.
pub fn markdown_table(rows: &[CoverageRow]) -> String {
    let mut out = String::from("| gas | wateq4f | minteq.v4 | pitzer |\n|---|---|---|---|\n");
    let gases: BTreeSet<&str> = rows.iter().map(|r| r.gas.as_str()).collect();
    for gas in gases {
        out.push_str(&format!("| `{gas}` |"));
        for route in ROUTES {
            let row = rows
                .iter()
                .find(|r| r.gas == gas && r.route == route)
                .unwrap();
            let cell = match row.status {
                GasStatus::Native => "native".to_string(),
                GasStatus::Imported => {
                    format!("**imported** from {}", row.attempts.last().unwrap().donor)
                }
                GasStatus::Refused => {
                    let parts: Vec<String> = row
                        .attempts
                        .iter()
                        .map(|a| {
                            let why = a.refusal.map(GasRefusal::as_str).unwrap_or("");
                            let what = a
                                .links
                                .last()
                                .map(|l| format!(" `{}`", l.species))
                                .unwrap_or_default();
                            if a.donor.is_empty() {
                                why.to_string()
                            } else {
                                format!("{}: {why}{what}", a.donor)
                            }
                        })
                        .collect();
                    format!("refused ({})", parts.join("; "))
                }
            };
            out.push_str(&format!(" {cell} |"));
        }
        out.push('\n');
    }
    out
}

/// Markdown table of headspace ↔ acid/base coupling per route.
pub fn coupling_table(rows: &[CoverageRow]) -> String {
    let mut out = String::from(
        "| gas | coupled to aqueous acid/base in finite headspace | wateq4f | minteq.v4 | pitzer |\n|---|---|---|---|---|\n",
    );
    for gas in VOLATILE_ACID_BASE {
        let coupled = headspace_coupled(gas);
        out.push_str(&format!(
            "| `{gas}` | {} |",
            if coupled {
                "yes"
            } else {
                "no (inert inventory)"
            }
        ));
        for route in ROUTES {
            let defined = rows
                .iter()
                .find(|r| r.gas == gas && r.route == route)
                .map(|r| r.status != GasStatus::Refused)
                .unwrap_or(false);
            let cell = match (coupled, defined) {
                (true, true) => "coupled",
                (true, false) => "refused (gas-phase-unavailable)",
                (false, true) => "phase defined, not posed",
                (false, false) => "no phase",
            };
            out.push_str(&format!(" {cell} |"));
        }
        out.push('\n');
    }
    out
}

/// The import checks on arbitrary database bytes (for boundary tests).
pub fn check_import(donor: &[u8], target: &[u8], gas: &str) -> (Option<GasRefusal>, Vec<Link>) {
    let donor = RawDb::parse(donor);
    let target = RawDb::parse(target);
    if !donor.phases.contains_key(gas) {
        return (Some(GasRefusal::NoReviewedDonorDefinition), vec![]);
    }
    check(&donor, &target, gas)
}
