//! THERMO-003: Validate that phase-specific property records carry ranges
//! and sources through the resolution ladder.

use kerotakis_data::*;
use std::fs;
use std::path::Path;

#[test]
fn registry_phase_properties_resolve_with_provenance() {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/registry/registry-source-v1.json");
    let content = fs::read_to_string(&path).unwrap();
    let doc: RegistryDocument = serde_json::from_str(&content).unwrap();

    // Water molar mass should resolve
    let res = resolve_phase_property(
        &doc,
        "water",
        &PhaseProperty::MolarMass,
        &Conditions::default(),
    );
    match res {
        Resolution::Resolved(v) => {
            assert!((v.value - 18.015).abs() < 0.01, "water M = {}", v.value);
            assert!(!v.source_id.is_empty(), "source_id must be present");
            assert!(!v.method_detail.is_empty(), "method must be present");
        }
        Resolution::Unavailable { reason } => panic!("water molar mass unavailable: {reason}"),
    }

    // NaCl molar mass
    let res = resolve_phase_property(
        &doc,
        "NaCl",
        &PhaseProperty::MolarMass,
        &Conditions::default(),
    );
    assert!(res.is_available(), "NaCl molar mass should be available");

    // A made-up species should be unavailable
    let res = resolve_phase_property(
        &doc,
        "unobtainium",
        &PhaseProperty::MolarMass,
        &Conditions::default(),
    );
    assert!(!res.is_available(), "unobtainium should be unavailable");
}

#[test]
fn every_phase_property_has_a_source() {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/registry/registry-source-v1.json");
    let content = fs::read_to_string(&path).unwrap();
    let doc: RegistryDocument = serde_json::from_str(&content).unwrap();

    for record in &doc.phase_thermodynamics {
        assert!(
            !record.quantity.source_id.is_empty(),
            "phase_thermo record {} has no source_id",
            record.id
        );
        assert!(
            record.quantity.value.is_finite(),
            "phase_thermo record {} has non-finite value",
            record.id
        );
    }
}
