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
    Nasa7Thermo, OrderTerm, PressureDependence, Range, RateExpression, RateLaw, ReactionNetwork,
    SiteTerm, StoichiometricTerm, ThirdBody, Troe, Uncertainty, Validity,
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
}

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
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct RawUnits {
    length: Option<String>,
    time: Option<String>,
    quantity: Option<String>,
    activation_energy: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawPhase {
    name: String,
    thermo: String,
    species: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct RawSpecies {
    name: String,
    composition: BTreeMap<String, f64>,
    #[serde(default)]
    thermo: Option<RawThermo>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct RawThermo {
    model: String,
    temperature_ranges: Vec<f64>,
    data: Vec<Vec<f64>>,
    #[serde(default)]
    reference_pressure: Option<Scalar>,
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
    efficiencies: BTreeMap<String, f64>,
    #[serde(default = "unit_efficiency")]
    default_efficiency: f64,
    #[serde(default, rename = "Troe")]
    troe: Option<RawTroe>,
}

fn elementary() -> String {
    "elementary".to_string()
}

const fn unit_efficiency() -> f64 {
    1.0
}

#[derive(Debug, Deserialize)]
struct RawRate {
    #[serde(rename = "A")]
    pre_exponential: f64,
    #[serde(default)]
    b: f64,
    #[serde(rename = "Ea")]
    activation_energy: Scalar,
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
        if phase.thermo != "ideal-gas" {
            return Err(MechanismError::UnsupportedPhase {
                phase: phase.name.clone(),
                thermo: phase.thermo.clone(),
            });
        }
        for species in &phase.species {
            if !names.contains(species) {
                return Err(MechanismError::UnknownPhaseSpecies {
                    phase: phase.name.clone(),
                    species: species.clone(),
                });
            }
            if assignments.insert(species.clone(), Phase::Gas).is_some() {
                return Err(MechanismError::UnassignedSpecies {
                    species: species.clone(),
                });
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
        let kind = match reaction.r#type.as_str() {
            "elementary" => ReactionKind::Elementary,
            "three-body" => ReactionKind::ThirdBody,
            "falloff" => ReactionKind::Falloff,
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
            (ReactionKind::Elementary, true) => {
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
        let mut stoichiometry = BTreeMap::<String, f64>::new();
        for (name, coefficient) in equation.reactants {
            if !species_by_name.contains_key(name.as_str()) {
                return Err(MechanismError::UnknownReactionSpecies {
                    reaction: number,
                    species: name,
                });
            }
            *stoichiometry.entry(name).or_default() -= coefficient;
        }
        for (name, coefficient) in equation.products {
            if !species_by_name.contains_key(name.as_str()) {
                return Err(MechanismError::UnknownReactionSpecies {
                    reaction: number,
                    species: name,
                });
            }
            *stoichiometry.entry(name).or_default() += coefficient;
        }
        stoichiometry.retain(|_, coefficient| coefficient.abs() > 1e-14);
        if stoichiometry.is_empty() {
            return Err(MechanismError::InvalidReaction {
                reaction: number,
                detail: "reaction has no net stoichiometric change".to_string(),
            });
        }
        validate_balance(number, &stoichiometry, &species_by_name)?;

        let orders: Vec<_> = stoichiometry
            .iter()
            .filter(|(_, coefficient)| **coefficient < 0.0)
            .map(|(name, coefficient)| OwnedOrder {
                species: name.clone(),
                phase: species_by_name[name.as_str()].phase,
                order: -*coefficient,
            })
            .collect();
        let reverse_orders = reversible.then(|| {
            stoichiometry
                .iter()
                .filter(|(_, coefficient)| **coefficient > 0.0)
                .map(|(name, coefficient)| OwnedOrder {
                    species: name.clone(),
                    phase: species_by_name[name.as_str()].phase,
                    order: *coefficient,
                })
                .collect::<Vec<_>>()
        });
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
                    }
                    .to_string(),
                    low_pressure_pre_exponential: match &reaction.pressure_dependence {
                        OwnedPressureDependence::Falloff { low_pressure, .. } => {
                            Some(low_pressure.pre_exponential)
                        }
                        _ => None,
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
        let activation_j_per_mol =
            activation_unit(raw.activation_energy.as_deref().unwrap_or("J/kmol"))?;
        Ok(Self {
            concentration_mol_per_litre: quantity_moles / volume_litres,
            seconds,
            activation_j_per_mol,
        })
    }
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
        "K" => Ok(super::R),
        unit => Err(unsupported("activation-energy", unit)),
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
    let pre_exponential = raw.pre_exponential
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
    if !activation_energy.is_finite() || activation_energy < 0.0 {
        return Err(MechanismError::InvalidRate {
            reaction,
            field: "Ea",
            value: activation_energy,
        });
    }
    Ok(RateLaw {
        pre_exponential,
        temperature_exponent: raw.b,
        activation_energy,
    })
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
