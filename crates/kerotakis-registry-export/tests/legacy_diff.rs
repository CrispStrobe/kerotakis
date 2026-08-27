use std::collections::BTreeMap;

use kerotakis_core::{
    species::{Colour, Phase as LegacyPhase, SpeciesData, REGISTRY},
    spectrum::BAND_NM,
    stoich::parse_formula,
};
use kerotakis_data::{
    Dimension, Method, NumericRecord, Phase, PhaseProperty, RegistryDocument, SourceLane,
    Uncertainty,
};
use kerotakis_registry_export::export_current_registry;

const IMPORT_METHOD: &str = "verbatim export from kerotakis_core::species::REGISTRY";

#[test]
fn checked_in_source_document_is_current_byte_for_byte() {
    let mut generated =
        serde_json::to_string_pretty(&export_current_registry().expect("export current registry"))
            .expect("serialize current registry");
    generated.push('\n');
    let checked_in = include_str!("../../../data/registry/registry-source-v1.json");
    if generated != checked_in {
        let mismatch = generated
            .lines()
            .zip(checked_in.lines())
            .position(|(current, committed)| current != committed)
            .map_or_else(
                || generated.lines().count().min(checked_in.lines().count()) + 1,
                |index| index + 1,
            );
        panic!(
            "checked-in source registry differs at line {mismatch}; regenerate with \
             `cargo run -p kerotakis-registry-export -- \
             data/registry/registry-source-v1.json`"
        );
    }
}

#[test]
fn every_legacy_field_is_present_and_unchanged() {
    let document = export_current_registry().expect("export current registry");
    document.validate().expect("export validates");

    assert_eq!(
        document
            .sources
            .iter()
            .filter(|source| source.id.starts_with("legacy/"))
            .count(),
        REGISTRY
            .iter()
            .filter(|species| {
                !matches!(
                    species.key,
                    "isopropanol" | "sucrose" | "Fe2O3" | "epsomite" | "SiO2"
                )
            })
            .count()
    );
    assert_eq!(document.material_recipes.len(), 44);
    let lugol = document
        .material_recipe("Lugol-Lösung_1%", Some("de"))
        .expect("localized dilute Lugol recipe");
    assert_eq!(lugol.canonical_key, "lugol_solution_1_percent");
    assert_eq!(lugol.components.len(), 3);
    assert!(lugol.unresolved_fraction.is_none());
    assert_eq!(document.identities.len(), REGISTRY.len());
    assert_eq!(document.compositions.len(), REGISTRY.len());
    assert_eq!(
        document.phase_thermodynamics.len(),
        REGISTRY.len() * 3
            + REGISTRY
                .iter()
                .filter(|species| species.dissolution_enthalpy_kj.is_some())
                .count()
    );
    assert_eq!(
        document.optical.len(),
        REGISTRY
            .iter()
            .filter(|species| has_optical(species))
            .count()
    );
    assert_eq!(
        document.model_parameters.len(),
        REGISTRY.len()
            + REGISTRY
                .iter()
                .filter(|species| species.colour.is_some())
                .count()
            + REGISTRY
                .iter()
                .filter(|species| species.forms_only_above_k.is_some())
                .count()
            + REGISTRY
                .iter()
                .filter(|species| species.aqueous_solubility_g_per_100_ml.is_some())
                .count()
    );
    assert!(document.transport.is_empty());
    assert!(document.safety.is_empty());
    assert!(document.microstates.is_empty());

    for species in REGISTRY {
        compare_species(&document, species);
    }
}

fn compare_species(document: &RegistryDocument, species: &SpeciesData) {
    let source_id = match species.key {
        "isopropanol" => "us-federal/isopropanol-chris".to_string(),
        "sucrose" => "kerotakis/sucrose-teaching-properties-v1".to_string(),
        "Fe2O3" => "us-federal/nasa-cea-hematite".to_string(),
        "epsomite" => "us-federal/usgs-epsomite".to_string(),
        "SiO2" => "us-federal/pubchem-silica".to_string(),
        _ => format!("legacy/{}", species.key),
    };
    let source = document
        .sources
        .iter()
        .find(|record| record.id == source_id)
        .unwrap_or_else(|| panic!("missing source for {}", species.key));
    assert_eq!(
        source.citation, species.provenance,
        "{} provenance",
        species.key
    );
    if species.key == "isopropanol" {
        assert_eq!(source.lane, SourceLane::Runtime);
        assert_eq!(source.licence, "LicenseRef-US-Public-Domain");
        assert_eq!(
            source.origin.as_deref(),
            Some("https://pubchem.ncbi.nlm.nih.gov/compound/3776")
        );
    } else if species.key == "sucrose" {
        assert_eq!(source.lane, SourceLane::Runtime);
        assert_eq!(source.licence, "AGPL-3.0-or-later");
        assert_eq!(
            source.origin.as_deref(),
            Some("crates/kerotakis-registry-export/src/lib.rs")
        );
    } else if species.key == "Fe2O3" {
        assert_eq!(source.lane, SourceLane::Runtime);
        assert_eq!(source.licence, "LicenseRef-US-Public-Domain");
        assert_eq!(source.origin.as_deref(), Some("vendor/nasa-cea/thermo.inp"));
    } else if species.key == "epsomite" {
        assert_eq!(source.lane, SourceLane::Runtime);
        assert_eq!(source.licence, "LicenseRef-US-Public-Domain");
        assert_eq!(
            source.origin.as_deref(),
            Some("vendor/iphreeqc/database/wateq4f.dat")
        );
    } else if species.key == "SiO2" {
        assert_eq!(source.lane, SourceLane::Runtime);
        assert_eq!(source.licence, "LicenseRef-US-Public-Domain");
        assert_eq!(
            source.origin.as_deref(),
            Some("https://pubchem.ncbi.nlm.nih.gov/compound/24261")
        );
    } else {
        assert_eq!(source.lane, SourceLane::BuildOracle, "{} lane", species.key);
        assert_eq!(
            source.licence, "LicenseRef-Kerotakis-Legacy-Provenance-Review-Required",
            "{} licence boundary",
            species.key
        );
        assert_eq!(
            source.origin.as_deref(),
            Some("crates/kerotakis-core/src/species.rs")
        );
        assert_eq!(source.revision, None);
        assert_eq!(source.retrieved, None);
    }

    let identity = document
        .identities
        .iter()
        .find(|record| record.id == species.key)
        .unwrap_or_else(|| panic!("missing identity for {}", species.key));
    assert_eq!(identity.name, species.name, "{} name", species.key);
    assert_eq!(
        identity.canonical_key,
        if species.inchikey.is_empty() {
            format!("legacy:species/{}", species.key)
        } else {
            species.inchikey.to_string()
        },
        "{} canonical key",
        species.key
    );
    let expected_identifiers = if species.inchikey.is_empty() {
        BTreeMap::new()
    } else {
        BTreeMap::from([("inchikey".to_string(), species.inchikey.to_string())])
    };
    assert_eq!(
        identity.identifiers, expected_identifiers,
        "{} identifiers",
        species.key
    );
    assert!(identity.synonyms.is_empty());
    assert_imported_evidence(
        &identity.evidence.source_id,
        &identity.evidence.method,
        &source_id,
    );

    let composition = document
        .compositions
        .iter()
        .find(|record| record.species_id == species.key)
        .unwrap_or_else(|| panic!("missing composition for {}", species.key));
    assert_eq!(
        composition.formula, species.formula,
        "{} formula",
        species.key
    );
    let parsed = parse_formula(species.formula).expect("legacy formula remains parseable");
    let exported_counts: BTreeMap<_, _> = composition
        .elements
        .iter()
        .map(|element| (element.element.clone(), element.count.value))
        .collect();
    assert_eq!(exported_counts, parsed.counts, "{} elements", species.key);
    assert_eq!(
        composition.net_charge.value, parsed.charge,
        "{} charge",
        species.key
    );
    assert_imported_evidence(
        &composition.evidence.source_id,
        &composition.evidence.method,
        &source_id,
    );

    let phase = phase(species.standard_phase);
    assert_property(
        document,
        species,
        PhaseProperty::MolarMass,
        species.molar_mass,
        "g/mol",
        Dimension::MolarMass,
        phase,
        &source_id,
    );
    assert_property(
        document,
        species,
        PhaseProperty::MolarHeatCapacity,
        species.heat_capacity,
        "J/(mol.K)",
        Dimension::MolarHeatCapacity,
        phase,
        &source_id,
    );
    assert_property(
        document,
        species,
        PhaseProperty::MassDensity,
        species.density,
        "g/mL",
        Dimension::MassDensity,
        phase,
        &source_id,
    );
    match species.dissolution_enthalpy_kj {
        Some(value) => assert_property(
            document,
            species,
            PhaseProperty::EnthalpyOfDissolution,
            value,
            "kJ/mol",
            Dimension::MolarEnergy,
            phase,
            &source_id,
        ),
        None => assert!(document.phase_thermodynamics.iter().all(|record| {
            record.species_id != species.key
                || record.property != PhaseProperty::EnthalpyOfDissolution
        })),
    }

    compare_optical(document, species, phase, &source_id);
    compare_model_parameters(document, species, phase, &source_id);
}

// Each argument names one column of the exported property record; bundling
// them would make the table-like call sites less explicit.
#[allow(clippy::too_many_arguments)]
fn assert_property(
    document: &RegistryDocument,
    species: &SpeciesData,
    property: PhaseProperty,
    value: f64,
    symbol: &str,
    dimension: Dimension,
    phase: Phase,
    source_id: &str,
) {
    let record = document
        .phase_thermodynamics
        .iter()
        .find(|record| record.species_id == species.key && record.property == property)
        .unwrap_or_else(|| panic!("missing {property:?} for {}", species.key));
    assert_eq!(record.phase, phase, "{} {property:?} phase", species.key);
    assert_imported_quantity(&record.quantity, value, symbol, dimension, phase, source_id);
}

fn compare_optical(
    document: &RegistryDocument,
    species: &SpeciesData,
    phase: Phase,
    source_id: &str,
) {
    let record = document
        .optical
        .iter()
        .find(|record| record.species_id == species.key);
    if !has_optical(species) {
        assert!(
            record.is_none(),
            "unexpected optical record for {}",
            species.key
        );
        return;
    }
    let record = record.unwrap();
    assert_eq!(record.phase, phase);
    assert_eq!(record.appearance.as_deref(), species.appearance);
    assert_eq!(record.flame_colour.as_deref(), species.flame_colour);
    assert_eq!(record.reflective_srgb, species.colour.map(rgb_hex));
    assert_imported_evidence(
        &record.evidence.source_id,
        &record.evidence.method,
        source_id,
    );
    match species.spectrum {
        Some(bands) => {
            let values = *bands;
            assert_eq!(record.spectrum.len(), BAND_NM.len());
            for ((sample, wavelength), value) in record.spectrum.iter().zip(BAND_NM).zip(values) {
                assert_imported_quantity(
                    &sample.wavelength,
                    wavelength,
                    "nm",
                    Dimension::Wavelength,
                    phase,
                    source_id,
                );
                assert_imported_quantity(
                    &sample.molar_absorptivity,
                    value,
                    "L/(mol.cm)",
                    Dimension::MolarAbsorptivity,
                    phase,
                    source_id,
                );
            }
        }
        None => assert!(record.spectrum.is_empty()),
    }
}

fn compare_model_parameters(
    document: &RegistryDocument,
    species: &SpeciesData,
    phase: Phase,
    source_id: &str,
) {
    let dissolves = parameter(
        document,
        &format!("dissolves-without-speciation/{}", species.key),
    );
    assert_eq!(
        dissolves.quantity.value,
        if species.dissolves_without_speciation {
            1.0
        } else {
            0.0
        }
    );
    assert_eq!(dissolves.quantity.uncertainty, Uncertainty::Exact);
    assert_eq!(dissolves.quantity.source_id, source_id);

    if let Some(colour) = species.colour {
        let tint = parameter(document, &format!("legacy-tint-strength/{}", species.key));
        assert_imported_quantity(
            &tint.quantity,
            colour.strength,
            "L/(mol.cm)",
            Dimension::MolarAbsorptivity,
            phase,
            source_id,
        );
    }
    if let Some(temperature) = species.forms_only_above_k {
        let threshold = parameter(document, &format!("forms-only-above/{}", species.key));
        assert_imported_quantity(
            &threshold.quantity,
            temperature,
            "K",
            Dimension::Temperature,
            phase,
            source_id,
        );
    }
}

fn parameter<'a>(
    document: &'a RegistryDocument,
    id: &str,
) -> &'a kerotakis_data::ModelParameterRecord {
    document
        .model_parameters
        .iter()
        .find(|record| record.id == id)
        .unwrap_or_else(|| panic!("missing model parameter {id}"))
}

fn assert_imported_quantity(
    quantity: &NumericRecord,
    value: f64,
    symbol: &str,
    dimension: Dimension,
    phase: Phase,
    source_id: &str,
) {
    assert_eq!(quantity.value, value);
    assert_eq!(quantity.unit.symbol, symbol);
    assert_eq!(quantity.unit.dimension, dimension);
    assert_eq!(quantity.conditions.phase, Some(phase));
    assert_eq!(quantity.uncertainty, Uncertainty::NotReported);
    assert_eq!(quantity.source_id, source_id);
    assert_eq!(quantity.method, Method::Imported(IMPORT_METHOD.to_string()));
}

fn assert_imported_evidence(source_id: &str, method: &Method, expected_source: &str) {
    assert_eq!(source_id, expected_source);
    assert_eq!(method, &Method::Imported(IMPORT_METHOD.to_string()));
}

fn has_optical(species: &SpeciesData) -> bool {
    species.appearance.is_some()
        || species.flame_colour.is_some()
        || species.colour.is_some()
        || species.spectrum.is_some()
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
