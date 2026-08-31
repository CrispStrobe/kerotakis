//! Build-only conversion of the handwritten seed registry into DATA-001's
//! reviewable source-record contract.
//!
//! This crate is intentionally not a dependency of any simulation or app
//! crate. Its output remains an oracle-side review artefact until each legacy
//! provenance statement has been cleared into the runtime source lane.

use std::collections::BTreeMap;

use kerotakis_core::{
    species::{Colour, Phase as LegacyPhase, SpeciesData, REGISTRY},
    spectrum::BAND_NM,
    stoich::parse_formula,
};
use kerotakis_data::{
    Applicability, CompositionRecord, Dimension, ElementAmount, Evidence, FractionRange,
    IdentityRecord, Interval, MaterialBasis, MaterialComponent, MaterialConfidence,
    MaterialExpansionPolicy, MaterialGeometry, MaterialPhysicalForm, MaterialRecipe, MaterialRole,
    Method, ModelParameterRecord, ModelSubject, NumericRecord, OpticalRecord, Phase, PhaseProperty,
    PhaseThermodynamicRecord, RegistryDocument, SourceLane, SourceRecord, SpectralSample,
    Uncertainty, Unit,
};

const IMPORT_METHOD: &str = "verbatim export from kerotakis_core::species::REGISTRY";
const LEGACY_LICENCE: &str = "LicenseRef-Kerotakis-Legacy-Provenance-Review-Required";
const ISOPROPANOL_SOURCE: &str = "us-federal/isopropanol-chris";
const ISOPROPANOL_CITATION: &str = "PubChem CID 3776 identity crosswalk plus U.S. Coast Guard CHRIS isopropanol liquid density (0.785 at 68 F) and liquid heat capacity (0.605 BTU/lb-F at 70 F); molar heat capacity converted to SI; retrieved 2026-08-27";
const SUCROSE_SOURCE: &str = "kerotakis/sucrose-teaching-properties-v1";
const SUCROSE_CITATION: &str = "PubChem CID 5988 identity crosswalk; Kerotakis room-temperature teaching approximations: crystal density 1.59 g/mL, conservative aqueous solubility 200 g/100 mL water, and 484 J/(mol.K) solid heat capacity estimated by Kopp's rule. These are explicit editorial parameters, not redistributed NIST SRD data; retrieved 2026-08-27";
const IRON_III_OXIDE_SOURCE: &str = "us-federal/nasa-cea-hematite";
const IRON_III_OXIDE_CITATION: &str = "PubChem CID 14833 identity crosswalk; NASA CEA thermo.inp Fe2O3(cr) record cites Pankratz (1983) for hematite thermochemistry. Room-temperature density 5.24 g/mL and reddish-brown appearance are explicit teaching properties; retrieved 2026-08-27";
const EPSOMITE_SOURCE: &str = "us-federal/usgs-epsomite";
const EPSOMITE_CITATION: &str = "PubChem CID 24843 identity crosswalk for magnesium sulfate heptahydrate; USGS PHREEQC wateq4f.dat Epsomite phase supplies MgSO4:7H2O dissolution stoichiometry. Density 1.68 g/mL and 360 J/(mol.K) heat capacity are explicit room-temperature teaching approximations; dissolution enthalpy remains unclaimed; retrieved 2026-08-27";
/// EXP-33. Transition temperatures get their own source record rather than
/// riding each species' molar-mass citation, because they are a different
/// claim with a different origin: the melting-point apparatus prints THIS
/// line, and a reader who wants to check the number must land on the book
/// the number came from.
const PHASE_TRANSITION_SOURCE: &str = "kerotakis/phase-transitions-v1";
const PHASE_TRANSITION_CITATION: &str = "Kerotakis curated phase-transition tranche v1: normal melting, boiling, sublimation, decomposition and hydrate-dehydration temperatures at 101.325 kPa. Each value is an individually entered editorial constant, taken from the standard published values for these substances and cross-checked against general reference tables. It is NOT a transcription from a positively identified copy of any single handbook edition, and no edition-level provenance is claimed: CRC Handbook of Chemistry and Physics, 97th ed. is the intended primary reference and every value here is flagged for reviewer confirmation against a positively identified copy of it before any stronger claim is made. Values are recorded only to the precision a school apparatus resolves; where a substance decomposes or sublimes rather than melting, no melting point is claimed at all; and where two general references disagreed the value was dropped rather than averaged or guessed. Compiled 2026-08-29";
const CHALCANTHITE_SOURCE: &str = "us-federal/usgs-chalcanthite";
const SILICA_SOURCE: &str = "us-federal/pubchem-silica";
const SILICA_CITATION: &str = "PubChem CID 24261 silica identity crosswalk. Quartz-like room-temperature teaching properties use molar mass 60.084 g/mol, density 2.65 g/mL and heat capacity 44.6 J/(mol.K); polymorph, grain coatings and natural-sand impurities remain separate material assumptions; retrieved 2026-08-27";

// Bootstrap a new data-driven species exactly once. After the generated source
// document is checked in, core's build script includes it in REGISTRY and the
// normal export loop reproduces the same record.
const ISOPROPANOL_SEED: SpeciesData = SpeciesData {
    key: "isopropanol",
    name: "isopropanol",
    formula: "C3H8O",
    inchikey: "KFZMGEQAYNKOFK-UHFFFAOYSA-N",
    molar_mass: 60.096,
    heat_capacity: 152.2,
    density: 0.785,
    standard_phase: LegacyPhase::Liquid,
    appearance: Some("colourless"),
    flame_colour: None,
    colour: None,
    spectrum: None,
    dissolution_enthalpy_kj: None,
    dissolves_without_speciation: false,
    aqueous_solubility_g_per_100_ml: None,
    forms_only_above_k: None,
    magnetic: false,
    transitions: None,
    provenance: ISOPROPANOL_CITATION,
};

const SUCROSE_SEED: SpeciesData = SpeciesData {
    key: "sucrose",
    name: "sucrose",
    formula: "C12H22O11",
    inchikey: "CZMRCDWAGMRECN-SFOFJGFUSA-N",
    molar_mass: 342.2965,
    heat_capacity: 484.0,
    density: 1.59,
    standard_phase: LegacyPhase::Solid,
    appearance: Some("white"),
    flame_colour: None,
    colour: None,
    spectrum: None,
    dissolution_enthalpy_kj: None,
    dissolves_without_speciation: true,
    aqueous_solubility_g_per_100_ml: Some(200.0),
    forms_only_above_k: None,
    magnetic: false,
    transitions: None,
    provenance: SUCROSE_CITATION,
};

const IRON_III_OXIDE_SEED: SpeciesData = SpeciesData {
    key: "Fe2O3",
    name: "iron(III) oxide (hematite)",
    formula: "Fe2O3",
    inchikey: "JEIPFZHSYJVQDO-UHFFFAOYSA-N",
    molar_mass: 159.687,
    heat_capacity: 103.9,
    density: 5.24,
    standard_phase: LegacyPhase::Solid,
    appearance: Some("reddish brown"),
    flame_colour: None,
    colour: Some(Colour {
        r: 145,
        g: 66,
        b: 54,
        strength: 0.0,
    }),
    spectrum: None,
    dissolution_enthalpy_kj: None,
    dissolves_without_speciation: false,
    aqueous_solubility_g_per_100_ml: None,
    forms_only_above_k: None,
    magnetic: false,
    transitions: None,
    provenance: IRON_III_OXIDE_CITATION,
};

const EPSOMITE_SEED: SpeciesData = SpeciesData {
    key: "epsomite",
    name: "magnesium sulfate heptahydrate (epsomite)",
    formula: "MgSO4·7H2O",
    inchikey: "WRUGWIBCXHJTDG-UHFFFAOYSA-L",
    molar_mass: 246.471,
    heat_capacity: 360.0,
    density: 1.68,
    standard_phase: LegacyPhase::Solid,
    appearance: Some("colourless to white crystals"),
    flame_colour: None,
    colour: None,
    spectrum: None,
    dissolution_enthalpy_kj: None,
    dissolves_without_speciation: false,
    aqueous_solubility_g_per_100_ml: None,
    forms_only_above_k: None,
    magnetic: false,
    transitions: None,
    provenance: EPSOMITE_CITATION,
};

const SILICA_SEED: SpeciesData = SpeciesData {
    key: "SiO2",
    name: "silicon dioxide (quartz)",
    formula: "SiO2",
    inchikey: "VYPSYNLAJGMNEJ-UHFFFAOYSA-N",
    molar_mass: 60.084,
    heat_capacity: 44.6,
    density: 2.65,
    standard_phase: LegacyPhase::Solid,
    appearance: Some("colourless to white grains"),
    flame_colour: None,
    colour: None,
    spectrum: None,
    dissolution_enthalpy_kj: None,
    dissolves_without_speciation: false,
    aqueous_solubility_g_per_100_ml: None,
    forms_only_above_k: None,
    magnetic: false,
    transitions: None,
    provenance: SILICA_CITATION,
};

/// Export every current declaration without changing or replacing the runtime
/// registry. All legacy sources remain build-oracle material pending explicit
/// licence and provenance review.
pub fn export_current_registry() -> Result<RegistryDocument, String> {
    let mut document = RegistryDocument::empty();
    for species in REGISTRY {
        export_species(&mut document, species)?;
    }
    if !REGISTRY.iter().any(|species| species.key == "isopropanol") {
        export_species(&mut document, &ISOPROPANOL_SEED)?;
    }
    if !REGISTRY.iter().any(|species| species.key == "sucrose") {
        export_species(&mut document, &SUCROSE_SEED)?;
    }
    if !REGISTRY.iter().any(|species| species.key == "Fe2O3") {
        export_species(&mut document, &IRON_III_OXIDE_SEED)?;
    }
    if !REGISTRY.iter().any(|species| species.key == "epsomite") {
        export_species(&mut document, &EPSOMITE_SEED)?;
    }
    if !REGISTRY.iter().any(|species| species.key == "SiO2") {
        export_species(&mut document, &SILICA_SEED)?;
    }
    // EXP-33: one source record for the whole transition tranche, pushed
    // once and only if something actually cites it — an orphan source is a
    // claim nobody made.
    if document
        .phase_thermodynamics
        .iter()
        .any(|record| record.quantity.source_id == PHASE_TRANSITION_SOURCE)
    {
        document.sources.push(SourceRecord {
            id: PHASE_TRANSITION_SOURCE.to_string(),
            citation: PHASE_TRANSITION_CITATION.to_string(),
            licence: "AGPL-3.0-or-later".to_string(),
            lane: SourceLane::Runtime,
            origin: Some("crates/kerotakis-registry-export/src/lib.rs".to_string()),
            revision: Some("v1".to_string()),
            retrieved: Some("2026-08-29".to_string()),
        });
    }
    export_material_recipes(&mut document);
    document.validate().map_err(|error| error.to_string())?;
    Ok(document)
}

fn export_material_recipes(document: &mut RegistryDocument) {
    const SOURCE: &str = "kerotakis/material-recipes-v1";
    document.sources.push(SourceRecord {
        id: SOURCE.to_string(),
        citation: "Kerotakis household-material assumptions v1: explicit unbranded teaching surrogates for common household substances; ACS middle-school chemistry uses 3% peroxide for yeast catalysis, documents detergent-lowered surface tension, teaches that vegetable oil is less dense than water and does not dissolve in it, demonstrates that detergent helps oil and water mix, and its Colors on the Move activity records detergent driving food colouring rapidly across whole milk; a Journal of Chemical Education baker's-yeast gasometer study measures CO2 evolution, induction, steady production and nutrient depletion, while FAO fermentation material gives the balanced hexose-to-ethanol-and-CO2 pathway; American Society of Baking compressed-yeast technical guidance reports 70% moisture and 30% solids; USDA ERS reports cow's milk as approximately 87% water with the balance milk fat and skim solids; ACS Making Glue and Mississippi State Extension describe vinegar separating milk casein into heavy white curds and liquid whey; USDA FoodData Central's white all-purpose wheat flour entry reports starch as the large majority of its carbohydrate, with protein, moisture, fibre, lipid and ash making up the rest, and its unsweetened apple-juice entry reports roughly 88% water with the sugars dominated by fructose and glucose rather than sucrose and the acidity carried mainly by malic acid; ordinary flat glass is a soda-lime composition of roughly three quarters silica with soda, lime, magnesia and alumina as network modifiers; solid paraffin candle wax and a sheet of office paper are dispensed against room-temperature bulk densities of 0.90 and 0.80 g/mL".to_string(),
        licence: "AGPL-3.0-or-later".to_string(),
        lane: SourceLane::Runtime,
        origin: Some("crates/kerotakis-registry-export/src/lib.rs".to_string()),
        revision: Some("2".to_string()),
        retrieved: Some("2026-08-29".to_string()),
    });
    let component = |species_id: &str, fraction: f64| MaterialComponent {
        species_id: species_id.to_string(),
        fraction: FractionRange {
            lower: fraction,
            upper: fraction,
        },
        evidence: Evidence {
            source_id: SOURCE.to_string(),
            method: Method::Curated("fixed teaching-surrogate composition".to_string()),
        },
    };
    let evidence = || Evidence {
        source_id: SOURCE.to_string(),
        method: Method::Editorial(
            "unbranded household teaching surrogate with an explicit concentration".to_string(),
        ),
    };
    let density = |value: f64| NumericRecord {
        value,
        unit: Unit {
            symbol: "g/mL".to_string(),
            dimension: Dimension::MassDensity,
        },
        conditions: Applicability::default(),
        uncertainty: Uncertainty::NotReported,
        source_id: SOURCE.to_string(),
        method: Method::Editorial("room-temperature teaching-surrogate density".to_string()),
    };
    let familiar_solid = |id: &str,
                          canonical_key: &str,
                          name: &str,
                          species: &str,
                          shape: &str,
                          de_aliases: &[&str],
                          en_aliases: &[&str],
                          assumptions: &[&str]| MaterialRecipe {
        id: id.to_string(),
        version: 1,
        canonical_key: canonical_key.to_string(),
        name: name.to_string(),
        aliases: BTreeMap::from([
            (
                "de".to_string(),
                de_aliases
                    .iter()
                    .map(|alias| (*alias).to_string())
                    .collect(),
            ),
            (
                "en".to_string(),
                en_aliases
                    .iter()
                    .map(|alias| (*alias).to_string())
                    .collect(),
            ),
        ]),
        basis: MaterialBasis::MassFraction,
        bulk_density: None,
        components: vec![component(species, 1.0)],
        unresolved_fraction: None,
        physical_form: MaterialPhysicalForm::CompositeObject {
            geometry: Some(MaterialGeometry {
                shape: Some(shape.to_string()),
                surface_area_m2: None,
                characteristic_length_m: None,
            }),
        },
        roles: Vec::new(),
        preparation: Some(format!(
            "idealised {name} represented by the installed {species} species"
        )),
        lot_assumptions: assumptions
            .iter()
            .map(|assumption| (*assumption).to_string())
            .collect(),
        substitutions: Vec::new(),
        confidence: MaterialConfidence::Curated,
        expansion_policy: MaterialExpansionPolicy::Fixed,
        evidence: evidence(),
    };
    let transparent_colour = |id: &str,
                              canonical_key: &str,
                              name: &str,
                              dye: &str,
                              concentration: f64,
                              medium: &str,
                              de_aliases: &[&str],
                              en_aliases: &[&str]| {
        MaterialRecipe {
        id: id.to_string(),
        version: 1,
        canonical_key: canonical_key.to_string(),
        name: name.to_string(),
        aliases: BTreeMap::from([
            (
                "de".to_string(),
                de_aliases.iter().map(|alias| (*alias).to_string()).collect(),
            ),
            (
                "en".to_string(),
                en_aliases.iter().map(|alias| (*alias).to_string()).collect(),
            ),
        ]),
        basis: MaterialBasis::MassFraction,
        bulk_density: Some(density(1.0)),
        components: vec![component(dye, concentration), component("water", 1.0 - concentration)],
        unresolved_fraction: None,
        physical_form: MaterialPhysicalForm::HomogeneousLiquid,
        roles: if canonical_key.starts_with("food_colour_") {
            vec![MaterialRole::SurfaceColourant {
                srgb: match canonical_key {
                    "food_colour_red" => [210, 35, 55],
                    "food_colour_yellow" => [245, 190, 25],
                    "food_colour_blue" => [35, 90, 210],
                    _ => [128, 128, 128],
                },
            }]
        } else {
            Vec::new()
        },
        preparation: Some(format!(
            "{}% w/w {dye} aqueous {medium}; unbranded transparent optical teaching surrogate",
            concentration * 100.0
        )),
        lot_assumptions: vec![
            "the named chromophore and concentration define this surrogate; it is not a claim about a retail product's ingredients".to_string(),
        ],
        substitutions: Vec::new(),
        confidence: MaterialConfidence::Surrogate,
        expansion_policy: MaterialExpansionPolicy::Fixed,
        evidence: evidence(),
    }
    };
    // These Gaussian bands are illustrative optical surrogates, not measured
    // spectra. Quantize them before export so their JSON representation remains
    // byte-stable after parse/serialize pack round trips on every platform.
    let stable_surrogate_coefficient = |value: f64| {
        if value == 0.0 {
            return 0.0;
        }
        format!("{value:.11e}")
            .parse::<f64>()
            .expect("formatted finite pigment coefficient must parse")
    };
    let pigment_bands = |peaks: &[(f64, f64, f64)]| {
        BAND_NM
            .iter()
            .map(|wavelength| {
                stable_surrogate_coefficient(
                    peaks
                        .iter()
                        .map(|(centre, strength, width)| {
                            let offset = (wavelength - centre) / width;
                            strength * (-0.5 * offset * offset).exp()
                        })
                        .sum(),
                )
            })
            .collect::<Vec<f64>>()
    };
    let acrylic_colour = |id: &str,
                          canonical_key: &str,
                          name: &str,
                          absorption: Vec<f64>,
                          scattering: f64,
                          de_aliases: &[&str],
                          en_aliases: &[&str]| {
        MaterialRecipe {
            id: id.to_string(),
            version: 1,
            canonical_key: canonical_key.to_string(),
            name: name.to_string(),
            aliases: BTreeMap::from([
                (
                    "de".to_string(),
                    de_aliases.iter().map(|alias| (*alias).to_string()).collect(),
                ),
                (
                    "en".to_string(),
                    en_aliases.iter().map(|alias| (*alias).to_string()).collect(),
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.1)),
            components: vec![component("water", 0.45)],
            unresolved_fraction: Some(FractionRange {
                lower: 0.55,
                upper: 0.55,
            }),
            physical_form: MaterialPhysicalForm::Suspension,
            roles: vec![MaterialRole::OpaquePigment {
                absorption,
                scattering: vec![scattering; BAND_NM.len()],
            }],
            preparation: Some(
                "waterborne opaque acrylic-paint optical surrogate; effective pigment and binder remain unresolved"
                    .to_string(),
            ),
            lot_assumptions: vec![
                "effective K/S coefficients describe this teaching color only; they do not match a brand, named artist pigment, gloss or drying film".to_string(),
                "the layer is treated as optically thick, so the substrate does not affect the computed swatch".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        }
    };
    document.material_recipes.extend([
        MaterialRecipe {
            id: "household/isopropanol-70-percent-vv".to_string(),
            version: 1,
            canonical_key: "isopropanol_70_percent".to_string(),
            name: "70% isopropanol".to_string(),
            aliases: BTreeMap::from([
                (
                    "en".to_string(),
                    vec![
                        "rubbing alcohol 70%".to_string(),
                        "isopropyl alcohol 70%".to_string(),
                        "rubbing_alcohol_70%".to_string(),
                    ],
                ),
                (
                    "de".to_string(),
                    vec![
                        "Isopropanol 70%".to_string(),
                        "Isopropylalkohol 70%".to_string(),
                        "Isopropanol_70%".to_string(),
                    ],
                ),
            ]),
            basis: MaterialBasis::VolumeFraction,
            bulk_density: None,
            components: vec![component("isopropanol", 0.70), component("water", 0.30)],
            unresolved_fraction: None,
            physical_form: MaterialPhysicalForm::HomogeneousLiquid,
            roles: Vec::new(),
            preparation: Some("70% v/v isopropanol in water at room temperature".to_string()),
            lot_assumptions: vec![
                "component volumes use the explicit runtime additive-volume teaching approximation; real mixing contracts slightly".to_string(),
                "denaturants, fragrance and brand-specific additives are not represented".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Curated,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/chlorine-bleach-5-percent".to_string(),
            version: 1,
            canonical_key: "chlorine_bleach_5_percent".to_string(),
            name: "5% chlorine bleach".to_string(),
            aliases: BTreeMap::from([
                (
                    "en".to_string(),
                    vec!["household bleach".to_string(), "bleach_5%".to_string()],
                ),
                (
                    "de".to_string(),
                    vec![
                        "Chlorreiniger 5%".to_string(),
                        "Chlorbleiche 5%".to_string(),
                        "Chlorreiniger_5%".to_string(),
                    ],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.08)),
            components: vec![component("NaOCl", 0.05), component("water", 0.95)],
            unresolved_fraction: None,
            physical_form: MaterialPhysicalForm::HomogeneousLiquid,
            roles: Vec::new(),
            preparation: Some("5% w/w sodium-hypochlorite teaching solution".to_string()),
            lot_assumptions: vec![
                "stabilisers, sodium chloride and brand-specific additives are not resolved in v1".to_string(),
                "never mix real bleach with acids or ammonia cleaners".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/ammonia-cleaner-5-percent".to_string(),
            version: 1,
            canonical_key: "ammonia_cleaner_5_percent".to_string(),
            name: "5% ammonia cleaner".to_string(),
            aliases: BTreeMap::from([
                (
                    "en".to_string(),
                    vec!["household ammonia".to_string(), "ammonia_cleaner_5%".to_string()],
                ),
                (
                    "de".to_string(),
                    vec![
                        "Ammoniakreiniger 5%".to_string(),
                        "Salmiakgeist 5%".to_string(),
                        "Ammoniakreiniger_5%".to_string(),
                    ],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(0.98)),
            components: vec![component("NH3", 0.05), component("water", 0.95)],
            unresolved_fraction: None,
            physical_form: MaterialPhysicalForm::HomogeneousLiquid,
            roles: Vec::new(),
            preparation: Some("5% w/w aqueous-ammonia teaching solution".to_string()),
            lot_assumptions: vec![
                "surfactants, fragrance and brand-specific additives are not resolved in v1".to_string(),
                "never mix real ammonia cleaner with bleach".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/sparkling-water-surrogate".to_string(),
            version: 1,
            canonical_key: "sparkling_water".to_string(),
            name: "sparkling water".to_string(),
            aliases: BTreeMap::from([
                (
                    "en".to_string(),
                    vec!["carbonated water".to_string(), "fizzy water".to_string()],
                ),
                (
                    "de".to_string(),
                    vec![
                        "Sprudel".to_string(),
                        "Mineralwasser mit Kohlensäure".to_string(),
                        "Sprudelwasser".to_string(),
                    ],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.0)),
            components: vec![component("CO2", 0.006), component("water", 0.994)],
            unresolved_fraction: None,
            physical_form: MaterialPhysicalForm::HomogeneousLiquid,
            roles: Vec::new(),
            preparation: Some("0.6% w/w CO2-in-water carbonation teaching surrogate".to_string()),
            lot_assumptions: vec![
                "dissolved minerals and brand-specific carbonation are not represented".to_string(),
                "dispensing into an open vessel lets the installed gas-liquid model decide how much CO2 escapes".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/cola-drink-surrogate".to_string(),
            version: 1,
            canonical_key: "cola_drink".to_string(),
            name: "cola drink surrogate".to_string(),
            aliases: BTreeMap::from([
                (
                    "en".to_string(),
                    vec!["fizzy cola".to_string()],
                ),
                (
                    "de".to_string(),
                    vec!["Cola".to_string(), "Colagetränk".to_string()],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.04)),
            components: vec![
                component("CO2", 0.006),
                component("H3PO4", 0.0005),
                component("water", 0.885),
            ],
            unresolved_fraction: Some(FractionRange {
                lower: 0.1085,
                upper: 0.1085,
            }),
            physical_form: MaterialPhysicalForm::HomogeneousLiquid,
            roles: Vec::new(),
            preparation: Some(
                "carbonated phosphoric-acid cola teaching surrogate; 10.85% unresolved solids and flavour blend"
                    .to_string(),
            ),
            lot_assumptions: vec![
                "sugar or sweetener identity, caramel colour, caffeine and flavour compounds remain unresolved".to_string(),
                "this is not a nutritional or brand-specific formulation".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/mineralised-tap-water-surrogate".to_string(),
            version: 1,
            canonical_key: "tap_water".to_string(),
            name: "mineralised tap-water surrogate".to_string(),
            aliases: BTreeMap::from([
                ("en".to_string(), vec!["tap water".to_string()]),
                ("de".to_string(), vec!["Leitungswasser".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(0.998)),
            components: vec![
                component("CaCl2", 0.0002),
                component("MgSO4", 0.0001),
                component("NaHCO3", 0.0004),
                component("water", 0.9993),
            ],
            unresolved_fraction: None,
            physical_form: MaterialPhysicalForm::HomogeneousLiquid,
            roles: Vec::new(),
            preparation: Some(
                "700 mg/kg mineralised hard-tap-water teaching surrogate".to_string(),
            ),
            lot_assumptions: vec![
                "real tap-water composition varies strongly by location, season and treatment".to_string(),
                "trace disinfectant, dissolved gases and organic matter are not represented".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/seawater-35-per-mille-surrogate".to_string(),
            version: 1,
            canonical_key: "seawater".to_string(),
            name: "3.5% seawater surrogate".to_string(),
            aliases: BTreeMap::from([
                (
                    "en".to_string(),
                    vec!["sea water".to_string(), "salt water".to_string()],
                ),
                (
                    "de".to_string(),
                    vec!["Meerwasser".to_string(), "Salzwasser".to_string()],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.025)),
            components: vec![
                component("CaCl2", 0.0012),
                component("KCl", 0.0007),
                component("MgSO4", 0.0017),
                component("NaCl", 0.027),
                component("water", 0.965),
            ],
            unresolved_fraction: Some(FractionRange {
                lower: 0.0044,
                upper: 0.0044,
            }),
            physical_form: MaterialPhysicalForm::HomogeneousLiquid,
            roles: Vec::new(),
            preparation: Some(
                "3.5% w/w simplified seawater teaching surrogate with major installed salts"
                    .to_string(),
            ),
            lot_assumptions: vec![
                "minor ions, alkalinity, dissolved gases and organic matter remain in the explicit unresolved fraction".to_string(),
                "this is not a recipe for biological or analytical artificial seawater".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/hydrogen-peroxide-3-percent".to_string(),
            version: 1,
            canonical_key: "hydrogen_peroxide_3_percent".to_string(),
            name: "3% hydrogen peroxide".to_string(),
            aliases: BTreeMap::from([
                (
                    "en".to_string(),
                    vec![
                        "household peroxide 3%".to_string(),
                        "peroxide_3%".to_string(),
                    ],
                ),
                (
                    "de".to_string(),
                    vec![
                        "Wasserstoffperoxid 3%".to_string(),
                        "Wasserstoffperoxid_3%".to_string(),
                    ],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.01)),
            components: vec![component("H2O2", 0.03), component("water", 0.97)],
            unresolved_fraction: None,
            physical_form: MaterialPhysicalForm::HomogeneousLiquid,
            roles: Vec::new(),
            preparation: Some("3% w/w aqueous solution at room temperature".to_string()),
            lot_assumptions: vec![
                "stabilisers and trace impurities are not resolved in v1".to_string()
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Curated,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/white-vinegar-5-percent".to_string(),
            version: 1,
            canonical_key: "white_vinegar_5_percent".to_string(),
            name: "5% white vinegar".to_string(),
            aliases: BTreeMap::from([
                ("en".to_string(), vec!["household vinegar".to_string()]),
                (
                    "de".to_string(),
                    vec![
                        "Essig".to_string(),
                        "Haushaltsessig 5%".to_string(),
                        "Haushaltsessig_5%".to_string(),
                    ],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.006)),
            components: vec![component("CH3COOH", 0.05), component("water", 0.95)],
            unresolved_fraction: None,
            physical_form: MaterialPhysicalForm::HomogeneousLiquid,
            roles: Vec::new(),
            preparation: Some("5% w/w acetic-acid teaching surrogate".to_string()),
            lot_assumptions: vec![
                "flavour compounds and brand-specific residues are not represented".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/dish-soap-surrogate".to_string(),
            version: 1,
            canonical_key: "dish_soap".to_string(),
            name: "dish soap".to_string(),
            aliases: BTreeMap::from([
                ("en".to_string(), vec!["washing-up_liquid".to_string()]),
                (
                    "de".to_string(),
                    vec!["Spülmittel".to_string()],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.03)),
            components: vec![component("water", 0.80)],
            unresolved_fraction: Some(FractionRange {
                lower: 0.20,
                upper: 0.20,
            }),
            physical_form: MaterialPhysicalForm::HomogeneousLiquid,
            roles: vec![
                MaterialRole::FoamStabilizer {
                    trapping_efficiency: 0.85,
                    gas_volume_fraction: 0.90,
                    half_life_seconds: 180.0,
                    saturation_amount: 0.4,
                },
                MaterialRole::SurfaceTensionReducer {
                    saturation_amount: 0.10,
                    max_cleared_fraction: 0.90,
                },
                MaterialRole::AqueousEmulsifier {
                    saturation_amount: 0.10,
                    max_dispersed_fraction: 0.92,
                    half_life_seconds: 300.0,
                },
            ],
            preparation: Some(
                "unbranded aqueous dish-soap teaching surrogate; surfactant blend unresolved"
                    .to_string(),
            ),
            lot_assumptions: vec![
                "brand-specific surfactants, salts, fragrance, dye and preservatives remain in the explicit unresolved fraction".to_string(),
                "emulsification parameters are a bounded stirred classroom observable, not a universal detergent specification".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/ground-black-pepper-surrogate".to_string(),
            version: 1,
            canonical_key: "ground_black_pepper".to_string(),
            name: "ground black pepper".to_string(),
            aliases: BTreeMap::from([
                (
                    "de".to_string(),
                    vec!["Pfeffer".to_string(), "schwarzer Pfeffer".to_string()],
                ),
                ("en".to_string(), vec!["ground pepper".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: None,
            components: Vec::new(),
            unresolved_fraction: Some(FractionRange {
                lower: 1.0,
                upper: 1.0,
            }),
            physical_form: MaterialPhysicalForm::Powder,
            roles: vec![MaterialRole::SurfaceFloater {
                saturation_amount: 0.08,
            }],
            preparation: Some(
                "dry ground black-pepper teaching surrogate for the quiet-water surface demonstration"
                    .to_string(),
            ),
            lot_assumptions: vec![
                "plant composition, grind distribution and brand-specific wetting remain unresolved; only the reviewed floating-grain observable is modeled".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/whole-milk-surrogate".to_string(),
            version: 1,
            canonical_key: "whole_milk".to_string(),
            name: "whole milk surrogate".to_string(),
            aliases: BTreeMap::from([
                (
                    "en".to_string(),
                    vec!["whole milk".to_string(), "cow's milk".to_string()],
                ),
                (
                    "de".to_string(),
                    vec!["Milch".to_string(), "Vollmilch".to_string()],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.03)),
            components: vec![component("water", 0.87)],
            unresolved_fraction: Some(FractionRange {
                lower: 0.13,
                upper: 0.13,
            }),
            physical_form: MaterialPhysicalForm::HomogeneousLiquid,
            roles: vec![
                MaterialRole::OpaqueLiquidColloid {
                    srgb: [248, 247, 240],
                    opacity_saturation_g_per_litre: 60.0,
                },
                MaterialRole::AcidCurdlingColloid {
                    acid_species: "CH3COOH".to_string(),
                    onset_moles_per_gram: 0.0001,
                    full_moles_per_gram: 0.0005,
                    max_curdled_fraction: 0.28,
                    max_opacity_reduction: 0.85,
                    curd_srgb: [250, 248, 230],
                },
            ],
            preparation: Some(
                "generic unflavoured whole-cow's-milk teaching surrogate".to_string(),
            ),
            lot_assumptions: vec![
                "USDA's approximately 87% water is resolved; milk fat, protein, lactose, minerals and natural variation remain together as conserved unresolved milk solids rather than fictional molecules".to_string(),
                "1.03 g/mL and 60 g/L full-opacity are explicit room-temperature visual geometry parameters, not product specifications".to_string(),
                "acid curdling is a bounded dose response calibrated to the familiar milk-and-vinegar classroom ratio; its 28% aggregate-solids ceiling separates estimated curd solids from wet-curd yield and its opacity response is independent".to_string(),
                "detergent-driven colour motion is a bounded surface-state response; spoilage and fermentation require separate state transitions".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/dry-yeast-catalase-surrogate".to_string(),
            version: 1,
            canonical_key: "dry_yeast".to_string(),
            name: "dry yeast".to_string(),
            aliases: BTreeMap::from([
                ("en".to_string(), vec!["baker's_yeast".to_string()]),
                (
                    "de".to_string(),
                    vec!["Hefe".to_string(), "Trockenhefe".to_string()],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: None,
            components: vec![component("catalase", 0.000_001)],
            unresolved_fraction: Some(FractionRange {
                lower: 0.999_999,
                upper: 0.999_999,
            }),
            physical_form: MaterialPhysicalForm::Granules,
            roles: vec![MaterialRole::FermentationCulture {
                reference_rate_per_second_per_gram: 0.0002,
                optimum_temperature_k: 308.15,
                temperature_width_k: 18.0,
                requires_hydration: true,
            }],
            preparation: Some(
                "dry baker's yeast represented as a catalase activity proxy; hydrate with warm water in the experiment"
                    .to_string(),
            ),
            lot_assumptions: vec![
                "enzyme activity varies strongly by brand and age; the bounded hydration ramp is a teaching surrogate, not a universal activity per gram".to_string(),
                "the sucrose-fermentation rate is an editorial classroom timescale; strain growth, oxygen switching, inhibition and secondary metabolites remain unresolved".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/fresh-compressed-yeast-surrogate".to_string(),
            version: 1,
            canonical_key: "fresh_yeast".to_string(),
            name: "fresh compressed yeast".to_string(),
            aliases: BTreeMap::from([
                (
                    "en".to_string(),
                    vec!["fresh yeast".to_string(), "compressed yeast".to_string()],
                ),
                (
                    "de".to_string(),
                    vec!["Frischhefe".to_string(), "Presshefe".to_string()],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: None,
            components: vec![component("water", 0.70), component("catalase", 0.000_000_3)],
            unresolved_fraction: Some(FractionRange {
                lower: 0.299_999_7,
                upper: 0.299_999_7,
            }),
            physical_form: MaterialPhysicalForm::Other {
                description: "moist compressed block".to_string(),
            },
            roles: vec![MaterialRole::FermentationCulture {
                reference_rate_per_second_per_gram: 0.0002,
                optimum_temperature_k: 308.15,
                temperature_width_k: 18.0,
                requires_hydration: false,
            }],
            preparation: Some(
                "fresh compressed baker's-yeast surrogate; already hydrated and scaled to 30% solids"
                    .to_string(),
            ),
            lot_assumptions: vec![
                "70% water and 30% solids follow compressed-yeast technical guidance".to_string(),
                "catalase activity is scaled from the dry-yeast teaching surrogate by dry solids; strain, age, storage and brand variation remain unresolved".to_string(),
                "the sucrose-fermentation rate is an editorial classroom timescale shared by equal dry solids; strain growth, oxygen switching, inhibition and secondary metabolites remain unresolved".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/granulated-table-sugar".to_string(),
            version: 1,
            canonical_key: "table_sugar".to_string(),
            name: "granulated table sugar".to_string(),
            aliases: BTreeMap::from([
                (
                    "en".to_string(),
                    vec!["table sugar".to_string(), "granulated sugar".to_string()],
                ),
                (
                    "de".to_string(),
                    vec!["Haushaltszucker".to_string(), "Kristallzucker".to_string()],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: None,
            components: vec![component("sucrose", 1.0)],
            unresolved_fraction: None,
            physical_form: MaterialPhysicalForm::Granules,
            roles: Vec::new(),
            preparation: Some("pure crystalline sucrose teaching surrogate".to_string()),
            lot_assumptions: vec![
                "moisture, invert sugar, anti-caking agents and source-crop residues are omitted"
                    .to_string(),
                "bare sugar/Zucker remains unclaimed because glucose, fructose and mixed sugars are distinct materials"
                    .to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Curated,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/vegetable-oil-surrogate".to_string(),
            version: 1,
            canonical_key: "vegetable_oil".to_string(),
            name: "vegetable oil".to_string(),
            aliases: BTreeMap::from([
                (
                    "en".to_string(),
                    vec!["cooking oil".to_string(), "plant oil".to_string()],
                ),
                (
                    "de".to_string(),
                    vec!["Pflanzenöl".to_string(), "Speiseöl".to_string()],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(0.92)),
            components: Vec::new(),
            unresolved_fraction: Some(FractionRange {
                lower: 1.0,
                upper: 1.0,
            }),
            physical_form: MaterialPhysicalForm::HomogeneousLiquid,
            roles: vec![MaterialRole::AqueousImmiscibleLiquid {
                srgb: [238, 218, 112],
                colour_word: "pale yellow".to_string(),
            }],
            preparation: Some(
                "generic room-temperature vegetable-oil teaching surrogate".to_string(),
            ),
            lot_assumptions: vec![
                "the oil remains a conserved unresolved triglyceride mixture; crop, refining, age, additives and brand are not guessed".to_string(),
                "0.92 g/mL is an explicit representative geometry parameter, not a product specification".to_string(),
                "the bounded role supports an upper oil layer on water; emulsions, oxidation, hydrolysis and combustion remain unmodelled".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/baking-soda".to_string(),
            version: 1,
            canonical_key: "baking_soda".to_string(),
            name: "baking soda".to_string(),
            aliases: BTreeMap::from([
                (
                    "en".to_string(),
                    vec!["bicarbonate of soda".to_string(), "sodium bicarbonate".to_string()],
                ),
                (
                    "de".to_string(),
                    vec!["Natron".to_string(), "Speisesoda".to_string(), "Backnatron".to_string()],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: None,
            components: vec![component("NaHCO3", 1.0)],
            unresolved_fraction: None,
            physical_form: MaterialPhysicalForm::Powder,
            roles: Vec::new(),
            preparation: Some("pure sodium-bicarbonate teaching surrogate".to_string()),
            lot_assumptions: vec![
                "anti-caking agents and moisture in retail products are omitted".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Curated,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/heat-activated-baking-powder-surrogate".to_string(),
            version: 1,
            canonical_key: "baking_powder".to_string(),
            name: "heat-activated baking powder surrogate".to_string(),
            aliases: BTreeMap::from([
                (
                    "en".to_string(),
                    vec!["baking powder".to_string()],
                ),
                (
                    "de".to_string(),
                    vec!["Backpulver".to_string()],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: None,
            components: vec![component("NaHCO3", 0.30), component("starch", 0.25)],
            unresolved_fraction: Some(FractionRange {
                lower: 0.45,
                upper: 0.45,
            }),
            physical_form: MaterialPhysicalForm::Powder,
            roles: Vec::new(),
            preparation: Some(
                "heat-activated baking-powder teaching surrogate with 30% sodium bicarbonate, 25% starch and 45% unresolved acid-salt blend"
                    .to_string(),
            ),
            lot_assumptions: vec![
                "heating uses the installed bicarbonate decomposition route and does not require the unresolved acid salts".to_string(),
                "wet or double-acting activation is not claimed until the acid salts and their dissolution kinetics are installed".to_string(),
                "this is not a formulation or substitution recommendation for food".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/washing-soda".to_string(),
            version: 1,
            canonical_key: "washing_soda".to_string(),
            name: "washing soda".to_string(),
            aliases: BTreeMap::from([
                ("en".to_string(), vec!["soda ash".to_string()]),
                (
                    "de".to_string(),
                    vec!["Waschsoda".to_string(), "Reine Soda".to_string()],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: None,
            components: vec![component("Na2CO3", 1.0)],
            unresolved_fraction: None,
            physical_form: MaterialPhysicalForm::Powder,
            roles: Vec::new(),
            preparation: Some("anhydrous sodium-carbonate teaching surrogate".to_string()),
            lot_assumptions: vec![
                "hydrated crystal soda and brand-specific additives are not represented".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Curated,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/cornstarch".to_string(),
            version: 1,
            canonical_key: "corn_starch".to_string(),
            name: "cornstarch".to_string(),
            aliases: BTreeMap::from([
                ("en".to_string(), vec!["corn starch".to_string()]),
                (
                    "de".to_string(),
                    vec!["Speisestärke".to_string(), "Maisstärke".to_string(), "Staerke".to_string()],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: None,
            components: vec![component("starch", 1.0)],
            unresolved_fraction: None,
            physical_form: MaterialPhysicalForm::Powder,
            roles: Vec::new(),
            preparation: Some("dry purified-starch teaching surrogate".to_string()),
            lot_assumptions: vec![
                "botanical source, moisture, lipids and protein traces are omitted".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "school/lugol-solution-1-percent".to_string(),
            version: 1,
            canonical_key: "lugol_solution_1_percent".to_string(),
            name: "1% Lugol iodine solution".to_string(),
            aliases: BTreeMap::from([
                (
                    "de".to_string(),
                    vec![
                        "Lugolsche Lösung 1%".to_string(),
                        "Lugol-Lösung 1%".to_string(),
                        "Lugol-Lösung_1%".to_string(),
                        "Iod-Kaliumiodid-Lösung 1%".to_string(),
                    ],
                ),
                (
                    "en".to_string(),
                    vec!["dilute Lugol solution".to_string(), "Lugol 1%".to_string()],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.02)),
            components: vec![
                component("I2", 0.01),
                component("KI", 0.02),
                component("water", 0.97),
            ],
            unresolved_fraction: None,
            physical_form: MaterialPhysicalForm::HomogeneousLiquid,
            roles: Vec::new(),
            preparation: Some(
                "dilute school-test surrogate: 1% w/w iodine and 2% w/w potassium iodide in water"
                    .to_string(),
            ),
            lot_assumptions: vec![
                "iodide-assisted iodine solubilisation is represented with retained KI and aqueous I2 bookkeeping; individual I3-/polyiodide equilibria are not resolved".to_string(),
                "the starch-complex optical response is calibrated to the broad literature band near 600-650 nm; starch source, amylose fraction, chain length and temperature shift its colour and intensity".to_string(),
                "this is a laboratory test reagent model, not a food or medical product".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/liquid-hand-soap-surrogate".to_string(),
            version: 1,
            canonical_key: "liquid_hand_soap".to_string(),
            name: "liquid hand soap".to_string(),
            aliases: BTreeMap::from([
                ("en".to_string(), vec!["hand wash".to_string()]),
                (
                    "de".to_string(),
                    vec![
                        "Flüssigseife".to_string(),
                        "Fluessigseife".to_string(),
                        "flüssige Seife".to_string(),
                        "Handseife".to_string(),
                    ],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.03)),
            components: vec![component("water", 0.75)],
            unresolved_fraction: Some(FractionRange {
                lower: 0.25,
                upper: 0.25,
            }),
            physical_form: MaterialPhysicalForm::HomogeneousLiquid,
            roles: vec![MaterialRole::FoamStabilizer {
                trapping_efficiency: 0.75,
                gas_volume_fraction: 0.88,
                half_life_seconds: 120.0,
                saturation_amount: 0.5,
            }],
            preparation: Some(
                "unbranded aqueous liquid-hand-soap teaching surrogate; surfactant blend unresolved"
                    .to_string(),
            ),
            lot_assumptions: vec![
                "bar soap is a different material; brand-specific surfactants, moisturisers, salts, fragrance, dye and preservatives remain unresolved".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        transparent_colour(
            "household/food-colour-red-betanin",
            "food_colour_red",
            "red food colouring",
            "betanin",
            0.001,
            "dropper solution",
            &["rote Lebensmittelfarbe", "Lebensmittelfarbe_rot"],
            &["red food color", "red_food_colouring", "red_food_color"],
        ),
        transparent_colour(
            "household/food-colour-yellow-curcumin",
            "food_colour_yellow",
            "yellow food colouring",
            "curcumin",
            0.001,
            "dropper solution",
            &["gelbe Lebensmittelfarbe", "Lebensmittelfarbe_gelb"],
            &["yellow food color", "yellow_food_colouring", "yellow_food_color"],
        ),
        transparent_colour(
            "household/food-colour-blue-indigo-carmine",
            "food_colour_blue",
            "blue food colouring",
            "indigo_carmine",
            0.001,
            "dropper solution",
            &["blaue Lebensmittelfarbe", "Lebensmittelfarbe_blau"],
            &["blue food color", "blue_food_colouring", "blue_food_color"],
        ),
        transparent_colour(
            "school/watercolor-red-betanin",
            "watercolour_red",
            "red watercolor",
            "betanin",
            0.0002,
            "wash",
            &["rote Wasserfarbe", "Wasserfarbe_rot"],
            &["red watercolour", "watercolor_red", "red_watercolor", "red_watercolour"],
        ),
        transparent_colour(
            "school/watercolor-yellow-curcumin",
            "watercolour_yellow",
            "yellow watercolor",
            "curcumin",
            0.0002,
            "wash",
            &["gelbe Wasserfarbe", "Wasserfarbe_gelb"],
            &["yellow watercolour", "watercolor_yellow", "yellow_watercolor", "yellow_watercolour"],
        ),
        transparent_colour(
            "school/watercolor-blue-indigo-carmine",
            "watercolour_blue",
            "blue watercolor",
            "indigo_carmine",
            0.0002,
            "wash",
            &["blaue Wasserfarbe", "Wasserfarbe_blau"],
            &["blue watercolour", "watercolor_blue", "blue_watercolor", "blue_watercolour"],
        ),
        acrylic_colour(
            "school/acrylic-red-surrogate",
            "acrylic_red",
            "red acrylic paint",
            pigment_bands(&[(470.0, 3.5, 65.0), (545.0, 2.0, 45.0)]),
            1.0,
            &["rote Acrylfarbe", "Acrylfarbe_rot"],
            &["red acrylic", "red_acrylic", "acrylic_paint_red"],
        ),
        acrylic_colour(
            "school/acrylic-yellow-surrogate",
            "acrylic_yellow",
            "yellow acrylic paint",
            pigment_bands(&[(440.0, 4.0, 45.0)]),
            1.0,
            &["gelbe Acrylfarbe", "Acrylfarbe_gelb"],
            &["yellow acrylic", "yellow_acrylic", "acrylic_paint_yellow"],
        ),
        acrylic_colour(
            "school/acrylic-blue-surrogate",
            "acrylic_blue",
            "blue acrylic paint",
            pigment_bands(&[(650.0, 4.0, 65.0)]),
            1.0,
            &["blaue Acrylfarbe", "Acrylfarbe_blau"],
            &["blue acrylic", "blue_acrylic", "acrylic_paint_blue"],
        ),
        acrylic_colour(
            "school/acrylic-white-surrogate",
            "acrylic_white",
            "white acrylic paint",
            vec![0.0; BAND_NM.len()],
            4.0,
            &["weiße Acrylfarbe", "weisse Acrylfarbe", "Acrylfarbe_weiss"],
            &["white acrylic", "white_acrylic", "acrylic_paint_white"],
        ),
        acrylic_colour(
            "school/acrylic-black-surrogate",
            "acrylic_black",
            "black acrylic paint",
            vec![8.0; BAND_NM.len()],
            0.2,
            &["schwarze Acrylfarbe", "Acrylfarbe_schwarz"],
            &["black acrylic", "black_acrylic", "acrylic_paint_black"],
        ),
        familiar_solid(
            "household/table-salt",
            "table_salt",
            "table salt",
            "NaCl",
            "granular crystals",
            &["Speisesalz", "Tafelsalz", "Kochsalz"],
            &["cooking salt"],
            &["moisture, iodine additives and anti-caking agents are omitted; the bare words salt and Salz remain unclaimed because they name a chemical class"],
        ),
        MaterialRecipe {
            id: "household/epsom-salt-heptahydrate".to_string(),
            version: 1,
            canonical_key: "epsom_salt".to_string(),
            name: "Epsom salt".to_string(),
            aliases: BTreeMap::from([
                (
                    "de".to_string(),
                    vec![
                        "Bittersalz".to_string(),
                        "Epsomsalz".to_string(),
                        "Magnesiumsulfat-Heptahydrat".to_string(),
                    ],
                ),
                (
                    "en".to_string(),
                    vec!["magnesium sulfate heptahydrate".to_string()],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: None,
            components: vec![component("epsomite", 1.0)],
            unresolved_fraction: None,
            physical_form: MaterialPhysicalForm::Granules,
            roles: Vec::new(),
            preparation: Some(
                "pure magnesium-sulfate-heptahydrate crystal teaching surrogate".to_string(),
            ),
            lot_assumptions: vec![
                "moisture, dehydration, grain size and retail additives are omitted; anhydrous magnesium sulfate is a distinct material".to_string(),
                "the seven waters remain bound in the dry crystal inventory until a phase model dissolves or transforms it".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Curated,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "school/iron-filings".to_string(),
            version: 1,
            canonical_key: "iron_filings".to_string(),
            name: "iron filings".to_string(),
            aliases: BTreeMap::from([
                (
                    "de".to_string(),
                    vec!["Eisenfeilspäne".to_string(), "Eisenspäne".to_string()],
                ),
                ("en".to_string(), vec!["iron powder".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: None,
            components: vec![component("Fe", 1.0)],
            unresolved_fraction: None,
            physical_form: MaterialPhysicalForm::Powder,
            roles: Vec::new(),
            preparation: Some("clean elemental-iron filings teaching surrogate".to_string()),
            lot_assumptions: vec![
                "surface oxide, cutting oil, particle-size distribution and alloying are omitted"
                    .to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Curated,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/quartz-sand-surrogate".to_string(),
            version: 1,
            canonical_key: "quartz_sand".to_string(),
            name: "quartz-rich sand surrogate".to_string(),
            aliases: BTreeMap::from([
                (
                    "de".to_string(),
                    vec!["Sand".to_string(), "Quarzsand".to_string(), "Spielsand".to_string()],
                ),
                (
                    "en".to_string(),
                    vec!["play sand".to_string(), "quartz sand".to_string()],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: None,
            components: vec![component("SiO2", 0.95)],
            unresolved_fraction: Some(FractionRange {
                lower: 0.05,
                upper: 0.05,
            }),
            physical_form: MaterialPhysicalForm::Granules,
            roles: Vec::new(),
            preparation: Some(
                "washed quartz-rich play-sand teaching surrogate with a conserved variable mineral fraction"
                    .to_string(),
            ),
            lot_assumptions: vec![
                "natural and retail sands vary widely; clay, feldspar, shell, iron minerals, organics, moisture and coatings remain in the explicit 5% unresolved fraction".to_string(),
                "grain-size distribution and colour are not yet resolved".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        familiar_solid(
            "school/calcium-carbonate-chalk-stick",
            "chalk_stick",
            "calcium-carbonate chalk stick",
            "CaCO3",
            "cylindrical stick",
            &["Kreidestück", "Tafelkreide", "Calciumcarbonat-Kreide"],
            &["chalk stick", "calcium carbonate chalk"],
            &["represented as pure calcium carbonate; gypsum chalk and product binders are different materials and are not implied"],
        ),
        familiar_solid(
            "school/magnesium-ribbon",
            "magnesium_ribbon",
            "magnesium ribbon",
            "Mg",
            "thin ribbon",
            &["Magnesiumband"],
            &["magnesium strip"],
            &["represented as clean elemental magnesium; surface oxide and reaction-rate effects wait for the surface-state model"],
        ),
        familiar_solid(
            "school/zinc-strip",
            "zinc_strip",
            "zinc strip",
            "Zn",
            "metal strip",
            &["Zinkstreifen", "Zinkblech"],
            &["zinc sheet"],
            &["represented as clean elemental zinc; galvanised coatings and alloying are not implied"],
        ),
        familiar_solid(
            "household/iron-nail-surrogate",
            "iron_nail",
            "iron nail surrogate",
            "Fe",
            "nail",
            &["Eisennagel"],
            &["iron nail"],
            &["represented as elemental iron; ordinary steel nails, coatings and corrosion products require alloy and surface-state recipes"],
        ),
        MaterialRecipe {
            id: "household/steel-wool-iron-surrogate".to_string(),
            version: 1,
            canonical_key: "steel_wool".to_string(),
            name: "steel wool surrogate".to_string(),
            aliases: BTreeMap::from([
                (
                    "de".to_string(),
                    vec!["Stahlwolle".to_string(), "Eisenwolle".to_string()],
                ),
                ("en".to_string(), vec!["steel wool".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: None,
            components: vec![component("Fe", 0.98)],
            unresolved_fraction: Some(FractionRange {
                lower: 0.02,
                upper: 0.02,
            }),
            physical_form: MaterialPhysicalForm::CompositeObject {
                geometry: Some(MaterialGeometry {
                    shape: Some("porous bundle of fine metal fibres".to_string()),
                    surface_area_m2: None,
                    characteristic_length_m: None,
                }),
            },
            roles: Vec::new(),
            preparation: Some(
                "fine low-carbon steel fibres represented by 98% resolved iron and 2% conserved unresolved alloy/coating fraction"
                    .to_string(),
            ),
            lot_assumptions: vec![
                "grade, carbon, alloying elements, oil, soap and coatings vary; the 98% iron fraction is a bounded teaching surrogate, not a product specification".to_string(),
                "fibre geometry explains why steel wool can ignite more readily than a nail, but the current ignition operator applies an explicit hot-zone threshold rather than claiming a measured area-dependent rate".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        familiar_solid(
            "school/copper-wire",
            "copper_wire",
            "copper wire",
            "Cu",
            "wire",
            &["Kupferdraht"],
            &["bare copper wire"],
            &["represented as bare elemental copper; insulation, lacquer and alloying are omitted"],
        ),
        familiar_solid(
            "household/aluminium-foil",
            "aluminium_foil",
            "aluminium foil",
            "Al",
            "thin foil",
            &["Alufolie", "Aluminiumfolie"],
            &["aluminum foil", "tin foil"],
            &["represented as elemental aluminium inventory; the native oxide layer and passivation kinetics are not yet modeled, so the engine must retain its explicit reaction boundary"],
        ),
        MaterialRecipe {
            id: "household/paraffin-candle-wax".to_string(),
            version: 1,
            canonical_key: "candle_wax".to_string(),
            name: "candle wax".to_string(),
            aliases: BTreeMap::from([
                (
                    "de".to_string(),
                    vec!["Kerzenwachs".to_string(), "Paraffinwachs".to_string()],
                ),
                ("en".to_string(), vec!["paraffin wax".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(0.90)),
            components: Vec::new(),
            unresolved_fraction: Some(FractionRange {
                lower: 1.0,
                upper: 1.0,
            }),
            physical_form: MaterialPhysicalForm::CompositeObject {
                geometry: Some(MaterialGeometry {
                    shape: Some("moulded block".to_string()),
                    surface_area_m2: None,
                    characteristic_length_m: None,
                }),
            },
            roles: vec![MaterialRole::ConservedUnresolvedSolid {
                srgb: [242, 240, 232],
                colour_word: "off-white".to_string(),
            }],
            preparation: Some(
                "solid paraffin candle wax, conserved whole because none of its long-chain alkanes is an installed species"
                    .to_string(),
            ),
            lot_assumptions: vec![
                "candle wax is a variable blend of long-chain alkanes, often with stearic acid, dyes and scent; no component of it is an installed species, so the whole mass stays conserved and unresolved rather than being given a stand-in molecule".to_string(),
                "melting is not claimed: the installed state model derives its transitions from water's enthalpies of fusion and vaporisation and covers no other substance, so heating named wax must reach the engine's ordinary model boundary instead of a curated melt".to_string(),
                "burning is not claimed either: a candle flame needs feed thermochemistry the engine has not installed, and a wick is an object the bench does not have".to_string(),
                "the bare words wax and Wachs remain unclaimed because beeswax, soy wax and paraffin wax are different materials, and bare paraffin remains unclaimed because it names a lamp fuel in British English".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/paper-sheet".to_string(),
            version: 1,
            canonical_key: "paper_sheet".to_string(),
            name: "paper".to_string(),
            aliases: BTreeMap::from([
                (
                    "de".to_string(),
                    vec!["Papier".to_string(), "Druckerpapier".to_string()],
                ),
                (
                    "en".to_string(),
                    vec!["sheet of paper".to_string(), "office paper".to_string()],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(0.80)),
            components: Vec::new(),
            unresolved_fraction: Some(FractionRange {
                lower: 1.0,
                upper: 1.0,
            }),
            physical_form: MaterialPhysicalForm::CompositeObject {
                geometry: Some(MaterialGeometry {
                    shape: Some("thin sheet".to_string()),
                    surface_area_m2: None,
                    characteristic_length_m: None,
                }),
            },
            roles: vec![MaterialRole::ConservedUnresolvedSolid {
                srgb: [246, 246, 242],
                colour_word: "white".to_string(),
            }],
            preparation: Some(
                "a sheet of ordinary white paper, conserved whole because cellulose is not an installed species"
                    .to_string(),
            ),
            lot_assumptions: vec![
                "cellulose is not in the runtime registry, so the whole sheet is conserved unresolved rather than resolved into a fibre that is not there; the identity is stated instead of guessed".to_string(),
                "mineral filler, sizing, coatings and optical brighteners vary by grade and stay inside the same unresolved mass; a carbonate-filled office paper and an unfilled newsprint are not distinguished".to_string(),
                "burning is not claimed: feed thermochemistry for cellulose is not installed, so ignition must reach the engine's model boundary".to_string(),
                "wetting, tearing and pulping are not claimed, and this material is not the filter paper the filtration apparatus uses".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/wheat-flour-surrogate".to_string(),
            version: 1,
            canonical_key: "wheat_flour".to_string(),
            name: "wheat flour".to_string(),
            aliases: BTreeMap::from([
                (
                    "de".to_string(),
                    vec![
                        "Mehl".to_string(),
                        "Weizenmehl".to_string(),
                        "Weißmehl".to_string(),
                    ],
                ),
                (
                    "en".to_string(),
                    vec![
                        "flour".to_string(),
                        "plain flour".to_string(),
                        "all-purpose flour".to_string(),
                    ],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(0.55)),
            components: vec![component("starch", 0.70)],
            unresolved_fraction: Some(FractionRange {
                lower: 0.30,
                upper: 0.30,
            }),
            physical_form: MaterialPhysicalForm::Powder,
            roles: Vec::new(),
            preparation: Some(
                "white wheat flour with a resolved 70% starch fraction and a conserved 30% protein, moisture, fibre, lipid and ash remainder"
                    .to_string(),
            ),
            lot_assumptions: vec![
                "the resolved fraction is starch, which is why a named iodine indicator finds it; the gluten proteins, sorbed moisture, fibre, lipids and ash stay in the conserved 30% remainder because none of them is an installed species".to_string(),
                "the flour's moisture is deliberately not resolved as liquid water: it is sorbed in the grain, and adding it to the vessel's free liquid would repeat the error the Epsom-salt recipe exists to avoid".to_string(),
                "mill type, ash grade, protein content, malted amylase and ascorbic-acid improvers vary widely; wholemeal, rye and spelt flours are different materials and are not implied".to_string(),
                "no gluten network, dough rheology, starch gelatinisation, enzymatic hydrolysis or baking behaviour is claimed".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/flour-and-water-dough".to_string(),
            version: 1,
            canonical_key: "flour_water_dough".to_string(),
            name: "flour-and-water dough".to_string(),
            aliases: BTreeMap::from([
                (
                    "de".to_string(),
                    vec!["Teig".to_string(), "Mehlteig".to_string()],
                ),
                (
                    "en".to_string(),
                    vec!["dough".to_string(), "simple dough".to_string()],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: None,
            components: vec![component("starch", 0.42)],
            unresolved_fraction: Some(FractionRange {
                lower: 0.58,
                upper: 0.58,
            }),
            physical_form: MaterialPhysicalForm::Other {
                description: "kneaded soft solid mass".to_string(),
            },
            roles: Vec::new(),
            preparation: Some(
                "a plain kneaded dough of three parts wheat flour to two parts water, resolving the flour's 42% starch and conserving the rest"
                    .to_string(),
            ),
            lot_assumptions: vec![
                "the dough's water is held in the flour matrix and is conserved inside the unresolved remainder rather than poured into the vessel's free liquid; the bench has no water-activity or matrix model, so it must not report a pool of water that a real dough would not release".to_string(),
                "no bulk density is claimed, so dough is dispensed by mass; a kneaded mass has no reviewed packing figure and guessing one would be a visible number without provenance".to_string(),
                "gluten development, kneading, viscoelasticity, proving, gelatinisation and baking are not claimed; adding yeast to this material does not make bread".to_string(),
                "hydration ratio, salt, flour grade and resting time vary; this is the plain flour-and-water teaching mixture only".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/apple-juice-surrogate".to_string(),
            version: 1,
            canonical_key: "apple_juice".to_string(),
            name: "apple juice surrogate".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Apfelsaft".to_string()]),
                ("en".to_string(), vec!["apple juice".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.045)),
            components: vec![component("sucrose", 0.02), component("water", 0.88)],
            unresolved_fraction: Some(FractionRange {
                lower: 0.10,
                upper: 0.10,
            }),
            physical_form: MaterialPhysicalForm::HomogeneousLiquid,
            roles: Vec::new(),
            preparation: Some(
                "clear apple-juice teaching surrogate: 88% water, the 2% that really is sucrose, and a conserved 10% remainder"
                    .to_string(),
            ),
            lot_assumptions: vec![
                "most of apple juice's sugar is fructose and glucose, and neither is an installed species; that roughly 8.5% stays in the conserved unresolved fraction instead of being relabelled sucrose, so the resolved sugar here is only the sucrose that is genuinely sucrose".to_string(),
                "acidity is not modelled: apple juice owes its tartness mainly to malic acid, which is not in the registry, and substituting an acid the engine does happen to have would compute a pH from the wrong molecule. The surrogate therefore makes no pH claim and behaves as a neutral sugar solution".to_string(),
                "pectin, minerals, vitamin C, colour and aroma compounds share the same unresolved remainder; cloudy, concentrate-reconstituted and fresh-pressed juices differ and are not distinguished".to_string(),
                "no juicing, browning, pasteurisation, fermentation or nutritional claim is made".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/soda-lime-glass".to_string(),
            version: 1,
            canonical_key: "glass".to_string(),
            name: "soda-lime glass".to_string(),
            aliases: BTreeMap::from([
                (
                    "de".to_string(),
                    vec![
                        "Glas".to_string(),
                        "Fensterglas".to_string(),
                        "Glasscherbe".to_string(),
                    ],
                ),
                ("en".to_string(), vec!["window glass".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(2.5)),
            components: vec![component("SiO2", 0.73)],
            unresolved_fraction: Some(FractionRange {
                lower: 0.27,
                upper: 0.27,
            }),
            physical_form: MaterialPhysicalForm::CompositeObject {
                geometry: Some(MaterialGeometry {
                    shape: Some("flat pane fragment".to_string()),
                    surface_area_m2: None,
                    characteristic_length_m: None,
                }),
            },
            roles: Vec::new(),
            preparation: Some(
                "an ordinary soda-lime glass object: 73% resolved silica with a conserved 27% network-modifier remainder"
                    .to_string(),
            ),
            lot_assumptions: vec![
                "the silica is reported as the installed SiO2 species so the object's silicon and oxygen are real inventory; in glass that silica is an amorphous network rather than the quartz grains of play sand, and no polymorph, devitrification or network claim follows from sharing one species key".to_string(),
                "the soda, lime, magnesia and alumina that make a melt workable are network modifiers, not free oxides in a beaker; resolving them as CaO or MgO would invent an alkaline dissolution that glass does not perform, so they stay in the conserved 27% remainder".to_string(),
                "glass is unreactive in the terms the aqueous engine models, which is not the same as inert: hydrofluoric-acid etching and hot-alkali attack are real chemistry the bench does not have and does not claim".to_string(),
                "softening, annealing, thermal shock, breaking and sharp edges are physics this recipe does not claim; borosilicate, lead crystal and fused quartz are different materials".to_string(),
                "this names the material. Laboratory glassware is created with the vessel commands, not dispensed from the shelf".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
    ]);
}

fn export_species(document: &mut RegistryDocument, species: &SpeciesData) -> Result<(), String> {
    let source_id = match species.key {
        "isopropanol" => ISOPROPANOL_SOURCE.to_string(),
        "sucrose" => SUCROSE_SOURCE.to_string(),
        "Fe2O3" => IRON_III_OXIDE_SOURCE.to_string(),
        "epsomite" => EPSOMITE_SOURCE.to_string(),
        "chalcanthite" => CHALCANTHITE_SOURCE.to_string(),
        "SiO2" => SILICA_SOURCE.to_string(),
        _ => format!("legacy/{}", species.key),
    };
    let curated_isopropanol = species.key == "isopropanol";
    let curated_sucrose = species.key == "sucrose";
    let curated_iron_iii_oxide = species.key == "Fe2O3";
    let curated_epsomite = species.key == "epsomite";
    let curated_chalcanthite = species.key == "chalcanthite";
    let curated_silica = species.key == "SiO2";
    let runtime_source = curated_isopropanol
        || curated_sucrose
        || curated_iron_iii_oxide
        || curated_epsomite
        || curated_chalcanthite
        || curated_silica;
    document.sources.push(SourceRecord {
        id: source_id.clone(),
        citation: species.provenance.to_string(),
        licence: match species.key {
            "isopropanol" => "LicenseRef-US-Public-Domain",
            "sucrose" => "AGPL-3.0-or-later",
            "Fe2O3" => "LicenseRef-US-Public-Domain",
            "epsomite" => "LicenseRef-US-Public-Domain",
            "chalcanthite" => "LicenseRef-US-Public-Domain",
            "SiO2" => "LicenseRef-US-Public-Domain",
            _ => LEGACY_LICENCE,
        }
        .to_string(),
        lane: if runtime_source {
            SourceLane::Runtime
        } else {
            SourceLane::BuildOracle
        },
        origin: Some(match species.key {
            "isopropanol" => "https://pubchem.ncbi.nlm.nih.gov/compound/3776".to_string(),
            "sucrose" => "crates/kerotakis-registry-export/src/lib.rs".to_string(),
            "Fe2O3" => "vendor/nasa-cea/thermo.inp".to_string(),
            "epsomite" => "vendor/iphreeqc/database/wateq4f.dat".to_string(),
            "chalcanthite" => "vendor/iphreeqc/database/wateq4f.dat".to_string(),
            "SiO2" => "https://pubchem.ncbi.nlm.nih.gov/compound/24261".to_string(),
            _ => "crates/kerotakis-core/src/species.rs".to_string(),
        }),
        revision: match species.key {
            "isopropanol" => Some("CID 3776".to_string()),
            "sucrose" => Some("v1".to_string()),
            "Fe2O3" => Some("NASA CEA Fe2O3(cr), PubChem CID 14833".to_string()),
            "epsomite" => Some("USGS WATEQ4F Epsomite, PubChem CID 24843".to_string()),
            "chalcanthite" => Some("USGS WATEQ4F Chalcanthite".to_string()),
            "SiO2" => Some("PubChem CID 24261".to_string()),
            _ => None,
        },
        retrieved: runtime_source.then(|| {
            if curated_chalcanthite {
                "2026-08-29".to_string()
            } else {
                "2026-08-27".to_string()
            }
        }),
    });

    let mut identifiers = BTreeMap::new();
    if !species.inchikey.is_empty() {
        identifiers.insert("inchikey".to_string(), species.inchikey.to_string());
    }
    document.identities.push(IdentityRecord {
        id: species.key.to_string(),
        canonical_key: if species.inchikey.is_empty() {
            format!("legacy:species/{}", species.key)
        } else {
            species.inchikey.to_string()
        },
        name: species.name.to_string(),
        identifiers,
        synonyms: Vec::new(),
        evidence: evidence(&source_id),
    });

    let formula = parse_formula(species.formula)
        .map_err(|error| format!("{} formula '{}': {error}", species.key, species.formula))?;
    document.compositions.push(CompositionRecord {
        id: format!("composition/{}", species.key),
        species_id: species.key.to_string(),
        formula: species.formula.to_string(),
        elements: formula
            .counts
            .into_iter()
            .map(|(element, count)| ElementAmount {
                element,
                count: exact_number(count, "1", Dimension::Dimensionless, &source_id),
            })
            .collect(),
        net_charge: exact_number(formula.charge, "1", Dimension::Dimensionless, &source_id),
        evidence: evidence(&source_id),
    });

    let phase = phase(species.standard_phase);
    for (suffix, property, value, symbol, dimension) in [
        (
            "molar-mass",
            PhaseProperty::MolarMass,
            species.molar_mass,
            "g/mol",
            Dimension::MolarMass,
        ),
        (
            "heat-capacity",
            PhaseProperty::MolarHeatCapacity,
            species.heat_capacity,
            "J/(mol.K)",
            Dimension::MolarHeatCapacity,
        ),
        (
            "density",
            PhaseProperty::MassDensity,
            species.density,
            "g/mL",
            Dimension::MassDensity,
        ),
    ] {
        document
            .phase_thermodynamics
            .push(PhaseThermodynamicRecord {
                id: format!("{suffix}/{}", species.key),
                species_id: species.key.to_string(),
                phase,
                property,
                quantity: imported_number(value, symbol, dimension, phase, &source_id),
            });
    }
    // EXP-33: melting/boiling ride the typed properties; sublimation,
    // decomposition and dehydration ride `Other`, because the schema has no
    // variant for them and inventing one would claim a dimension check the
    // schema cannot make. Every record carries the same transition source.
    if let Some(t) = species.transitions {
        for (suffix, property, value) in [
            (
                "melting-point",
                PhaseProperty::MeltingTemperature,
                t.melting_k,
            ),
            (
                "boiling-point",
                PhaseProperty::BoilingTemperature,
                t.boiling_k,
            ),
            (
                "sublimation-point",
                PhaseProperty::Other("sublimation_temperature".to_string()),
                t.sublimation_k,
            ),
            (
                "decomposition-point",
                PhaseProperty::Other("decomposition_temperature".to_string()),
                t.decomposition_k,
            ),
            (
                "dehydration-point",
                PhaseProperty::Other("dehydration_temperature".to_string()),
                t.dehydration_k,
            ),
        ] {
            let Some(value) = value else { continue };
            document
                .phase_thermodynamics
                .push(PhaseThermodynamicRecord {
                    id: format!("{suffix}/{}", species.key),
                    species_id: species.key.to_string(),
                    phase,
                    property,
                    quantity: NumericRecord {
                        value,
                        unit: Unit {
                            symbol: "K".to_string(),
                            dimension: Dimension::Temperature,
                        },
                        conditions: Applicability {
                            phase: Some(phase),
                            pressure: Some(Interval {
                                lower: 101_325.0,
                                upper: 101_325.0,
                                unit: Unit {
                                    symbol: "Pa".to_string(),
                                    dimension: Dimension::Pressure,
                                },
                            }),
                            notes: t.boundary.map(str::to_string),
                            ..Applicability::default()
                        },
                        uncertainty: Uncertainty::NotReported,
                        source_id: PHASE_TRANSITION_SOURCE.to_string(),
                        method: Method::Curated(
                            "EXP-33 curated transition tranche; value corroborated against \
                             the handbook literature named in the source citation"
                                .to_string(),
                        ),
                    },
                });
        }
    }
    if let Some(value) = species.dissolution_enthalpy_kj {
        document
            .phase_thermodynamics
            .push(PhaseThermodynamicRecord {
                id: format!("dissolution-enthalpy/{}", species.key),
                species_id: species.key.to_string(),
                phase,
                property: PhaseProperty::EnthalpyOfDissolution,
                quantity: imported_number(
                    value,
                    "kJ/mol",
                    Dimension::MolarEnergy,
                    phase,
                    &source_id,
                ),
            });
    }

    export_optical(document, species, &source_id, phase);
    export_model_parameters(document, species, &source_id);
    Ok(())
}

fn export_optical(
    document: &mut RegistryDocument,
    species: &SpeciesData,
    source_id: &str,
    phase: Phase,
) {
    if species.appearance.is_none()
        && species.flame_colour.is_none()
        && species.colour.is_none()
        && species.spectrum.is_none()
    {
        return;
    }
    let spectrum = species
        .spectrum
        .copied()
        .into_iter()
        .flat_map(|values| BAND_NM.into_iter().zip(values))
        .map(|(wavelength, molar_absorptivity)| SpectralSample {
            wavelength: imported_number(wavelength, "nm", Dimension::Wavelength, phase, source_id),
            molar_absorptivity: imported_number(
                molar_absorptivity,
                "L/(mol.cm)",
                Dimension::MolarAbsorptivity,
                phase,
                source_id,
            ),
        })
        .collect();
    document.optical.push(OpticalRecord {
        id: format!("optical/{}", species.key),
        species_id: species.key.to_string(),
        phase,
        appearance: species.appearance.map(str::to_string),
        flame_colour: species.flame_colour.map(str::to_string),
        reflective_srgb: species.colour.map(rgb_hex),
        spectrum,
        evidence: evidence(source_id),
    });
}

fn export_model_parameters(
    document: &mut RegistryDocument,
    species: &SpeciesData,
    source_id: &str,
) {
    document.model_parameters.push(ModelParameterRecord {
        id: format!("dissolves-without-speciation/{}", species.key),
        subject: ModelSubject::Species(species.key.to_string()),
        model: "legacy-runtime".to_string(),
        parameter: "dissolves-without-speciation".to_string(),
        quantity: exact_number(
            if species.dissolves_without_speciation {
                1.0
            } else {
                0.0
            },
            "1",
            Dimension::Dimensionless,
            source_id,
        ),
    });
    if let Some(solubility) = species.aqueous_solubility_g_per_100_ml {
        document.model_parameters.push(ModelParameterRecord {
            id: format!("aqueous-solubility/{}", species.key),
            subject: ModelSubject::Species(species.key.to_string()),
            model: "bounded-room-temperature-dissolution".to_string(),
            parameter: "aqueous-solubility-g-per-100-ml".to_string(),
            quantity: imported_number(
                solubility,
                "g/100mL",
                Dimension::MassConcentration,
                phase(species.standard_phase),
                source_id,
            ),
        });
    }
    if let Some(colour) = species.colour {
        document.model_parameters.push(ModelParameterRecord {
            id: format!("legacy-tint-strength/{}", species.key),
            subject: ModelSubject::Species(species.key.to_string()),
            model: "legacy-tint".to_string(),
            parameter: "strength".to_string(),
            quantity: imported_number(
                colour.strength,
                "L/(mol.cm)",
                Dimension::MolarAbsorptivity,
                phase(species.standard_phase),
                source_id,
            ),
        });
    }
    if let Some(temperature) = species.forms_only_above_k {
        document.model_parameters.push(ModelParameterRecord {
            id: format!("forms-only-above/{}", species.key),
            subject: ModelSubject::Species(species.key.to_string()),
            model: "legacy-metastability".to_string(),
            parameter: "forms-only-above".to_string(),
            quantity: imported_number(
                temperature,
                "K",
                Dimension::Temperature,
                phase(species.standard_phase),
                source_id,
            ),
        });
    }
    if species.magnetic {
        document.model_parameters.push(ModelParameterRecord {
            id: format!("magnetic/{}", species.key),
            subject: ModelSubject::Species(species.key.to_string()),
            model: "legacy-runtime".to_string(),
            parameter: "magnetic".to_string(),
            quantity: exact_number(1.0, "1", Dimension::Dimensionless, source_id),
        });
    }
}

fn imported_number(
    value: f64,
    symbol: &str,
    dimension: Dimension,
    phase: Phase,
    source_id: &str,
) -> NumericRecord {
    NumericRecord {
        value,
        unit: Unit {
            symbol: symbol.to_string(),
            dimension,
        },
        conditions: Applicability {
            phase: Some(phase),
            ..Applicability::default()
        },
        uncertainty: Uncertainty::NotReported,
        source_id: source_id.to_string(),
        method: Method::Imported(IMPORT_METHOD.to_string()),
    }
}

fn exact_number(value: f64, symbol: &str, dimension: Dimension, source_id: &str) -> NumericRecord {
    NumericRecord {
        value,
        unit: Unit {
            symbol: symbol.to_string(),
            dimension,
        },
        conditions: Applicability::default(),
        uncertainty: Uncertainty::Exact,
        source_id: source_id.to_string(),
        method: Method::Derived("parsed from the verbatim legacy declaration".to_string()),
    }
}

fn evidence(source_id: &str) -> Evidence {
    Evidence {
        source_id: source_id.to_string(),
        method: Method::Imported(IMPORT_METHOD.to_string()),
    }
}

fn phase(value: LegacyPhase) -> Phase {
    match value {
        LegacyPhase::Solid => Phase::Solid,
        LegacyPhase::Liquid => Phase::Liquid,
        LegacyPhase::Aqueous => Phase::Aqueous,
        LegacyPhase::Gas => Phase::Gas,
    }
}

fn rgb_hex(colour: Colour) -> String {
    format!("#{:02X}{:02X}{:02X}", colour.r, colour.g, colour.b)
}
