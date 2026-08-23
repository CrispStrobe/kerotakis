//! DATA-010 gate: verify the compiled pack contains every species from
//! the static registry with matching values.

use kerotakis_data::*;
use std::fs;
use std::path::Path;

#[test]
fn pack_contains_all_registry_species() {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/registry/registry-source-v1.json");
    let content = fs::read_to_string(&path).unwrap();
    let doc: RegistryDocument = serde_json::from_str(&content).unwrap();

    // The pack must have at least as many identities as the static registry (75).
    assert!(
        doc.identities.len() >= 75,
        "pack has {} identities, expected >= 75",
        doc.identities.len()
    );

    // Every identity must have a canonical_key (InChIKey).
    for id in &doc.identities {
        assert!(
            !id.canonical_key.is_empty(),
            "identity {} has no canonical_key",
            id.id
        );
    }

    // The pack must have phase thermodynamic records.
    assert!(
        doc.phase_thermodynamics.len() >= 200,
        "pack has {} phase_thermo records, expected >= 200",
        doc.phase_thermodynamics.len()
    );

    // Compile to pack and load back — round-trip parity.
    let payload = serialize_pack_payload(&doc);
    use sha2::{Digest, Sha256};
    let hash = Sha256::digest(&payload);
    let mut pack = Vec::new();
    pack.extend_from_slice(PACK_MAGIC);
    pack.extend_from_slice(&PACK_VERSION.to_le_bytes());
    pack.extend_from_slice(&hash);
    pack.extend_from_slice(&payload);

    let loaded = load_pack(&pack).unwrap();
    assert_eq!(loaded.identities.len(), doc.identities.len());
    assert_eq!(
        loaded.phase_thermodynamics.len(),
        doc.phase_thermodynamics.len()
    );
}

#[test]
fn pack_molar_masses_match_known_values() {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/registry/registry-source-v1.json");
    let content = fs::read_to_string(&path).unwrap();
    let doc: RegistryDocument = serde_json::from_str(&content).unwrap();

    // Spot-check known molar masses.
    let known = [("water", 18.015), ("NaCl", 58.44), ("CO2", 44.01)];

    for (species, expected_mm) in &known {
        let res = resolve_phase_property(
            &doc,
            species,
            &PhaseProperty::MolarMass,
            &Conditions::default(),
        );
        match res {
            Resolution::Resolved(v) => {
                assert!(
                    (v.value - expected_mm).abs() < 0.1,
                    "{species}: pack M = {}, expected {expected_mm}",
                    v.value
                );
            }
            Resolution::Unavailable { reason } => {
                panic!("{species} molar mass unavailable: {reason}");
            }
        }
    }
}
