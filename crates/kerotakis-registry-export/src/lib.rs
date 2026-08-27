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
    MaterialPhysicalForm, MaterialRecipe, Method, ModelParameterRecord, ModelSubject,
    NumericRecord, OpticalRecord, Phase, PhaseProperty, PhaseThermodynamicRecord, RegistryDocument,
    SourceLane, SourceRecord, SpectralSample, Uncertainty, Unit,
};

const IMPORT_METHOD: &str = "verbatim export from kerotakis_core::species::REGISTRY";
const LEGACY_LICENCE: &str = "LicenseRef-Kerotakis-Legacy-Provenance-Review-Required";

/// Export every current declaration without changing or replacing the runtime
/// registry. All legacy sources remain build-oracle material pending explicit
/// licence and provenance review.
pub fn export_current_registry() -> Result<RegistryDocument, String> {
    let mut document = RegistryDocument::empty();
    for species in REGISTRY {
        export_species(&mut document, species)?;
    }
    export_material_recipes(&mut document);
    document.validate().map_err(|error| error.to_string())?;
    Ok(document)
}

fn export_material_recipes(document: &mut RegistryDocument) {
    const SOURCE: &str = "kerotakis/material-recipes-v1";
    document.sources.push(SourceRecord {
        id: SOURCE.to_string(),
        citation: "Kerotakis household-material assumptions v1: 3% hydrogen peroxide and 5% white vinegar are explicit unbranded teaching surrogates; ACS middle-school chemistry uses 3% peroxide for the yeast-catalysis activity".to_string(),
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
    document.material_recipes.extend([
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
            preparation: Some("5% w/w acetic-acid teaching surrogate".to_string()),
            lot_assumptions: vec![
                "flavour compounds and brand-specific residues are not represented".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
    ]);
}

fn export_species(document: &mut RegistryDocument, species: &SpeciesData) -> Result<(), String> {
    let source_id = format!("legacy/{}", species.key);
    document.sources.push(SourceRecord {
        id: source_id.clone(),
        citation: species.provenance.to_string(),
        licence: LEGACY_LICENCE.to_string(),
        lane: SourceLane::BuildOracle,
        origin: Some("crates/kerotakis-core/src/species.rs".to_string()),
        revision: None,
        retrieved: None,
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
