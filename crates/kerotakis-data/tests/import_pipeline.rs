//! DATA-006: End-to-end test of the CC0 Wikidata import pipeline.
//!
//! Reads the tiny Wikidata import, converts to RegistryDocument records,
//! validates, compiles to a pack, loads back, and verifies round-trip.

use kerotakis_data::*;
use std::path::Path;

#[derive(serde::Deserialize)]
struct WikidataImport {
    source: WikidataSource,
    identities: Vec<WikidataIdentity>,
}

#[derive(serde::Deserialize)]
struct WikidataSource {
    id: String,
    citation: String,
    licence: String,
    lane: String,
    origin: String,
    retrieved: String,
}

#[derive(serde::Deserialize)]
struct WikidataIdentity {
    species_id: String,
    wikidata_qid: String,
    cas_rn: String,
    pubchem_cid: String,
    inchikey: String,
    iupac_name: String,
}

fn load_wikidata_import() -> WikidataImport {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../data/imports/wikidata-cc0-identities.json");
    let content = std::fs::read_to_string(&path).unwrap();
    serde_json::from_str(&content).unwrap()
}

fn wikidata_to_registry(import: &WikidataImport) -> RegistryDocument {
    let mut doc = RegistryDocument::empty();

    let lane = match import.source.lane.as_str() {
        "runtime" => SourceLane::Runtime,
        "build_oracle" => SourceLane::BuildOracle,
        _ => SourceLane::ExternalOracle,
    };

    doc.sources.push(SourceRecord {
        id: import.source.id.clone(),
        citation: import.source.citation.clone(),
        licence: import.source.licence.clone(),
        lane,
        origin: Some(import.source.origin.clone()),
        revision: None,
        retrieved: Some(import.source.retrieved.clone()),
    });

    for ident in &import.identities {
        let mut identifiers = std::collections::BTreeMap::new();
        identifiers.insert("inchikey".into(), ident.inchikey.clone());
        identifiers.insert("cas_rn".into(), ident.cas_rn.clone());
        identifiers.insert("pubchem_cid".into(), ident.pubchem_cid.clone());
        identifiers.insert("wikidata_qid".into(), ident.wikidata_qid.clone());

        doc.identities.push(IdentityRecord {
            id: format!("{}/{}", import.source.id, ident.species_id),
            canonical_key: ident.inchikey.clone(),
            name: ident.iupac_name.clone(),
            identifiers,
            synonyms: vec![],
            evidence: Evidence {
                source_id: import.source.id.clone(),
                method: Method::Imported(format!(
                    "Wikidata {} CC0 identity crosswalk",
                    ident.wikidata_qid
                )),
            },
        });
    }

    doc
}

#[test]
fn wikidata_cc0_import_validates() {
    let import = load_wikidata_import();
    let doc = wikidata_to_registry(&import);

    // Source is CC0, lane is runtime → eligible for pack.
    assert_eq!(doc.sources[0].licence, "CC0-1.0");
    assert!(doc.sources[0].lane.may_enter_runtime_pack());

    // Validate.
    doc.validate().expect("imported records should validate");
}

#[test]
fn wikidata_cc0_import_round_trips_through_pack() {
    let import = load_wikidata_import();
    let doc = wikidata_to_registry(&import);

    // Compile to pack.
    let payload = serialize_pack_payload(&doc);
    use sha2::{Digest, Sha256};
    let hash = Sha256::digest(&payload);

    let mut pack = Vec::new();
    pack.extend_from_slice(PACK_MAGIC);
    pack.extend_from_slice(&PACK_VERSION.to_le_bytes());
    pack.extend_from_slice(&hash);
    pack.extend_from_slice(&payload);

    // Load back.
    let loaded = load_pack(&pack).unwrap();
    assert_eq!(loaded.identities.len(), 3);
    assert_eq!(loaded.identities[0].name, "oxidane");
    assert_eq!(loaded.identities[1].name, "sodium chloride");
    assert_eq!(loaded.identities[2].name, "calcium carbonate");
}

#[test]
fn pubchem_compatible_fields_are_accepted_and_incompatible_rejected() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../data/imports/pubchem-approved-properties.json");
    let content = std::fs::read_to_string(&path).unwrap();
    let import: serde_json::Value = serde_json::from_str(&content).unwrap();

    let properties = import["properties"].as_array().unwrap();
    let compatible: Vec<_> = properties
        .iter()
        .filter(|p| p["compatible"].as_bool() == Some(true))
        .collect();
    let incompatible: Vec<_> = properties
        .iter()
        .filter(|p| p["compatible"].as_bool() == Some(false))
        .collect();

    // 3 compatible fields should be accepted.
    assert_eq!(compatible.len(), 3, "expected 3 compatible PubChem fields");

    // 1 incompatible field should be rejected.
    assert_eq!(
        incompatible.len(),
        1,
        "expected 1 incompatible PubChem field"
    );
    assert_eq!(incompatible[0]["field"].as_str(), Some("patent_count"));
    assert!(incompatible[0]["rejection_reason"]
        .as_str()
        .unwrap()
        .contains("not a chemical property"));

    // The source is build_oracle, not runtime — PubChem data is used for
    // validation only, not shipped in the app pack.
    assert_eq!(import["source"]["lane"].as_str(), Some("build_oracle"));
}

#[test]
fn wikidata_identities_carry_crosswalk_ids() {
    let import = load_wikidata_import();
    let doc = wikidata_to_registry(&import);
    let water = &doc.identities[0];
    assert_eq!(water.identifiers["cas_rn"], "7732-18-5");
    assert_eq!(water.identifiers["pubchem_cid"], "962");
    assert_eq!(water.identifiers["wikidata_qid"], "Q283");
}
