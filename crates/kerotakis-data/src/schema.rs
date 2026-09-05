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
    /// Familiar named mixtures and objects. Recipes remain distinct from
    /// canonical species identities and expand into those identities only
    /// when material is dispensed.
    #[serde(default)]
    pub material_recipes: Vec<MaterialRecipe>,
}

/// A versioned, reviewable description of a familiar mixture or object.
///
/// Fractions all use the recipe's common basis. A ranged recipe samples one
/// deterministic position through every component interval; this preserves
/// correlations and never invents ambient randomness. Any balance that is not
/// chemically resolved remains explicit in `unresolved_fraction`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterialRecipe {
    pub id: String,
    pub version: u32,
    pub canonical_key: String,
    pub name: String,
    /// BCP-47 language tag to aliases in that language (for example `en` and
    /// `de`). The canonical key is always accepted independently.
    #[serde(default)]
    pub aliases: BTreeMap<String, Vec<String>>,
    pub basis: MaterialBasis,
    /// Bulk density for converting a dispensed volume into the recipe basis.
    /// Required when a mass-fraction recipe accepts mL input; absent means the
    /// caller must use the native basis rather than guessing a density.
    #[serde(default)]
    pub bulk_density: Option<NumericRecord>,
    pub components: Vec<MaterialComponent>,
    #[serde(default)]
    pub unresolved_fraction: Option<FractionRange>,
    pub physical_form: MaterialPhysicalForm,
    /// Bounded functional behavior supplied by the named material rather than
    /// by any one resolved molecule (for example a proprietary detergent's
    /// ability to stabilize foam).
    #[serde(default)]
    pub roles: Vec<MaterialRole>,
    #[serde(default)]
    pub preparation: Option<String>,
    #[serde(default)]
    pub lot_assumptions: Vec<String>,
    #[serde(default)]
    pub substitutions: Vec<MaterialSubstitution>,
    pub confidence: MaterialConfidence,
    pub expansion_policy: MaterialExpansionPolicy,
    pub evidence: Evidence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterialBasis {
    MassFraction,
    MoleFraction,
    VolumeFraction,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterialComponent {
    pub species_id: String,
    pub fraction: FractionRange,
    pub evidence: Evidence,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FractionRange {
    pub lower: f64,
    pub upper: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MaterialPhysicalForm {
    HomogeneousLiquid,
    Suspension,
    Powder,
    Granules,
    BulkSolid,
    GasMixture,
    CompositeObject {
        #[serde(default)]
        geometry: Option<MaterialGeometry>,
    },
    Other {
        description: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MaterialRole {
    CoherentObject,
    OsmoticMembrane {
        internal_osmolarity_mol_per_litre: f64,
    },
    BrowningSurface,
    FattySoapEquivalent {
        moles_per_gram: f64,
    },
    /// Empirical bridge from gas made by chemistry to a foam visual. Values
    /// are recipe-level teaching surrogates, not claims about a hidden exact
    /// surfactant formulation.
    FoamStabilizer {
        trapping_efficiency: f64,
        gas_volume_fraction: f64,
        half_life_seconds: f64,
        /// Amount of unresolved functional blend, in the recipe basis, at
        /// which the bounded foam effect reaches full strength.
        saturation_amount: f64,
    },
    /// Effective Kubelka–Munk coefficients for an opaque pigment/binder
    /// surrogate. Samples align with the runtime visible bands (405–705 nm in
    /// 20 nm steps). They are K and S model coefficients, never display RGB.
    OpaquePigment {
        absorption: Vec<f64>,
        scattering: Vec<f64>,
    },
    /// A named unresolved powder whose grains remain visibly afloat on a
    /// quiet water surface. This is a bounded classroom-observable role, not
    /// a molecular composition claim.
    SurfaceFloater {
        /// Recipe-basis amount at which the visible surface is treated as
        /// fully covered.
        saturation_amount: f64,
    },
    /// Empirical bridge from a surfactant-containing recipe to the familiar
    /// pepper-and-soap spreading demonstration. It describes only the
    /// reviewed dose response, not a universal surface-tension coefficient.
    SurfaceTensionReducer {
        /// Amount of unresolved functional blend, in the recipe basis, at
        /// which the bounded clearing effect reaches full strength.
        saturation_amount: f64,
        max_cleared_fraction: f64,
    },
    /// A recipe-level liquid mixture that remains separate from an aqueous
    /// phase and forms the upper layer. This is deliberately a visible,
    /// bounded material property: it does not invent one molecular species
    /// for a variable household mixture or claim a full LLE model.
    AqueousImmiscibleLiquid {
        /// Display colour for the unresolved bulk layer.
        srgb: [u8; 3],
        colour_word: String,
    },
    /// Bounded recipe-level surfactant behavior under mechanical stirring.
    /// The parameters describe a teaching observable, not a molecular CMC,
    /// droplet-size distribution, or universal detergent formulation.
    AqueousEmulsifier {
        saturation_amount: f64,
        max_dispersed_fraction: f64,
        half_life_seconds: f64,
    },
    /// A stable, opaque household colloid such as milk. The unresolved
    /// fraction stays conserved as a named material while this role exposes
    /// only the bounded visual consequence of its dispersed solids/fat.
    OpaqueLiquidColloid {
        srgb: [u8; 3],
        /// Unresolved material concentration (g/L) at full opacity.
        opacity_saturation_g_per_litre: f64,
    },
    /// Bounded acid-dose response for a named colloid that forms visible
    /// curds. This is a recipe observable, not a protein speciation model.
    AcidCurdlingColloid {
        acid_species: String,
        onset_moles_per_gram: f64,
        full_moles_per_gram: f64,
        max_curdled_fraction: f64,
        max_opacity_reduction: f64,
        curd_srgb: [u8; 3],
    },
    /// A dilute, resolved colourant that can remain as a visible surface
    /// drop on an opaque colloid until detergent spreads it or mechanical
    /// mixing releases it into the bulk optical model.
    SurfaceColourant {
        srgb: [u8; 3],
    },
    /// A named solid whose substance the registry does not resolve into any
    /// installed species, and which is therefore conserved as named matter
    /// rather than silently discarded or given a stand-in molecule.
    ///
    /// The role carries exactly what the bench can honestly say about such a
    /// material: that a visible piece of it is in the vessel, and what colour
    /// it is. It deliberately claims no reactivity in either direction —
    /// neither that the material is inert nor that it takes part in anything,
    /// and it adds no chemistry that could mask an operator's ordinary
    /// `NotYetModelled` answer.
    ConservedUnresolvedSolid {
        srgb: [u8; 3],
        colour_word: String,
    },
    /// A conserved unresolved baker's-yeast fraction with a bounded sucrose
    /// fermentation response. Parameters describe a classroom gas-evolution
    /// timescale, not strain growth or a universal product specification.
    FermentationCulture {
        reference_rate_per_second_per_gram: f64,
        optimum_temperature_k: f64,
        temperature_width_k: f64,
        requires_hydration: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterialGeometry {
    #[serde(default)]
    pub shape: Option<String>,
    #[serde(default)]
    pub surface_area_m2: Option<f64>,
    #[serde(default)]
    pub characteristic_length_m: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterialSubstitution {
    pub component_species_id: String,
    pub substitute_species_id: String,
    /// Amount of substitute in the recipe basis per unit of the component.
    pub ratio: f64,
    pub evidence: Evidence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterialConfidence {
    Measured,
    Curated,
    Estimated,
    Surrogate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MaterialExpansionPolicy {
    /// Every component interval must be a single exact value.
    Fixed,
    /// The caller supplies a sample seed. Equal recipe/version/seed triples
    /// always select the same point through every declared interval.
    Seeded { salt: String },
}

/// One deterministic expansion in the recipe's declared basis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterialExpansion {
    pub recipe_id: String,
    pub recipe_version: u32,
    pub basis: MaterialBasis,
    pub total_amount: f64,
    pub components: Vec<ExpandedMaterialComponent>,
    pub unresolved_amount: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpandedMaterialComponent {
    pub species_id: String,
    pub fraction: f64,
    pub amount: f64,
}

impl MaterialRecipe {
    /// Match a canonical key, display name, or localized alias. Matching is
    /// Unicode-lowercased and whitespace-normalized; language limits which
    /// alias list participates but canonical identity remains universal.
    pub fn matches(&self, query: &str, language: Option<&str>) -> bool {
        let query = normalize_material_name(query);
        if normalize_material_name(&self.canonical_key) == query
            || normalize_material_name(&self.name) == query
        {
            return true;
        }
        self.aliases
            .iter()
            .filter(|(tag, _)| language.is_none_or(|wanted| tag.eq_ignore_ascii_case(wanted)))
            .flat_map(|(_, aliases)| aliases)
            .any(|alias| normalize_material_name(alias) == query)
    }

    /// Expand a positive amount without discarding the unresolved balance.
    pub fn expand(&self, total_amount: f64, sample_seed: u64) -> Option<MaterialExpansion> {
        if !total_amount.is_finite() || total_amount <= 0.0 {
            return None;
        }
        let position = match &self.expansion_policy {
            MaterialExpansionPolicy::Fixed => 0.0,
            MaterialExpansionPolicy::Seeded { salt } => deterministic_unit_interval(&format!(
                "{}\0{}\0{}\0{sample_seed}",
                self.id, self.version, salt
            )),
        };
        let components = self
            .components
            .iter()
            .map(|component| {
                let fraction = component.fraction.lower
                    + (component.fraction.upper - component.fraction.lower) * position;
                ExpandedMaterialComponent {
                    species_id: component.species_id.clone(),
                    fraction,
                    amount: total_amount * fraction,
                }
            })
            .collect::<Vec<_>>();
        let resolved: f64 = components.iter().map(|component| component.fraction).sum();
        Some(MaterialExpansion {
            recipe_id: self.id.clone(),
            recipe_version: self.version,
            basis: self.basis,
            total_amount,
            components,
            unresolved_amount: total_amount * (1.0 - resolved).max(0.0),
        })
    }
}

impl RegistryDocument {
    pub fn material_recipe(&self, query: &str, language: Option<&str>) -> Option<&MaterialRecipe> {
        self.material_recipes
            .iter()
            .find(|recipe| recipe.matches(query, language))
    }
}

/// One spelling for every way a name can be written down.
///
/// KID-1: the `.lab` grammar splits on whitespace, so an alias that
/// contains a space — `household vinegar`, `whole milk`, `table sugar` —
/// could never be typed at all, and thirty of the fifty shipped recipes
/// had only that shape. Underscore, hyphen and space are the same
/// separator here, so the writable form of any alias reaches its recipe.
/// Checked to introduce no collision across the shipped recipes, their
/// aliases, or the species keys (`no_alias_collides_after_normalization`).
fn normalize_material_name(value: &str) -> String {
    value
        .split(|c: char| c.is_whitespace() || c == '_' || c == '-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn deterministic_unit_interval(value: &str) -> f64 {
    // FNV-1a is deliberately small and specified here; this is stable sample
    // selection, not security or statistical random-number generation.
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    (hash as f64) / (u64::MAX as f64)
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
    /// A molecular-scale physical length, such as the PC-SAFT segment
    /// diameter. This remains distinct from optical wavelength even when
    /// both source values happen to be expressed in nanometres.
    MolecularLength,
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
