//! Generated element-to-content coverage for the progressive periodic table.
//!
//! This is deliberately derived from the live registries.  A shelf key in the
//! report is therefore something a client can actually dispense; material
//! recipes contribute the formulae of their deterministic expansion rather
//! than pretending the material name is a chemical formula.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{material, species, stoich};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ElementCapability {
    IdentityOnly,
    AddObserve,
    PropertyBacked,
    Reacting,
    LessonBacked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShelfItemKind {
    Species,
    MaterialRecipe,
}

/// Provenance that lets clients round-trip every displayed example to the
/// actual shelf registry and explain which formula caused the match.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ElementShelfItem {
    pub shelf_key: String,
    pub display_name: String,
    pub kind: ShelfItemKind,
    pub formula_species_keys: Vec<String>,
    pub formulas: Vec<String>,
    pub property_backed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunnableElementRoute {
    pub key: String,
    pub label: String,
    /// Real shelf keys, not internal species identifiers.
    pub required_shelf_keys: Vec<String>,
    pub lesson: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ElementCoverageEntry {
    pub symbol: String,
    pub capability: ElementCapability,
    pub examples: Vec<ElementShelfItem>,
    pub routes: Vec<RunnableElementRoute>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ElementCoverageReport {
    pub schema: u32,
    pub elements: Vec<ElementCoverageEntry>,
}

/// Atomic-number order. Structural identities are intentionally independent
/// of installed chemical/property claims, so uncovered cells remain honest.
pub const ELEMENT_SYMBOLS: [&str; 118] = [
    "H", "He", "Li", "Be", "B", "C", "N", "O", "F", "Ne", "Na", "Mg", "Al", "Si", "P", "S", "Cl",
    "Ar", "K", "Ca", "Sc", "Ti", "V", "Cr", "Mn", "Fe", "Co", "Ni", "Cu", "Zn", "Ga", "Ge", "As",
    "Se", "Br", "Kr", "Rb", "Sr", "Y", "Zr", "Nb", "Mo", "Tc", "Ru", "Rh", "Pd", "Ag", "Cd", "In",
    "Sn", "Sb", "Te", "I", "Xe", "Cs", "Ba", "La", "Ce", "Pr", "Nd", "Pm", "Sm", "Eu", "Gd", "Tb",
    "Dy", "Ho", "Er", "Tm", "Yb", "Lu", "Hf", "Ta", "W", "Re", "Os", "Ir", "Pt", "Au", "Hg", "Tl",
    "Pb", "Bi", "Po", "At", "Rn", "Fr", "Ra", "Ac", "Th", "Pa", "U", "Np", "Pu", "Am", "Cm", "Bk",
    "Cf", "Es", "Fm", "Md", "No", "Lr", "Rf", "Db", "Sg", "Bh", "Hs", "Mt", "Ds", "Rg", "Cn", "Nh",
    "Fl", "Mc", "Lv", "Ts", "Og",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ElementCoverageError {
    InvalidSpeciesFormula {
        shelf_key: String,
        formula: String,
    },
    MissingRecipeSpecies {
        shelf_key: String,
        species_key: String,
    },
    MissingRouteSpecies {
        route: String,
        species_key: String,
    },
}

impl std::fmt::Display for ElementCoverageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSpeciesFormula { shelf_key, formula } => {
                write!(f, "shelf item {shelf_key} has invalid formula {formula}")
            }
            Self::MissingRecipeSpecies {
                shelf_key,
                species_key,
            } => write!(
                f,
                "material shelf item {shelf_key} expands to missing species {species_key}"
            ),
            Self::MissingRouteSpecies { route, species_key } => {
                write!(
                    f,
                    "route {route} requires missing shelf species {species_key}"
                )
            }
        }
    }
}

impl std::error::Error for ElementCoverageError {}

/// A lesson supplied by a client/codex pack.  Core accepts only shelf keys and
/// validates them against the same generated inventory before advertising it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledLessonRoute {
    pub key: String,
    pub label: String,
    pub required_shelf_keys: Vec<String>,
}

/// Replay-proved runnable content supplied by the integration that owns its
/// executor. `lesson` distinguishes a bare reaction route from a guided lesson.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledRunnableRoute {
    pub key: String,
    pub label: String,
    pub required_shelf_keys: Vec<String>,
    pub lesson: bool,
}

/// Generate coverage from built-in plus pack-loaded species/materials. Routes
/// stay empty until an owning integration injects replay-proved content.
pub fn element_coverage_report() -> Result<ElementCoverageReport, ElementCoverageError> {
    element_coverage_report_with_lessons(&[])
}

/// Portable host/wasm boundary: clients consume the same deterministic JSON
/// without duplicating formula parsing or coverage rules in TypeScript.
pub fn element_coverage_json() -> Result<String, String> {
    let report = element_coverage_report().map_err(|error| error.to_string())?;
    serde_json::to_string(&report).map_err(|error| error.to_string())
}

pub fn element_coverage_report_with_lessons(
    lessons: &[InstalledLessonRoute],
) -> Result<ElementCoverageReport, ElementCoverageError> {
    let routes: Vec<_> = lessons
        .iter()
        .map(|lesson| InstalledRunnableRoute {
            key: lesson.key.clone(),
            label: lesson.label.clone(),
            required_shelf_keys: lesson.required_shelf_keys.clone(),
            lesson: true,
        })
        .collect();
    element_coverage_report_with_routes(&routes)
}

pub fn element_coverage_report_with_routes(
    installed_routes: &[InstalledRunnableRoute],
) -> Result<ElementCoverageReport, ElementCoverageError> {
    let all_species = species::all_species();
    let species_by_key: BTreeMap<&str, _> = all_species.iter().map(|row| (row.key, *row)).collect();
    let mut items: BTreeMap<String, ElementShelfItem> = BTreeMap::new();

    for row in &all_species {
        stoich::parse_formula(row.formula).map_err(|_| {
            ElementCoverageError::InvalidSpeciesFormula {
                shelf_key: row.key.to_string(),
                formula: row.formula.to_string(),
            }
        })?;
        items.insert(
            row.key.to_string(),
            ElementShelfItem {
                shelf_key: row.key.to_string(),
                display_name: row.name.to_string(),
                kind: ShelfItemKind::Species,
                formula_species_keys: vec![row.key.to_string()],
                formulas: vec![row.formula.to_string()],
                property_backed: has_observable_property(row),
            },
        );
    }

    for recipe in material::all() {
        let expansion = recipe
            .expand(1.0, 0)
            .expect("a positive deterministic registry expansion");
        let mut component_keys = BTreeSet::new();
        let mut formulas = BTreeSet::new();
        let mut property_backed = !recipe.roles.is_empty();
        for component in expansion.components {
            let row = species_by_key
                .get(component.species_id.as_str())
                .ok_or_else(|| ElementCoverageError::MissingRecipeSpecies {
                    shelf_key: recipe.canonical_key.clone(),
                    species_key: component.species_id.clone(),
                })?;
            component_keys.insert(component.species_id);
            formulas.insert(row.formula.to_string());
            property_backed |= has_observable_property(row);
        }
        items.insert(
            recipe.canonical_key.clone(),
            ElementShelfItem {
                shelf_key: recipe.canonical_key,
                display_name: recipe.name,
                kind: ShelfItemKind::MaterialRecipe,
                formula_species_keys: component_keys.into_iter().collect(),
                formulas: formulas.into_iter().collect(),
                property_backed,
            },
        );
    }

    let shelf_keys: BTreeSet<&str> = items.keys().map(String::as_str).collect();
    // Runnable links are injected only after their owning integration has
    // replay-proved them.  Core must not infer executability from merely
    // finding reactant keys in a curated equation.
    let mut routes = Vec::new();
    for installed in installed_routes {
        let mut required = installed.required_shelf_keys.clone();
        required.sort();
        required.dedup();
        for key in &required {
            if !shelf_keys.contains(key.as_str()) {
                return Err(ElementCoverageError::MissingRouteSpecies {
                    route: installed.key.clone(),
                    species_key: key.clone(),
                });
            }
        }
        routes.push(RunnableElementRoute {
            key: installed.key.clone(),
            label: installed.label.clone(),
            required_shelf_keys: required,
            lesson: installed.lesson,
        });
    }
    routes.sort_by(|a, b| a.key.cmp(&b.key));
    routes.dedup_by(|a, b| a.key == b.key);

    let mut by_element: BTreeMap<String, Vec<ElementShelfItem>> = BTreeMap::new();
    for item in items.values() {
        let mut symbols = BTreeSet::new();
        for formula in &item.formulas {
            let parsed = stoich::parse_formula(formula).map_err(|_| {
                ElementCoverageError::InvalidSpeciesFormula {
                    shelf_key: item.shelf_key.clone(),
                    formula: formula.clone(),
                }
            })?;
            symbols.extend(parsed.counts.keys().cloned());
        }
        for symbol in symbols {
            by_element.entry(symbol).or_default().push(item.clone());
        }
    }

    let mut elements = Vec::with_capacity(ELEMENT_SYMBOLS.len());
    for symbol in ELEMENT_SYMBOLS {
        let examples = by_element.remove(symbol).unwrap_or_default();
        let example_keys: BTreeSet<&str> = examples.iter().map(|i| i.shelf_key.as_str()).collect();
        let element_routes: Vec<_> = routes
            .iter()
            .filter(|route| {
                route
                    .required_shelf_keys
                    .iter()
                    .any(|key| example_keys.contains(key.as_str()))
            })
            .cloned()
            .collect();
        let capability = if examples.is_empty() {
            ElementCapability::IdentityOnly
        } else if element_routes.iter().any(|route| route.lesson) {
            ElementCapability::LessonBacked
        } else if !element_routes.is_empty() {
            ElementCapability::Reacting
        } else if examples.iter().any(|item| item.property_backed) {
            ElementCapability::PropertyBacked
        } else {
            ElementCapability::AddObserve
        };
        elements.push(ElementCoverageEntry {
            symbol: symbol.to_string(),
            capability,
            examples,
            routes: element_routes,
        });
    }

    Ok(ElementCoverageReport {
        schema: 1,
        elements,
    })
}

fn has_observable_property(row: &species::SpeciesData) -> bool {
    row.appearance.is_some()
        || row.flame_colour.is_some()
        || row.colour.is_some()
        || row.spectrum.is_some()
        || row.dissolution_enthalpy_kj.is_some()
        || row.aqueous_solubility_g_per_100_ml.is_some()
        || row.forms_only_above_k.is_some()
        || row.magnetic
}
