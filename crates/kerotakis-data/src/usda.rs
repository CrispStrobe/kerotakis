//! BRD-013: the USDA FoodData Central Foundation Foods adapter.
//!
//! # What this is for
//!
//! BRD-014 needs household food materials — milk, juice, flour, oil, sugar —
//! described as [`MaterialRecipe`](crate::MaterialRecipe)s: a list of
//! resolved species plus an honest, conserved unresolved remainder. USDA's
//! Foundation Foods are the only large, public-domain, analytically measured
//! source of that composition for generic foods.
//!
//! # What it refuses to do
//!
//! A nutrient panel is not a molecular composition, and this adapter is built
//! so that it cannot pretend otherwise.
//!
//! * `protein`, `fat`, `ash` and `dietary fibre` are populations of thousands
//!   of molecules. They stay named unresolved aggregates. No amino acid is
//!   invented, no triglyceride is invented.
//! * A sugar or organic acid becomes a species **only** where USDA determined
//!   that compound individually. A record that reports `Sugars, Total` and no
//!   individual determination keeps all of it in `other_carbohydrate`.
//! * A mineral is an *elemental total*: USDA measured how much sodium is in
//!   the food, never which salt it was in. Minerals are reported as an element
//!   inventory inside `ash` and never become `Na+` or `NaCl`. Table salt is
//!   the clearest case — its record states 38.7 g of sodium and no chlorine at
//!   all, so `NaCl` is an inference this adapter declines to make.
//! * `Carbohydrate, by difference` is upstream's closure term, not a
//!   measurement. It is read, but the reconciliation report says so.
//!
//! # Units
//!
//! Every Foundation Foods amount is per 100 g of edible portion, spelled `g`,
//! `mg` or `µg`. Those are bare masses, which `crate::units` rejects by design
//! — a mass with no basis cannot be a composition. So the conversion to the
//! record's own declared basis happens *here*, before any unit reaches a
//! candidate: a nutrient amount becomes grams per 100 g, carrying the reviewed
//! spelling [`BASIS_UNIT`], which normalizes onto [`Dimension::MassPerMass`].
//! Everything that is not a mass — `kcal`, `kJ`, `IU` — is a typed rejection
//! that keeps the original spelling ([`NutrientRejection::UnitIsNotAMass`]).
//!
//! # Where it stops
//!
//! Adapters stop at quarantine. Nothing here writes a registry record, a
//! material pack, or a species. [`candidates`] produces
//! [`QuarantinedCandidate`]s and [`crate::lint_promotion`] is the gate.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};

use crate::adapter::{
    CandidateField, PromotionPolicy, QuarantinedCandidate, RuntimeFieldPolicy,
    ADAPTER_SCHEMA_VERSION,
};
use crate::provenance::EligibleFieldList;
use crate::schema::Dimension;

/// The adapter id every candidate and manifest from this importer carries.
pub const ADAPTER_ID: &str = "usda-fdc";

/// The provenance source id in `provenance/sources.toml`.
pub const SOURCE_ID: &str = "usda-fdc-foundation";

/// FoodData Central is a work of the US federal government, released without
/// copyright. The lint's runtime lane accepts `CC0-1.0`; the *record* still
/// sits in the quarantine lane until a human promotes it.
pub const LICENCE: &str = "CC0-1.0";

/// The only `dataType` this adapter accepts. Branded records are volatile
/// manufacturer reformulations with label-rounded numbers.
pub const ACCEPTED_DATA_TYPE: &str = "Foundation";

/// Foundation Foods amounts are per 100 g of edible portion.
pub const BASIS_GRAMS: f64 = 100.0;

/// The reviewed unit spelling the adapter derives. `crate::units` normalizes
/// it onto [`Dimension::MassPerMass`] with a factor of 0.01, so 11.7 g/100g
/// becomes the mass fraction 0.117.
pub const BASIS_UNIT: &str = "g/100g";

/// Reported values are rounded to two or three significant figures, so a
/// ledger can never close more tightly than this even when every component
/// reports a spread of zero.
pub const MIN_TOLERANCE_GRAMS: f64 = 0.05;

// ── the pinned snapshot ─────────────────────────────────────────────────────

/// One Foundation Foods record, in the projection `tools/fetch-usda-snapshot.py`
/// pins. Unknown fields are ignored rather than refused so that an upstream
/// addition shows up as a refresh diff instead of a parse failure.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FoodRecord {
    #[serde(rename = "fdcId")]
    pub fdc_id: u64,
    pub description: String,
    #[serde(rename = "dataType")]
    pub data_type: String,
    #[serde(rename = "foodClass", default)]
    pub food_class: Option<String>,
    #[serde(rename = "publicationDate", default)]
    pub publication_date: Option<String>,
    #[serde(rename = "ndbNumber", default)]
    pub ndb_number: Option<Value>,
    #[serde(default)]
    pub footnote: Option<String>,
    #[serde(rename = "foodCategory", default)]
    pub food_category: Option<FoodCategory>,
    #[serde(rename = "foodNutrients", default)]
    pub food_nutrients: Vec<FoodNutrient>,
    #[serde(rename = "inputFoods", default)]
    pub input_foods: Vec<InputFood>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FoodCategory {
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputFood {
    #[serde(rename = "foodDescription", default)]
    pub food_description: Option<String>,
    #[serde(rename = "inputFdcId", default)]
    pub input_fdc_id: Option<u64>,
    #[serde(rename = "inputDataType", default)]
    pub input_data_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FoodNutrient {
    pub nutrient: NutrientMeta,
    #[serde(default)]
    pub amount: Option<f64>,
    #[serde(rename = "dataPoints", default)]
    pub data_points: Option<u64>,
    #[serde(default)]
    pub min: Option<f64>,
    #[serde(default)]
    pub max: Option<f64>,
    #[serde(default)]
    pub median: Option<f64>,
    #[serde(default)]
    pub derivation: Option<Derivation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NutrientMeta {
    pub id: u32,
    #[serde(default)]
    pub number: Option<String>,
    pub name: String,
    #[serde(rename = "unitName")]
    pub unit_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Derivation {
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

/// Parse a pinned snapshot. The caller verifies the
/// [`SnapshotManifest`](crate::SnapshotManifest) first; this refuses
/// anything that is not a Foundation Foods record so a mixed snapshot cannot
/// slip a Branded formulation into a candidate.
pub fn parse_snapshot(raw: &[u8]) -> Result<Vec<FoodRecord>, UsdaError> {
    let records: Vec<FoodRecord> =
        serde_json::from_slice(raw).map_err(|error| UsdaError::Malformed(error.to_string()))?;
    if records.is_empty() {
        return Err(UsdaError::EmptySnapshot);
    }
    let mut seen = BTreeSet::new();
    for record in &records {
        if record.data_type != ACCEPTED_DATA_TYPE {
            return Err(UsdaError::NotFoundationFood {
                fdc_id: record.fdc_id,
                data_type: record.data_type.clone(),
            });
        }
        if !seen.insert(record.fdc_id) {
            return Err(UsdaError::DuplicateRecord {
                fdc_id: record.fdc_id,
            });
        }
    }
    Ok(records)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UsdaError {
    Malformed(String),
    EmptySnapshot,
    NotFoundationFood { fdc_id: u64, data_type: String },
    DuplicateRecord { fdc_id: u64 },
}

impl std::fmt::Display for UsdaError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Malformed(detail) => write!(formatter, "malformed USDA snapshot: {detail}"),
            Self::EmptySnapshot => write!(formatter, "USDA snapshot contains no records"),
            Self::NotFoundationFood { fdc_id, data_type } => write!(
                formatter,
                "record {fdc_id} is dataType {data_type}, not {ACCEPTED_DATA_TYPE}"
            ),
            Self::DuplicateRecord { fdc_id } => {
                write!(formatter, "record {fdc_id} appears twice in the snapshot")
            }
        }
    }
}

impl std::error::Error for UsdaError {}

// ── the reviewed nutrient map ───────────────────────────────────────────────

const NUTRIENT_WATER: u32 = 1051;
const NUTRIENT_PROTEIN: u32 = 1003;
const NUTRIENT_FAT: u32 = 1004;
const NUTRIENT_ASH: u32 = 1007;
const NUTRIENT_CARBOHYDRATE_BY_DIFFERENCE: u32 = 1005;
const NUTRIENT_CARBOHYDRATE_BY_SUMMATION: u32 = 1050;
const NUTRIENT_FIBRE: u32 = 1079;
const NUTRIENT_STARCH: u32 = 1009;
const NUTRIENT_ALCOHOL: u32 = 1018;

/// Individually determined sugars. The third column is the registry species
/// the adapter *proposes*; `None` means Kerotakis names no species for that
/// compound, so its mass stays a named unresolved component under its own
/// name rather than being folded into an anonymous remainder.
const SUGARS: &[(u32, &str, Option<&str>)] = &[
    (1010, "sucrose", Some("sucrose")),
    (1011, "glucose", Some("glucose")),
    (1012, "fructose", Some("fructose")),
    (1013, "lactose", None),
    (1014, "maltose", Some("maltose")),
    (1075, "galactose", None),
];

/// Individually determined organic acids, including ascorbic acid: it is an
/// organic acid that upstream files under vitamins, and like the others it
/// lands inside `Carbohydrate, by difference`.
const ORGANIC_ACIDS: &[(u32, &str, Option<&str>)] = &[
    (1026, "acetic_acid", Some("CH3COOH")),
    (1032, "citric_acid", Some("citric_acid")),
    (1038, "lactic_acid", None),
    (1039, "malic_acid", Some("malic_acid")),
    (1040, "tartaric_acid", None),
    (1041, "oxalic_acid", None),
    (1044, "quinic_acid", None),
    (1162, "ascorbic_acid", Some("ascorbic_acid")),
];

/// Elemental mineral determinations. These never become species; see the
/// module docs.
const MINERALS: &[(u32, &str)] = &[
    (1087, "Ca"),
    (1088, "Cl"),
    (1089, "Fe"),
    (1090, "Mg"),
    (1091, "P"),
    (1092, "K"),
    (1093, "Na"),
    (1094, "S"),
    (1095, "Zn"),
    (1096, "Cr"),
    (1097, "Co"),
    (1098, "Cu"),
    (1099, "F"),
    (1100, "I"),
    (1101, "Mn"),
    (1102, "Mo"),
    (1103, "Se"),
    (1146, "Ni"),
];

/// Totals that restate something the ledger already carries. Reading them
/// would double-count, so they are rejected under their own reason rather
/// than being lumped in with genuinely unmapped nutrients.
const DUPLICATE_TOTALS: &[(u32, &str)] = &[
    (1002, "protein is derived from this nitrogen determination"),
    (1050, "restates the carbohydrate the ledger already carries"),
    (1063, "restates the individually determined sugars"),
    (
        1085,
        "an ingredient-label fat total, not the analytical `Total lipid (fat)`",
    ),
    (1257, "a fatty-acid class inside the record's own fat total"),
    (1258, "a fatty-acid class inside the record's own fat total"),
    (1292, "a fatty-acid class inside the record's own fat total"),
    (1293, "a fatty-acid class inside the record's own fat total"),
    (2000, "restates the individually determined sugars"),
    (
        2033,
        "a fibre fraction inside the record's own dietary-fibre total",
    ),
    (
        2038,
        "a fibre fraction inside the record's own dietary-fibre total",
    ),
    (
        2065,
        "a fibre fraction inside the record's own dietary-fibre total",
    ),
];

/// The named aggregates that stand in for populations no model resolves.
pub const AGGREGATE_PROTEIN: &str = "protein";
pub const AGGREGATE_FAT: &str = "fat";
pub const AGGREGATE_ASH: &str = "ash";
pub const AGGREGATE_FIBRE: &str = "dietary_fibre";
pub const AGGREGATE_OTHER_CARBOHYDRATE: &str = "other_carbohydrate";

// ── the mapped composition ──────────────────────────────────────────────────

/// What one component of a food is, in registry terms.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "disposition", rename_all = "snake_case")]
pub enum ComponentDisposition {
    /// The adapter proposes this registry species id. Proposing is not
    /// promoting: whether the species exists is [`registry_gaps`]'s question.
    Species { species_id: String },
    /// Named, conserved, and explicitly not a molecule.
    NamedUnresolved { reason: UnresolvedReason },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnresolvedReason {
    /// USDA measured a population, not a compound: protein, fat, ash, fibre,
    /// and the carbohydrate no individual determination accounts for.
    AggregatePopulation,
    /// USDA named one compound, and Kerotakis has no species for it. The mass
    /// keeps that compound's name so a later review can resolve it.
    NoRegistrySpecies,
}

/// One line of a food's mass ledger, in grams per [`BASIS_GRAMS`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Component {
    pub label: String,
    #[serde(flatten)]
    pub disposition: ComponentDisposition,
    pub grams_per_basis: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_nutrient_id: Option<u32>,
    pub source_field: String,
    /// USDA's derivation code (`A` analytical, `AS` summed, `NC` calculated),
    /// or `adapter:residual` for the carbohydrate the adapter itself derives.
    pub derivation: String,
    /// Half the record's own `min..max` range, where it reports one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spread_grams: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_points: Option<u64>,
}

impl Component {
    pub fn is_resolved(&self) -> bool {
        matches!(self.disposition, ComponentDisposition::Species { .. })
    }

    pub fn species_id(&self) -> Option<&str> {
        match &self.disposition {
            ComponentDisposition::Species { species_id } => Some(species_id.as_str()),
            ComponentDisposition::NamedUnresolved { .. } => None,
        }
    }
}

/// A mineral element inside `ash`. Never a species: see the module docs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MineralElement {
    pub element: String,
    pub grams_per_basis: f64,
    pub source_nutrient_id: u32,
    pub source_field: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spread_grams: Option<f64>,
}

/// A nutrient the adapter did not read, and why. Every one is reported: an
/// importer that silently drops fields cannot be audited.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "reason", rename_all = "snake_case")]
pub enum NutrientRejection {
    /// A grouping row (`Proximates`, `Minerals`) that carries no amount.
    CategoryHeader { nutrient_id: u32, name: String },
    /// The determination is not a mass, so it cannot be part of a composition.
    /// The upstream spelling is preserved rather than guessed at.
    UnitIsNotAMass {
        nutrient_id: u32,
        name: String,
        unit: String,
    },
    /// The amount is absent, negative, or not finite.
    UnusableAmount {
        nutrient_id: u32,
        name: String,
        detail: String,
    },
    /// USDA reports the total mass of an element, never the salt or ion it was
    /// in. The element inventory is reported inside `ash` instead.
    ElementalTotalNotSpeciated {
        nutrient_id: u32,
        element: String,
        grams_per_basis: f64,
    },
    /// A total that restates something already in the ledger.
    DuplicateTotal {
        nutrient_id: u32,
        name: String,
        detail: String,
    },
    /// A mass with no reviewed component: individual fatty acids inside `fat`,
    /// amino acids inside `protein`, vitamins and sterols inside
    /// `other_carbohydrate`. Their mass is already carried by the aggregate
    /// that contains them, so reading them separately would double-count.
    NoReviewedComponent {
        nutrient_id: u32,
        name: String,
        unit: String,
        amount: f64,
    },
}

/// How the record closes its own mass balance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CarbohydrateClosure {
    /// `Carbohydrate, by difference` — upstream's own closure term, defined as
    /// 100 g minus water, protein, fat, ash and alcohol. A ledger built on it
    /// closes by construction; the reconciliation says so rather than claiming
    /// an independent check.
    ByDifference,
    /// `Carbohydrate, by summation` — an independent determination, so the
    /// ledger's closure is a real check.
    BySummation,
    /// No carbohydrate determination at all.
    Absent,
}

/// Why a food cannot be shipped as a candidate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "conflict", rename_all = "snake_case")]
pub enum ReconciliationConflict {
    /// The record does not state one of the five proximates. Absent is not
    /// zero: assuming it were would invent the missing mass.
    MissingProximate { missing: Vec<String> },
    /// The individually determined sugars, starch, fibre and organic acids add
    /// up to more carbohydrate than the record declares, by more than the
    /// record's own spread allows.
    CarbohydrateOverSubscribed {
        carbohydrate_grams: f64,
        subscribed_grams: f64,
        excess_grams: f64,
        tolerance_grams: f64,
    },
    /// The mineral elements outweigh the ash that is supposed to contain them.
    MineralsExceedAsh {
        mineral_grams: f64,
        ash_grams: f64,
        tolerance_grams: f64,
    },
    /// The ledger does not add up to the declared basis within the record's
    /// own stated uncertainty.
    ResidualOutsideUncertainty {
        residual_grams: f64,
        tolerance_grams: f64,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Reconciliation {
    pub basis_grams: f64,
    pub resolved_grams: f64,
    pub named_unresolved_grams: f64,
    pub total_grams: f64,
    /// `basis_grams - total_grams`. Positive means the ledger is short.
    pub residual_grams: f64,
    /// The sum of the record's own half-ranges, floored at
    /// [`MIN_TOLERANCE_GRAMS`].
    pub tolerance_grams: f64,
    pub closure: CarbohydrateClosure,
    /// The carbohydrate determinations over-subscribed the total, but by less
    /// than the record's own spread. The remainder was clamped to zero rather
    /// than carried negative.
    pub oversubscribed_within_uncertainty: bool,
    pub conflicts: Vec<ReconciliationConflict>,
}

impl Reconciliation {
    /// Whether this food may become a candidate at all.
    pub fn reconciles(&self) -> bool {
        self.conflicts.is_empty()
    }
}

/// One food, mapped.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FoodComposition {
    pub fdc_id: u64,
    pub description: String,
    pub data_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub food_category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publication_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ndb_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub footnote: Option<String>,
    pub basis: String,
    pub basis_grams: f64,
    /// Whether the record determined any sugar individually. A `false` here is
    /// why a starchy or floury food keeps all of its carbohydrate unresolved.
    pub sugars_reported_individually: bool,
    pub components: Vec<Component>,
    pub mineral_elements: Vec<MineralElement>,
    pub sample_input_foods: Vec<String>,
    pub rejections: Vec<NutrientRejection>,
    pub reconciliation: Reconciliation,
}

impl FoodComposition {
    pub fn component(&self, label: &str) -> Option<&Component> {
        self.components
            .iter()
            .find(|component| component.label == label)
    }

    pub fn external_record_id(&self) -> String {
        self.fdc_id.to_string()
    }
}

/// Convert a nutrient's amount into grams per [`BASIS_GRAMS`].
///
/// This is the only place a USDA unit spelling is interpreted. The result
/// carries [`BASIS_UNIT`], which the reviewed vocabulary knows; the bare `g`,
/// `mg`, `µg` spellings never reach a candidate.
fn grams_per_basis(unit: &str, amount: f64) -> Option<f64> {
    let factor = match unit.trim().to_ascii_lowercase().as_str() {
        "g" => 1.0,
        "mg" => 1e-3,
        "µg" | "ug" | "mcg" => 1e-6,
        _ => return None,
    };
    Some(amount * factor)
}

fn spread(nutrient: &FoodNutrient) -> Option<f64> {
    let (min, max) = (nutrient.min?, nutrient.max?);
    if !min.is_finite() || !max.is_finite() || max < min {
        return None;
    }
    grams_per_basis(&nutrient.nutrient.unit_name, (max - min) / 2.0)
}

fn derivation_code(nutrient: &FoodNutrient) -> String {
    nutrient
        .derivation
        .as_ref()
        .and_then(|derivation| derivation.code.clone())
        .unwrap_or_else(|| "unstated".to_owned())
}

/// Rust sums `f64` from an identity of `-0.0`, so a food with no components
/// at all — soybean oil states none of the five proximates — reports a
/// negative zero. It is numerically zero and every comparison agrees, but it
/// reads as a defect in a checked-in artifact a human reviews, so the sign is
/// normalized away at the one place the ledger totals are built.
fn without_negative_zero(value: f64) -> f64 {
    if value == 0.0 {
        0.0
    } else {
        value
    }
}

fn source_field(fdc_id: u64, nutrient_id: u32) -> String {
    format!("foods[fdcId={fdc_id}].foodNutrients[nutrient.id={nutrient_id}].amount")
}

/// Map every record in a pinned snapshot.
pub fn map_snapshot(records: &[FoodRecord]) -> Vec<FoodComposition> {
    let mut compositions: Vec<_> = records.iter().map(map_record).collect();
    compositions.sort_by_key(|composition| composition.fdc_id);
    compositions
}

/// Map one Foundation Foods record onto a mass ledger.
pub fn map_record(record: &FoodRecord) -> FoodComposition {
    let mut rejections = Vec::new();
    let mut components = Vec::new();
    let mut minerals = Vec::new();

    // Index the determinations the ledger reads, rejecting everything else
    // with a reason.
    let mut masses: BTreeMap<u32, (&FoodNutrient, f64)> = BTreeMap::new();
    for nutrient in &record.food_nutrients {
        let meta = &nutrient.nutrient;
        let Some(amount) = nutrient.amount else {
            rejections.push(NutrientRejection::CategoryHeader {
                nutrient_id: meta.id,
                name: meta.name.clone(),
            });
            continue;
        };
        let Some(grams) = grams_per_basis(&meta.unit_name, amount) else {
            rejections.push(NutrientRejection::UnitIsNotAMass {
                nutrient_id: meta.id,
                name: meta.name.clone(),
                unit: meta.unit_name.clone(),
            });
            continue;
        };
        if !grams.is_finite() || grams < 0.0 {
            rejections.push(NutrientRejection::UnusableAmount {
                nutrient_id: meta.id,
                name: meta.name.clone(),
                detail: format!("{amount} {}", meta.unit_name),
            });
            continue;
        }
        if masses.insert(meta.id, (nutrient, grams)).is_some() {
            rejections.push(NutrientRejection::UnusableAmount {
                nutrient_id: meta.id,
                name: meta.name.clone(),
                detail: "the record states this nutrient twice".to_owned(),
            });
        }
    }

    let mut read: BTreeSet<u32> = BTreeSet::new();

    // Water is the only proximate that is a single molecule.
    if let Some(&(nutrient, grams)) = masses.get(&NUTRIENT_WATER) {
        read.insert(NUTRIENT_WATER);
        components.push(Component {
            label: "water".to_owned(),
            disposition: ComponentDisposition::Species {
                species_id: "water".to_owned(),
            },
            grams_per_basis: grams,
            source_nutrient_id: Some(NUTRIENT_WATER),
            source_field: source_field(record.fdc_id, NUTRIENT_WATER),
            derivation: derivation_code(nutrient),
            spread_grams: spread(nutrient),
            data_points: nutrient.data_points,
        });
    }

    // The aggregate populations. Naming them is the whole point: they are
    // conserved, displayed, and never converted into a molecule.
    for (nutrient_id, label) in [
        (NUTRIENT_PROTEIN, AGGREGATE_PROTEIN),
        (NUTRIENT_FAT, AGGREGATE_FAT),
        (NUTRIENT_ASH, AGGREGATE_ASH),
        (NUTRIENT_FIBRE, AGGREGATE_FIBRE),
    ] {
        if let Some(&(nutrient, grams)) = masses.get(&nutrient_id) {
            read.insert(nutrient_id);
            components.push(Component {
                label: label.to_owned(),
                disposition: ComponentDisposition::NamedUnresolved {
                    reason: UnresolvedReason::AggregatePopulation,
                },
                grams_per_basis: grams,
                source_nutrient_id: Some(nutrient_id),
                source_field: source_field(record.fdc_id, nutrient_id),
                derivation: derivation_code(nutrient),
                spread_grams: spread(nutrient),
                data_points: nutrient.data_points,
            });
        }
    }

    // Individually determined compounds. `Sugars, Total` is deliberately not
    // read: a total is not a determination of any one sugar.
    let mut sugars_reported_individually = false;
    let mut subscribed = 0.0;
    for (nutrient_id, label, species) in SUGARS.iter().chain(ORGANIC_ACIDS.iter()) {
        let Some(&(nutrient, grams)) = masses.get(nutrient_id) else {
            continue;
        };
        read.insert(*nutrient_id);
        if SUGARS.iter().any(|(id, _, _)| id == nutrient_id) {
            sugars_reported_individually = true;
        }
        subscribed += grams;
        if grams == 0.0 {
            // A determined zero is real information — it is why this adapter
            // may say milk sugar is absent — but a zero-mass component would
            // only clutter the ledger.
            continue;
        }
        components.push(Component {
            label: (*label).to_owned(),
            disposition: match species {
                Some(species_id) => ComponentDisposition::Species {
                    species_id: (*species_id).to_owned(),
                },
                None => ComponentDisposition::NamedUnresolved {
                    reason: UnresolvedReason::NoRegistrySpecies,
                },
            },
            grams_per_basis: grams,
            source_nutrient_id: Some(*nutrient_id),
            source_field: source_field(record.fdc_id, *nutrient_id),
            derivation: derivation_code(nutrient),
            spread_grams: spread(nutrient),
            data_points: nutrient.data_points,
        });
    }

    if let Some(&(nutrient, grams)) = masses.get(&NUTRIENT_STARCH) {
        read.insert(NUTRIENT_STARCH);
        subscribed += grams;
        if grams > 0.0 {
            components.push(Component {
                label: "starch".to_owned(),
                disposition: ComponentDisposition::Species {
                    species_id: "starch".to_owned(),
                },
                grams_per_basis: grams,
                source_nutrient_id: Some(NUTRIENT_STARCH),
                source_field: source_field(record.fdc_id, NUTRIENT_STARCH),
                derivation: derivation_code(nutrient),
                spread_grams: spread(nutrient),
                data_points: nutrient.data_points,
            });
        }
    }

    if let Some(&(nutrient, grams)) = masses.get(&NUTRIENT_ALCOHOL) {
        read.insert(NUTRIENT_ALCOHOL);
        if grams > 0.0 {
            components.push(Component {
                label: "ethanol".to_owned(),
                disposition: ComponentDisposition::Species {
                    species_id: "ethanol".to_owned(),
                },
                grams_per_basis: grams,
                source_nutrient_id: Some(NUTRIENT_ALCOHOL),
                source_field: source_field(record.fdc_id, NUTRIENT_ALCOHOL),
                derivation: derivation_code(nutrient),
                spread_grams: spread(nutrient),
                data_points: nutrient.data_points,
            });
        }
    }

    // Fibre is already a ledger component; it is also inside the carbohydrate
    // total, so it counts against the carbohydrate subscription.
    if let Some(&(_, grams)) = masses.get(&NUTRIENT_FIBRE) {
        subscribed += grams;
    }

    // Minerals: reported, never speciated.
    for (nutrient_id, element) in MINERALS {
        let Some(&(nutrient, grams)) = masses.get(nutrient_id) else {
            continue;
        };
        read.insert(*nutrient_id);
        rejections.push(NutrientRejection::ElementalTotalNotSpeciated {
            nutrient_id: *nutrient_id,
            element: (*element).to_owned(),
            grams_per_basis: grams,
        });
        if grams > 0.0 {
            minerals.push(MineralElement {
                element: (*element).to_owned(),
                grams_per_basis: grams,
                source_nutrient_id: *nutrient_id,
                source_field: source_field(record.fdc_id, *nutrient_id),
                spread_grams: spread(nutrient),
            });
        }
    }

    // The carbohydrate remainder. This is the adapter's own derivation and is
    // labelled as such.
    let (carbohydrate, closure) = match (
        masses.get(&NUTRIENT_CARBOHYDRATE_BY_DIFFERENCE),
        masses.get(&NUTRIENT_CARBOHYDRATE_BY_SUMMATION),
    ) {
        (Some(&(_, grams)), _) => {
            read.insert(NUTRIENT_CARBOHYDRATE_BY_DIFFERENCE);
            (Some(grams), CarbohydrateClosure::ByDifference)
        }
        (None, Some(&(_, grams))) => {
            read.insert(NUTRIENT_CARBOHYDRATE_BY_SUMMATION);
            (Some(grams), CarbohydrateClosure::BySummation)
        }
        (None, None) => (None, CarbohydrateClosure::Absent),
    };

    // Everything the ledger did not read.
    for (nutrient_id, &(nutrient, grams)) in &masses {
        if read.contains(nutrient_id) {
            continue;
        }
        if let Some((_, detail)) = DUPLICATE_TOTALS.iter().find(|(id, _)| id == nutrient_id) {
            rejections.push(NutrientRejection::DuplicateTotal {
                nutrient_id: *nutrient_id,
                name: nutrient.nutrient.name.clone(),
                detail: (*detail).to_owned(),
            });
            continue;
        }
        rejections.push(NutrientRejection::NoReviewedComponent {
            nutrient_id: *nutrient_id,
            name: nutrient.nutrient.name.clone(),
            unit: nutrient.nutrient.unit_name.clone(),
            amount: grams,
        });
    }

    // Tolerance comes from the record's own spread, never from a number the
    // adapter picked to make the test pass.
    let mut tolerance: f64 = components
        .iter()
        .filter_map(|component| component.spread_grams)
        .sum();
    tolerance = tolerance.max(MIN_TOLERANCE_GRAMS);

    let mut conflicts = Vec::new();
    let mut oversubscribed_within_uncertainty = false;

    if let Some(carbohydrate) = carbohydrate {
        let remainder = carbohydrate - subscribed;
        if remainder < -tolerance {
            conflicts.push(ReconciliationConflict::CarbohydrateOverSubscribed {
                carbohydrate_grams: carbohydrate,
                subscribed_grams: subscribed,
                excess_grams: -remainder,
                tolerance_grams: tolerance,
            });
        } else if remainder < 0.0 {
            oversubscribed_within_uncertainty = true;
        }
        if remainder > 0.0 {
            components.push(Component {
                label: AGGREGATE_OTHER_CARBOHYDRATE.to_owned(),
                disposition: ComponentDisposition::NamedUnresolved {
                    reason: UnresolvedReason::AggregatePopulation,
                },
                grams_per_basis: remainder,
                source_nutrient_id: None,
                source_field: format!(
                    "foods[fdcId={}]: carbohydrate less every individual sugar, \
                     starch, fibre and organic-acid determination",
                    record.fdc_id
                ),
                derivation: "adapter:residual".to_owned(),
                spread_grams: None,
                data_points: None,
            });
        }
    }

    let missing: Vec<String> = [
        (masses.contains_key(&NUTRIENT_WATER), "water"),
        (masses.contains_key(&NUTRIENT_PROTEIN), "protein"),
        (masses.contains_key(&NUTRIENT_FAT), "total lipid (fat)"),
        (masses.contains_key(&NUTRIENT_ASH), "ash"),
        (carbohydrate.is_some(), "carbohydrate"),
    ]
    .into_iter()
    .filter(|(present, _)| !present)
    .map(|(_, name)| name.to_owned())
    .collect();
    if !missing.is_empty() {
        conflicts.push(ReconciliationConflict::MissingProximate { missing });
    }

    let mineral_grams: f64 = minerals.iter().map(|mineral| mineral.grams_per_basis).sum();
    let ash_grams = masses.get(&NUTRIENT_ASH).map_or(0.0, |&(_, grams)| grams);
    if masses.contains_key(&NUTRIENT_ASH) && mineral_grams > ash_grams + tolerance {
        conflicts.push(ReconciliationConflict::MineralsExceedAsh {
            mineral_grams,
            ash_grams,
            tolerance_grams: tolerance,
        });
    }

    components.sort_by(|a, b| a.label.cmp(&b.label));
    minerals.sort_by(|a, b| a.element.cmp(&b.element));
    rejections.sort_by_key(rejection_sort_key);

    let resolved_grams = without_negative_zero(
        components
            .iter()
            .filter(|component| component.is_resolved())
            .map(|component| component.grams_per_basis)
            .sum(),
    );
    let named_unresolved_grams = without_negative_zero(
        components
            .iter()
            .filter(|component| !component.is_resolved())
            .map(|component| component.grams_per_basis)
            .sum(),
    );
    let total_grams = without_negative_zero(resolved_grams + named_unresolved_grams);
    let residual_grams = without_negative_zero(BASIS_GRAMS - total_grams);
    if residual_grams.abs() > tolerance {
        conflicts.push(ReconciliationConflict::ResidualOutsideUncertainty {
            residual_grams,
            tolerance_grams: tolerance,
        });
    }

    FoodComposition {
        fdc_id: record.fdc_id,
        description: record.description.clone(),
        data_type: record.data_type.clone(),
        food_category: record
            .food_category
            .as_ref()
            .and_then(|category| category.description.clone()),
        publication_date: record.publication_date.clone(),
        ndb_number: record.ndb_number.as_ref().map(render_ndb),
        footnote: record.footnote.clone(),
        basis: format!("{BASIS_GRAMS:.0} g edible portion"),
        basis_grams: BASIS_GRAMS,
        sugars_reported_individually,
        components,
        mineral_elements: minerals,
        sample_input_foods: record
            .input_foods
            .iter()
            .filter_map(|input| input.food_description.clone())
            .collect(),
        rejections,
        reconciliation: Reconciliation {
            basis_grams: BASIS_GRAMS,
            resolved_grams,
            named_unresolved_grams,
            total_grams,
            residual_grams,
            tolerance_grams: tolerance,
            closure,
            oversubscribed_within_uncertainty,
            conflicts,
        },
    }
}

fn render_ndb(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

fn rejection_sort_key(rejection: &NutrientRejection) -> (u8, u32) {
    match rejection {
        NutrientRejection::CategoryHeader { nutrient_id, .. } => (0, *nutrient_id),
        NutrientRejection::UnitIsNotAMass { nutrient_id, .. } => (1, *nutrient_id),
        NutrientRejection::UnusableAmount { nutrient_id, .. } => (2, *nutrient_id),
        NutrientRejection::ElementalTotalNotSpeciated { nutrient_id, .. } => (3, *nutrient_id),
        NutrientRejection::DuplicateTotal { nutrient_id, .. } => (4, *nutrient_id),
        NutrientRejection::NoReviewedComponent { nutrient_id, .. } => (5, *nutrient_id),
    }
}

// ── registry gaps ───────────────────────────────────────────────────────────

/// A species the adapter proposes that the registry does not carry yet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistryGap {
    pub species_id: String,
    /// The component labels that would resolve onto it.
    pub components: Vec<String>,
    /// Which foods need it.
    pub fdc_ids: Vec<u64>,
}

/// Which proposed species the registry does not carry.
///
/// This is deliberately separate from mapping: a candidate must not change
/// shape because a species landed on another branch. The gap report is where
/// "we would resolve this if the registry had it" is said out loud.
pub fn registry_gaps(
    compositions: &[FoodComposition],
    installed_species: &BTreeSet<String>,
) -> Vec<RegistryGap> {
    #[derive(Default)]
    struct Users {
        components: BTreeSet<String>,
        fdc_ids: BTreeSet<u64>,
    }

    let mut gaps: BTreeMap<String, Users> = BTreeMap::new();
    for composition in compositions {
        for component in &composition.components {
            let Some(species_id) = component.species_id() else {
                continue;
            };
            if installed_species.contains(species_id) {
                continue;
            }
            let entry = gaps.entry(species_id.to_owned()).or_default();
            entry.components.insert(component.label.clone());
            entry.fdc_ids.insert(composition.fdc_id);
        }
    }
    gaps.into_iter()
        .map(|(species_id, users)| RegistryGap {
            species_id,
            components: users.components.into_iter().collect(),
            fdc_ids: users.fdc_ids.into_iter().collect(),
        })
        .collect()
}

// ── quarantine candidates ───────────────────────────────────────────────────

/// Field-name prefixes, so a policy and a reviewer agree on what a field is.
pub const FIELD_COMPONENT: &str = "component.";
pub const FIELD_UNRESOLVED: &str = "unresolved.";
pub const FIELD_MINERAL: &str = "mineral_element.";
pub const FIELD_SPREAD: &str = "spread.";

/// The metadata fields a promotion policy allowlists.
const PROMOTABLE_METADATA: &[&str] = &["description", "basis", "food_category", "publication_date"];

/// The metadata fields a candidate carries but no policy admits: they describe
/// the *import*, not the material.
const IMPORT_ONLY_METADATA: &[&str] = &[
    "data_type",
    "food_class",
    "ndb_number",
    "sample_input_foods",
];

fn number(value: f64) -> Value {
    Number::from_f64(value).map_or(Value::Null, Value::Number)
}

fn quantity(value: f64, source_field: &str) -> CandidateField {
    CandidateField::new(number(value), source_field, LICENCE).with_unit(BASIS_UNIT)
}

/// Turn reconciled compositions into quarantine candidates.
///
/// A food that does not reconcile is **not** here: it is in [`conflicts`]. A
/// composition whose own numbers do not add up is a report for a reviewer, not
/// a candidate for a pack.
pub fn candidates(compositions: &[FoodComposition]) -> Vec<QuarantinedCandidate> {
    compositions
        .iter()
        .filter(|composition| composition.reconciliation.reconciles())
        .map(candidate)
        .collect()
}

/// Every food whose ledger a reviewer must look at instead.
pub fn conflicts(compositions: &[FoodComposition]) -> Vec<&FoodComposition> {
    compositions
        .iter()
        .filter(|composition| !composition.reconciliation.reconciles())
        .collect()
}

fn candidate(composition: &FoodComposition) -> QuarantinedCandidate {
    let record_id = format!("{SOURCE_ID}:{}", composition.fdc_id);
    let mut fields: BTreeMap<String, CandidateField> = BTreeMap::new();

    let text = |value: &str, source: &str| {
        CandidateField::new(Value::String(value.to_owned()), source, LICENCE)
    };

    fields.insert(
        "description".to_owned(),
        text(
            &composition.description,
            &format!("foods[fdcId={}].description", composition.fdc_id),
        ),
    );
    fields.insert(
        "basis".to_owned(),
        // The basis is Foundation Foods' own convention, not a measured
        // quantity, so it stays a phrase rather than a bare mass.
        text(
            &composition.basis,
            "FoodData Central: Foundation Foods amounts are per 100 g edible portion",
        ),
    );
    fields.insert(
        "data_type".to_owned(),
        text(
            &composition.data_type,
            &format!("foods[fdcId={}].dataType", composition.fdc_id),
        ),
    );
    if let Some(category) = &composition.food_category {
        fields.insert(
            "food_category".to_owned(),
            text(
                category,
                &format!(
                    "foods[fdcId={}].foodCategory.description",
                    composition.fdc_id
                ),
            ),
        );
    }
    if let Some(published) = &composition.publication_date {
        fields.insert(
            "publication_date".to_owned(),
            text(
                published,
                &format!("foods[fdcId={}].publicationDate", composition.fdc_id),
            ),
        );
    }
    if let Some(ndb) = &composition.ndb_number {
        fields.insert(
            "ndb_number".to_owned(),
            text(
                ndb,
                &format!("foods[fdcId={}].ndbNumber", composition.fdc_id),
            ),
        );
    }
    if !composition.sample_input_foods.is_empty() {
        fields.insert(
            "sample_input_foods".to_owned(),
            CandidateField::new(
                Value::Array(
                    composition
                        .sample_input_foods
                        .iter()
                        .map(|input| Value::String(input.clone()))
                        .collect(),
                ),
                format!(
                    "foods[fdcId={}].inputFoods[].foodDescription",
                    composition.fdc_id
                ),
                LICENCE,
            ),
        );
    }

    for component in &composition.components {
        let prefix = if component.is_resolved() {
            FIELD_COMPONENT
        } else {
            FIELD_UNRESOLVED
        };
        fields.insert(
            format!("{prefix}{}", component.label),
            quantity(component.grams_per_basis, &component.source_field),
        );
        if let Some(spread) = component.spread_grams {
            fields.insert(
                format!("{FIELD_SPREAD}{}", component.label),
                quantity(
                    spread,
                    &component.source_field.replace(".amount", ".(max-min)/2"),
                ),
            );
        }
    }

    for mineral in &composition.mineral_elements {
        fields.insert(
            format!("{FIELD_MINERAL}{}", mineral.element),
            quantity(mineral.grams_per_basis, &mineral.source_field),
        );
    }

    QuarantinedCandidate {
        adapter_id: ADAPTER_ID.to_owned(),
        source_record_id: record_id,
        external_record_id: composition.external_record_id(),
        // A food is not a chemical identity, so there is no InChIKey to join
        // on. The legacy NDB number is the one stable key two Foundation
        // records could collide on, and a collision is worth reporting.
        identity_key: composition
            .ndb_number
            .as_ref()
            .map(|ndb| format!("usda-ndb:{ndb}")),
        fields,
    }
}

/// The reviewed promotion allowlist for this adapter.
///
/// Composition, its uncertainty and the material's own description may be
/// promoted; import bookkeeping may not. Building it from the candidate set
/// is what keeps it exact — a policy naming a field no record carries, or a
/// record carrying a field no policy names, is precisely what
/// [`crate::lint_promotion`] and `review_candidates` are for.
pub fn promotion_policy(candidates: &[QuarantinedCandidate]) -> PromotionPolicy {
    let mut fields: BTreeMap<String, RuntimeFieldPolicy> = BTreeMap::new();
    let licences = [LICENCE];

    for candidate in candidates {
        for name in candidate.fields.keys() {
            if IMPORT_ONLY_METADATA.contains(&name.as_str()) {
                continue;
            }
            if PROMOTABLE_METADATA.contains(&name.as_str()) {
                fields.entry(name.clone()).or_insert_with(|| {
                    RuntimeFieldPolicy::new(format!("material.{name}"), licences)
                });
                continue;
            }
            for (prefix, target) in [
                (FIELD_COMPONENT, "material.component."),
                (FIELD_UNRESOLVED, "material.unresolved."),
                (FIELD_MINERAL, "material.mineral_element."),
                (FIELD_SPREAD, "material.spread."),
            ] {
                if let Some(rest) = name.strip_prefix(prefix) {
                    fields.entry(name.clone()).or_insert_with(|| {
                        RuntimeFieldPolicy::new(format!("{target}{rest}"), licences)
                            .with_dimension(Dimension::MassPerMass)
                    });
                    break;
                }
            }
        }
    }

    PromotionPolicy { fields }
}

/// The eligible-field lists a promotion dry run submits.
///
/// *Proposed*, not signed off: this is what the adapter would ask a reviewer
/// to approve — composition, its uncertainty, the mineral inventory, and the
/// material's own description. Import bookkeeping is not proposed at all, and
/// nothing here promotes anything.
pub fn proposed_eligible_fields(candidates: &[QuarantinedCandidate]) -> Vec<EligibleFieldList> {
    let policy = promotion_policy(candidates);
    candidates
        .iter()
        .map(|candidate| EligibleFieldList {
            adapter_id: candidate.adapter_id.clone(),
            external_record_id: candidate.external_record_id.clone(),
            fields: candidate
                .fields
                .keys()
                .filter(|field| policy.fields.contains_key(field.as_str()))
                .cloned()
                .collect(),
        })
        .collect()
}

// ── the import report ───────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FoodLedgerRow {
    pub fdc_id: u64,
    pub description: String,
    pub resolved_grams: f64,
    pub named_unresolved_grams: f64,
    pub total_grams: f64,
    pub residual_grams: f64,
    pub tolerance_grams: f64,
    pub closure: CarbohydrateClosure,
    pub sugars_reported_individually: bool,
    pub resolved_species: Vec<String>,
    pub named_unresolved: Vec<String>,
    pub conflicts: Vec<ReconciliationConflict>,
}

/// The deterministic artifact a reviewer reads: one row per food, and every
/// rejection class with its count.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportReport {
    pub schema: u32,
    pub adapter_id: String,
    pub source_id: String,
    pub licence: String,
    pub basis: String,
    pub food_count: usize,
    pub candidate_count: usize,
    pub conflict_count: usize,
    pub ledger: Vec<FoodLedgerRow>,
    pub rejection_counts: BTreeMap<String, usize>,
}

/// The deterministic ledger artifact.
///
/// It deliberately says nothing about the registry: it is a function of the
/// pinned snapshot alone, so it can be checked in and byte-compared without
/// breaking every time a species lands on another branch. The registry side
/// of the story is [`registry_gaps`], which is written beside it and is
/// expected to move.
pub fn import_report(compositions: &[FoodComposition]) -> ImportReport {
    let mut rejection_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut ledger = Vec::new();
    for composition in compositions {
        for rejection in &composition.rejections {
            *rejection_counts
                .entry(rejection_class(rejection).to_owned())
                .or_default() += 1;
        }
        ledger.push(FoodLedgerRow {
            fdc_id: composition.fdc_id,
            description: composition.description.clone(),
            resolved_grams: composition.reconciliation.resolved_grams,
            named_unresolved_grams: composition.reconciliation.named_unresolved_grams,
            total_grams: composition.reconciliation.total_grams,
            residual_grams: composition.reconciliation.residual_grams,
            tolerance_grams: composition.reconciliation.tolerance_grams,
            closure: composition.reconciliation.closure,
            sugars_reported_individually: composition.sugars_reported_individually,
            resolved_species: composition
                .components
                .iter()
                .filter_map(|component| component.species_id().map(str::to_owned))
                .collect(),
            named_unresolved: composition
                .components
                .iter()
                .filter(|component| !component.is_resolved())
                .map(|component| component.label.clone())
                .collect(),
            conflicts: composition.reconciliation.conflicts.clone(),
        });
    }

    ImportReport {
        schema: ADAPTER_SCHEMA_VERSION,
        adapter_id: ADAPTER_ID.to_owned(),
        source_id: SOURCE_ID.to_owned(),
        licence: LICENCE.to_owned(),
        basis: format!("{BASIS_GRAMS:.0} g edible portion"),
        food_count: compositions.len(),
        candidate_count: compositions
            .iter()
            .filter(|composition| composition.reconciliation.reconciles())
            .count(),
        conflict_count: compositions
            .iter()
            .filter(|composition| !composition.reconciliation.reconciles())
            .count(),
        ledger,
        rejection_counts,
    }
}

fn rejection_class(rejection: &NutrientRejection) -> &'static str {
    match rejection {
        NutrientRejection::CategoryHeader { .. } => "category_header",
        NutrientRejection::UnitIsNotAMass { .. } => "unit_is_not_a_mass",
        NutrientRejection::UnusableAmount { .. } => "unusable_amount",
        NutrientRejection::ElementalTotalNotSpeciated { .. } => "elemental_total_not_speciated",
        NutrientRejection::DuplicateTotal { .. } => "duplicate_total",
        NutrientRejection::NoReviewedComponent { .. } => "no_reviewed_component",
    }
}

/// Stable bytes for a checked-in report artifact.
pub fn canonical_report_bytes<T: Serialize>(report: &T) -> Result<Vec<u8>, serde_json::Error> {
    let mut bytes = serde_json::to_vec_pretty(report)?;
    bytes.push(b'\n');
    Ok(bytes)
}
