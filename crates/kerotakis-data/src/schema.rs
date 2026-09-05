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
    /// A named homogeneous liquid formulation that remains fully unresolved
    /// and conserved. This role makes no claim about its reactivity or colour.
    ConservedUnresolvedLiquid,
    /// A conserved unresolved baker's-yeast fraction with a bounded sucrose
    /// fermentation response. Parameters describe a classroom gas-evolution
    /// timescale, not strain growth or a universal product specification.
    FermentationCulture {
        reference_rate_per_second_per_gram: f64,
        optimum_temperature_k: f64,
        temperature_width_k: f64,
        requires_hydration: bool,
        /// Which balanced aggregate reaction this culture runs. Defaulted
        /// to the alcoholic route so the yeast recipes that predate the
        /// other three read unchanged.
        #[serde(default)]
        metabolism: CultureMetabolism,
    },
    /// A named food that BRINGS its own enzyme rather than having one
    /// weighed into the beaker beside it.
    ///
    /// The enzyme activity model reads its catalyst out of the vessel's
    /// species inventory, and an enzyme is deliberately not a registry
    /// identity — it is a catalyst with an approximate dose mass and no
    /// molecular formula. A recipe component must be an identity, so
    /// fresh pineapple had no way to carry its bromelain and the classic
    /// pineapple-and-jelly demonstration could not run. This role is that
    /// bridge, and it carries the one thing a component could not: how
    /// much activity, in the activity model's own dose units.
    EnzymeSource {
        /// Catalyst key from the engine's enzyme catalogue, for example
        /// `bromelain`.
        enzyme: String,
        /// Grams of that catalyst which reproduce this material's activity,
        /// per gram of the material AS DISPENSED. It is an activity
        /// equivalent expressed in the bounded activity model's dose units
        /// and is NOT a mass of enzyme in the food: nothing here claims to
        /// know how much bromelain a pineapple contains.
        catalyst_equivalent_per_gram: f64,
        /// Above this temperature the carried enzyme is treated as
        /// irreversibly denatured. Letting the beaker cool does not bring
        /// it back, which is the difference between fresh and cooked.
        denatures_above_k: f64,
    },
    /// The bulk DC electrical resistivity of a named solid object.
    ///
    /// A pure solid's resistivity rides its species record, because it is
    /// a constant of the substance and a handbook tabulates it as one.
    /// A porcelain insulator has no such record and could not: its
    /// resistivity belongs to the fired object rather than to the silica
    /// the recipe resolves, it is set by the alkali content of the glassy
    /// phase between the crystals, and no component of this recipe
    /// carries it. That is exactly what a role is for.
    ///
    /// It carries a span as well as a value on purpose. An insulator's
    /// resistivity is not one number the way copper's is — it moves by
    /// orders of magnitude with composition, temperature and surface
    /// condition, and a doped semiconductor's is set by a dopant
    /// concentration the recipe does not pin down. The single value is
    /// what the meter reads for THIS reviewed object; the span is what
    /// the class covers, and a reading that quotes one without the other
    /// is a confidence the data does not have.
    /// A named object that is a galvanic cell in its own right, sealed,
    /// and which the bench deliberately does not open.
    ///
    /// Corrosion is a battery nobody wanted; a battery is a corrosion cell
    /// somebody built on purpose. The difference is that the cell's two
    /// electrodes and its electrolyte are chosen, packaged and sealed, so
    /// nothing crosses the case and the object's mass is the same
    /// discharged as it was new. That last fact is the one a balance can
    /// check, and it is why this role exists beside the reaction rather
    /// than instead of it.
    ///
    /// The reaction is CURATED PROSE. It is written down, not run: the
    /// products of an alkaline discharge have no species in this registry,
    /// no charge is tracked, and a reaction that moved matter would have
    /// to invent both. So the ledger is untouched and the sentence is the
    /// claim.
    SealedCell {
        /// Nominal open-circuit voltage of the couple, V.
        open_circuit_volts: f64,
        /// The balanced discharge reaction, as written.
        reaction: String,
        /// What moves inside while it discharges.
        why: String,
        /// What this row does not claim.
        boundary: String,
        /// The citation that travels with the voltage and the equation.
        source: String,
    },
    /// How a named polymer answers heat — the one property that separates
    /// the two families of plastic, and the reason they are two families.
    ///
    /// A thermoplastic is a tangle of separate chains held to one another
    /// by nothing stronger than the attraction between them. Heat it past
    /// the point where those give way and the chains slide: it softens, it
    /// can be moulded, and on cooling it sets again in the new shape. That
    /// is why it can be recycled by melting.
    ///
    /// A thermoset was cured, and curing built covalent bonds BETWEEN the
    /// chains. There are no separate chains left to slide — the object is
    /// one molecule — so there is no melting point to reach. Raise the
    /// temperature far enough and the bonds that hold it together break
    /// instead: it chars, and nothing brings it back. `softens_above_k`
    /// being `None` is that claim, stated as an absence rather than as a
    /// very large number.
    PolymerHeatResponse {
        /// Specific heat capacity, J/(g.K), so that heating an object made
        /// of this actually warms something. Without it a vessel holding a
        /// two-gram block reports itself empty to the heater.
        specific_heat_j_per_g_k: f64,
        /// Temperature, K, above which the chains slide and the object
        /// softens and can be reshaped. `None` for a cross-linked network,
        /// and the `None` is the claim.
        softens_above_k: Option<f64>,
        /// Temperature, K, above which the polymer decomposes. This one is
        /// not reversible in either family.
        chars_above_k: f64,
        /// What this row does not claim.
        boundary: String,
        /// The citation that travels with the two temperatures.
        source: String,
    },
    /// How a named material answers ultraviolet light: the transmitted
    /// fraction at the standard test film, in the two bands a sun-protection
    /// label is defined over. A labelled SPF is an erythemal dose ratio that
    /// the UV-B band dominates; a broad-spectrum claim adds a UV-A protection
    /// factor. Neither is a spectrum — this row states attenuation per band,
    /// and says why it is not absorption: mineral filters scatter as much as
    /// they absorb, so the Beer–Lambert path the question invites would be
    /// the wrong physics for them.
    UvAttenuation {
        /// Sun protection factor: the UV-B transmitted fraction is 1/SPF.
        spf: f64,
        /// UV-A protection factor: the UV-A transmitted fraction is 1/UVA-PF.
        uva_protection_factor: f64,
        /// The film the factors are defined at, mg/cm² (2.0 in the standard).
        film_mg_per_cm2: f64,
        /// How the light is stopped, in words.
        mechanism: String,
        /// What this row does not claim.
        boundary: String,
        /// The citation that travels with the factors.
        source: String,
    },
    BulkElectricalResistivity {
        /// Room-temperature bulk DC volume resistivity of this reviewed
        /// object, in ohm.m. Must lie inside the declared span.
        ohm_m: f64,
        /// Lower bound of the span this class of material covers, ohm.m.
        span_lower_ohm_m: f64,
        /// Upper bound of the same span, ohm.m.
        span_upper_ohm_m: f64,
        /// What this row does not claim: temperature dependence, surface
        /// leakage, anisotropy, dopant level.
        boundary: String,
        /// The citation that must travel with the reading. Unlike the
        /// other roles this one carries its own source, for the reason
        /// `corrosion::Barrier` does: the meter prints the number, so it
        /// has to be able to print the book behind it, and a runtime
        /// recipe cannot reach the registry's source records.
        source: String,
    },
}

/// The balanced aggregate reaction a declared culture runs.
///
/// Each is one reviewed equation on a disaccharide, or on ethanol, and
/// each conserves mass exactly. What a culture IS — its species, its
/// strain, what else it makes and whether the result is safe to eat —
/// is deliberately outside this enum.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CultureMetabolism {
    /// Baker's yeast: C12H22O11 + H2O -> 4 C2H5OH + 4 CO2.
    #[default]
    Alcoholic,
    /// Homofermentative lactic bacteria, the yoghurt route:
    /// C12H22O11 + H2O -> 4 C3H6O3. No gas, which is why a yoghurt pot
    /// does not rise and a sourdough does.
    Homolactic,
    /// Heterofermentative lactic bacteria beside wild yeast, the sourdough
    /// route: C12H22O11 + H2O -> 2 C3H6O3 + 2 C2H5OH + 2 CO2. Acid and
    /// gas from the same sugar.
    Heterolactic,
    /// Acetic acid bacteria, the vinegar route: C2H5OH + O2 -> CH3COOH +
    /// H2O. An oxidation, so it stops when the air does.
    Acetic,
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
