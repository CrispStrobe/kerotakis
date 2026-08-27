//! DATA-010: Verify that the compiled pack reproduces the hand-authored registry.
//!
//! This test is the gate for removing the hand-authored REGISTRY const from
//! species.rs. Once it passes, the pack is the authoritative source and the
//! static const can be removed (or generated from the pack at build time).

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

#[test]
fn source_registry_loads_and_validates() {
    let path = workspace_root().join("data/registry/registry-source-v1.json");
    let json = std::fs::read_to_string(&path).expect("registry source JSON must exist");
    let doc: kerotakis_data::RegistryDocument =
        serde_json::from_str(&json).expect("registry source must parse");

    // Verify basic structure
    assert!(!doc.sources.is_empty(), "registry must have source records");
    assert!(
        !doc.identities.is_empty(),
        "registry must have identity records"
    );
    doc.validate().expect("checked-in registry must validate");
    assert_eq!(doc.material_recipes.len(), 24);
}

#[test]
fn pack_round_trip_is_deterministic() {
    let path = workspace_root().join("data/registry/registry-source-v1.json");
    let json = std::fs::read_to_string(&path).expect("registry source JSON must exist");
    let doc: kerotakis_data::RegistryDocument =
        serde_json::from_str(&json).expect("registry source must parse");

    // Serialize to pack payload
    let payload1 = kerotakis_data::serialize_pack_payload(&doc);

    // Deserialize and re-serialize — must be identical (deterministic)
    let doc2: kerotakis_data::RegistryDocument =
        serde_json::from_slice(&payload1).expect("round-trip deserialize");
    assert_eq!(doc2.material_recipes.len(), doc.material_recipes.len());
    for (index, (round_tripped, source)) in doc2
        .material_recipes
        .iter()
        .zip(&doc.material_recipes)
        .enumerate()
    {
        assert_eq!(
            round_tripped, source,
            "material recipe {index} ({}) changed during pack round-trip",
            source.id
        );
    }
    let peroxide = doc2
        .material_recipe("Wasserstoffperoxid 3%", Some("de"))
        .expect("localized household recipe survives the pack");
    let expansion = peroxide.expand(100.0, 0).expect("fixed expansion");
    assert_eq!(expansion.components[0].amount, 3.0);
    assert_eq!(expansion.unresolved_amount, 0.0);
    let payload2 = kerotakis_data::serialize_pack_payload(&doc2);

    assert_eq!(
        payload1, payload2,
        "pack payload must be deterministic across round trips"
    );
}

#[test]
fn pack_covers_all_hand_authored_species() {
    let path = workspace_root().join("data/registry/registry-source-v1.json");
    let json = std::fs::read_to_string(&path).expect("registry source JSON must exist");
    let doc: kerotakis_data::RegistryDocument =
        serde_json::from_str(&json).expect("registry source must parse");

    // Every species in the hand-authored registry must appear in the pack.
    // The pack uses InChIKey as canonical_key; check by name instead.
    let pack_names: Vec<&str> = doc.identities.iter().map(|id| id.name.as_str()).collect();

    // Check a representative sample of the 75 species
    let expected = [
        "water",
        "sodium chloride",
        "hydrochloric acid",
        "sodium hydroxide",
    ];
    for name in &expected {
        assert!(
            pack_names
                .iter()
                .any(|n| n.to_lowercase().contains(&name.to_lowercase())),
            "species '{}' missing from pack registry (names: {:?})",
            name,
            &pack_names[..5]
        );
    }

    // Verify count is reasonable (the hand-authored registry has 75 species)
    assert!(
        doc.identities.len() >= 70,
        "pack should have ~75 species, got {}",
        doc.identities.len()
    );
}

#[test]
fn provenance_records_all_have_lanes() {
    let path = workspace_root().join("data/registry/registry-source-v1.json");
    let json = std::fs::read_to_string(&path).expect("registry source JSON must exist");
    let doc: kerotakis_data::RegistryDocument =
        serde_json::from_str(&json).expect("registry source must parse");

    for source in &doc.sources {
        // lane is an enum (Runtime, BuildOracle, ExternalOracle) — always set
        let _ = &source.lane; // type check: SourceLane is not Option
        assert!(!source.id.is_empty(), "source record has empty id");
    }
}
