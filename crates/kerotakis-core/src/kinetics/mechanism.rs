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
    KineticReaction, Locality, OrderTerm, RateExpression, RateLaw, ReactionNetwork, SiteTerm,
    StoichiometricTerm, Uncertainty, Validity,
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
}

#[derive(Debug, Clone, PartialEq)]
struct MechanismSpecies {
    name: String,
    composition: BTreeMap<String, f64>,
    phase: Phase,
}

#[derive(Debug, Clone, PartialEq)]
struct OwnedReaction {
    id: String,
    equation: String,
    stoichiometry: Vec<OwnedTerm>,
    orders: Vec<OwnedOrder>,
    rate: RateLaw,
    phase: Phase,
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
    #[error("reaction {reaction}: {detail}")]
    InvalidReaction { reaction: usize, detail: String },
    #[error("reaction {reaction}: reversible elementary reactions need thermodynamic reverse rates, which are not in the KIN-006 subset")]
    UnsupportedReversible { reaction: usize },
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
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct RawReaction {
    equation: String,
    #[serde(default = "elementary")]
    r#type: String,
    rate_constant: RawRate,
}

fn elementary() -> String {
    "elementary".to_string()
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
            Ok(MechanismSpecies {
                name: entry.name,
                composition: entry.composition,
                phase,
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
        if reaction.r#type != "elementary" {
            return Err(MechanismError::UnsupportedReactionType {
                reaction: number,
                kind: reaction.r#type,
            });
        }
        let equation = parse_equation(&reaction.equation, number)?;
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
        let total_order: f64 = orders.iter().map(|order| order.order).sum();
        let pre_exponential = reaction.rate_constant.pre_exponential
            * units.concentration_mol_per_litre.powf(1.0 - total_order)
            / units.seconds;
        let activation_energy = parse_activation_energy(
            &reaction.rate_constant.activation_energy,
            units.activation_j_per_mol,
        )?;
        validate_rate(number, "A", pre_exponential)?;
        if !reaction.rate_constant.b.is_finite() {
            return Err(MechanismError::InvalidRate {
                reaction: number,
                field: "b",
                value: reaction.rate_constant.b,
            });
        }
        if !activation_energy.is_finite() || activation_energy < 0.0 {
            return Err(MechanismError::InvalidRate {
                reaction: number,
                field: "Ea",
                value: activation_energy,
            });
        }

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
            rate: RateLaw {
                pre_exponential,
                temperature_exponent: reaction.rate_constant.b,
                activation_energy,
            },
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
            .alloc_str("validity range not declared by the loaded mechanism");
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
            KineticReaction {
                id: arena.storage.alloc_str(&reaction.id),
                equation: arena.storage.alloc_str(&reaction.equation),
                stoichiometry,
                locality: Locality::Bulk(reaction.phase),
                forward: RateExpression {
                    arrhenius: reaction.rate,
                    orders,
                },
                reverse: None,
                catalysts: &[],
                sites: &[] as &[SiteTerm<'_>],
                electrons: 0.0,
                validity: Validity {
                    temperature_k: None,
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

struct Equation {
    reactants: Vec<(String, f64)>,
    products: Vec<(String, f64)>,
}

fn parse_equation(text: &str, reaction: usize) -> Result<Equation, MechanismError> {
    if text.contains("(+") || text.split_whitespace().any(|token| token == "M") {
        return Err(MechanismError::UnsupportedReactionType {
            reaction,
            kind: "three-body/falloff".to_string(),
        });
    }
    if text.contains("<=>") {
        return Err(MechanismError::UnsupportedReversible { reaction });
    }
    let (left, right) = text
        .split_once("=>")
        .ok_or_else(|| MechanismError::InvalidReaction {
            reaction,
            detail: "expected the irreversible arrow '=>'".to_string(),
        })?;
    Ok(Equation {
        reactants: parse_side(left, reaction)?,
        products: parse_side(right, reaction)?,
    })
}

fn parse_side(text: &str, reaction: usize) -> Result<Vec<(String, f64)>, MechanismError> {
    let mut terms = Vec::new();
    for raw in text.split(" + ") {
        let raw = raw.trim();
        if raw.is_empty() {
            return Err(MechanismError::InvalidReaction {
                reaction,
                detail: "empty equation term".to_string(),
            });
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
    Ok(terms)
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
    fn rejects_unbalanced_unknown_and_reversible_reactions() {
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
        assert!(matches!(
            parse_yaml(&reversible),
            Err(MechanismError::UnsupportedReversible { .. })
        ));
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
}
