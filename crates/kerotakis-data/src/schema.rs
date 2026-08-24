use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Version of the source-record contract, independent of future runtime-pack
/// versions.
pub const REGISTRY_SCHEMA_VERSION: u32 = 1;

/// A complete reviewable source registry.
///
/// The record families are separate arrays on purpose: identity corrections,
/// thermodynamic models, safety classifications, and visual observations have
/// different sources and review cycles. They meet only through stable species
/// identifiers.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RegistryDocument {
    pub schema: u32,
    #[serde(default)]
    pub sources: Vec<SourceRecord>,
    #[serde(default)]
    pub identities: Vec<IdentityRecord>,
    #[serde(default)]
    pub compositions: Vec<CompositionRecord>,
    #[serde(default)]
    pub phase_thermodynamics: Vec<PhaseThermodynamicRecord>,
    #[serde(default)]
    pub transport: Vec<TransportRecord>,
    #[serde(default)]
    pub optical: Vec<OpticalRecord>,
    #[serde(default)]
    pub safety: Vec<SafetyRecord>,
    #[serde(default)]
    pub microstates: Vec<MicrostateRecord>,
    #[serde(default)]
    pub model_parameters: Vec<ModelParameterRecord>,
}

impl RegistryDocument {
    pub fn empty() -> Self {
        Self {
            schema: REGISTRY_SCHEMA_VERSION,
            ..Self::default()
        }
    }
}

/// Where a source may participate in the product.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceLane {
    /// Cleared material that may be compiled into distributed app data.
    Runtime,
    /// A build/test dependency whose raw records never enter an app pack.
    BuildOracle,
    /// A remote comparison service; only reviewed derived facts may cross it.
    ExternalOracle,
    /// Species added for a specific EXP task; eligible for the runtime pack.
    ExperimentData,
}

impl SourceLane {
    /// Whether records from this source are eligible to enter a distributed
    /// runtime pack. Licence review assigns the lane; the pack compiler does
    /// not reinterpret licence strings or copy oracle material.
    pub const fn may_enter_runtime_pack(self) -> bool {
        matches!(self, Self::Runtime | Self::ExperimentData)
    }
}

/// Bibliographic and distribution identity for one source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceRecord {
    pub id: String,
    pub citation: String,
    /// SPDX expression or a project-local `LicenseRef-*` identifier.
    pub licence: String,
    pub lane: SourceLane,
    #[serde(default)]
    pub origin: Option<String>,
    #[serde(default)]
    pub revision: Option<String>,
    #[serde(default)]
    pub retrieved: Option<String>,
}

/// Stable names and non-numeric identifiers only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityRecord {
    pub id: String,
    pub canonical_key: String,
    pub name: String,
    #[serde(default)]
    pub identifiers: BTreeMap<String, String>,
    #[serde(default)]
    pub synonyms: Vec<String>,
    pub evidence: Evidence,
}

/// Formula, elemental makeup, and net charge.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositionRecord {
    pub id: String,
    pub species_id: String,
    pub formula: String,
    pub elements: Vec<ElementAmount>,
    pub net_charge: NumericRecord,
    /// Evidence for the formula itself; elemental counts and charge retain
    /// their own numeric evidence.
    pub evidence: Evidence,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ElementAmount {
    pub element: String,
    /// Dimensionless stoichiometric count. Isotopologues may use a non-integer
    /// reviewed value; validation therefore requires non-negative, not integer.
    pub count: NumericRecord,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Solid,
    Liquid,
    Aqueous,
    Gas,
    Plasma,
    Supercritical,
}

/// A single phase-specific thermodynamic fact. One property per record keeps
/// its conditions, uncertainty, source, and method indivisible.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhaseThermodynamicRecord {
    pub id: String,
    pub species_id: String,
    pub phase: Phase,
    pub property: PhaseProperty,
    pub quantity: NumericRecord,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhaseProperty {
    MolarMass,
    MassDensity,
    MolarHeatCapacity,
    StandardEnthalpyOfFormation,
    StandardGibbsEnergyOfFormation,
    StandardMolarEntropy,
    MeltingTemperature,
    BoilingTemperature,
    VapourPressure,
    EnthalpyOfFusion,
    EnthalpyOfVaporisation,
    EnthalpyOfDissolution,
    Other(String),
}

impl PhaseProperty {
    pub fn expected_dimension(&self) -> Option<Dimension> {
        use Dimension::*;
        match self {
            Self::MolarMass => Some(MolarMass),
            Self::MassDensity => Some(MassDensity),
            Self::MolarHeatCapacity => Some(MolarHeatCapacity),
            Self::StandardEnthalpyOfFormation
            | Self::StandardGibbsEnergyOfFormation
            | Self::EnthalpyOfFusion
            | Self::EnthalpyOfVaporisation
            | Self::EnthalpyOfDissolution => Some(MolarEnergy),
            Self::StandardMolarEntropy => Some(MolarEntropy),
            Self::MeltingTemperature | Self::BoilingTemperature => Some(Temperature),
            Self::VapourPressure => Some(Pressure),
            Self::Other(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransportRecord {
    pub id: String,
    pub species_id: String,
    pub phase: Phase,
    pub property: TransportProperty,
    pub quantity: NumericRecord,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransportProperty {
    Diffusivity,
    DynamicViscosity,
    ThermalConductivity,
    SurfaceTension,
    ElectricalConductivity,
    Other(String),
}

impl TransportProperty {
    pub fn expected_dimension(&self) -> Option<Dimension> {
        use Dimension::*;
        match self {
            Self::Diffusivity => Some(Diffusivity),
            Self::DynamicViscosity => Some(DynamicViscosity),
            Self::ThermalConductivity => Some(ThermalConductivity),
            Self::SurfaceTension => Some(SurfaceTension),
            Self::ElectricalConductivity => Some(ElectricalConductivity),
            Self::Other(_) => None,
        }
    }
}

/// Human-visible properties and, where available, quantitative spectra.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpticalRecord {
    pub id: String,
    pub species_id: String,
    pub phase: Phase,
    #[serde(default)]
    pub appearance: Option<String>,
    /// Characteristic atomic-emission observation, kept distinct from the
    /// material's reflective or transmitted colour.
    #[serde(default)]
    pub flame_colour: Option<String>,
    /// `#RRGGBB`; encoded as text so an RGB triplet cannot become untraced
    /// scientific numeric data.
    #[serde(default)]
    pub reflective_srgb: Option<String>,
    #[serde(default)]
    pub spectrum: Vec<SpectralSample>,
    /// Evidence for qualitative appearance and encoded sRGB. Quantitative
    /// spectrum points carry their own evidence.
    pub evidence: Evidence,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpectralSample {
    pub wavelength: NumericRecord,
    pub molar_absorptivity: NumericRecord,
}

/// Qualitative classifications plus quantitative safety limits.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SafetyRecord {
    pub id: String,
    pub species_id: String,
    #[serde(default)]
    pub classifications: Vec<String>,
    #[serde(default)]
    pub statements: Vec<String>,
    #[serde(default)]
    pub limits: Vec<SafetyLimit>,
    pub evidence: Evidence,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SafetyLimit {
    pub kind: SafetyLimitKind,
    pub quantity: NumericRecord,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SafetyLimitKind {
    ExposureConcentration,
    AcuteDose,
    FlashTemperature,
    AutoignitionTemperature,
    LowerExplosiveFraction,
    UpperExplosiveFraction,
    Other(String),
}

impl SafetyLimitKind {
    pub fn expected_dimension(&self) -> Option<Dimension> {
        use Dimension::*;
        match self {
            Self::ExposureConcentration => Some(MassConcentration),
            Self::AcuteDose => Some(MassPerMass),
            Self::FlashTemperature | Self::AutoignitionTemperature => Some(Temperature),
            Self::LowerExplosiveFraction | Self::UpperExplosiveFraction => Some(Dimensionless),
            Self::Other(_) => None,
        }
    }
}

/// Protonation, tautomeric, spin, oxidation, or conformational state that a
/// resolver may distinguish without pretending it is a separate identity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MicrostateRecord {
    pub id: String,
    pub species_id: String,
    pub label: String,
    pub kind: MicrostateKind,
    pub formal_charge: NumericRecord,
    #[serde(default)]
    pub relative_energy: Option<NumericRecord>,
    #[serde(default)]
    pub equilibrium_fraction: Option<NumericRecord>,
    /// Evidence for the existence and classification of the microstate;
    /// numeric fields remain independently sourced.
    pub evidence: Evidence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MicrostateKind {
    Protonation,
    Tautomer,
    Oxidation,
    Spin,
    Conformer,
    Other(String),
}

/// A model-specific number that is not an intrinsic species property.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelParameterRecord {
    pub id: String,
    pub subject: ModelSubject,
    pub model: String,
    pub parameter: String,
    pub quantity: NumericRecord,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "id", rename_all = "snake_case")]
pub enum ModelSubject {
    Species(String),
    Reaction(String),
    Apparatus(String),
    Material(String),
}

/// Physical dimension, kept separate from the unit spelling so incompatible
/// units can be rejected before conversion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Dimension {
    Dimensionless,
    Amount,
    MolarMass,
    Temperature,
    Pressure,
    MassDensity,
    MolarHeatCapacity,
    MolarEnergy,
    MolarEntropy,
    Concentration,
    MassConcentration,
    MassPerMass,
    Diffusivity,
    DynamicViscosity,
    ThermalConductivity,
    SurfaceTension,
    ElectricalConductivity,
    Wavelength,
    MolarAbsorptivity,
    Time,
    Area,
    Volume,
    RateConstant,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Unit {
    /// UCUM spelling where available, otherwise a documented project spelling.
    pub symbol: String,
    pub dimension: Dimension,
}

/// One scientific numeric fact and all metadata needed to judge or reproduce
/// it. Conditions inherit this record's source and method; they are not
/// free-standing property claims.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NumericRecord {
    pub value: f64,
    pub unit: Unit,
    #[serde(default)]
    pub conditions: Applicability,
    pub uncertainty: Uncertainty,
    pub source_id: String,
    pub method: Method,
}

/// Provenance for non-numeric claims. Numeric claims embed the same two
/// fields directly so their evidence cannot be separated from the value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Evidence {
    pub source_id: String,
    pub method: Method,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Applicability {
    #[serde(default)]
    pub temperature: Option<Interval>,
    #[serde(default)]
    pub pressure: Option<Interval>,
    #[serde(default)]
    pub ph: Option<Interval>,
    #[serde(default)]
    pub ionic_strength: Option<Interval>,
    #[serde(default)]
    pub medium: Option<String>,
    #[serde(default)]
    pub phase: Option<Phase>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Interval {
    pub lower: f64,
    pub upper: f64,
    pub unit: Unit,
}

/// Explicit uncertainty, including the honest states "exact by definition"
/// and "the source did not report one".
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Uncertainty {
    Exact,
    NotReported,
    Absolute { plus_minus: f64 },
    Relative { fraction: f64 },
    Interval { lower: f64, upper: f64 },
}

/// How the value entered the registry. Free text is required inside every
/// variant so "calculated" cannot hide which model or transformation was used.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "detail", rename_all = "snake_case")]
pub enum Method {
    Measured(String),
    Calculated(String),
    Derived(String),
    Imported(String),
    Editorial(String),
    Curated(String),
}

impl Method {
    pub(crate) fn detail(&self) -> &str {
        match self {
            Self::Measured(detail)
            | Self::Calculated(detail)
            | Self::Derived(detail)
            | Self::Imported(detail)
            | Self::Editorial(detail)
            | Self::Curated(detail) => detail,
        }
    }
}
