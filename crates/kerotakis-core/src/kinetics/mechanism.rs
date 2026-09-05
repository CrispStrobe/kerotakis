//! Runtime mechanism documents and their strict YAML front end.
//!
//! Parsing and execution are deliberately separated. [`ParsedMechanism`]
//! owns untrusted file data and can be inspected without constructing solver
//! state. [`ParsedMechanism::compile_in`] then lowers that validated data into
//! the same borrowed reaction-network IR used by curated kinetics. The caller
//! owns the arena, so runtime mechanisms neither leak memory nor acquire a
//! second evaluator.

use std::collections::{BTreeMap, BTreeSet};

use bumpalo::Bump;
use serde::{Deserialize, Serialize};

use super::{
    ColliderEfficiency, EquilibriumTerm, IdealGasEquilibrium, KineticReaction, Locality,
    Nasa7Thermo, OrderTerm, PressureDependence, PressureRate, Range, RateExpression, RateLaw,
    ReactionNetwork, SiteTerm, StoichiometricTerm, ThirdBody, Troe, Uncertainty, Validity,
};
use crate::species::Phase;

/// Storage for a compiled mechanism network.
///
/// Drop this value after every network borrowed from it. Reusing an arena for
/// several compilations is supported, but retains all of their allocations
/// until the arena itself is dropped.
#[derive(Debug, Default)]
pub struct MechanismArena {
    storage: Bump,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedMechanism {
    name: String,
    species: Vec<MechanismSpecies>,
    reactions: Vec<OwnedReaction>,
    units: ResolvedUnits,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MechanismSummary {
    pub name: String,
    pub species: usize,
    pub reactions: usize,
    pub elements: Vec<String>,
    pub reaction_details: Vec<MechanismReactionSummary>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MechanismReactionSummary {
    pub id: String,
    pub equation: String,
    pub total_order: f64,
    pub pre_exponential: f64,
    pub temperature_exponent: f64,
    pub activation_energy_j_per_mol: f64,
    pub rate_model: String,
    pub low_pressure_pre_exponential: Option<f64>,
    pub pressure_points_pa: Vec<f64>,
    pub reversible: bool,
}

#[derive(Debug, Clone, PartialEq)]
struct MechanismSpecies {
    name: String,
    composition: BTreeMap<String, f64>,
    phase: Phase,
    thermo: Option<Nasa7Thermo>,
}

#[derive(Debug, Clone, PartialEq)]
struct OwnedReaction {
    id: String,
    equation: String,
    stoichiometry: Vec<OwnedTerm>,
    orders: Vec<OwnedOrder>,
    reverse_orders: Option<Vec<OwnedOrder>>,
    rate: RateLaw,
    pressure_dependence: OwnedPressureDependence,
    equilibrium: Vec<OwnedEquilibriumTerm>,
    validity_temperature_k: Option<Range>,
    phase: Phase,
}

#[derive(Debug, Clone, PartialEq)]
struct OwnedEquilibriumTerm {
    species: String,
    coefficient: f64,
    thermo: Nasa7Thermo,
}

#[derive(Debug, Clone, PartialEq)]
enum OwnedPressureDependence {
    None,
    ThirdBody {
        collider: OwnedThirdBody,
    },
    Falloff {
        collider: OwnedThirdBody,
        low_pressure: RateLaw,
        troe: Option<Troe>,
    },
    Plog {
        rates: Vec<PressureRate>,
    },
}

#[derive(Debug, Clone, PartialEq)]
struct OwnedThirdBody {
    default_efficiency: f64,
    efficiencies: Vec<(String, f64)>,
}

#[derive(Debug, Clone, PartialEq)]
struct OwnedTerm {
    species: String,
    coefficient: f64,
    phase: Phase,
}

#[derive(Debug, Clone, PartialEq)]
struct OwnedOrder {
    species: String,
    phase: Phase,
    order: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ResolvedUnits {
    concentration_mol_per_litre: f64,
    seconds: f64,
    activation_j_per_mol: f64,
    pressure_pa: f64,
}

#[derive(Debug, thiserror::Error)]
pub enum MechanismError {
    #[error("invalid mechanism YAML: {0}")]
    Yaml(#[from] serde_yaml_ng::Error),
    #[error("mechanism has no phases")]
    MissingPhases,
    #[error("mechanism has no species")]
    MissingSpecies,
    #[error("mechanism has no reactions")]
    MissingReactions,
    #[error("duplicate {kind} name '{name}'")]
    DuplicateName { kind: &'static str, name: String },
    #[error(
        "phase '{phase}': only ideal-gas kinetics are supported in KIN-006 (got thermo '{thermo}')"
    )]
    UnsupportedPhase { phase: String, thermo: String },
    #[error("phase '{phase}' names unknown species '{species}'")]
    UnknownPhaseSpecies { phase: String, species: String },
    #[error("species '{species}' is not assigned to exactly one phase")]
    UnassignedSpecies { species: String },
    #[error("species '{species}' is claimed by more than one phase")]
    DuplicatePhaseAssignment { species: String },
    #[error("species '{species}' has invalid element count for '{element}': {count}")]
    InvalidComposition {
        species: String,
        element: String,
        count: f64,
    },
    #[error("species '{species}' has invalid NASA7 thermochemistry: {detail}")]
    InvalidThermo { species: String, detail: String },
    #[error("reaction {reaction}: {detail}")]
    InvalidReaction { reaction: usize, detail: String },
    #[error("reaction {reaction}: unsupported reaction type '{kind}'")]
    UnsupportedReactionType { reaction: usize, kind: String },
    #[error("reaction {reaction}: unknown species '{species}'")]
    UnknownReactionSpecies { reaction: usize, species: String },
    #[error("reaction {reaction}: element {element} has net coefficient {imbalance:+.6}")]
    ElementImbalance {
        reaction: usize,
        element: String,
        imbalance: f64,
    },
    #[error("unsupported {kind} unit '{unit}'")]
    UnsupportedUnit { kind: &'static str, unit: String },
    #[error("reaction {reaction}: {field} must be finite and positive (got {value})")]
    InvalidRate {
        reaction: usize,
        field: &'static str,
        value: f64,
    },
    /// BRD-041: `Ea` in a fitted Arrhenius expression is a fitted parameter,
    /// not a barrier height, so it may be negative — barrierless
    /// radical-radical reactions such as `CO + OH -> CO2 + H` genuinely get
    /// faster as they cool. BRD-040 counted four negative activation energies
    /// in Cantera's own `h2o2.yaml` and thirty-two in `gri30.yaml`, and
    /// recommended (§7, item 3) that the `Ea >= 0` guard become a finiteness
    /// check. It has. What is still refused is a value that is not a number:
    /// a NaN activation energy makes every rate downstream of it NaN, and
    /// silently.
    #[error("reaction {reaction}: Ea must be finite (got {value})")]
    NonFiniteActivationEnergy { reaction: usize, value: f64 },
    /// A Cantera YAML key the portable subset does not model.
    ///
    /// Every such key is refused rather than skipped: silently dropping a field
    /// that changes a rate law, a reaction order, or a unit scale would answer
    /// a different question than the file asks.
    #[error("{owner}: unsupported Cantera field '{field}' (BRD-040: not modelled by the portable subset)")]
    UnsupportedField { owner: String, field: String },
    /// A named top-level document section the portable subset cannot attribute
    /// to a phase, so it cannot know whether its contents are in play.
    #[error("unsupported Cantera document section '{section}' (BRD-040: only description/units/phases/species/reactions and ck2yaml provenance keys are modelled)")]
    UnsupportedSection { section: String },
    /// A supported key carrying a value outside the portable subset.
    #[error("{owner}: unsupported value '{value}' for Cantera field '{field}' (BRD-040)")]
    UnsupportedFieldValue {
        owner: String,
        field: &'static str,
        value: String,
    },
}

/// YAML keys the portable subset may ignore without changing any answer.
///
/// Everything outside these lists is refused by [`reject_unknown_fields`]. The
/// asymmetry is deliberate: an allowlist of provenance/annotation keys is
/// auditable, whereas serde's default of dropping unknown keys silently would
/// let a future Cantera rate modifier change a mechanism's meaning unnoticed.
const IGNORABLE_DOCUMENT_KEYS: &[&str] = &[
    "generator",
    "input-files",
    "cantera-version",
    "date",
    "note",
    "references",
    "elements",
];
const IGNORABLE_PHASE_KEYS: &[&str] = &[
    "elements",
    "note",
    "state",
    "transport",
    "adjacent-phases",
    "skip-undeclared-elements",
    "skip-undeclared-third-bodies",
    "explicit-third-body-duplicates",
];
/// Species keys that cannot reach an ideal-gas rate. Transport data is never
/// read by the kinetics path. `equation-of-state` carries real-gas parameters
/// (Redlich-Kwong coefficients in Cantera's own `h2o2.yaml` and
/// `nDodecane_Reitz.yaml`) that Cantera itself ignores for an `ideal-gas`
/// phase, and only `ideal-gas` phases are accepted here.
const IGNORABLE_SPECIES_KEYS: &[&str] = &[
    "note",
    "transport",
    "critical-parameters",
    "equation-of-state",
];
const IGNORABLE_THERMO_KEYS: &[&str] = &["note"];
/// `duplicate` is an assertion, not a rate modifier: Cantera keeps duplicate
/// reactions separate and sums their rates, which is exactly what compiling
/// each entry independently already does.
const IGNORABLE_REACTION_KEYS: &[&str] = &["note", "id", "duplicate"];

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct RawMechanism {
    #[serde(default)]
    description: String,
    #[serde(default)]
    units: RawUnits,
    #[serde(default)]
    phases: Vec<RawPhase>,
    #[serde(default)]
    species: Vec<RawSpecies>,
    #[serde(default)]
    reactions: Vec<RawReaction>,
    #[serde(flatten)]
    extra: BTreeMap<String, serde_yaml_ng::Value>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct RawUnits {
    length: Option<String>,
    time: Option<String>,
    quantity: Option<String>,
    energy: Option<String>,
    activation_energy: Option<String>,
    pressure: Option<String>,
    temperature: Option<String>,
    #[serde(flatten)]
    extra: BTreeMap<String, serde_yaml_ng::Value>,
}

#[derive(Debug, Deserialize)]
struct RawPhase {
    name: String,
    thermo: String,
    /// Cantera accepts a species list, the string `all` (also the default when
    /// the key is absent), or cross-file section references. The reference
    /// forms are refused by name rather than through a serde type error.
    #[serde(default)]
    species: Option<serde_yaml_ng::Value>,
    #[serde(default)]
    kinetics: Option<String>,
    #[serde(default)]
    reactions: Option<serde_yaml_ng::Value>,
    #[serde(flatten)]
    extra: BTreeMap<String, serde_yaml_ng::Value>,
}

#[derive(Debug, Deserialize)]
struct RawSpecies {
    name: String,
    composition: BTreeMap<String, f64>,
    #[serde(default)]
    thermo: Option<RawThermo>,
    #[serde(flatten)]
    extra: BTreeMap<String, serde_yaml_ng::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct RawThermo {
    model: String,
    temperature_ranges: Vec<f64>,
    data: Vec<Vec<f64>>,
    #[serde(default)]
    reference_pressure: Option<Scalar>,
    #[serde(flatten)]
    extra: BTreeMap<String, serde_yaml_ng::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct RawReaction {
    equation: String,
    #[serde(default = "elementary")]
    r#type: String,
    #[serde(default)]
    rate_constant: Option<RawRate>,
    #[serde(default, rename = "high-P-rate-constant")]
    high_p_rate_constant: Option<RawRate>,
    #[serde(default, rename = "low-P-rate-constant")]
    low_p_rate_constant: Option<RawRate>,
    #[serde(default)]
    rate_constants: Vec<RawPressureRate>,
    #[serde(default)]
    efficiencies: BTreeMap<String, f64>,
    #[serde(default = "unit_efficiency")]
    default_efficiency: f64,
    #[serde(default, rename = "Troe")]
    troe: Option<RawTroe>,
    /// BRD-041: explicit reaction orders, replacing the exponents the
    /// equation would otherwise imply. A global step is a curve fit to a
    /// flame rather than a molecular event, and its orders are measured
    /// separately from its stoichiometry — Westbrook and Dryer's methane
    /// fit is first order overall while the equation says third.
    #[serde(default)]
    orders: BTreeMap<String, f64>,
    /// Cantera's acknowledgement flag for an order below zero. A negative
    /// order says the fuel INHIBITS its own consumption, which is a real
    /// and well-documented feature of global hydrocarbon fits and a
    /// spectacular typo if it was not meant, so it must be declared.
    #[serde(default)]
    negative_orders: bool,
    #[serde(flatten)]
    extra: BTreeMap<String, serde_yaml_ng::Value>,
}

/// Resolve a phase's `species` selector into concrete names.
///
/// Cantera's default (key absent) and the explicit string `all` both mean every
/// species declared in the document. Cross-file and per-section selectors are
/// refused by name.
fn phase_species(
    owner: &str,
    selector: Option<&serde_yaml_ng::Value>,
    declared: &BTreeSet<String>,
) -> Result<Vec<String>, MechanismError> {
    let unsupported = |value: &serde_yaml_ng::Value| MechanismError::UnsupportedFieldValue {
        owner: owner.to_string(),
        field: "species",
        value: render_yaml_value(value),
    };
    match selector {
        None => Ok(declared.iter().cloned().collect()),
        Some(value) if value.as_str() == Some("all") => Ok(declared.iter().cloned().collect()),
        Some(value) => {
            let items = value.as_sequence().ok_or_else(|| unsupported(value))?;
            items
                .iter()
                .map(|item| {
                    item.as_str()
                        .map(ToString::to_string)
                        .ok_or_else(|| unsupported(item))
                })
                .collect()
        }
    }
}

/// Refuse any key outside the modelled set, naming the offending field.
fn reject_unknown_fields(
    owner: &str,
    extra: &BTreeMap<String, serde_yaml_ng::Value>,
    ignorable: &[&str],
) -> Result<(), MechanismError> {
    for field in extra.keys() {
        if !ignorable.contains(&field.as_str()) {
            return Err(MechanismError::UnsupportedField {
                owner: owner.to_string(),
                field: field.clone(),
            });
        }
    }
    Ok(())
}

fn elementary() -> String {
    "elementary".to_string()
}

const fn unit_efficiency() -> f64 {
    1.0
}

#[derive(Debug, Deserialize)]
struct RawRate {
    /// Cantera also accepts a unit-bearing string here (`A: 1.0e12 cm^3/mol/s`).
    /// Accepting the scalar form and typing the string form keeps the failure
    /// legible instead of surfacing serde's "invalid type: string".
    #[serde(rename = "A")]
    pre_exponential: Scalar,
    #[serde(default)]
    b: f64,
    #[serde(rename = "Ea")]
    activation_energy: Scalar,
}

#[derive(Debug, Deserialize)]
struct RawPressureRate {
    #[serde(rename = "P")]
    pressure: Scalar,
    #[serde(flatten)]
    rate: RawRate,
}

#[derive(Debug, Deserialize)]
struct RawTroe {
    #[serde(rename = "A")]
    a: f64,
    #[serde(rename = "T3")]
    t3: f64,
    #[serde(rename = "T1")]
    t1: f64,
    #[serde(default, rename = "T2")]
    t2: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReactionKind {
    Elementary,
    ThirdBody,
    Falloff,
    Plog,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Scalar {
    Number(f64),
    Text(String),
}

/// Parse and validate the KIN-006 portable mechanism subset.
pub fn parse_yaml(text: &str) -> Result<ParsedMechanism, MechanismError> {
    let raw: RawMechanism = serde_yaml_ng::from_str(text)?;
    for section in raw.extra.keys() {
        if !IGNORABLE_DOCUMENT_KEYS.contains(&section.as_str()) {
            return Err(MechanismError::UnsupportedSection {
                section: section.clone(),
            });
        }
    }
    if raw.phases.is_empty() {
        return Err(MechanismError::MissingPhases);
    }
    if raw.species.is_empty() {
        return Err(MechanismError::MissingSpecies);
    }
    if raw.reactions.is_empty() {
        return Err(MechanismError::MissingReactions);
    }

    let units = ResolvedUnits::new(&raw.units)?;
    let mut names = BTreeSet::new();
    for species in &raw.species {
        reject_unknown_fields(
            &format!("species '{}'", species.name),
            &species.extra,
            IGNORABLE_SPECIES_KEYS,
        )?;
        if let Some(thermo) = &species.thermo {
            reject_unknown_fields(
                &format!("species '{}' thermo", species.name),
                &thermo.extra,
                IGNORABLE_THERMO_KEYS,
            )?;
        }
        if !names.insert(species.name.clone()) {
            return Err(MechanismError::DuplicateName {
                kind: "species",
                name: species.name.clone(),
            });
        }
        for (element, count) in &species.composition {
            if element.is_empty() || !count.is_finite() || *count <= 0.0 {
                return Err(MechanismError::InvalidComposition {
                    species: species.name.clone(),
                    element: element.clone(),
                    count: *count,
                });
            }
        }
    }

    let mut assignments = BTreeMap::new();
    for phase in &raw.phases {
        let owner = format!("phase '{}'", phase.name);
        // The phase model is checked first so that a surface or edge phase is
        // named as such, rather than tripping over whichever of its own keys
        // (`site-density`, `adjacent-phases`) happens to be inspected first.
        if phase.thermo != "ideal-gas" {
            return Err(MechanismError::UnsupportedPhase {
                phase: phase.name.clone(),
                thermo: phase.thermo.clone(),
            });
        }
        reject_unknown_fields(&owner, &phase.extra, IGNORABLE_PHASE_KEYS)?;
        // A phase may restrict which reaction sections apply to it. Only the
        // Cantera default (`all`) can be honoured without modelling section
        // selection, so `none`, `declared-species` and section lists are
        // refused rather than quietly compiling every reaction in the file.
        if let Some(selector) = &phase.reactions {
            let all = selector.as_str().is_some_and(|value| value == "all");
            if !all {
                return Err(MechanismError::UnsupportedFieldValue {
                    owner: owner.clone(),
                    field: "reactions",
                    value: render_yaml_value(selector),
                });
            }
        }
        if let Some(kinetics) = phase.kinetics.as_deref() {
            if kinetics != "gas" && kinetics != "bulk" {
                return Err(MechanismError::UnsupportedFieldValue {
                    owner: owner.clone(),
                    field: "kinetics",
                    value: kinetics.to_string(),
                });
            }
        }
        let members = phase_species(&owner, phase.species.as_ref(), &names)?;
        for species in members {
            if !names.contains(&species) {
                return Err(MechanismError::UnknownPhaseSpecies {
                    phase: phase.name.clone(),
                    species: species.clone(),
                });
            }
            if assignments.insert(species.clone(), Phase::Gas).is_some() {
                return Err(MechanismError::DuplicatePhaseAssignment { species });
            }
        }
    }

    let species: Vec<_> = raw
        .species
        .into_iter()
        .map(|entry| {
            let phase = assignments.get(&entry.name).copied().ok_or_else(|| {
                MechanismError::UnassignedSpecies {
                    species: entry.name.clone(),
                }
            })?;
            let thermo = entry
                .thermo
                .as_ref()
                .map(|raw| validate_nasa7(&entry.name, raw))
                .transpose()?;
            Ok(MechanismSpecies {
                name: entry.name,
                composition: entry.composition,
                phase,
                thermo,
            })
        })
        .collect::<Result<_, MechanismError>>()?;
    let species_by_name: BTreeMap<_, _> = species
        .iter()
        .map(|species| (species.name.as_str(), species))
        .collect();

    let mut reactions = Vec::with_capacity(raw.reactions.len());
    for (offset, reaction) in raw.reactions.into_iter().enumerate() {
        let number = offset + 1;
        reject_unknown_fields(
            &format!("reaction {number}"),
            &reaction.extra,
            IGNORABLE_REACTION_KEYS,
        )?;
        let kind = match reaction.r#type.as_str() {
            "elementary" => ReactionKind::Elementary,
            "three-body" => ReactionKind::ThirdBody,
            "falloff" => ReactionKind::Falloff,
            "pressure-dependent-Arrhenius" => ReactionKind::Plog,
            _ => {
                return Err(MechanismError::UnsupportedReactionType {
                    reaction: number,
                    kind: reaction.r#type,
                })
            }
        };
        let equation = parse_equation(&reaction.equation, number)?;
        if equation.reversible && kind != ReactionKind::Elementary {
            return Err(MechanismError::InvalidReaction {
                reaction: number,
                detail: "reversible pressure-dependent reactions are not supported yet".to_string(),
            });
        }
        let reversible = equation.reversible;
        match (kind, equation.has_collider) {
            (ReactionKind::Elementary | ReactionKind::Plog, true) => {
                return Err(MechanismError::InvalidReaction {
                    reaction: number,
                    detail: "elementary reaction contains a third-body marker".to_string(),
                })
            }
            (ReactionKind::ThirdBody | ReactionKind::Falloff, false) => {
                return Err(MechanismError::InvalidReaction {
                    reaction: number,
                    detail: "pressure-dependent reaction is missing M or (+M)".to_string(),
                })
            }
            _ => {}
        }
        // Mass-action orders follow the coefficients written on each SIDE of the
        // equation, not the net stoichiometry. `H + 2 O2 <=> HO2 + O2` is second
        // order in O2 even though O2's net coefficient is -1; deriving orders
        // from the net vector silently produced a second-order rate law with a
        // third-order pre-exponential, mis-scaling A by one concentration unit.
        let mut reactant_totals = BTreeMap::<String, f64>::new();
        for (name, coefficient) in equation.reactants {
            if !species_by_name.contains_key(name.as_str()) {
                return Err(MechanismError::UnknownReactionSpecies {
                    reaction: number,
                    species: name,
                });
            }
            *reactant_totals.entry(name).or_default() += coefficient;
        }
        let mut product_totals = BTreeMap::<String, f64>::new();
        for (name, coefficient) in equation.products {
            if !species_by_name.contains_key(name.as_str()) {
                return Err(MechanismError::UnknownReactionSpecies {
                    reaction: number,
                    species: name,
                });
            }
            *product_totals.entry(name).or_default() += coefficient;
        }
        let mut stoichiometry = BTreeMap::<String, f64>::new();
        for (name, coefficient) in &reactant_totals {
            *stoichiometry.entry(name.clone()).or_default() -= *coefficient;
        }
        for (name, coefficient) in &product_totals {
            *stoichiometry.entry(name.clone()).or_default() += *coefficient;
        }
        stoichiometry.retain(|_, coefficient| coefficient.abs() > 1e-14);
        if stoichiometry.is_empty() {
            return Err(MechanismError::InvalidReaction {
                reaction: number,
                detail: "reaction has no net stoichiometric change".to_string(),
            });
        }
        validate_balance(number, &stoichiometry, &species_by_name)?;

        let side_orders = |totals: &BTreeMap<String, f64>| {
            totals
                .iter()
                .map(|(name, coefficient)| OwnedOrder {
                    species: name.clone(),
                    phase: species_by_name[name.as_str()].phase,
                    order: *coefficient,
                })
                .collect::<Vec<_>>()
        };
        let mut orders = side_orders(&reactant_totals);
        if reaction.negative_orders && reaction.orders.is_empty() {
            return Err(MechanismError::InvalidReaction {
                reaction: number,
                detail: "negative-orders is declared without any orders to relax".to_string(),
            });
        }
        if !reaction.orders.is_empty() && (kind != ReactionKind::Elementary || reversible) {
            return Err(MechanismError::InvalidReaction {
                reaction: number,
                detail: "explicit orders are modelled only for irreversible elementary reactions"
                    .to_string(),
            });
        }
        apply_explicit_orders(
            number,
            &mut orders,
            &reaction.orders,
            reaction.negative_orders,
        )?;
        let reverse_orders = reversible.then(|| side_orders(&product_totals));
        let (equilibrium, validity_temperature_k) = if reversible {
            let mut min_temperature_k: f64 = 0.0;
            let mut max_temperature_k = f64::INFINITY;
            let mut terms = Vec::with_capacity(stoichiometry.len());
            for (name, coefficient) in &stoichiometry {
                let thermo = species_by_name[name.as_str()].thermo.ok_or_else(|| {
                    MechanismError::InvalidReaction {
                        reaction: number,
                        detail: format!(
                            "reversible reaction species '{name}' is missing NASA7 thermochemistry"
                        ),
                    }
                })?;
                min_temperature_k = min_temperature_k.max(thermo.min_temperature_k);
                max_temperature_k = max_temperature_k.min(thermo.max_temperature_k);
                terms.push(OwnedEquilibriumTerm {
                    species: name.clone(),
                    coefficient: *coefficient,
                    thermo,
                });
            }
            if min_temperature_k > max_temperature_k {
                return Err(MechanismError::InvalidReaction {
                    reaction: number,
                    detail: "reversible species have no shared NASA7 temperature range".to_string(),
                });
            }
            (
                terms,
                Some(Range {
                    min: min_temperature_k,
                    max: max_temperature_k,
                }),
            )
        } else {
            (Vec::new(), None)
        };
        let total_order: f64 = orders.iter().map(|order| order.order).sum();
        if kind != ReactionKind::Plog && !reaction.rate_constants.is_empty() {
            return Err(MechanismError::InvalidReaction {
                reaction: number,
                detail: "rate-constants requires type pressure-dependent-Arrhenius".to_string(),
            });
        }
        let collider = validate_collider(
            number,
            reaction.default_efficiency,
            reaction.efficiencies,
            &species_by_name,
        )?;
        let (rate, pressure_dependence) = match kind {
            ReactionKind::Elementary => {
                if collider.is_some() || reaction.troe.is_some() {
                    return Err(MechanismError::InvalidReaction {
                        reaction: number,
                        detail: "elementary reaction declares collider/falloff parameters"
                            .to_string(),
                    });
                }
                let raw_rate = reaction
                    .rate_constant
                    .as_ref()
                    .ok_or_else(|| missing_rate(number, "rate-constant"))?;
                (
                    normalize_rate(raw_rate, total_order, units, number)?,
                    OwnedPressureDependence::None,
                )
            }
            ReactionKind::ThirdBody => {
                let raw_rate = reaction
                    .rate_constant
                    .as_ref()
                    .ok_or_else(|| missing_rate(number, "rate-constant"))?;
                (
                    normalize_rate(raw_rate, total_order + 1.0, units, number)?,
                    OwnedPressureDependence::ThirdBody {
                        collider: collider.unwrap_or_else(default_collider),
                    },
                )
            }
            ReactionKind::Falloff => {
                let high = reaction
                    .high_p_rate_constant
                    .as_ref()
                    .ok_or_else(|| missing_rate(number, "high-P-rate-constant"))?;
                let low = reaction
                    .low_p_rate_constant
                    .as_ref()
                    .ok_or_else(|| missing_rate(number, "low-P-rate-constant"))?;
                let troe = reaction
                    .troe
                    .map(|parameters| validate_troe(number, parameters))
                    .transpose()?;
                let high = normalize_rate(high, total_order, units, number)?;
                let low = normalize_rate(low, total_order + 1.0, units, number)?;
                (
                    high,
                    OwnedPressureDependence::Falloff {
                        collider: collider.unwrap_or_else(default_collider),
                        low_pressure: low,
                        troe,
                    },
                )
            }
            ReactionKind::Plog => {
                if collider.is_some()
                    || reaction.troe.is_some()
                    || reaction.rate_constant.is_some()
                    || reaction.high_p_rate_constant.is_some()
                    || reaction.low_p_rate_constant.is_some()
                {
                    return Err(MechanismError::InvalidReaction {
                        reaction: number,
                        detail:
                            "pressure-dependent-Arrhenius reaction declares incompatible rate fields"
                                .to_string(),
                    });
                }
                if reaction.rate_constants.is_empty() {
                    return Err(missing_rate(number, "rate-constants"));
                }
                let mut rates = reaction
                    .rate_constants
                    .iter()
                    .map(|entry| {
                        Ok(PressureRate {
                            pressure_pa: parse_reaction_pressure(
                                &entry.pressure,
                                units.pressure_pa,
                                number,
                            )?,
                            arrhenius: normalize_rate(&entry.rate, total_order, units, number)?,
                        })
                    })
                    .collect::<Result<Vec<_>, MechanismError>>()?;
                rates.sort_by(|left, right| left.pressure_pa.total_cmp(&right.pressure_pa));
                if rates
                    .windows(2)
                    .all(|pair| pair[0].pressure_pa == pair[1].pressure_pa)
                {
                    return Err(MechanismError::InvalidReaction {
                        reaction: number,
                        detail:
                            "rate-constants requires at least two distinct interpolation pressures"
                                .to_string(),
                    });
                }
                (rates[0].arrhenius, OwnedPressureDependence::Plog { rates })
            }
        };

        let terms = stoichiometry
            .into_iter()
            .map(|(name, coefficient)| OwnedTerm {
                phase: species_by_name[name.as_str()].phase,
                species: name,
                coefficient,
            })
            .collect();
        reactions.push(OwnedReaction {
            id: format!("reaction-{number}"),
            equation: reaction.equation,
            stoichiometry: terms,
            orders,
            reverse_orders,
            rate,
            pressure_dependence,
            equilibrium,
            validity_temperature_k,
            phase: Phase::Gas,
        });
    }

    Ok(ParsedMechanism {
        name: if raw.description.trim().is_empty() {
            "runtime-mechanism".to_string()
        } else {
            raw.description
        },
        species,
        reactions,
        units,
    })
}

impl ParsedMechanism {
    /// Species names admitted by this validated mechanism, in document order.
    pub fn species_names(&self) -> impl ExactSizeIterator<Item = &str> {
        self.species.iter().map(|species| species.name.as_str())
    }

    /// Call one species by another name everywhere the mechanism uses it:
    /// the species list, every stoichiometric term, every order, every
    /// equilibrium term and every third-body efficiency. Returns how many
    /// places changed.
    ///
    /// A pack is written in the formulas its evaluation used — `CH4`,
    /// `H2O` — and the bench keeps its ledger by registry key — `methane`,
    /// `water`. The two have to agree before a network can read a vessel
    /// or write products into it, and the honest place to reconcile them
    /// is here, on the validated document, not by editing the pack text
    /// the tests and the provenance point at. The `equation` strings are
    /// left as written: they are what a learner reads.
    pub fn rename_species(&mut self, from: &str, to: &str) -> usize {
        let mut changed = 0usize;
        let mut rename = |name: &mut String| {
            if name == from {
                *name = to.to_string();
                changed += 1;
            }
        };
        for species in &mut self.species {
            rename(&mut species.name);
        }
        for reaction in &mut self.reactions {
            for term in &mut reaction.stoichiometry {
                rename(&mut term.species);
            }
            for order in &mut reaction.orders {
                rename(&mut order.species);
            }
            if let Some(orders) = &mut reaction.reverse_orders {
                for order in orders {
                    rename(&mut order.species);
                }
            }
            for term in &mut reaction.equilibrium {
                rename(&mut term.species);
            }
            match &mut reaction.pressure_dependence {
                OwnedPressureDependence::ThirdBody { collider }
                | OwnedPressureDependence::Falloff { collider, .. } => {
                    for (name, _) in &mut collider.efficiencies {
                        rename(name);
                    }
                }
                OwnedPressureDependence::None | OwnedPressureDependence::Plog { .. } => {}
            }
        }
        changed
    }

    pub fn summary(&self) -> MechanismSummary {
        let elements = self
            .species
            .iter()
            .flat_map(|species| species.composition.keys().cloned())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        MechanismSummary {
            name: self.name.clone(),
            species: self.species.len(),
            reactions: self.reactions.len(),
            elements,
            reaction_details: self
                .reactions
                .iter()
                .map(|reaction| MechanismReactionSummary {
                    id: reaction.id.clone(),
                    equation: reaction.equation.clone(),
                    total_order: reaction.orders.iter().map(|order| order.order).sum(),
                    pre_exponential: reaction.rate.pre_exponential,
                    temperature_exponent: reaction.rate.temperature_exponent,
                    activation_energy_j_per_mol: reaction.rate.activation_energy,
                    rate_model: match &reaction.pressure_dependence {
                        OwnedPressureDependence::None => "elementary",
                        OwnedPressureDependence::ThirdBody { .. } => "three_body",
                        OwnedPressureDependence::Falloff { troe: Some(_), .. } => "troe",
                        OwnedPressureDependence::Falloff { troe: None, .. } => "lindemann",
                        OwnedPressureDependence::Plog { .. } => "pressure_dependent_arrhenius",
                    }
                    .to_string(),
                    low_pressure_pre_exponential: match &reaction.pressure_dependence {
                        OwnedPressureDependence::Falloff { low_pressure, .. } => {
                            Some(low_pressure.pre_exponential)
                        }
                        _ => None,
                    },
                    pressure_points_pa: match &reaction.pressure_dependence {
                        OwnedPressureDependence::Plog { rates } => unique_pressures(rates),
                        _ => Vec::new(),
                    },
                    reversible: reaction.reverse_orders.is_some(),
                })
                .collect(),
        }
    }

    /// Lower this validated document into the runtime reaction-network IR.
    pub fn compile_in<'a>(&self, arena: &'a MechanismArena) -> ReactionNetwork<'a> {
        let provenance = arena
            .storage
            .alloc_str("runtime mechanism YAML; parameters supplied by the loaded document");
        let note = arena
            .storage
            .alloc_str("runtime mechanism validity, including shared thermochemistry range");
        let reactions = self.reactions.iter().map(|reaction| {
            let stoichiometry =
                arena
                    .storage
                    .alloc_slice_fill_iter(reaction.stoichiometry.iter().map(|term| {
                        StoichiometricTerm {
                            species: arena.storage.alloc_str(&term.species),
                            coefficient: term.coefficient,
                            phase: term.phase,
                        }
                    }));
            let orders = arena
                .storage
                .alloc_slice_fill_iter(reaction.orders.iter().map(|term| OrderTerm {
                    species: arena.storage.alloc_str(&term.species),
                    phase: Some(term.phase),
                    order: term.order,
                }));
            let reverse = reaction.reverse_orders.as_ref().map(|terms| {
                let orders = arena
                    .storage
                    .alloc_slice_fill_iter(terms.iter().map(|term| OrderTerm {
                        species: arena.storage.alloc_str(&term.species),
                        phase: Some(term.phase),
                        order: term.order,
                    }));
                RateExpression {
                    arrhenius: reaction.rate,
                    orders,
                }
            });
            let equilibrium = (!reaction.equilibrium.is_empty()).then(|| {
                let terms = arena
                    .storage
                    .alloc_slice_fill_iter(reaction.equilibrium.iter().map(|term| {
                        EquilibriumTerm {
                            species: arena.storage.alloc_str(&term.species),
                            coefficient: term.coefficient,
                            thermo: term.thermo,
                        }
                    }));
                IdealGasEquilibrium { terms }
            });
            let pressure_dependence = match &reaction.pressure_dependence {
                OwnedPressureDependence::None => None,
                OwnedPressureDependence::ThirdBody { collider } => {
                    Some(PressureDependence::ThirdBody {
                        collider: compile_collider(collider, arena),
                    })
                }
                OwnedPressureDependence::Falloff {
                    collider,
                    low_pressure,
                    troe,
                } => Some(PressureDependence::Falloff {
                    collider: compile_collider(collider, arena),
                    low_pressure: *low_pressure,
                    troe: *troe,
                }),
                OwnedPressureDependence::Plog { rates } => {
                    let rates = arena.storage.alloc_slice_copy(rates);
                    Some(PressureDependence::Plog { rates })
                }
            };
            KineticReaction {
                id: arena.storage.alloc_str(&reaction.id),
                equation: arena.storage.alloc_str(&reaction.equation),
                stoichiometry,
                locality: Locality::Bulk(reaction.phase),
                forward: RateExpression {
                    arrhenius: reaction.rate,
                    orders,
                },
                reverse,
                equilibrium,
                pressure_dependence,
                catalysts: &[],
                sites: &[] as &[SiteTerm<'_>],
                electrons: 0.0,
                validity: Validity {
                    temperature_k: reaction.validity_temperature_k,
                    pressure_pa: None,
                    note,
                },
                uncertainty: Uncertainty {
                    relative: None,
                    note: "uncertainty not declared by the loaded mechanism",
                },
                source_ids: &[],
                provenance,
            }
        });
        let reactions = arena.storage.alloc_slice_fill_iter(reactions);
        ReactionNetwork {
            id: arena.storage.alloc_str(&self.name),
            reactions,
        }
    }

    pub fn concentration_unit_mol_per_litre(&self) -> f64 {
        self.units.concentration_mol_per_litre
    }
}

fn unique_pressures(rates: &[PressureRate]) -> Vec<f64> {
    let mut pressures = rates
        .iter()
        .map(|rate| rate.pressure_pa)
        .collect::<Vec<_>>();
    pressures.dedup();
    pressures
}

fn compile_collider<'a>(collider: &OwnedThirdBody, arena: &'a MechanismArena) -> ThirdBody<'a> {
    let efficiencies = arena
        .storage
        .alloc_slice_fill_iter(collider.efficiencies.iter().map(|(species, efficiency)| {
            ColliderEfficiency {
                species: arena.storage.alloc_str(species),
                efficiency: *efficiency,
            }
        }));
    ThirdBody {
        default_efficiency: collider.default_efficiency,
        efficiencies,
    }
}

impl ResolvedUnits {
    fn new(raw: &RawUnits) -> Result<Self, MechanismError> {
        reject_unknown_fields("units", &raw.extra, &["mass", "current"])?;
        // Cantera accepts a `temperature` default but rejects any scale with a
        // non-unity conversion from kelvin, so anything but K is refused here.
        if let Some(temperature) = raw.temperature.as_deref() {
            if temperature.trim() != "K" {
                return Err(unsupported("temperature", temperature));
            }
        }
        let volume_litres: f64 = match raw.length.as_deref().unwrap_or("m") {
            "m" => 1_000.0,
            "cm" => 1.0e-3,
            "mm" => 1.0e-6,
            unit => return Err(unsupported("length", unit)),
        };
        let quantity_moles = match raw.quantity.as_deref().unwrap_or("kmol") {
            "kmol" => 1_000.0,
            "mol" => 1.0,
            unit => return Err(unsupported("quantity", unit)),
        };
        let seconds = match raw.time.as_deref().unwrap_or("s") {
            "s" => 1.0,
            "ms" => 1.0e-3,
            "min" => 60.0,
            unit => return Err(unsupported("time", unit)),
        };
        // Cantera: "Setting default units for `energy` and `quantity` will
        // determine the default units of `activation-energy`, which can be
        // overridden by explicitly giving the desired units". Defaulting to a
        // fixed J/kmol instead misread `units: {quantity: mol}` — a very common
        // ck2yaml output shape — by a factor of one thousand.
        let energy_joules = match raw.energy.as_deref().unwrap_or("J") {
            "J" => 1.0,
            "kJ" => 1_000.0,
            "cal" => 4.184,
            "kcal" => 4_184.0,
            unit => return Err(unsupported("energy", unit)),
        };
        let activation_j_per_mol = match raw.activation_energy.as_deref() {
            Some(unit) => activation_unit(unit)?,
            None => energy_joules / quantity_moles,
        };
        let pressure_pa = pressure_unit(raw.pressure.as_deref().unwrap_or("Pa"))?;
        Ok(Self {
            concentration_mol_per_litre: quantity_moles / volume_litres,
            seconds,
            activation_j_per_mol,
            pressure_pa,
        })
    }
}

/// Render a rejected YAML value compactly so the error names what was refused.
fn render_yaml_value(value: &serde_yaml_ng::Value) -> String {
    value.as_str().map_or_else(
        || {
            serde_yaml_ng::to_string(value)
                .unwrap_or_else(|_| "<unprintable>".to_string())
                .trim()
                .replace('\n', " ")
        },
        ToString::to_string,
    )
}

fn unsupported(kind: &'static str, unit: &str) -> MechanismError {
    MechanismError::UnsupportedUnit {
        kind,
        unit: unit.to_string(),
    }
}

fn activation_unit(unit: &str) -> Result<f64, MechanismError> {
    match unit.trim() {
        "J/mol" => Ok(1.0),
        "J/kmol" => Ok(1.0e-3),
        "kJ/mol" => Ok(1_000.0),
        "cal/mol" => Ok(4.184),
        "kcal/mol" => Ok(4_184.0),
        "K" => Ok(crate::constants::GAS_CONSTANT),
        unit => Err(unsupported("activation-energy", unit)),
    }
}

fn pressure_unit(unit: &str) -> Result<f64, MechanismError> {
    match unit.trim() {
        "Pa" => Ok(1.0),
        "kPa" => Ok(1_000.0),
        "MPa" => Ok(1_000_000.0),
        "bar" => Ok(100_000.0),
        "atm" => Ok(101_325.0),
        unit => Err(unsupported("pressure", unit)),
    }
}

fn thermo_error(species: &str, detail: impl Into<String>) -> MechanismError {
    MechanismError::InvalidThermo {
        species: species.to_string(),
        detail: detail.into(),
    }
}

fn parse_reference_pressure(species: &str, value: Option<&Scalar>) -> Result<f64, MechanismError> {
    let pressure = match value {
        None => 101_325.0,
        Some(Scalar::Number(value)) => *value,
        Some(Scalar::Text(text)) => {
            let (number, unit) = text.trim().split_once(char::is_whitespace).ok_or_else(|| {
                thermo_error(species, format!("invalid reference pressure '{text}'"))
            })?;
            let number = number.parse::<f64>().map_err(|_| {
                thermo_error(species, format!("invalid reference pressure '{text}'"))
            })?;
            number
                * match unit.trim() {
                    "Pa" => 1.0,
                    "kPa" => 1_000.0,
                    "bar" => 100_000.0,
                    "atm" => 101_325.0,
                    _ => {
                        return Err(thermo_error(
                            species,
                            format!("unsupported reference-pressure unit '{}'", unit.trim()),
                        ))
                    }
                }
        }
    };
    if pressure.is_finite() && pressure > 0.0 {
        Ok(pressure)
    } else {
        Err(thermo_error(
            species,
            format!("reference pressure must be finite and positive (got {pressure})"),
        ))
    }
}

fn nasa_coefficients(species: &str, row: &[f64]) -> Result<[f64; 7], MechanismError> {
    if row.len() != 7 || row.iter().any(|coefficient| !coefficient.is_finite()) {
        return Err(thermo_error(
            species,
            "each NASA7 data row must contain seven finite coefficients",
        ));
    }
    Ok(row.try_into().expect("NASA7 row length was checked"))
}

fn validate_nasa7(species: &str, raw: &RawThermo) -> Result<Nasa7Thermo, MechanismError> {
    if raw.model != "NASA7" {
        return Err(thermo_error(
            species,
            format!("unsupported model '{}'", raw.model),
        ));
    }
    let (min_temperature_k, midpoint_temperature_k, max_temperature_k, low, high) =
        match (raw.temperature_ranges.as_slice(), raw.data.as_slice()) {
            ([min, max], [coefficients]) => (*min, *max, *max, coefficients, coefficients),
            ([min, midpoint, max], [low, high]) => (*min, *midpoint, *max, low, high),
            _ => {
                return Err(thermo_error(
                    species,
                    "NASA7 requires two temperature bounds and one data row, or three bounds and two data rows",
                ))
            }
        };
    if !min_temperature_k.is_finite()
        || !midpoint_temperature_k.is_finite()
        || !max_temperature_k.is_finite()
        || min_temperature_k <= 0.0
        || min_temperature_k >= midpoint_temperature_k
        || midpoint_temperature_k > max_temperature_k
        || (raw.data.len() == 2 && midpoint_temperature_k >= max_temperature_k)
    {
        return Err(thermo_error(
            species,
            "temperature ranges must be finite, positive, and strictly increasing",
        ));
    }
    Ok(Nasa7Thermo {
        min_temperature_k,
        midpoint_temperature_k,
        max_temperature_k,
        low_coefficients: nasa_coefficients(species, low)?,
        high_coefficients: nasa_coefficients(species, high)?,
        reference_pressure_pa: parse_reference_pressure(species, raw.reference_pressure.as_ref())?,
    })
}

fn parse_activation_energy(value: &Scalar, default_scale: f64) -> Result<f64, MechanismError> {
    match value {
        Scalar::Number(value) => Ok(*value * default_scale),
        Scalar::Text(text) => {
            let (number, unit) = text
                .trim()
                .split_once(char::is_whitespace)
                .ok_or_else(|| unsupported("activation-energy", text))?;
            let number = number
                .parse::<f64>()
                .map_err(|_| unsupported("activation-energy", text))?;
            Ok(number * activation_unit(unit.trim())?)
        }
    }
}

fn parse_reaction_pressure(
    value: &Scalar,
    default_scale: f64,
    reaction: usize,
) -> Result<f64, MechanismError> {
    let pressure = match value {
        Scalar::Number(value) => *value * default_scale,
        Scalar::Text(text) => {
            let (number, unit) = text.trim().split_once(char::is_whitespace).ok_or_else(|| {
                MechanismError::InvalidReaction {
                    reaction,
                    detail: format!("invalid pressure '{text}'"),
                }
            })?;
            let number = number
                .parse::<f64>()
                .map_err(|_| MechanismError::InvalidReaction {
                    reaction,
                    detail: format!("invalid pressure '{text}'"),
                })?;
            number * pressure_unit(unit.trim())?
        }
    };
    if pressure.is_finite() && pressure > 0.0 {
        Ok(pressure)
    } else {
        Err(MechanismError::InvalidReaction {
            reaction,
            detail: format!("pressure must be finite and positive (got {pressure})"),
        })
    }
}

fn validate_rate(reaction: usize, field: &'static str, value: f64) -> Result<(), MechanismError> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        Err(MechanismError::InvalidRate {
            reaction,
            field,
            value,
        })
    }
}

fn missing_rate(reaction: usize, field: &str) -> MechanismError {
    MechanismError::InvalidReaction {
        reaction,
        detail: format!("missing {field}"),
    }
}

fn normalize_rate(
    raw: &RawRate,
    dimensional_order: f64,
    units: ResolvedUnits,
    reaction: usize,
) -> Result<RateLaw, MechanismError> {
    let declared = match &raw.pre_exponential {
        Scalar::Number(value) => *value,
        Scalar::Text(text) => {
            // Cantera allows `A: 1.0e12 cm^3/mol/s`. Per-value rate units are
            // not modelled, so refuse them by name instead of leaving serde to
            // report an opaque type mismatch.
            return Err(unsupported("pre-exponential", text));
        }
    };
    let pre_exponential = declared
        * units
            .concentration_mol_per_litre
            .powf(1.0 - dimensional_order)
        / units.seconds;
    validate_rate(reaction, "A", pre_exponential)?;
    if !raw.b.is_finite() {
        return Err(MechanismError::InvalidRate {
            reaction,
            field: "b",
            value: raw.b,
        });
    }
    let activation_energy =
        parse_activation_energy(&raw.activation_energy, units.activation_j_per_mol)?;
    if !activation_energy.is_finite() {
        return Err(MechanismError::NonFiniteActivationEnergy {
            reaction,
            value: activation_energy,
        });
    }
    Ok(RateLaw {
        pre_exponential,
        temperature_exponent: raw.b,
        activation_energy,
    })
}

/// Replace the equation-derived reactant orders with the ones the document
/// states, and refuse everything the portable subset does not model.
///
/// Three refusals, each of which changes an answer rather than a style:
///
/// - an order on a species that is not on the reactant side is Cantera's
///   `nonreactant-orders`, a different rate law shape (the CO step of a
///   two-step hydrocarbon mechanism depends on water, which it neither
///   consumes nor produces). It is refused by name rather than dropped;
/// - an order below zero without `negative-orders: true` is refused,
///   because Cantera requires the same acknowledgement and because a
///   stray minus sign turns a fuel into an inhibitor;
/// - a non-finite order is refused, since every downstream rate would be
///   NaN.
fn apply_explicit_orders(
    reaction: usize,
    orders: &mut [OwnedOrder],
    declared: &BTreeMap<String, f64>,
    negative_allowed: bool,
) -> Result<(), MechanismError> {
    for (species, order) in declared {
        if !order.is_finite() {
            return Err(MechanismError::InvalidReaction {
                reaction,
                detail: format!("order for '{species}' must be finite (got {order})"),
            });
        }
        if *order < 0.0 && !negative_allowed {
            return Err(MechanismError::InvalidReaction {
                reaction,
                detail: format!(
                    "order for '{species}' is {order}; a negative order must declare negative-orders: true"
                ),
            });
        }
        let Some(slot) = orders.iter_mut().find(|term| term.species == *species) else {
            return Err(MechanismError::InvalidReaction {
                reaction,
                detail: format!(
                    "orders names '{species}', which is not a reactant of this equation (nonreactant-orders is not modelled)"
                ),
            });
        };
        slot.order = *order;
    }
    Ok(())
}

fn default_collider() -> OwnedThirdBody {
    OwnedThirdBody {
        default_efficiency: 1.0,
        efficiencies: Vec::new(),
    }
}

fn validate_collider(
    reaction: usize,
    default_efficiency: f64,
    efficiencies: BTreeMap<String, f64>,
    species: &BTreeMap<&str, &MechanismSpecies>,
) -> Result<Option<OwnedThirdBody>, MechanismError> {
    if !default_efficiency.is_finite() || default_efficiency < 0.0 {
        return Err(MechanismError::InvalidReaction {
            reaction,
            detail: format!("invalid default collider efficiency {default_efficiency}"),
        });
    }
    for (name, efficiency) in &efficiencies {
        if !species.contains_key(name.as_str()) {
            return Err(MechanismError::UnknownReactionSpecies {
                reaction,
                species: name.clone(),
            });
        }
        if !efficiency.is_finite() || *efficiency < 0.0 {
            return Err(MechanismError::InvalidReaction {
                reaction,
                detail: format!("invalid collider efficiency for '{name}': {efficiency}"),
            });
        }
    }
    if efficiencies.is_empty() && default_efficiency == 1.0 {
        Ok(None)
    } else {
        Ok(Some(OwnedThirdBody {
            default_efficiency,
            efficiencies: efficiencies.into_iter().collect(),
        }))
    }
}

fn validate_troe(reaction: usize, raw: RawTroe) -> Result<Troe, MechanismError> {
    if !raw.a.is_finite() || !(0.0..=1.0).contains(&raw.a) {
        return Err(MechanismError::InvalidReaction {
            reaction,
            detail: format!(
                "Troe A must be finite and between zero and one (got {})",
                raw.a
            ),
        });
    }
    for (name, value) in [("T3", raw.t3), ("T1", raw.t1)] {
        if !value.is_finite() || value <= 0.0 {
            return Err(MechanismError::InvalidReaction {
                reaction,
                detail: format!("Troe {name} must be finite and positive (got {value})"),
            });
        }
    }
    if raw
        .t2
        .is_some_and(|value| !value.is_finite() || value <= 0.0)
    {
        return Err(MechanismError::InvalidReaction {
            reaction,
            detail: format!("Troe T2 must be finite and positive (got {:?})", raw.t2),
        });
    }
    Ok(Troe {
        a: raw.a,
        t3: raw.t3,
        t1: raw.t1,
        t2: raw.t2,
    })
}

struct Equation {
    reactants: Vec<(String, f64)>,
    products: Vec<(String, f64)>,
    has_collider: bool,
    reversible: bool,
}

fn parse_equation(text: &str, reaction: usize) -> Result<Equation, MechanismError> {
    let (left, right, reversible) = if let Some((left, right)) = text.split_once("<=>") {
        (left, right, true)
    } else if let Some((left, right)) = text.split_once("=>") {
        (left, right, false)
    } else {
        return Err(MechanismError::InvalidReaction {
            reaction,
            detail: "expected the reaction arrow '=>' or '<=>'".to_string(),
        });
    };
    let (reactants, left_collider) = parse_side(left, reaction)?;
    let (products, right_collider) = parse_side(right, reaction)?;
    if left_collider != right_collider {
        return Err(MechanismError::InvalidReaction {
            reaction,
            detail: "third-body marker must appear on both sides of the equation".to_string(),
        });
    }
    Ok(Equation {
        reactants,
        products,
        has_collider: left_collider,
        reversible,
    })
}

fn parse_side(text: &str, reaction: usize) -> Result<(Vec<(String, f64)>, bool), MechanismError> {
    let mut terms = Vec::new();
    let parenthetical = text.contains("(+M)");
    let cleaned = text.replace("(+M)", "");
    let mut bare = false;
    for raw in cleaned.split(" + ") {
        let raw = raw.trim();
        if raw.is_empty() {
            return Err(MechanismError::InvalidReaction {
                reaction,
                detail: "empty equation term".to_string(),
            });
        }
        if raw == "M" {
            bare = true;
            continue;
        }
        let split = raw.find(char::is_whitespace);
        let (coefficient, species) = match split {
            Some(index) if raw[..index].parse::<f64>().is_ok() => (
                raw[..index].parse::<f64>().unwrap_or_default(),
                raw[index..].trim(),
            ),
            _ => (1.0, raw),
        };
        if !coefficient.is_finite() || coefficient <= 0.0 || species.is_empty() {
            return Err(MechanismError::InvalidReaction {
                reaction,
                detail: format!("invalid equation term '{raw}'"),
            });
        }
        terms.push((species.to_string(), coefficient));
    }
    Ok((terms, parenthetical || bare))
}

fn validate_balance(
    reaction: usize,
    stoichiometry: &BTreeMap<String, f64>,
    species: &BTreeMap<&str, &MechanismSpecies>,
) -> Result<(), MechanismError> {
    let mut elements = BTreeMap::<&str, f64>::new();
    for (name, coefficient) in stoichiometry {
        for (element, count) in &species[name.as_str()].composition {
            *elements.entry(element).or_default() += coefficient * count;
        }
    }
    for (element, imbalance) in elements {
        if imbalance.abs() > 1e-9 {
            return Err(MechanismError::ElementImbalance {
                reaction,
                element: element.to_string(),
                imbalance,
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kinetics::advance_network;
    use crate::units::{Liters, Moles};
    use crate::vessel::{Headspace, Vessel, VesselId};

    const HYDROGEN: &str = r#"
description: minimal hydrogen oxidation
units: {length: cm, quantity: mol, activation-energy: cal/mol}
phases:
- name: gas
  thermo: ideal-gas
  species: [H2, O2, H2O]
species:
- name: H2
  composition: {H: 2}
- name: O2
  composition: {O: 2}
- name: H2O
  composition: {H: 2, O: 1}
reactions:
- equation: 2 H2 + O2 => 2 H2O
  rate-constant: {A: 1.0e12, b: 0.5, Ea: 10.0 kcal/mol}
"#;

    #[test]
    fn parses_validates_and_lowers_elementary_mechanism() {
        let parsed = parse_yaml(HYDROGEN).unwrap();
        assert_eq!(parsed.summary().species, 3);
        assert_eq!(parsed.summary().reactions, 1);
        let arena = MechanismArena::default();
        let network = parsed.compile_in(&arena);
        assert_eq!(network.id, "minimal hydrogen oxidation");
        assert_eq!(network.reactions[0].stoichiometry.len(), 3);
        assert_eq!(network.reactions[0].forward.orders.len(), 2);
        assert_eq!(
            network.reactions[0].forward.arrhenius.temperature_exponent,
            0.5
        );
        assert_eq!(
            network.reactions[0].forward.arrhenius.activation_energy,
            41_840.0
        );
        // mol/cm³ -> mol/L. A third-order rate constant carries C^-2.
        assert_eq!(
            network.reactions[0].forward.arrhenius.pre_exponential,
            1.0e6
        );
    }

    #[test]
    fn default_kmol_per_cubic_metre_matches_moles_per_litre() {
        let text = HYDROGEN.replace(
            "units: {length: cm, quantity: mol, activation-energy: cal/mol}",
            "units: {activation-energy: J/kmol}",
        );
        let text = text.replace("10.0 kcal/mol", "41840000");
        let parsed = parse_yaml(&text).unwrap();
        let arena = MechanismArena::default();
        let network = parsed.compile_in(&arena);
        assert_eq!(
            network.reactions[0].forward.arrhenius.pre_exponential,
            1.0e12
        );
        assert_eq!(
            network.reactions[0].forward.arrhenius.activation_energy,
            41_840.0
        );
    }

    #[test]
    fn rejects_unbalanced_unknown_and_uncharacterised_reversible_reactions() {
        let unbalanced = HYDROGEN.replace("2 H2O", "H2O");
        assert!(matches!(
            parse_yaml(&unbalanced),
            Err(MechanismError::ElementImbalance { .. })
        ));
        let unknown = HYDROGEN.replace("2 H2O", "2 OH");
        assert!(matches!(
            parse_yaml(&unknown),
            Err(MechanismError::UnknownReactionSpecies { .. })
        ));
        let reversible = HYDROGEN.replace("=>", "<=>");
        let error = parse_yaml(&reversible).unwrap_err().to_string();
        assert!(error.contains("missing NASA7 thermochemistry"), "{error}");
    }

    const REVERSIBLE: &str = r#"
description: reversible isomerisation
phases:
- name: gas
  thermo: ideal-gas
  species: [A, B]
species:
- name: A
  composition: {X: 1}
  thermo:
    model: NASA7
    temperature-ranges: [200.0, 1000.0, 3000.0]
    data:
    - [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
    - [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
- name: B
  composition: {X: 1}
  thermo:
    model: NASA7
    reference-pressure: 1 atm
    temperature-ranges: [200.0, 1000.0, 3000.0]
    data:
    - [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.3862943611198906]
    - [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.3862943611198906]
reactions:
- equation: A <=> B
  rate-constant: {A: 1.0, b: 0, Ea: 0}
"#;

    #[test]
    fn nasa7_reversible_rate_obeys_detailed_balance() {
        let parsed = parse_yaml(REVERSIBLE).unwrap();
        assert!(parsed.summary().reaction_details[0].reversible);
        let arena = MechanismArena::default();
        let network = parsed.compile_in(&arena);
        let reaction = &network.reactions[0];
        assert_eq!(reaction.validity.temperature_k.unwrap().min, 200.0);
        assert_eq!(reaction.validity.temperature_k.unwrap().max, 3000.0);
        assert!(
            (reaction
                .equilibrium
                .unwrap()
                .concentration_equilibrium_constant(500.0)
                - 4.0)
                .abs()
                < 1e-12
        );

        let mut vessel = sealed_gas(&[("A", 0.5), ("B", 0.5)]);
        vessel.temperature = crate::Kelvin(500.0);
        vessel.refresh_pressure();
        assert!((reaction.rate_now(&vessel) - 0.375).abs() < 1e-12);

        let equilibrium = sealed_gas(&[("A", 0.2), ("B", 0.8)]);
        assert!(reaction.rate_now(&equilibrium).abs() < 1e-12);

        let mut evolving = sealed_gas(&[("A", 1.0)]);
        evolving.temperature = crate::Kelvin(500.0);
        evolving.refresh_pressure();
        advance_network(&mut evolving, 20.0, &network).unwrap();
        let a = evolving.moles_of(&crate::SpeciesId::new("A")).0;
        let b = evolving.moles_of(&crate::SpeciesId::new("B")).0;
        assert!((a - 0.2).abs() < 1e-6, "{a}");
        assert!((b - 0.8).abs() < 1e-6, "{b}");

        let mut outside_validity = sealed_gas(&[("A", 1.0)]);
        outside_validity.temperature = crate::Kelvin(100.0);
        outside_validity.refresh_pressure();
        assert!(advance_network(&mut outside_validity, 20.0, &network)
            .unwrap()
            .is_empty());
        assert_eq!(
            outside_validity.moles_of(&crate::SpeciesId::new("A")).0,
            1.0
        );
    }

    #[test]
    fn nasa7_schema_errors_name_the_species() {
        let malformed = REVERSIBLE.replace(
            "temperature-ranges: [200.0, 1000.0, 3000.0]",
            "temperature-ranges: [3000.0, 1000.0, 200.0]",
        );
        let error = parse_yaml(&malformed).unwrap_err().to_string();
        assert!(error.contains("species 'A'"), "{error}");
        assert!(error.contains("temperature ranges"), "{error}");
    }

    #[test]
    fn modified_arrhenius_temperature_exponent_is_executed() {
        let law = RateLaw {
            pre_exponential: 2.0,
            temperature_exponent: 0.5,
            activation_energy: 0.0,
        };
        assert!((law.rate_constant(400.0) - 40.0).abs() < 1e-12);
    }

    const COLLIDER_SPECIES: &str = r#"
description: collider tests
phases:
- name: gas
  thermo: ideal-gas
  species: [H, O, O2, HO2, AR]
species:
- name: H
  composition: {H: 1}
- name: O
  composition: {O: 1}
- name: O2
  composition: {O: 2}
- name: HO2
  composition: {H: 1, O: 2}
- name: AR
  composition: {Ar: 1}
"#;

    fn sealed_gas(contents: &[(&str, f64)]) -> Vessel {
        let mut vessel = Vessel::new(VesselId(0), "reactor");
        vessel.headspace = Headspace::Sealed {
            volume: Liters(1.0),
        };
        for (species, moles) in contents {
            vessel.deposit(crate::SpeciesId::new(species), Moles(*moles), Phase::Gas);
        }
        vessel.refresh_pressure();
        vessel
    }

    const PLOG: &str = r#"
description: pressure grid
units: {pressure: atm, activation-energy: J/mol}
phases:
- name: gas
  thermo: ideal-gas
  species: [A, B]
species:
- name: A
  composition: {X: 1}
- name: B
  composition: {X: 1}
reactions:
- equation: A => B
  type: pressure-dependent-Arrhenius
  rate-constants:
  - {P: 1, A: 1, b: 0, Ea: 0}
  - {P: 1, A: 3, b: 0, Ea: 0}
  - {P: 100, A: 400, b: 0, Ea: 0}
"#;

    fn sealed_gas_at_pressure(species: &str, pressure_pa: f64) -> Vessel {
        let temperature_k = 300.0;
        let concentration = pressure_pa / (8_314.462_618 * temperature_k);
        let mut vessel = sealed_gas(&[(species, concentration)]);
        vessel.temperature = crate::Kelvin(temperature_k);
        vessel.refresh_pressure();
        vessel
    }

    #[test]
    fn plog_sums_duplicate_pressures_interpolates_logs_and_executes_implicitly() {
        let parsed = parse_yaml(PLOG).unwrap();
        let summary = parsed.summary();
        assert_eq!(
            summary.reaction_details[0].rate_model,
            "pressure_dependent_arrhenius"
        );
        assert_eq!(
            summary.reaction_details[0].pressure_points_pa,
            vec![101_325.0, 10_132_500.0]
        );
        let arena = MechanismArena::default();
        let network = parsed.compile_in(&arena);
        let reaction = &network.reactions[0];

        for (pressure_pa, expected_k) in [
            (10_132.5, 4.0),
            (101_325.0, 4.0),
            (1_013_250.0, 40.0),
            (10_132_500.0, 400.0),
            (101_325_000.0, 400.0),
        ] {
            let vessel = sealed_gas_at_pressure("A", pressure_pa);
            let concentration = pressure_pa / (8_314.462_618 * 300.0);
            let actual_k = reaction.rate_now(&vessel) / concentration;
            assert!((actual_k - expected_k).abs() < 1e-10, "{actual_k}");
        }

        let mut vessel = sealed_gas_at_pressure("A", 1_013_250.0);
        let initial = vessel.moles_of(&crate::SpeciesId::new("A")).0;
        advance_network(&mut vessel, 0.01, &network).unwrap();
        let remaining = vessel.moles_of(&crate::SpeciesId::new("A")).0;
        assert!((remaining - initial * (-0.4f64).exp()).abs() < 1e-6);
    }

    #[test]
    fn plog_rejects_missing_or_degenerate_pressure_grids() {
        let missing = PLOG.replace(
            "  - {P: 1, A: 1, b: 0, Ea: 0}\n  - {P: 1, A: 3, b: 0, Ea: 0}\n  - {P: 100, A: 400, b: 0, Ea: 0}",
            "  - {P: 1, A: 1, b: 0, Ea: 0}",
        );
        let error = parse_yaml(&missing).unwrap_err().to_string();
        assert!(
            error.contains("two distinct interpolation pressures"),
            "{error}"
        );

        let invalid = PLOG.replace("P: 100", "P: 0");
        let error = parse_yaml(&invalid).unwrap_err().to_string();
        assert!(
            error.contains("pressure must be finite and positive"),
            "{error}"
        );
    }

    #[test]
    fn three_body_efficiencies_change_the_effective_collider_concentration() {
        let yaml = format!(
            "{COLLIDER_SPECIES}\nreactions:\n- equation: 2 O + M => O2 + M\n  type: three-body\n  rate-constant: {{A: 10.0, b: 0, Ea: 0}}\n  efficiencies: {{AR: 0.5}}\n"
        );
        let parsed = parse_yaml(&yaml).unwrap();
        assert_eq!(
            parsed.summary().reaction_details[0].rate_model,
            "three_body"
        );
        let arena = MechanismArena::default();
        let network = parsed.compile_in(&arena);
        let vessel = sealed_gas(&[("O", 0.2), ("AR", 0.8)]);
        // [M]eff = [O] + 0.5[Ar] = 0.6 mol/L.
        let rate = network.reactions[0].rate_now(&vessel);
        assert!((rate - 10.0 * 0.2f64.powi(2) * 0.6).abs() < 1e-12, "{rate}");
    }

    #[test]
    fn lindemann_and_troe_match_their_closed_form_rates() {
        let base = format!(
            "{COLLIDER_SPECIES}\nreactions:\n- equation: H + O2 (+M) => HO2 (+M)\n  type: falloff\n  high-P-rate-constant: {{A: 100.0, b: 0, Ea: 0}}\n  low-P-rate-constant: {{A: 1000.0, b: 0, Ea: 0}}\n"
        );
        let parsed = parse_yaml(&base).unwrap();
        assert_eq!(parsed.summary().reaction_details[0].rate_model, "lindemann");
        let arena = MechanismArena::default();
        let network = parsed.compile_in(&arena);
        let vessel = sealed_gas(&[("H", 0.01), ("O2", 0.02), ("AR", 0.97)]);
        // Pr = k0[M]/kinf = 10 and k = kinf Pr/(1+Pr).
        let expected = (100.0 * 10.0 / 11.0) * 0.01 * 0.02;
        let lindemann = network.reactions[0].rate_now(&vessel);
        assert!((lindemann - expected).abs() < 1e-12, "{lindemann}");

        let troe_yaml = format!("{base}  Troe: {{A: 0.5, T3: 1000.0, T1: 10000.0, T2: 5000.0}}\n");
        let troe_parsed = parse_yaml(&troe_yaml).unwrap();
        assert_eq!(troe_parsed.summary().reaction_details[0].rate_model, "troe");
        let troe_arena = MechanismArena::default();
        let troe_network = troe_parsed.compile_in(&troe_arena);
        let broadened = troe_network.reactions[0].rate_now(&vessel);
        let parameters = Troe {
            a: 0.5,
            t3: 1000.0,
            t1: 10000.0,
            t2: Some(5000.0),
        };
        let expected_broadened = expected * parameters.broadening(298.15, 10.0);
        assert!(
            (broadened - expected_broadened).abs() < 1e-12,
            "{broadened}"
        );
        assert!(broadened < lindemann);
    }

    #[test]
    fn parsed_gas_network_advances_in_a_finite_headspace() {
        let yaml = r#"
description: gas integration
phases:
- name: gas
  thermo: ideal-gas
  species: [H2, H]
species:
- name: H2
  composition: {H: 2}
- name: H
  composition: {H: 1}
reactions:
- equation: H2 => 2 H
  rate-constant: {A: 1.0, b: 0, Ea: 0}
"#;
        let parsed = parse_yaml(yaml).unwrap();
        let arena = MechanismArena::default();
        let network = parsed.compile_in(&arena);
        let mut vessel = sealed_gas(&[("H2", 1.0)]);
        let initial_pressure = vessel.pressure.0;
        let extents = advance_network(&mut vessel, 0.1, &network).unwrap();
        assert!(!extents.is_empty());
        assert!(vessel.moles_of(&crate::SpeciesId::new("H2")).0 < 1.0);
        assert!(vessel.moles_of(&crate::SpeciesId::new("H")).0 > 0.0);
        assert!(vessel.pressure.0 > initial_pressure);
    }

    #[test]
    fn pressure_dependent_schema_errors_are_explicit() {
        let missing_marker = format!(
            "{COLLIDER_SPECIES}\nreactions:\n- equation: 2 O => O2\n  type: three-body\n  rate-constant: {{A: 1.0, b: 0, Ea: 0}}\n"
        );
        let error = parse_yaml(&missing_marker).unwrap_err().to_string();
        assert!(error.contains("missing M or (+M)"), "{error}");

        let unknown_efficiency = format!(
            "{COLLIDER_SPECIES}\nreactions:\n- equation: 2 O + M => O2 + M\n  type: three-body\n  rate-constant: {{A: 1.0, b: 0, Ea: 0}}\n  efficiencies: {{NOPE: 2.0}}\n"
        );
        assert!(matches!(
            parse_yaml(&unknown_efficiency),
            Err(MechanismError::UnknownReactionSpecies { .. })
        ));
    }
}
