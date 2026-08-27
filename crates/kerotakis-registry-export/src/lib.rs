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
    IdentityRecord, MaterialBasis, MaterialComponent, MaterialConfidence, MaterialExpansionPolicy,
    MaterialPhysicalForm, MaterialRecipe, MaterialRole, Method, ModelParameterRecord, ModelSubject,
    NumericRecord, OpticalRecord, Phase, PhaseProperty, PhaseThermodynamicRecord, RegistryDocument,
    SourceLane, SourceRecord, SpectralSample, Uncertainty, Unit,
};

const IMPORT_METHOD: &str = "verbatim export from kerotakis_core::species::REGISTRY";
const LEGACY_LICENCE: &str = "LicenseRef-Kerotakis-Legacy-Provenance-Review-Required";
const ISOPROPANOL_SOURCE: &str = "us-federal/isopropanol-chris";
const ISOPROPANOL_CITATION: &str = "PubChem CID 3776 identity crosswalk plus U.S. Coast Guard CHRIS isopropanol liquid density (0.785 at 68 F) and liquid heat capacity (0.605 BTU/lb-F at 70 F); molar heat capacity converted to SI; retrieved 2026-08-27";

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
    forms_only_above_k: None,
    magnetic: false,
    provenance: ISOPROPANOL_CITATION,
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
    export_material_recipes(&mut document);
    document.validate().map_err(|error| error.to_string())?;
    Ok(document)
}

fn export_material_recipes(document: &mut RegistryDocument) {
    const SOURCE: &str = "kerotakis/material-recipes-v1";
    document.sources.push(SourceRecord {
        id: SOURCE.to_string(),
        citation: "Kerotakis household-material assumptions v1: explicit unbranded teaching surrogates for common household substances; ACS middle-school chemistry uses 3% peroxide for the yeast-catalysis activity".to_string(),
        licence: "AGPL-3.0-or-later".to_string(),
        lane: SourceLane::Runtime,
        origin: Some("crates/kerotakis-registry-export/src/lib.rs".to_string()),
        revision: Some("1".to_string()),
        retrieved: Some("2026-08-27".to_string()),
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
        roles: Vec::new(),
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
            roles: vec![MaterialRole::FoamStabilizer {
                trapping_efficiency: 0.85,
                gas_volume_fraction: 0.90,
                half_life_seconds: 180.0,
                saturation_amount: 0.4,
            }],
            preparation: Some(
                "unbranded aqueous dish-soap teaching surrogate; surfactant blend unresolved"
                    .to_string(),
            ),
            lot_assumptions: vec![
                "brand-specific surfactants, salts, fragrance, dye and preservatives remain in the explicit unresolved fraction".to_string(),
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
            roles: Vec::new(),
            preparation: Some(
                "dry baker's yeast represented as a catalase activity proxy; hydrate with warm water in the experiment"
                    .to_string(),
            ),
            lot_assumptions: vec![
                "enzyme activity varies strongly by brand and age; the bounded hydration ramp is a teaching surrogate, not a universal activity per gram".to_string(),
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
    ]);
}

fn export_species(document: &mut RegistryDocument, species: &SpeciesData) -> Result<(), String> {
    let source_id = if species.key == "isopropanol" {
        ISOPROPANOL_SOURCE.to_string()
    } else {
        format!("legacy/{}", species.key)
    };
    let curated_isopropanol = species.key == "isopropanol";
    document.sources.push(SourceRecord {
        id: source_id.clone(),
        citation: species.provenance.to_string(),
        licence: if curated_isopropanol {
            "LicenseRef-US-Public-Domain"
        } else {
            LEGACY_LICENCE
        }
        .to_string(),
        lane: if curated_isopropanol {
            SourceLane::Runtime
        } else {
            SourceLane::BuildOracle
        },
        origin: Some(
            if curated_isopropanol {
                "https://pubchem.ncbi.nlm.nih.gov/compound/3776"
            } else {
                "crates/kerotakis-core/src/species.rs"
            }
            .to_string(),
        ),
        revision: curated_isopropanol.then(|| "CID 3776".to_string()),
        retrieved: curated_isopropanol.then(|| "2026-08-27".to_string()),
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
