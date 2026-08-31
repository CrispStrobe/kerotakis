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
