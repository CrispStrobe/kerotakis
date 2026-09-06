//! The net ionic equation, derived from the solved speciation (GUI-092).
//!
//! A molecular equation is a bookkeeping statement about bottles; the net
//! ionic one is a statement about what actually happened in the beaker, and
//! it is the only one of the two a bench can *compute*. Silver nitrate met
//! by table salt is not `AgNO3 + NaCl → AgCl + NaNO3` in any physical
//! sense — there is no silver nitrate in the beaker to react. There are
//! four ions, two of which find each other. Which two is not something to
//! memorise per reaction: it falls out of the speciation the solver
//! reports, and so does the fact that the other two are still floating
//! there afterwards, unchanged, which is the whole content of the word
//! "spectator".
//!
//! # How it is derived
//!
//! For a precipitation the engine hands us the solid that formed and the
//! full dissolved species distribution. From those:
//!
//! 1. The solid's formula is parsed into element counts and charge.
//! 2. For each element of the solid other than hydrogen and oxygen — the
//!    two the solvent itself supplies — the *most abundant dissolved
//!    species carrying that element* is chosen, provided it brings in no
//!    element the solid does not contain. That is the partner: not a
//!    lookup, the solver's own answer about what that element is in
//!    solution *as*.
//! 3. Water and whichever of H⁺/OH⁻ the solution actually holds more of
//!    are offered as balancing partners, so a hydroxide precipitating out
//!    of a basic solution is written with hydroxide and a carbonate out of
//!    a near-neutral one is written releasing a proton — which is what the
//!    chemistry does.
//! 4. The coefficients are solved as a small linear system over the
//!    elements *and* the charge, then verified. Nothing that fails to
//!    balance is shown.
//!
//! For a neutralisation the engine already computes the extent — the
//! amount of the solutes' unspent acidity that cancelled — and previously
//! spent it on heat and threw the number away. `H⁺ + OH⁻ → H₂O` is not a
//! stored equation here either: the two ions are read off the same
//! speciation, and if the solver never characterised the solution there is
//! nothing to say.
//!
//! # What it will not do
//!
//! Return a guess. Where the vessel carries no speciation, where a partner
//! cannot be chosen from the species actually present, or where the
//! coefficients do not balance exactly, the answer is `None` and the
//! reader is shown nothing rather than a plausible-looking equation. The
//! honest scope today is precipitation and neutralisation, the two cases
//! where the participants are knowable; redox and organic steps carry no
//! participant list yet and are not guessed at.

use serde::{Deserialize, Serialize};

use crate::ops::Event;
use crate::species::{self, Phase, SpeciesId};
use crate::stoich::{self, Formula, FormulaDialect};
use crate::vessel::{SpeciesDetail, Vessel, VesselId};

/// The only floor applied to *participants* is the solver's own.
///
/// A floor of our own was the first thing tried and it was wrong in the
/// exact case this module exists for: a precipitation leaves its ions
/// depleted by definition. Silver chloride dropping out of brine leaves
/// about 7 nmol/kgw of free silver — below any "surely that is a trace"
/// threshold, and the whole reason there is a solid in the beaker. So
/// anything the solver bothered to report is a candidate, and the
/// judgement about what is worth *naming* is applied to spectators only,
/// where it is a question about a sentence rather than about chemistry.
const TRACE_MOLALITY: f64 = 0.0;

/// A spectator has to be *there* to be worth naming. One percent of the
/// most abundant solute keeps the sentence to the ions a learner poured in
/// rather than the tail of minor complexes below them.
const SPECTATOR_FRACTION: f64 = 0.01;

/// How many spectator ions are worth naming before the list stops being a
/// sentence.
const MAX_SPECTATORS: usize = 6;

/// One term of a net ionic equation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IonTerm {
    /// The species as the engine names it: `Ag+`, `SO4-2`, `H2O`, `AgCl`.
    pub species: String,
    /// The same thing typeset for a reader: `Ag⁺`, `SO₄²⁻`, `H₂O`.
    pub label: String,
    /// Stoichiometric coefficient. Always ≥ 1; a term that cancelled is
    /// not carried.
    pub coefficient: u32,
    /// Net charge on one formula unit.
    pub charge: i32,
    pub phase: Phase,
}

impl IonTerm {
    fn new(species: &str, coefficient: u32, charge: i32, phase: Phase) -> IonTerm {
        IonTerm {
            species: species.to_string(),
            label: typeset(species),
            coefficient,
            charge,
            phase,
        }
    }

    /// `2 OH⁻(aq)` — the term as it appears in the written equation.
    pub fn written(&self) -> String {
        let coefficient = if self.coefficient == 1 {
            String::new()
        } else {
            format!("{} ", self.coefficient)
        };
        format!("{coefficient}{}{}", self.label, phase_suffix(self.phase))
    }
}

/// Which computed fact the equation was derived from. Not a reaction
/// *class* guess: each variant names an engine result that carries its own
/// participants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IonicBasis {
    /// A solid the aqueous solver brought out of solution.
    Precipitation,
    /// Unspent acidity that cancelled when its opposite arrived.
    Neutralisation,
}

/// A net ionic equation and the solution it was read out of.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NetIonic {
    pub vessel: VesselId,
    pub basis: IonicBasis,
    pub reactants: Vec<IonTerm>,
    pub products: Vec<IonTerm>,
    /// Ions the solver left in solution, taking no part. Empty is a real
    /// answer — there is nothing spectating in a neutralisation of pure
    /// acid by pure base.
    pub spectators: Vec<IonTerm>,
    /// The equation as one line: `Ag⁺(aq) + Cl⁻(aq) → AgCl(s)`.
    pub equation: String,
    /// The solver whose speciation this was read out of, where the vessel
    /// records one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance: Option<String>,
}

impl NetIonic {
    fn assemble(
        vessel: VesselId,
        basis: IonicBasis,
        reactants: Vec<IonTerm>,
        products: Vec<IonTerm>,
        spectators: Vec<IonTerm>,
        provenance: Option<String>,
    ) -> NetIonic {
        let side = |terms: &[IonTerm]| {
            terms
                .iter()
                .map(IonTerm::written)
                .collect::<Vec<_>>()
                .join(" + ")
        };
        let equation = format!("{} → {}", side(&reactants), side(&products));
        NetIonic {
            vessel,
            basis,
            reactants,
            products,
            spectators,
            equation,
            provenance,
        }
    }

    /// The spectator ions as one phrase: `Na⁺, NO₃⁻`. `None` when the
    /// solver left nothing beside the reaction.
    pub fn spectator_phrase(&self) -> Option<String> {
        (!self.spectators.is_empty()).then(|| {
            self.spectators
                .iter()
                .map(|t| t.label.clone())
                .collect::<Vec<_>>()
                .join(", ")
        })
    }
}

/// The net ionic equations a step's events earn — the producer side, shared
/// by every host so the CLI, the wasm bench and the shell say the identical
/// thing about the identical chemistry (the shape `chart::charts_for_events`
/// established).
///
/// Empty is the common case and an honest one: most operators are not a
/// reaction between ions.
pub fn net_ionic_for(events: &[Event], vessels: &[Vessel]) -> Vec<NetIonic> {
    let mut out: Vec<NetIonic> = Vec::new();
    for event in events.iter().filter(|e| e.is_observable()) {
        let Some(vessel_id) = subject_vessel(event) else {
            continue;
        };
        let Some(vessel) = vessels.iter().find(|v| v.id == vessel_id) else {
            continue;
        };
        if let Some(net) = net_ionic(event, vessel) {
            if !out.iter().any(|prior| prior.equation == net.equation) {
                out.push(net);
            }
        }
    }
    out
}

/// The net ionic equation one event implies in one vessel, or `None` where
/// the engine does not know enough to write one.
pub fn net_ionic(event: &Event, vessel: &Vessel) -> Option<NetIonic> {
    match event {
        Event::Precipitated { species, .. } => from_precipitate(vessel, species),
        Event::Neutralised { .. } => from_neutralisation(vessel),
        _ => None,
    }
}

fn subject_vessel(event: &Event) -> Option<VesselId> {
    match event {
        Event::Precipitated { vessel, .. } | Event::Neutralised { vessel, .. } => Some(*vessel),
        _ => None,
    }
}

// ── precipitation ───────────────────────────────────────────────────

fn from_precipitate(vessel: &Vessel, solid_id: &SpeciesId) -> Option<NetIonic> {
    let solid_formula = species::lookup(solid_id)
        .and_then(|d| stoich::parse_formula(d.formula).ok())
        .or_else(|| stoich::parse_formula(&solid_id.0).ok())?;
    let dissolved = dissolved_species(vessel)?;

    // Hydrogen and oxygen are the solvent's to supply, so they do not
    // select a partner; every other element of the solid must have come
    // from something the solver says is in solution.
    let skeleton: Vec<String> = solid_formula
        .counts
        .keys()
        .filter(|e| *e != "H" && *e != "O")
        .cloned()
        .collect();
    if skeleton.is_empty() {
        return None;
    }

    let mut participants: Vec<(String, Formula)> = Vec::new();
    for element in &skeleton {
        let (detail, formula) = dissolved
            .iter()
            // It carries the element…
            .filter(|(_, f)| f.counts.contains_key(element))
            // …and brings in nothing the solid does not contain. A
            // chloro-complex cannot be the source of the silver in a
            // sulfate.
            .filter(|(_, f)| {
                f.counts
                    .keys()
                    .all(|k| k == "H" || k == "O" || solid_formula.counts.contains_key(k))
            })
            // Charge first, then abundance. This is the *ionic* equation:
            // it is written in terms of the ions the solution holds, and
            // the ordering is a rule rather than a tie-break. In silver
            // chloride the neutral AgCl(aq) complex is the more abundant
            // silver species by two decades, and taking it would reduce
            // the whole thing to `AgCl(aq) → AgCl(s)`: true, and not what
            // anyone came to see. That the complex is nonetheless there is
            // worth showing one day — beside the free ions, not instead.
            .max_by(|a, b| {
                let ion = |f: &Formula| f.charge != 0.0;
                ion(&a.1)
                    .cmp(&ion(&b.1))
                    .then_with(|| a.0.molality.total_cmp(&b.0.molality))
            })?;
        if !participants.iter().any(|(name, _)| name == &detail.name) {
            participants.push((detail.name.clone(), formula.clone()));
        }
    }

    // Balancing partners: water, and whichever end of the water axis this
    // solution actually holds more of. Both are read off the speciation —
    // a solution the solver never reported H⁺ or OH⁻ for gets water alone,
    // and if that cannot close the equation nothing is shown.
    let mut balancers: Vec<(String, Formula)> = Vec::new();
    if let Some(proton) = ["H+", "OH-"]
        .iter()
        .filter_map(|name| dissolved.iter().find(|(d, _)| d.name == *name))
        .max_by(|a, b| a.0.molality.total_cmp(&b.0.molality))
    {
        balancers.push((proton.0.name.clone(), proton.1.clone()));
    }
    balancers.push((
        "H2O".to_string(),
        stoich::parse_formula("H2O").expect("H2O is a formula"),
    ));

    let terms: Vec<(String, Formula)> = participants
        .iter()
        .chain(balancers.iter())
        .cloned()
        .collect();
    let coefficients = balance_against(&terms, &solid_formula)?;

    // Every chosen partner must actually take part: a "net ionic equation"
    // in which the silver ion has coefficient zero is not one.
    if coefficients[..participants.len()].iter().any(|c| *c <= 0.0) {
        return None;
    }
    let scale = integer_scale(&coefficients)?;

    let mut reactants: Vec<IonTerm> = Vec::new();
    let mut products: Vec<IonTerm> = vec![IonTerm::new(&solid_id.0, scale, 0, Phase::Solid)];
    for ((name, formula), coefficient) in terms.iter().zip(coefficients.iter()) {
        let n = coefficient * scale as f64;
        if n.abs() < 0.5 {
            continue;
        }
        let magnitude = n.abs().round() as u32;
        let term = IonTerm::new(
            name,
            magnitude,
            formula.charge.round() as i32,
            aqueous_or_solvent(name),
        );
        if n > 0.0 {
            reactants.push(term);
        } else {
            products.push(term);
        }
    }

    let participant_names: Vec<&str> = terms.iter().map(|(n, _)| n.as_str()).collect();
    Some(NetIonic::assemble(
        vessel.id,
        IonicBasis::Precipitation,
        reactants,
        products,
        spectators(&dissolved, &participant_names),
        provenance_of(vessel),
    ))
}

// ── neutralisation ──────────────────────────────────────────────────

fn from_neutralisation(vessel: &Vessel) -> Option<NetIonic> {
    let dissolved = dissolved_species(vessel)?;
    // Both halves have to be species the solver actually reports. Written
    // out of a table instead, this would be the one equation in the module
    // that is remembered rather than derived.
    let proton = dissolved.iter().find(|(d, _)| d.name == "H+")?;
    let hydroxide = dissolved.iter().find(|(d, _)| d.name == "OH-")?;
    let participants = [proton.0.name.as_str(), hydroxide.0.name.as_str(), "H2O"];
    Some(NetIonic::assemble(
        vessel.id,
        IonicBasis::Neutralisation,
        vec![
            IonTerm::new("H+", 1, 1, Phase::Aqueous),
            IonTerm::new("OH-", 1, -1, Phase::Aqueous),
        ],
        vec![IonTerm::new("H2O", 1, 0, Phase::Liquid)],
        spectators(&dissolved, &participants),
        provenance_of(vessel),
    ))
}

// ── the speciation the vessel carries ───────────────────────────────

/// The dissolved species worth reasoning about, with their formulas
/// parsed. `None` — not an empty list — where no solver has characterised
/// the solution: "we do not know" and "there is nothing there" are
/// different answers and only one of them is true here.
fn dissolved_species(vessel: &Vessel) -> Option<Vec<(&SpeciesDetail, Formula)>> {
    let solution = vessel.solution.as_ref()?;
    let parsed: Vec<(&SpeciesDetail, Formula)> = solution
        .species
        .iter()
        .filter(|d| d.molality > TRACE_MOLALITY)
        .filter_map(|d| {
            let formula =
                stoich::parse_formula_with(&d.name, FormulaDialect::PhreeqcMaster).ok()?;
            Some((d, formula))
        })
        .collect();
    (!parsed.is_empty()).then_some(parsed)
}

fn provenance_of(vessel: &Vessel) -> Option<String> {
    let p = vessel.solution.as_ref()?.provenance.as_ref()?;
    Some(format!("{} · {} · {}", p.engine, p.dataset, p.model))
}

/// The ions left over: charged, present in quantity, and not taking part.
fn spectators(dissolved: &[(&SpeciesDetail, Formula)], participants: &[&str]) -> Vec<IonTerm> {
    let ceiling = dissolved
        .iter()
        .filter(|(d, f)| f.charge != 0.0 && !is_water_axis(&d.name))
        .map(|(d, _)| d.molality)
        .fold(0.0f64, f64::max);
    if ceiling <= 0.0 {
        return Vec::new();
    }
    let mut candidates: Vec<&(&SpeciesDetail, Formula)> = dissolved
        .iter()
        .filter(|(d, f)| {
            f.charge != 0.0
                && !is_water_axis(&d.name)
                && !participants.contains(&d.name.as_str())
                && d.molality >= ceiling * SPECTATOR_FRACTION
        })
        .collect();
    candidates.sort_by(|a, b| b.0.molality.total_cmp(&a.0.molality));
    candidates.truncate(MAX_SPECTATORS);
    candidates
        .into_iter()
        .map(|(d, f)| IonTerm::new(&d.name, 1, f.charge.round() as i32, Phase::Aqueous))
        .collect()
}

fn is_water_axis(name: &str) -> bool {
    matches!(name, "H+" | "OH-" | "H2O" | "H3O+")
}

fn aqueous_or_solvent(name: &str) -> Phase {
    if name == "H2O" {
        Phase::Liquid
    } else {
        Phase::Aqueous
    }
}

// ── the small linear solve ──────────────────────────────────────────

/// Coefficients `x` such that `Σ xᵢ·termᵢ = target`, over every element
/// involved *and* over charge. Free variables are pinned to zero, which
/// picks the smallest set of participants that works; the answer is then
/// verified against every row, so a system with no exact solution returns
/// `None` rather than a least-squares fiction.
fn balance_against(terms: &[(String, Formula)], target: &Formula) -> Option<Vec<f64>> {
    let mut elements: Vec<String> = target.counts.keys().cloned().collect();
    for (_, f) in terms {
        for e in f.counts.keys() {
            if !elements.contains(e) {
                elements.push(e.clone());
            }
        }
    }
    elements.sort();

    let n = terms.len();
    let mut rows: Vec<Vec<f64>> = Vec::with_capacity(elements.len() + 1);
    for element in &elements {
        let mut row: Vec<f64> = terms
            .iter()
            .map(|(_, f)| f.counts.get(element).copied().unwrap_or(0.0))
            .collect();
        row.push(target.counts.get(element).copied().unwrap_or(0.0));
        rows.push(row);
    }
    let mut charge_row: Vec<f64> = terms.iter().map(|(_, f)| f.charge).collect();
    charge_row.push(target.charge);
    rows.push(charge_row);

    let solution = gauss_jordan(&mut rows, n)?;
    // Verify. Gauss–Jordan on an over-determined system happily reports a
    // pivot solution that satisfies the rows it used and nothing else.
    for row in &rows {
        let lhs: f64 = row[..n]
            .iter()
            .zip(solution.iter())
            .map(|(a, x)| a * x)
            .sum();
        if (lhs - row[n]).abs() > 1e-6 {
            return None;
        }
    }
    Some(solution)
}

/// Row-reduce an augmented `m × (n+1)` system in place and read off a
/// solution with free variables at zero. `None` if a row reduces to
/// `0 = c`, which is an inconsistent system.
fn gauss_jordan(rows: &mut [Vec<f64>], n: usize) -> Option<Vec<f64>> {
    let m = rows.len();
    let mut pivot_of: Vec<Option<usize>> = vec![None; n];
    let mut r = 0usize;
    for c in 0..n {
        let Some(p) = (r..m).max_by(|a, b| rows[*a][c].abs().total_cmp(&rows[*b][c].abs())) else {
            break;
        };
        if rows[p][c].abs() < 1e-9 {
            continue;
        }
        rows.swap(r, p);
        let lead = rows[r][c];
        for value in rows[r].iter_mut() {
            *value /= lead;
        }
        let pivot_row = rows[r].clone();
        for (other, row) in rows.iter_mut().enumerate().take(m) {
            if other == r {
                continue;
            }
            let factor = row[c];
            if factor.abs() < 1e-12 {
                continue;
            }
            for k in c..=n {
                row[k] -= factor * pivot_row[k];
            }
        }
        pivot_of[c] = Some(r);
        r += 1;
        if r == m {
            break;
        }
    }
    // An all-zero coefficient row with a non-zero constant is `0 = c`.
    for row in rows.iter() {
        if row[..n].iter().all(|v| v.abs() < 1e-9) && row[n].abs() > 1e-6 {
            return None;
        }
    }
    Some(
        (0..n)
            .map(|c| pivot_of[c].map(|r| rows[r][n]).unwrap_or(0.0))
            .collect(),
    )
}

/// The smallest whole number that turns every coefficient into a whole
/// number. Chemistry writes `2 OH⁻`, not `0.5`.
fn integer_scale(coefficients: &[f64]) -> Option<u32> {
    (1u32..=12).find(|k| {
        coefficients
            .iter()
            .all(|c| ((c * *k as f64) - (c * *k as f64).round()).abs() < 1e-6)
    })
}

// ── typography ──────────────────────────────────────────────────────

fn phase_suffix(phase: Phase) -> &'static str {
    match phase {
        Phase::Aqueous => "(aq)",
        Phase::Solid => "(s)",
        Phase::Liquid => "(l)",
        Phase::Gas => "(g)",
    }
}

/// `SO4-2` → `SO₄²⁻`. The engine and PHREEQC write charges flat; a reader
/// reads them raised, and the equation strip is the one place in the app
/// where that is worth the characters.
pub fn typeset(name: &str) -> String {
    let (body, charge) = split_trailing_charge(name);
    let mut out = String::with_capacity(name.len());
    for ch in body.chars() {
        out.push(match ch {
            '0'..='9' => SUBSCRIPTS[ch as usize - '0' as usize],
            other => other,
        });
    }
    let magnitude = charge.unsigned_abs();
    if magnitude > 1 {
        for digit in magnitude.to_string().chars() {
            out.push(SUPERSCRIPTS[digit as usize - '0' as usize]);
        }
    }
    match charge.signum() {
        1 => out.push('⁺'),
        -1 => out.push('⁻'),
        _ => {}
    }
    out
}

const SUBSCRIPTS: [char; 10] = ['₀', '₁', '₂', '₃', '₄', '₅', '₆', '₇', '₈', '₉'];
const SUPERSCRIPTS: [char; 10] = ['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'];

/// Split a trailing charge the way `stoich` does — digits *before* a sign
/// are a subscript (`NO3-` is nitrate, not "N with three O and charge −1"),
/// and a sign followed by digits is the magnitude (`SO4-2`).
fn split_trailing_charge(name: &str) -> (&str, i32) {
    let t = name.trim_end();
    let signs = t
        .chars()
        .rev()
        .take_while(|c| *c == '+' || *c == '-')
        .count();
    if signs > 0 {
        let cut = t.len() - signs;
        let sign = if t.as_bytes()[cut] == b'-' { -1 } else { 1 };
        return (&t[..cut], sign * signs as i32);
    }
    if let Some(pos) = t.rfind(['+', '-']) {
        let tail = &t[pos + 1..];
        if !tail.is_empty() && tail.chars().all(|c| c.is_ascii_digit()) {
            let sign = if t.as_bytes()[pos] == b'-' { -1 } else { 1 };
            if let Ok(magnitude) = tail.parse::<i32>() {
                return (&t[..pos], sign * magnitude);
            }
        }
    }
    (t, 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::units::{Kelvin, Moles};
    use crate::vessel::{Provenance, SolutionInfo};

    fn detail(name: &str, molality: f64) -> SpeciesDetail {
        SpeciesDetail {
            name: name.to_string(),
            molality,
            activity: molality,
        }
    }

    fn beaker(species: Vec<SpeciesDetail>) -> Vessel {
        let mut v = Vessel::new(VesselId(0), "beaker");
        v.temperature = Kelvin::STANDARD;
        v.solution = Some(SolutionInfo {
            pe: None,
            redox: Vec::new(),
            ph: 7.0,
            ionic_strength: 0.05,
            species,
            provenance: Some(Provenance {
                engine: "PHREEQC (IPhreeqc)".into(),
                dataset: "wateq4f.dat".into(),
                model: "Debye–Hückel".into(),
                dataset_sources: Vec::new(),
                routing: "the only aqueous engine wired in this test".into(),
            }),
        });
        v
    }

    /// Silver nitrate met by table salt. The equation the bench must write
    /// is about silver and chloride; the sodium and the nitrate are still
    /// floating there and must not appear in it.
    ///
    /// The molalities are the shape PHREEQC actually returns for this
    /// beaker, and they are hostile to two obvious shortcuts. Free silver
    /// sits at 7 nmol/kgw — a precipitation depletes its own ions, so any
    /// "trace" floor of our own would blind this to the case it exists
    /// for. And the neutral AgCl(aq) complex is fifty times more abundant
    /// than Ag⁺, so picking the most abundant silver species outright
    /// would write `AgCl(aq) → AgCl(s)`.
    #[test]
    fn silver_chloride_drops_its_spectators() {
        let v = beaker(vec![
            detail("Na+", 0.086),
            detail("NO3-", 0.059),
            detail("Cl-", 0.027),
            detail("AgCl", 3.6e-7),
            detail("H+", 1e-7),
            detail("OH-", 1e-7),
            detail("AgCl2-", 8.8e-10),
            detail("Ag+", 6.7e-9),
        ]);
        let event = Event::Precipitated {
            vessel: VesselId(0),
            species: SpeciesId::new("AgCl"),
            moles: Moles(0.0058),
            dry: false,
        };
        let net = net_ionic(&event, &v).expect("a precipitate with speciation is derivable");

        assert_eq!(net.basis, IonicBasis::Precipitation);
        assert_eq!(net.equation, "Ag⁺(aq) + Cl⁻(aq) → AgCl(s)");

        let names: Vec<&str> = net.reactants.iter().map(|t| t.species.as_str()).collect();
        assert!(names.contains(&"Ag+") && names.contains(&"Cl-"));
        assert!(
            !names.contains(&"Na+") && !names.contains(&"NO3-"),
            "the spectators must not be in the equation: {names:?}"
        );
        assert!(net.reactants.iter().all(|t| t.coefficient == 1));

        let spectators: Vec<&str> = net.spectators.iter().map(|t| t.species.as_str()).collect();
        assert_eq!(spectators, vec!["Na+", "NO3-"]);
        assert_eq!(net.spectator_phrase().as_deref(), Some("Na⁺, NO₃⁻"));
        assert!(net.provenance.as_deref().unwrap().contains("PHREEQC"));
    }

    /// Lye into magnesium chloride. The coefficient is not 1, and it has to
    /// be solved rather than assumed: two hydroxides per magnesium.
    #[test]
    fn magnesium_hydroxide_balances_its_hydroxide() {
        let v = beaker(vec![
            detail("Cl-", 0.10),
            detail("Na+", 0.10),
            detail("Mg+2", 1.0e-4),
            detail("OH-", 1.0e-3),
            detail("H+", 1.0e-11),
        ]);
        let event = Event::Precipitated {
            vessel: VesselId(0),
            species: SpeciesId::new("Mg(OH)2"),
            moles: Moles(0.01),
            dry: false,
        };
        let net = net_ionic(&event, &v).expect("hydroxide precipitation is derivable");
        assert_eq!(net.equation, "Mg²⁺(aq) + 2 OH⁻(aq) → Mg(OH)₂(s)");
        assert_eq!(
            net.spectators
                .iter()
                .map(|t| t.species.as_str())
                .collect::<Vec<_>>(),
            vec!["Cl-", "Na+"]
        );
    }

    /// Caustic soda met by hydrochloric acid. The engine computed how much
    /// acidity cancelled; the equation is about water, and nothing else in
    /// the beaker took part.
    #[test]
    fn neutralisation_is_water_forming() {
        let v = beaker(vec![
            detail("Na+", 0.10),
            detail("Cl-", 0.10),
            detail("H+", 1.0e-7),
            detail("OH-", 1.0e-7),
        ]);
        let event = Event::Neutralised {
            vessel: VesselId(0),
            moles: Moles(0.01),
        };
        let net = net_ionic(&event, &v).expect("a characterised solution can say this");
        assert_eq!(net.basis, IonicBasis::Neutralisation);
        assert_eq!(net.equation, "H⁺(aq) + OH⁻(aq) → H₂O(l)");
        assert_eq!(
            net.spectators
                .iter()
                .map(|t| t.species.as_str())
                .collect::<Vec<_>>(),
            vec!["Na+", "Cl-"]
        );
    }

    /// No speciation, no equation. A vessel no aqueous solver has looked at
    /// still precipitates things — a nonaqueous or curated path can emit
    /// the event — and the honest answer there is silence, not a guess at
    /// which ions "would" have been present.
    #[test]
    fn without_speciation_there_is_nothing_to_say() {
        let mut v = Vessel::new(VesselId(0), "beaker");
        v.solution = None;
        let event = Event::Precipitated {
            vessel: VesselId(0),
            species: SpeciesId::new("AgCl"),
            moles: Moles(0.0058),
            dry: false,
        };
        assert_eq!(net_ionic(&event, &v), None);

        // Characterised, but with nothing in it that could have made the
        // solid: still nothing to say rather than an invented partner.
        let barren = beaker(vec![detail("Na+", 0.1), detail("NO3-", 0.1)]);
        assert_eq!(net_ionic(&event, &barren), None);
    }

    /// An event that is not a reaction between ions earns no equation.
    #[test]
    fn other_events_earn_no_equation() {
        let v = beaker(vec![detail("Na+", 0.1), detail("Cl-", 0.1)]);
        let event = Event::Dissolved {
            vessel: VesselId(0),
            species: SpeciesId::new("NaCl"),
            moles: Moles(0.01),
        };
        assert_eq!(net_ionic(&event, &v), None);
    }

    /// Below the observability floor nothing is told, so nothing is
    /// derived either — the equation strip must not contradict the feed.
    #[test]
    fn a_trace_precipitate_is_not_announced() {
        let v = beaker(vec![
            detail("Ag+", 1e-5),
            detail("Cl-", 1e-5),
            detail("Na+", 0.1),
            detail("NO3-", 0.1),
        ]);
        let events = vec![Event::Precipitated {
            vessel: VesselId(0),
            species: SpeciesId::new("AgCl"),
            moles: Moles(1e-12),
            dry: false,
        }];
        assert!(net_ionic_for(&events, std::slice::from_ref(&v)).is_empty());
    }

    #[test]
    fn charges_are_typeset_the_way_they_are_read() {
        assert_eq!(typeset("Ag+"), "Ag⁺");
        assert_eq!(typeset("NO3-"), "NO₃⁻");
        assert_eq!(typeset("SO4-2"), "SO₄²⁻");
        assert_eq!(typeset("Ca+2"), "Ca²⁺");
        assert_eq!(typeset("H2O"), "H₂O");
        assert_eq!(typeset("Mg(OH)2"), "Mg(OH)₂");
    }
}
