use std::path::Path;

use kerotakis_data::canonical_quarantine_bytes;
use kerotakis_data::fluid_parameters::{
    parse_source_document, promotion_report, FluidParameterImportError, PILOT_IDENTITIES,
};
use serde_json::{json, Value};

fn fixture() -> Vec<u8> {
    std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/quarantine/brd031-fluid-synthetic-v1/source.json"),
    )
    .expect("read synthetic fixture")
}

#[test]
fn candidate_order_and_bytes_are_independent_of_source_order() {
    let raw = fixture();
    let first = parse_source_document(&raw).expect("fixture imports");
    assert_eq!(
        first
            .iter()
            .map(|candidate| candidate.external_record_id.as_str())
            .collect::<Vec<_>>(),
        PILOT_IDENTITIES.map(|(id, _)| id).to_vec()
    );

    let mut reordered: Value = serde_json::from_slice(&raw).unwrap();
    reordered["records"].as_array_mut().unwrap().reverse();
    let second = parse_source_document(&serde_json::to_vec(&reordered).unwrap()).unwrap();
    assert_eq!(
        canonical_quarantine_bytes(first).unwrap(),
        canonical_quarantine_bytes(second).unwrap()
    );
}

#[test]
fn pilot_ids_and_inchikeys_join_the_checked_registry() {
    let registry: Value = serde_json::from_slice(include_bytes!(
        "../../../data/registry/registry-source-v1.json"
    ))
    .unwrap();
    let identities = registry["identities"].as_array().unwrap();
    for (id, inchikey) in PILOT_IDENTITIES {
        let matches = identities
            .iter()
            .filter(|identity| identity["id"] == id && identity["canonical_key"] == inchikey)
            .count();
        assert_eq!(matches, 1, "{id} must join exactly one canonical identity");
    }
}

#[test]
fn malformed_and_nonfinite_numbers_are_refused() {
    assert!(matches!(
        parse_source_document(br#"{"records": [}"#),
        Err(FluidParameterImportError::MalformedDocument { .. })
    ));

    let text = String::from_utf8(fixture()).unwrap();
    let nonfinite = text.replacen("\"value\":1.0", "\"value\":1e999", 1);
    assert!(matches!(
        parse_source_document(nonfinite.as_bytes()),
        Err(FluidParameterImportError::MalformedDocument { .. })
            | Err(FluidParameterImportError::InvalidQuantity { .. })
    ));
}

#[test]
fn all_six_canonical_identities_are_required() {
    let mut document: Value = serde_json::from_slice(&fixture()).unwrap();
    document["records"]
        .as_array_mut()
        .unwrap()
        .retain(|record| record["id"] != Value::String("NH3".to_owned()));
    assert_eq!(
        parse_source_document(&serde_json::to_vec(&document).unwrap()).unwrap_err(),
        FluidParameterImportError::MissingCanonicalIdentity {
            id: "NH3".to_owned()
        }
    );
}

#[test]
fn a_source_licence_cannot_widen_the_runtime_policy() {
    let mut document: Value = serde_json::from_slice(&fixture()).unwrap();
    document["data_licence"] = json!("GPL-3.0-only");
    let candidates = parse_source_document(&serde_json::to_vec(&document).unwrap()).unwrap();
    let report = promotion_report(candidates);
    assert!(report
        .reviews
        .iter()
        .all(|review| review.accepted.is_empty()));
    assert!(report
        .reviews
        .iter()
        .all(|review| !review.rejected.is_empty()));
}

#[test]
fn synthetic_permissive_fixture_has_no_schema_or_licence_rejections() {
    let candidates = parse_source_document(&fixture()).unwrap();
    let report = promotion_report(candidates);
    assert!(report.identity_conflicts.is_empty());
    assert!(report
        .reviews
        .iter()
        .all(|review| review.rejected.is_empty()));
    assert!(report.reviews.iter().all(|review| {
        review
            .accepted
            .contains_key("model.pc_saft.segment_diameter")
    }));
}

#[test]
fn associating_fluids_require_complete_association_parameters() {
    let mut document: Value = serde_json::from_slice(&fixture()).unwrap();
    let water = document["records"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|record| record["id"] == "water")
        .unwrap();
    water["pc_saft"]
        .as_object_mut()
        .unwrap()
        .remove("association");
    assert_eq!(
        parse_source_document(&serde_json::to_vec(&document).unwrap()).unwrap_err(),
        FluidParameterImportError::MissingAssociation { id: "water".into() }
    );
}

#[test]
fn an_association_model_requires_at_least_one_site() {
    let mut document: Value = serde_json::from_slice(&fixture()).unwrap();
    let association = &mut document["records"][0]["pc_saft"]["association"];
    association["na"] = json!(0);
    association["nb"] = json!(0);
    association["nc"] = json!(0);
    assert!(matches!(
        parse_source_document(&serde_json::to_vec(&document).unwrap()),
        Err(FluidParameterImportError::InvalidQuantity { field, .. })
            if field == "pc_saft.association.site_count"
    ));
}

#[test]
fn a_quantity_with_the_wrong_dimension_is_rejected_by_review() {
    let mut document: Value = serde_json::from_slice(&fixture()).unwrap();
    document["records"][0]["pc_saft"]["molar_mass"]["unit"] = json!("K");
    let report =
        promotion_report(parse_source_document(&serde_json::to_vec(&document).unwrap()).unwrap());
    let ethanol = report
        .reviews
        .iter()
        .find(|review| review.external_record_id == "ethanol")
        .unwrap();
    assert!(ethanol
        .rejected
        .iter()
        .any(|rejection| rejection.field == "pc_saft.molar_mass"));
}

#[test]
fn segment_diameter_cannot_be_promoted_as_an_optical_wavelength() {
    let mut document: Value = serde_json::from_slice(&fixture()).unwrap();
    document["records"][0]["pc_saft"]["segment_diameter"]["unit"] = json!("pm");
    let report =
        promotion_report(parse_source_document(&serde_json::to_vec(&document).unwrap()).unwrap());
    let ethanol = report
        .reviews
        .iter()
        .find(|review| review.external_record_id == "ethanol")
        .unwrap();
    assert!(ethanol
        .rejected
        .iter()
        .any(|rejection| rejection.field == "pc_saft.segment_diameter"));
}
