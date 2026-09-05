//! BRD-020: the reaction-family intermediate representation.
//!
//! One audited rule applies a known transformation to structurally
//! matching substrates — without becoming an arbitrary predictor. The
//! tree already holds three partial answers, each grown ad hoc:
//!
//! * `curated.rs` — exact species pairs, with three bolted-on gates
//!   (`solvent`, `min_temp_k`, `catalyst`) added one experiment at a time;
//! * `kerotakis-org::templates` — SMIRKS transformations with their own
//!   `TemplateConditions`, reachable only through the `react` verb;
//! * `kinetics.rs` — rate laws with a third, separate catalyst notion.
//!
//! This module is the one vocabulary those grow toward: a versioned
//! record carrying the mapped transformation, the complete gate set, a
//! deterministic conflict order, the outcome model, provenance, and —
//! load-bearing, per the honesty rule — the requirement that a family
//! says *why it fired or declined* in words a learner can check.
//!
//! Structural matching is deliberately behind a trait. Today it is
//! implemented over `chematic` in `kerotakis-org`; BRD-022's selected
//! engine (Indigo or RDKit) replaces the implementation without touching
//! a single record, which is the point of an IR.
//!
//! What this deliberately is not (BRD-020's out-of-scope list): reaction
//! planning, retrosynthesis, learned outcome prediction, or rule mining.
//! Every record is authored, sourced, and linted.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::ops::{Event, NotModelledCause};
use crate::solve::{Applicability, CapabilityReport, Equilibrator, SolveError, SolverRouteKind};
use crate::species::{self, Phase, SpeciesId};
use crate::units::Moles;
use crate::vessel::Vessel;

/// The confidence vocabulary of ROADMAP R5, carried per record. A family
/// is never presented as more certain than its curation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FamilyConfidence {
    /// A verified template with condition gates: the family claims every
    /// structurally matching substrate inside its gates.
    CuratedFamily,
    /// Only the exact substrates named in the record are claimed; the
    /// template exists for checking, not for generalisation.
    CuratedInstance,
    /// Direction or site only — no quantitative yield or rate claim.
    Qualitative,
}

/// A temperature window in kelvin. Either bound may be open.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct KelvinWindow {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
}

impl KelvinWindow {
    pub fn admits(&self, t: f64) -> bool {
        self.min.is_none_or(|m| t >= m) && self.max.is_none_or(|m| t <= m)
    }
}

/// A pH window. Meaningful only where an aqueous solution is
/// characterised; a gate on pH in an uncharacterised vessel DECLINES
/// (with the reason) rather than assuming neutrality.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct PhWindow {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
}

/// What medium the family needs around its substrates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediumGate {
    /// Characterised aqueous solution (the engine's, not a label).
    Aqueous,
    /// A single named organic solvent, water absent — the CAP-23 bench.
    OrganicSolvent { solvent: String },
    /// Water must be absent entirely (the Grignard teaching case:
    /// EXP-46's moisture-kills-the-reagent is a computed verdict here).
    Anhydrous,
    /// Dry solids and gases — the thermal bench.
    Dry,
    /// The family does not care.
    Any,
}

/// A catalyst requirement. Present-but-not-consumed, like
/// `CuratedReaction::catalyst` — but named per family, with the honest
/// distinction between "this species" and "this kind of surface".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CatalystGate {
    Species {
        key: String,
    },
    /// Any of the named species satisfies the gate (nickel-class for
    /// EXP-46's couplings).
    AnyOf {
        keys: Vec<String>,
    },
}

/// Light requirement, for the photochemical families. The lamp state is
/// engine-owned (BRD-076's typed irradiation events); the gate merely
/// names what it needs.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LightGate {
    /// Maximum wavelength (nm) that can drive the transformation; the
    /// gate admits any applied wavelength at or below it.
    pub max_wavelength_nm: f64,
}

/// The complete condition set. Every field is a *gate*: all present
/// gates must admit the vessel, and the first gate that declines names
/// itself in the [`Declined`] explanation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GateSet {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub medium: Option<MediumGate>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature_k: Option<KelvinWindow>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ph: Option<PhWindow>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub catalyst: Option<CatalystGate>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub light: Option<LightGate>,
    /// Functional groups (by the `kerotakis-org` perception vocabulary)
    /// every mapped substrate must carry.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_groups: Vec<String>,
    /// Groups whose presence anywhere in the vessel forbids the family —
    /// the ORG-008 incompatibility idea, promoted into the IR.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub forbidden_groups: Vec<String>,
}

/// How the transformed matter behaves once the family fires.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutcomeModel {
    /// Runs to the limiting reagent — the honest default for the school
    /// preparations where equilibrium data is not curated.
    ToCompletion,
    /// An equilibrium constant with its provenance; the extent solver
    /// stops at K rather than at exhaustion.
    Equilibrium { log_k: f64, source: String },
    /// The family's time behaviour is owned by a registered kinetic law;
    /// firing the family means admitting that law, not reacting now.
    KineticLaw { kinetics_id: String },
}

/// A versioned, audited reaction family.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FamilyRecord {
    /// Stable identifier; never re-bound (the CAP/OPT/EXP law).
    pub id: String,
    pub version: u32,
    /// The atom-mapped transformation, in the SMIRKS dialect the
    /// structure oracle accepts. For `CuratedInstance` records this may
    /// be accompanied by the exact substrate list below.
    pub smirks: String,
    /// Exact substrates claimed (registry keys). Required for
    /// `CuratedInstance`; optional exemplars for `CuratedFamily`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub substrates: Vec<String>,
    /// Non-structural co-reactants and products by registry key with
    /// stoichiometric coefficients — the bridge into the conserved
    /// ledger for species the structural layer does not model (ions,
    /// water, the NaOAc ledger bridge the esterification tests state).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub ledger_reactants: BTreeMap<String, f64>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub ledger_products: BTreeMap<String, f64>,
    pub gates: GateSet,
    /// Deterministic conflict order: higher fires first. Ties break by
    /// gate specificity, then id — see [`conflict_order`].
    #[serde(default)]
    pub priority: i32,
    pub outcome: OutcomeModel,
    pub confidence: FamilyConfidence,
    /// Primary-literature or editorial provenance; the lint refuses an
    /// empty one, exactly as the codex refuses an empty `fails_at`.
    pub provenance: String,
    /// Where the family STOPS being claimed, in words. Shown when the
    /// family declines an almost-matching ask; the lint refuses records
    /// without one, because a rule without a boundary is presented as
    /// truth.
    pub refusal_domain: String,
}

/// Why a family fired: every clause a learner could check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fired {
    pub family: String,
    pub version: u32,
    /// The gates that were consulted and admitted, in order.
    pub gates_passed: Vec<String>,
    pub confidence: FamilyConfidence,
}

/// Why a family declined. `gate` names the first refusing gate; `reason`
/// is the sentence. A structural non-match is not a decline — a family
/// whose pattern does not match was never asked.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Declined {
    pub family: String,
    pub version: u32,
    pub gate: String,
    pub reason: String,
}

/// Deterministic conflict order over candidate families whose structure
/// and gates both admit the same vessel: priority descending, then more
/// total gates first (the more specific rule wins, the way `NH4` is
/// matched before the acetate signature in `derived.rs`), then id
/// ascending. Total, reproducible, and independent of record file order.
pub fn conflict_order(a: &FamilyRecord, b: &FamilyRecord) -> std::cmp::Ordering {
    b.priority
        .cmp(&a.priority)
        .then_with(|| gate_count(&b.gates).cmp(&gate_count(&a.gates)))
        .then_with(|| a.id.cmp(&b.id))
}

fn gate_count(g: &GateSet) -> usize {
    g.medium.is_some() as usize
        + g.temperature_k.is_some() as usize
        + g.ph.is_some() as usize
        + g.catalyst.is_some() as usize
        + g.light.is_some() as usize
        + g.required_groups.len()
        + g.forbidden_groups.len()
}

/// Structural services the IR needs, implemented outside this crate.
/// `kerotakis-org` provides the chematic-backed implementation today;
/// BRD-022's engine replaces it behind the same seam.
pub trait StructureOracle {
    /// Functional groups present on a species (by registry key), in the
    /// perception vocabulary `GateSet` gates on. `None` when the species
    /// has no modelled structure — which is an honest DECLINE for any
    /// family gating on groups, never a silent pass.
    fn groups_of(&self, species_key: &str) -> Option<Vec<String>>;
    /// Whether the family's mapped pattern matches the given substrates,
    /// and if so, the product species (by registry key where nameable).
    /// An unnameable product is an error by the honesty rule: the pool
    /// of the nameable is the boundary, exactly as in the thermal pool.
    fn apply(
        &self,
        record: &FamilyRecord,
        substrate_keys: &[&str],
    ) -> Result<Option<Vec<String>>, String>;
}

/// The record lint (BRD-020 acceptance): every family must conserve
/// atoms and charge across its ledger sides, carry provenance and a
/// refusal domain, and declare substrates when it claims only instances.
pub fn lint_record(record: &FamilyRecord) -> Vec<String> {
    let mut problems = Vec::new();
    if record.provenance.trim().is_empty() {
        problems.push(format!("{}: empty provenance", record.id));
    }
    if record.refusal_domain.trim().is_empty() {
        problems.push(format!(
            "{}: empty refusal_domain — a rule without a boundary is presented as truth",
            record.id
        ));
    }
    if record.confidence == FamilyConfidence::CuratedInstance && record.substrates.is_empty() {
        problems.push(format!(
            "{}: curated_instance claims exact substrates but names none",
            record.id
        ));
    }
    if record.smirks.trim().is_empty() && record.ledger_reactants.is_empty() {
        problems.push(format!(
            "{}: neither a mapped transformation nor ledger stoichiometry — the record does nothing",
            record.id
        ));
    }
    // Element/charge conservation over the ledger sides, where both are
    // expressed in registry keys with parsable formulas, is enforced by
    // the caller with the registry in hand (`lint_ledger_balance`); this
    // function checks what needs no registry.
    problems
}

// ── The router ──────────────────────────────────────────────────────
//
// BRD-020's open question was WHERE family matching sits. Decided
// 2026-09-05: immediately after the exact curated pairs and before every
// general engine (see `kerotakis-stack`). The bench has already screened
// the operator for safety and resolved every species name at parse time
// when a solver first sees the vessel, so this position is the IR's
// "after safety and identity resolution, before the honesty fallback".
// Curated rows answer first because an exact pair is the more specific
// claim; a family is asked only about structures the registry curated,
// and only once its gates admit the vessel.

const TRACE: f64 = 1e-12;

/// A family pack: `[[family]]` tables in TOML, every record linted on
/// load. A pack with one bad record is refused whole — a lint that let
/// the good records through would ship the bad one beside them.
pub fn load_records(toml_text: &str) -> Result<Vec<FamilyRecord>, String> {
    #[derive(Deserialize)]
    struct Pack {
        #[serde(default)]
        family: Vec<FamilyRecord>,
    }
    let pack: Pack =
        toml::from_str(toml_text).map_err(|e| format!("family pack does not parse: {e}"))?;
    let problems: Vec<String> = pack.family.iter().flat_map(lint_record).collect();
    if !problems.is_empty() {
        return Err(problems.join("; "));
    }
    Ok(pack.family)
}

/// How many mapped reactant molecules the SMIRKS names: top-level `.`
/// separators on its left-hand side, plus one. Zero for a ledger-only
/// record (empty SMIRKS), which fires on its ledger stoichiometry alone.
pub fn reactant_slots(smirks: &str) -> usize {
    let lhs = smirks.split(">>").next().unwrap_or("").trim();
    if lhs.is_empty() {
        return 0;
    }
    let mut depth = 0i32;
    let mut dots = 0usize;
    for ch in lhs.chars() {
        match ch {
            '[' => depth += 1,
            ']' => depth -= 1,
            '.' if depth == 0 => dots += 1,
            _ => {}
        }
    }
    dots + 1
}

/// One structural match of a record against a vessel: the substrates it
/// would consume (registry keys, coefficient one each) and the products
/// the oracle named for them.
#[derive(Debug, Clone)]
struct Matched {
    substrates: Vec<String>,
    products: Vec<String>,
}

/// A family whose structure matched and whose gates all admitted, with
/// the extent its outcome model allows.
#[derive(Debug, Clone)]
pub struct Ready {
    pub family: String,
    pub version: u32,
    pub gates_passed: Vec<String>,
    pub confidence: FamilyConfidence,
    pub substrates: Vec<String>,
    pub products: Vec<String>,
    /// Signed, in moles of the record as written. Negative runs the
    /// record toward its reactants — an equilibrium met from the product
    /// side.
    pub extent: f64,
}

/// What the router finds in a vessel, without touching it.
#[derive(Debug, Default)]
pub struct Evaluation {
    /// In conflict order.
    pub ready: Vec<Ready>,
    /// Structural matches a gate refused, each naming the gate.
    pub declined: Vec<Declined>,
    /// Structural matches the oracle could not carry into the registry —
    /// a product nobody curated, or a template that failed its own
    /// conservation check. Spoken as typed refusals, never dropped.
    pub refused: Vec<String>,
}

/// The reaction-family equilibrator: audited records, matched through a
/// structural oracle, gated, and applied to the vessel ledger.
pub struct FamilyRouter<O: StructureOracle> {
    oracle: O,
    records: Vec<FamilyRecord>,
}

impl<O: StructureOracle> FamilyRouter<O> {
    pub fn new(oracle: O, mut records: Vec<FamilyRecord>) -> Self {
        records.sort_by(conflict_order);
        FamilyRouter { oracle, records }
    }

    pub fn records(&self) -> &[FamilyRecord] {
        &self.records
    }

    /// Species in the vessel the oracle can see the structure of, in a
    /// fixed order. A species without a curated structure is never a
    /// candidate: a family asked about it has not matched — it was
    /// never asked.
    fn candidates(&self, vessel: &Vessel) -> Vec<String> {
        let mut keys: Vec<String> = vessel
            .contents
            .iter()
            .filter(|p| p.moles.0 > TRACE)
            .map(|p| p.species.0.clone())
            .filter(|k| self.oracle.groups_of(k).is_some())
            .collect();
        keys.sort();
        keys.dedup();
        keys
    }

    /// The first candidate tuple the record's pattern matches, in a
    /// fixed order — so the same vessel always yields the same match
    /// regardless of pouring order. Two-slot templates try ordered pairs
    /// of DISTINCT species; a record needing three mapped reactants is
    /// not asked here (none exists yet, and the IR says such a record
    /// should say which slot is which).
    fn find_match(
        &self,
        record: &FamilyRecord,
        candidates: &[String],
    ) -> Result<Option<Matched>, String> {
        let tuples: Vec<Vec<&str>> = match reactant_slots(&record.smirks) {
            0 => {
                return Ok(Some(Matched {
                    substrates: Vec::new(),
                    products: Vec::new(),
                }))
            }
            1 => candidates.iter().map(|k| vec![k.as_str()]).collect(),
            2 => {
                let mut pairs = Vec::new();
                for a in candidates {
                    for b in candidates {
                        if a != b {
                            pairs.push(vec![a.as_str(), b.as_str()]);
                        }
                    }
                }
                pairs
            }
            _ => return Ok(None),
        };
        for keys in tuples {
            if let Some(products) = self.oracle.apply(record, &keys)? {
                return Ok(Some(Matched {
                    substrates: keys.iter().map(|k| k.to_string()).collect(),
                    products,
                }));
            }
        }
        Ok(None)
    }

    /// Every gate in order; the first refusing gate names itself.
    fn check_gates(
        &self,
        record: &FamilyRecord,
        vessel: &Vessel,
        substrates: &[String],
    ) -> Result<Vec<String>, Declined> {
        let g = &record.gates;
        let declined = |gate: &str, reason: String| Declined {
            family: record.id.clone(),
            version: record.version,
            gate: gate.to_string(),
            reason,
        };
        let mut passed = Vec::new();
        if let Some(medium) = &g.medium {
            medium_admits(medium, vessel).map_err(|why| declined("medium", why))?;
            passed.push("medium".to_string());
        }
        if let Some(w) = &g.temperature_k {
            if !w.admits(vessel.temperature.0) {
                return Err(declined(
                    "temperature_k",
                    format!(
                        "the vessel stands at {:.1} K and the family needs {} K",
                        vessel.temperature.0,
                        describe_window(w.min, w.max)
                    ),
                ));
            }
            passed.push("temperature_k".to_string());
        }
        if let Some(w) = &g.ph {
            match vessel.solution.as_ref().map(|s| s.ph) {
                None => {
                    return Err(declined(
                        "ph",
                        "no characterised aqueous solution to read a pH from — the gate declines \
                         rather than assuming neutrality"
                            .to_string(),
                    ))
                }
                Some(ph) => {
                    if !(w.min.is_none_or(|m| ph >= m) && w.max.is_none_or(|m| ph <= m)) {
                        return Err(declined(
                            "ph",
                            format!("pH {ph:.2} is outside {}", describe_window(w.min, w.max)),
                        ));
                    }
                }
            }
            passed.push("ph".to_string());
        }
        if let Some(c) = &g.catalyst {
            let present = |key: &str| vessel.moles_of(&SpeciesId::new(key)).0 > TRACE;
            let (ok, wanted) = match c {
                CatalystGate::Species { key } => (present(key), key.clone()),
                CatalystGate::AnyOf { keys } => {
                    (keys.iter().any(|k| present(k)), keys.join(" or "))
                }
            };
            if !ok {
                return Err(declined(
                    "catalyst",
                    format!("no {wanted} in the vessel — the family needs its catalyst present"),
                ));
            }
            passed.push("catalyst".to_string());
        }
        if g.light.is_some() {
            return Err(declined(
                "light",
                "irradiation reaches the bench as a typed event, not as a state the vessel keeps \
                 (BRD-076); a light gate cannot yet be read, so it declines"
                    .to_string(),
            ));
        }
        if !g.required_groups.is_empty() {
            let mut seen: Vec<String> = Vec::new();
            for key in substrates {
                match self.oracle.groups_of(key) {
                    Some(groups) => seen.extend(groups),
                    None => {
                        return Err(declined(
                            "required_groups",
                            format!("{key} has no curated structure, so its groups cannot be seen"),
                        ))
                    }
                }
            }
            if let Some(missing) = g.required_groups.iter().find(|r| !seen.contains(r)) {
                return Err(declined(
                    "required_groups",
                    format!("no substrate carries a {missing} group"),
                ));
            }
            passed.push("required_groups".to_string());
        }
        if !g.forbidden_groups.is_empty() {
            for p in vessel.contents.iter().filter(|p| p.moles.0 > TRACE) {
                if let Some(groups) = self.oracle.groups_of(&p.species.0) {
                    if let Some(f) = g.forbidden_groups.iter().find(|f| groups.contains(f)) {
                        return Err(declined(
                            "forbidden_groups",
                            format!(
                                "{} carries a {f} group, which this family refuses to run beside",
                                p.species.0
                            ),
                        ));
                    }
                }
            }
            passed.push("forbidden_groups".to_string());
        }
        Ok(passed)
    }

    /// Look, without touching.
    pub fn evaluate(&self, vessel: &Vessel) -> Evaluation {
        let mut out = Evaluation::default();
        let candidates = self.candidates(vessel);
        for record in &self.records {
            let matched = match self.find_match(record, &candidates) {
                Ok(Some(m)) => m,
                Ok(None) => continue,
                Err(why) => {
                    out.refused.push(why);
                    continue;
                }
            };
            let gates = match self.check_gates(record, vessel, &matched.substrates) {
                Ok(g) => g,
                Err(d) => {
                    out.declined.push(d);
                    continue;
                }
            };
            let (reactants, products) = sides(record, &matched);
            match solve_extent(record, vessel, &reactants, &products) {
                Ok(x) if x.abs() > TRACE => out.ready.push(Ready {
                    family: record.id.clone(),
                    version: record.version,
                    gates_passed: gates,
                    confidence: record.confidence,
                    substrates: matched.substrates,
                    products: matched.products,
                    extent: x,
                }),
                Ok(_) => {}
                Err(why) => out.declined.push(Declined {
                    family: record.id.clone(),
                    version: record.version,
                    gate: "outcome".to_string(),
                    reason: why,
                }),
            }
        }
        out
    }
}

fn medium_admits(medium: &MediumGate, vessel: &Vessel) -> Result<(), String> {
    let liquid = |p: &crate::vessel::Portion| {
        matches!(p.phase, Phase::Liquid | Phase::Aqueous) && p.moles.0 > TRACE
    };
    let water_liquid = vessel
        .contents
        .iter()
        .any(|p| p.species.0 == "water" && liquid(p));
    let any_liquid = vessel.contents.iter().any(liquid);
    match medium {
        MediumGate::Any => Ok(()),
        MediumGate::Aqueous => {
            let watery = water_liquid
                && crate::nonaqueous::water_fraction_among_solvents(vessel)
                    .is_none_or(|f| f >= crate::nonaqueous::AQUEOUS_WATER_FRACTION_FLOOR);
            if watery || vessel.solution.is_some() {
                Ok(())
            } else {
                Err(
                    "the family runs in water, and this vessel holds no water-majority liquid"
                        .to_string(),
                )
            }
        }
        MediumGate::OrganicSolvent { solvent } => {
            match crate::nonaqueous::single_organic_solvent(vessel) {
                Some(s) if s == solvent => Ok(()),
                Some(s) => Err(format!(
                    "the liquid is {s}, and the family runs in {solvent}"
                )),
                None => Err(format!("no single water-free {solvent} phase to run in")),
            }
        }
        MediumGate::Anhydrous => {
            if vessel.moles_of(&SpeciesId::new("water")).0 > TRACE {
                Err("water is present, and this family needs it absent".to_string())
            } else {
                Ok(())
            }
        }
        MediumGate::Dry => {
            if any_liquid {
                Err(
                    "a liquid is present, and this family runs between dry solids and gases"
                        .to_string(),
                )
            } else {
                Ok(())
            }
        }
    }
}

fn describe_window(min: Option<f64>, max: Option<f64>) -> String {
    match (min, max) {
        (Some(a), Some(b)) => format!("between {a:.1} and {b:.1}"),
        (Some(a), None) => format!("at least {a:.1}"),
        (None, Some(b)) => format!("at most {b:.1}"),
        (None, None) => "any".to_string(),
    }
}

/// One side of a record as (registry key, coefficient) pairs.
type Side = Vec<(String, f64)>;

/// The two sides of a matched record: the structural substrates and
/// products at one each, then the ledger bridge for the species the
/// structural layer does not model.
fn sides(record: &FamilyRecord, m: &Matched) -> (Side, Side) {
    let mut reactants: Side = m.substrates.iter().map(|k| (k.clone(), 1.0)).collect();
    reactants.extend(record.ledger_reactants.iter().map(|(k, c)| (k.clone(), *c)));
    let mut products: Side = m.products.iter().map(|k| (k.clone(), 1.0)).collect();
    products.extend(record.ledger_products.iter().map(|(k, c)| (k.clone(), *c)));
    (reactants, products)
}

/// The extent the outcome model allows, signed. `ToCompletion` runs to
/// the limiting reagent. `Equilibrium` finds the extent at which the
/// mole-basis quotient meets K — valid where the stoichiometric sums
/// match so the volume cancels, which the record is checked for — and
/// may be negative when the mixture already stands past K.
fn solve_extent(
    record: &FamilyRecord,
    vessel: &Vessel,
    reactants: &[(String, f64)],
    products: &[(String, f64)],
) -> Result<f64, String> {
    let amount = |k: &str| vessel.moles_of(&SpeciesId::new(k)).0;
    let forward_max = reactants
        .iter()
        .map(|(k, c)| amount(k) / c)
        .fold(f64::INFINITY, f64::min);
    if !forward_max.is_finite() {
        return Ok(0.0);
    }
    match &record.outcome {
        OutcomeModel::ToCompletion => Ok(forward_max.max(0.0)),
        OutcomeModel::KineticLaw { kinetics_id } => Err(format!(
            "outcome model kinetic_law ({kinetics_id}) is not yet routed — BRD-050 owns admitting \
             a rate law from a family"
        )),
        OutcomeModel::Equilibrium { log_k, .. } => {
            let sum_r: f64 = reactants.iter().map(|(_, c)| c).sum();
            let sum_p: f64 = products.iter().map(|(_, c)| c).sum();
            if (sum_r - sum_p).abs() > 1e-9 {
                return Err(format!(
                    "the equilibrium model needs equal stoichiometric sums so the volume cancels; \
                     this record has {sum_r} in and {sum_p} out"
                ));
            }
            let reverse_max = products
                .iter()
                .map(|(k, c)| amount(k) / c)
                .fold(f64::INFINITY, f64::min);
            let ln_k = log_k * std::f64::consts::LN_10;
            // ln Q(x) − ln K: rises with x, because products grow and
            // reactants shrink. Its zero is the extent.
            let f = |x: f64| -> f64 {
                let mut v = -ln_k;
                for (k, c) in products {
                    v += c * (amount(k) + c * x).max(1e-300).ln();
                }
                for (k, c) in reactants {
                    v -= c * (amount(k) - c * x).max(1e-300).ln();
                }
                v
            };
            let lo = if reverse_max.is_finite() {
                -reverse_max
            } else {
                0.0
            };
            let hi = forward_max;
            if f(hi) <= 0.0 {
                return Ok(hi);
            }
            if f(lo) >= 0.0 {
                return Ok(lo);
            }
            let (mut a, mut b) = (lo, hi);
            for _ in 0..200 {
                let mid = 0.5 * (a + b);
                if f(mid) < 0.0 {
                    a = mid;
                } else {
                    b = mid;
                }
            }
            Ok(0.5 * (a + b))
        }
    }
}

fn phase_of(key: &str) -> Result<Phase, String> {
    species::lookup_key(key)
        .map(|d| d.standard_phase)
        .ok_or_else(|| format!("the registry has no species '{key}' to deposit"))
}

/// Move the matter. Every product phase is resolved BEFORE the first
/// withdrawal: a half-applied transformation would be the one thing
/// worse than none.
fn apply_extent(
    vessel: &mut Vessel,
    reactants: &[(String, f64)],
    products: &[(String, f64)],
    x: f64,
) -> Result<(), String> {
    let (consumed, formed) = if x >= 0.0 {
        (reactants, products)
    } else {
        (products, reactants)
    };
    let n = x.abs();
    let phases: Vec<Phase> = formed
        .iter()
        .map(|(k, _)| phase_of(k))
        .collect::<Result<_, _>>()?;
    for (k, c) in consumed {
        vessel.withdraw(&SpeciesId::new(k), Moles(n * c));
    }
    for ((k, c), phase) in formed.iter().zip(phases) {
        vessel.deposit(SpeciesId::new(k), Moles(n * c), phase);
    }
    // The aqueous tail reads this current; a family that moves ions has
    // to leave it right, exactly as `curated.rs` does.
    vessel.solute_charge = crate::displacement::solute_charge(vessel);
    Ok(())
}

fn equation_of(
    record: &FamilyRecord,
    reactants: &[(String, f64)],
    products: &[(String, f64)],
) -> String {
    let side = |s: &[(String, f64)]| {
        s.iter()
            .map(|(k, c)| {
                if (*c - 1.0).abs() < 1e-12 {
                    k.clone()
                } else {
                    format!("{c} {k}")
                }
            })
            .collect::<Vec<_>>()
            .join(" + ")
    };
    let arrow = match record.outcome {
        OutcomeModel::Equilibrium { .. } => "⇌",
        _ => "→",
    };
    format!("{} {arrow} {}", side(reactants), side(products))
}

fn boundary_of(record: &FamilyRecord, ready: &Ready) -> String {
    let gates = if ready.gates_passed.is_empty() {
        "none required".to_string()
    } else {
        ready.gates_passed.join(", ")
    };
    let direction = if ready.extent < 0.0 {
        "; ran toward the reactants, because the mixture stood past its equilibrium"
    } else {
        ""
    };
    format!(
        "{} (family {} v{}, {:?}; gates passed: {gates}{direction})",
        record.refusal_domain, record.id, record.version, record.confidence
    )
}

impl<O: StructureOracle> Equilibrator for FamilyRouter<O> {
    fn name(&self) -> &'static str {
        "reaction-families"
    }

    fn route_kind(&self) -> SolverRouteKind {
        SolverRouteKind::Curated
    }

    /// A family that would fire, or a match the registry cannot carry —
    /// both are this solver having something to say. A gate decline is
    /// not: the vessel was examined and the answer is "not under these
    /// conditions", which `capability` reports in words without adding
    /// a line to every step of a lesson that never meant to esterify.
    fn applies(&self, vessel: &Vessel) -> bool {
        let e = self.evaluate(vessel);
        !e.ready.is_empty() || !e.refused.is_empty()
    }

    fn capability(&self, vessel: &Vessel) -> CapabilityReport {
        let e = self.evaluate(vessel);
        let applicability = if !e.ready.is_empty() {
            Applicability::Applicable
        } else if !e.declined.is_empty() {
            Applicability::NotApplicable {
                reason: e
                    .declined
                    .iter()
                    .map(|d| {
                        format!(
                            "{} v{} declined at {}: {}",
                            d.family, d.version, d.gate, d.reason
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("; "),
            }
        } else {
            Applicability::NotApplicable {
                reason: "no reaction family's pattern matches this vessel".to_string(),
            }
        };
        CapabilityReport {
            solver: self.name(),
            applicability,
            is_chemistry: self.chemistry_applies(vessel),
            validity: None,
        }
    }

    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        let mut events = Vec::new();
        let mut spoken: Vec<String> = Vec::new();
        // One family per pass, then look again: what the first one made
        // may be what the next one needs, and an equilibrium record
        // re-evaluates to nothing once it stands at K.
        for _ in 0..8 {
            let evaluation = self.evaluate(vessel);
            for why in evaluation.refused {
                if spoken.contains(&why) {
                    continue;
                }
                let cause = if why.contains("cannot name") {
                    NotModelledCause::PhaseNotInRegistry
                } else {
                    NotModelledCause::ModelBoundary
                };
                events.push(Event::NotYetModeled {
                    cause,
                    vessel: vessel.id,
                    what: why.clone(),
                });
                spoken.push(why);
            }
            let Some(ready) = evaluation.ready.into_iter().next() else {
                break;
            };
            let record = self
                .records
                .iter()
                .find(|r| r.id == ready.family)
                .expect("a ready family is one of this router's records");
            let matched = Matched {
                substrates: ready.substrates.clone(),
                products: ready.products.clone(),
            };
            let (reactants, products) = sides(record, &matched);
            apply_extent(vessel, &reactants, &products, ready.extent).map_err(|detail| {
                SolveError::NotConverged {
                    solver: "reaction-families".to_string(),
                    detail,
                }
            })?;
            events.push(Event::OrgReacted {
                vessel: vessel.id,
                name: record.id.clone(),
                equation: equation_of(record, &reactants, &products),
                extent: Moles(ready.extent.abs()),
                boundary: boundary_of(record, &ready),
            });
        }
        Ok(events)
    }
}

#[cfg(test)]
mod router_tests {
    use super::*;
    use crate::vessel::VesselId;

    /// A structural oracle with no chemistry toolkit behind it: the
    /// router's own logic is what these tests weigh. `kerotakis-org`
    /// tests the same router over the real chematic oracle.
    struct FakeOracle;

    impl StructureOracle for FakeOracle {
        fn groups_of(&self, key: &str) -> Option<Vec<String>> {
            match key {
                "CH3COOH" => Some(vec!["carboxylic acid".to_string()]),
                "ethanol" | "methanol" => Some(vec!["alcohol".to_string()]),
                "ethyl_acetate" => Some(vec!["ester".to_string()]),
                "water" => Some(Vec::new()),
                _ => None,
            }
        }

        fn apply(
            &self,
            record: &FamilyRecord,
            keys: &[&str],
        ) -> Result<Option<Vec<String>>, String> {
            if !record.id.starts_with("fake-esterification") {
                return Ok(None);
            }
            match keys {
                ["CH3COOH", "ethanol"] => {
                    Ok(Some(vec!["ethyl_acetate".to_string(), "water".to_string()]))
                }
                ["CH3COOH", "methanol"] => Err(format!(
                    "family {} produced a structure the registry cannot name (COC(C)=O)",
                    record.id
                )),
                _ => Ok(None),
            }
        }
    }

    fn esterification(id: &str, priority: i32) -> FamilyRecord {
        FamilyRecord {
            id: id.to_string(),
            version: 1,
            smirks: "[C:1](=[O:2])[OH:3].[OH:4][C:5]>>[C:1](=[O:2])[O:4][C:5].[OH2:3]".to_string(),
            substrates: vec!["CH3COOH".to_string(), "ethanol".to_string()],
            ledger_reactants: BTreeMap::new(),
            ledger_products: BTreeMap::new(),
            gates: GateSet {
                temperature_k: Some(KelvinWindow {
                    min: Some(333.15),
                    max: None,
                }),
                catalyst: Some(CatalystGate::Species {
                    key: "H2SO4".to_string(),
                }),
                required_groups: vec!["carboxylic acid".to_string(), "alcohol".to_string()],
                ..GateSet::default()
            },
            priority,
            outcome: OutcomeModel::Equilibrium {
                log_k: 4f64.log10(),
                source: "test".to_string(),
            },
            confidence: FamilyConfidence::CuratedFamily,
            provenance: "test".to_string(),
            refusal_domain: "test boundary".to_string(),
        }
    }

    fn vessel(portions: &[(&str, f64)], kelvin: f64) -> Vessel {
        let mut v = Vessel::new(VesselId(0), "v1");
        for (key, moles) in portions {
            let phase = species::lookup_key(key)
                .map(|d| d.standard_phase)
                .unwrap_or(Phase::Liquid);
            v.deposit(SpeciesId::new(key), Moles(*moles), phase);
        }
        v.temperature = crate::units::Kelvin(kelvin);
        v
    }

    fn mass_g(v: &Vessel) -> f64 {
        v.contents
            .iter()
            .map(|p| {
                p.moles.0
                    * species::lookup(&p.species)
                        .expect("registry species")
                        .molar_mass
            })
            .sum()
    }

    fn router() -> FamilyRouter<FakeOracle> {
        FamilyRouter::new(FakeOracle, vec![esterification("fake-esterification", 0)])
    }

    #[test]
    fn hot_and_catalysed_it_runs_to_k_and_conserves_mass() {
        let mut v = vessel(
            &[("CH3COOH", 0.1), ("ethanol", 0.1), ("H2SO4", 0.001)],
            340.0,
        );
        let before = mass_g(&v);
        let events = router().equilibrate(&mut v).expect("equilibrates");
        // K = 4 for an equimolar pair: x/(0.1 − x) = 2, x = 0.0667.
        let ester = v.moles_of(&SpeciesId::new("ethyl_acetate")).0;
        assert!((ester - 0.2 / 3.0).abs() < 1e-6, "ester {ester}");
        assert!((v.moles_of(&SpeciesId::new("water")).0 - 0.2 / 3.0).abs() < 1e-6);
        assert!((mass_g(&v) - before).abs() < 1e-9, "mass drifted");
        assert!(events.iter().any(|e| matches!(
            e,
            Event::OrgReacted { name, equation, .. }
                if name == "fake-esterification" && equation.contains("⇌")
        )));
        // The catalyst is present, not consumed.
        assert!((v.moles_of(&SpeciesId::new("H2SO4")).0 - 0.001).abs() < 1e-12);
    }

    #[test]
    fn cold_it_declines_and_names_the_temperature_gate() {
        let v = vessel(
            &[("CH3COOH", 0.1), ("ethanol", 0.1), ("H2SO4", 0.001)],
            298.15,
        );
        let r = router();
        assert!(!r.applies(&v));
        let e = r.evaluate(&v);
        assert_eq!(e.declined.len(), 1);
        assert_eq!(e.declined[0].gate, "temperature_k");
        match r.capability(&v).applicability {
            Applicability::NotApplicable { reason } => assert!(reason.contains("temperature_k")),
            other => panic!("expected a decline, got {other:?}"),
        }
    }

    #[test]
    fn without_its_catalyst_it_declines_by_name() {
        let v = vessel(&[("CH3COOH", 0.1), ("ethanol", 0.1)], 340.0);
        let e = router().evaluate(&v);
        assert_eq!(e.declined[0].gate, "catalyst");
        assert!(e.declined[0].reason.contains("H2SO4"));
    }

    #[test]
    fn added_water_pushes_the_equilibrium_back() {
        let mut dry = vessel(
            &[("CH3COOH", 0.1), ("ethanol", 0.1), ("H2SO4", 0.001)],
            340.0,
        );
        let mut wet = vessel(
            &[
                ("CH3COOH", 0.1),
                ("ethanol", 0.1),
                ("H2SO4", 0.001),
                ("water", 1.0),
            ],
            340.0,
        );
        router().equilibrate(&mut dry).unwrap();
        router().equilibrate(&mut wet).unwrap();
        let ester = |v: &Vessel| v.moles_of(&SpeciesId::new("ethyl_acetate")).0;
        assert!(
            ester(&wet) < 0.5 * ester(&dry),
            "{} vs {}",
            ester(&wet),
            ester(&dry)
        );
        // 3x² − 1.8x + 0.04 = 0 with a mole of water present.
        assert!((ester(&wet) - 0.023_1).abs() < 5e-4, "{}", ester(&wet));
    }

    #[test]
    fn past_its_equilibrium_the_record_runs_backward() {
        // Mostly ester and water, a trace of acid and alcohol: Q > K.
        let mut v = vessel(
            &[
                ("CH3COOH", 0.001),
                ("ethanol", 0.001),
                ("ethyl_acetate", 0.1),
                ("water", 0.1),
                ("H2SO4", 0.001),
            ],
            340.0,
        );
        let events = router().equilibrate(&mut v).unwrap();
        assert!(v.moles_of(&SpeciesId::new("CH3COOH")).0 > 0.001);
        assert!(v.moles_of(&SpeciesId::new("ethyl_acetate")).0 < 0.1);
        assert!(events.iter().any(|e| matches!(
            e,
            Event::OrgReacted { boundary, .. } if boundary.contains("toward the reactants")
        )));
    }

    #[test]
    fn pouring_order_does_not_change_the_answer() {
        let mut ab = vessel(
            &[("CH3COOH", 0.1), ("ethanol", 0.15), ("H2SO4", 0.001)],
            340.0,
        );
        let mut ba = vessel(
            &[("H2SO4", 0.001), ("ethanol", 0.15), ("CH3COOH", 0.1)],
            340.0,
        );
        router().equilibrate(&mut ab).unwrap();
        router().equilibrate(&mut ba).unwrap();
        for key in ["CH3COOH", "ethanol", "ethyl_acetate", "water"] {
            let id = SpeciesId::new(key);
            assert!(
                (ab.moles_of(&id).0 - ba.moles_of(&id).0).abs() < 1e-12,
                "{key}"
            );
        }
    }

    #[test]
    fn a_species_without_a_structure_is_never_asked() {
        let v = vessel(
            &[("CH3COOH", 0.1), ("hexane", 0.1), ("H2SO4", 0.001)],
            340.0,
        );
        let e = router().evaluate(&v);
        assert!(e.ready.is_empty() && e.declined.is_empty() && e.refused.is_empty());
        assert!(!router().applies(&v));
    }

    #[test]
    fn a_product_the_registry_cannot_name_is_a_typed_refusal() {
        let mut v = vessel(
            &[("CH3COOH", 0.1), ("methanol", 0.1), ("H2SO4", 0.001)],
            340.0,
        );
        let mut r = router();
        assert!(r.applies(&v), "the router has something to say");
        let events = r.equilibrate(&mut v).unwrap();
        assert_eq!(events.len(), 1);
        assert!(matches!(
            &events[0],
            Event::NotYetModeled { cause: NotModelledCause::PhaseNotInRegistry, what, .. }
                if what.contains("cannot name")
        ));
        // Said once, not once per pass.
        assert!((v.moles_of(&SpeciesId::new("CH3COOH")).0 - 0.1).abs() < 1e-12);
    }

    #[test]
    fn to_completion_consumes_the_limiting_reagent() {
        let mut record = esterification("fake-esterification-complete", 0);
        record.outcome = OutcomeModel::ToCompletion;
        let mut r = FamilyRouter::new(FakeOracle, vec![record]);
        let mut v = vessel(
            &[("CH3COOH", 0.1), ("ethanol", 0.25), ("H2SO4", 0.001)],
            340.0,
        );
        let events = r.equilibrate(&mut v).unwrap();
        assert!(v.moles_of(&SpeciesId::new("CH3COOH")).0 < 1e-12);
        assert!((v.moles_of(&SpeciesId::new("ethanol")).0 - 0.15).abs() < 1e-12);
        assert!(events.iter().any(|e| matches!(
            e,
            Event::OrgReacted { equation, extent, .. }
                if equation.contains("→") && (extent.0 - 0.1).abs() < 1e-12
        )));
    }

    #[test]
    fn the_higher_priority_record_answers_first() {
        let mut fast = esterification("fake-esterification-priority", 5);
        fast.outcome = OutcomeModel::ToCompletion;
        let slow = esterification("fake-esterification", 0);
        let mut r = FamilyRouter::new(FakeOracle, vec![slow, fast]);
        assert_eq!(r.records()[0].id, "fake-esterification-priority");
        let mut v = vessel(
            &[("CH3COOH", 0.1), ("ethanol", 0.1), ("H2SO4", 0.001)],
            340.0,
        );
        let events = r.equilibrate(&mut v).unwrap();
        // The completion record ran and left nothing for the equilibrium
        // record to weigh: exactly one firing.
        let fired: Vec<&String> = events
            .iter()
            .filter_map(|e| match e {
                Event::OrgReacted { name, .. } => Some(name),
                _ => None,
            })
            .collect();
        assert_eq!(fired, vec!["fake-esterification-priority"]);
    }

    #[test]
    fn a_pack_loads_and_a_bad_record_refuses_the_pack() {
        let good = r#"
[[family]]
id = "x"
version = 1
smirks = "[C:1](=[O:2])[OH:3].[OH:4][C:5]>>[C:1](=[O:2])[O:4][C:5].[OH2:3]"
confidence = "curated_family"
provenance = "p"
refusal_domain = "r"
[family.gates]
medium = { organic_solvent = { solvent = "ethanol" } }
[family.gates.catalyst]
any_of = { keys = ["H2SO4", "HCl"] }
[family.outcome.equilibrium]
log_k = 0.6
source = "s"
"#;
        let records = load_records(good).expect("parses and lints");
        assert_eq!(records.len(), 1);
        assert!(matches!(
            records[0].gates.medium,
            Some(MediumGate::OrganicSolvent { ref solvent }) if solvent == "ethanol"
        ));
        assert!(
            matches!(records[0].outcome, OutcomeModel::Equilibrium { log_k, .. } if (log_k - 0.6).abs() < 1e-12)
        );
        let bad = good.replace("refusal_domain = \"r\"", "refusal_domain = \"\"");
        let err = load_records(&bad).expect_err("a boundaryless rule refuses the pack");
        assert!(err.contains("without a boundary"));
    }

    #[test]
    fn reactant_slots_counts_top_level_separators_only() {
        assert_eq!(reactant_slots(""), 0);
        assert_eq!(
            reactant_slots("[C:1][Br:2].[OH-:3]>>[C:1][OH:3].[Br-:2]"),
            2
        );
        assert_eq!(reactant_slots("[CH2:1]=[CH2:2]>>[CH2:1][CH2:2]"), 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal(id: &str, priority: i32, gates: GateSet) -> FamilyRecord {
        FamilyRecord {
            id: id.to_string(),
            version: 1,
            smirks: "[C:1](=[O:2])[OH:3].[OH:4][C:5]>>[C:1](=[O:2])[O:4][C:5].[OH2:3]".to_string(),
            substrates: Vec::new(),
            ledger_reactants: BTreeMap::new(),
            ledger_products: BTreeMap::new(),
            gates,
            priority,
            outcome: OutcomeModel::ToCompletion,
            confidence: FamilyConfidence::CuratedFamily,
            provenance: "test".to_string(),
            refusal_domain: "test boundary".to_string(),
        }
    }

    #[test]
    fn conflict_order_is_total_and_deterministic() {
        let plain = minimal("b-plain", 0, GateSet::default());
        let gated = minimal(
            "a-gated",
            0,
            GateSet {
                medium: Some(MediumGate::Anhydrous),
                ..GateSet::default()
            },
        );
        let prioritised = minimal("z-priority", 1, GateSet::default());
        let mut set = [&plain, &gated, &prioritised];
        set.sort_by(|a, b| conflict_order(a, b));
        // Priority beats specificity beats id.
        assert_eq!(
            set.map(|r| r.id.as_str()),
            ["z-priority", "a-gated", "b-plain"]
        );
        // Reversing the input changes nothing: the order is total.
        let mut reversed = [&prioritised, &gated, &plain];
        reversed.sort_by(|a, b| conflict_order(a, b));
        assert_eq!(set.map(|r| r.id.as_str()), reversed.map(|r| r.id.as_str()));
    }

    #[test]
    fn the_lint_refuses_a_boundaryless_rule() {
        let mut r = minimal("no-boundary", 0, GateSet::default());
        r.refusal_domain = String::new();
        let problems = lint_record(&r);
        assert!(problems.iter().any(|p| p.contains("without a boundary")));
    }

    #[test]
    fn the_lint_refuses_an_instance_without_substrates() {
        let mut r = minimal("instance", 0, GateSet::default());
        r.confidence = FamilyConfidence::CuratedInstance;
        let problems = lint_record(&r);
        assert!(problems.iter().any(|p| p.contains("names none")));
    }

    #[test]
    fn records_round_trip_through_serde() {
        let r = minimal(
            "esterification",
            2,
            GateSet {
                medium: Some(MediumGate::OrganicSolvent {
                    solvent: "ethanol".into(),
                }),
                temperature_k: Some(KelvinWindow {
                    min: Some(298.15),
                    max: None,
                }),
                catalyst: Some(CatalystGate::Species {
                    key: "H2SO4".into(),
                }),
                required_groups: vec!["carboxylic_acid".into(), "alcohol".into()],
                ..GateSet::default()
            },
        );
        let json = serde_json::to_string(&r).unwrap();
        let back: FamilyRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, r.id);
        assert_eq!(back.gates.required_groups, r.gates.required_groups);
        assert!(matches!(
            back.gates.catalyst,
            Some(CatalystGate::Species { ref key }) if key == "H2SO4"
        ));
    }

    #[test]
    fn windows_admit_and_refuse() {
        let w = KelvinWindow {
            min: Some(300.0),
            max: Some(400.0),
        };
        assert!(w.admits(350.0));
        assert!(!w.admits(299.9));
        assert!(!w.admits(400.1));
        assert!(KelvinWindow::default().admits(5.0));
    }
}
