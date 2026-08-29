use std::collections::BTreeMap;

use kerotakis_data::*;
use serde_json::json;

fn field(value: serde_json::Value, source_field: &str, licence: &str) -> CandidateField {
    CandidateField::new(value, source_field, licence)
}

fn candidate(record: &str, name: &str) -> QuarantinedCandidate {
    QuarantinedCandidate {
        adapter_id: "synthetic-v1".into(),
        source_record_id: format!("snapshot/{record}"),
        external_record_id: record.into(),
        identity_key: Some("XLYOFNOQVPJJNP-UHFFFAOYSA-N".into()),
        fields: BTreeMap::from([
            (
                "canonical_name".into(),
                field(json!(name), "record.name", "CC0-1.0"),
            ),
            (
                "patent_count".into(),
                field(json!(12345), "record.patents", "LicenseRef-Unknown"),
            ),
            (
                "hazard_text".into(),
                field(json!("irritant"), "record.hazard", "LicenseRef-Restricted"),
            ),
        ]),
    }
}

#[test]
fn pinned_snapshot_refuses_changed_bytes() {
    let raw = br#"{"records":["water"]}"#;
    let manifest = SnapshotManifest {
        schema: ADAPTER_SCHEMA_VERSION,
        adapter_id: "synthetic-v1".into(),
        source_id: "synthetic".into(),
        source_revision: "2026-08".into(),
        retrieved: "2026-08-28".into(),
        raw_artifact: "raw/synthetic-2026-08.json".into(),
        record_count: 1,
        sha256: snapshot_sha256(raw),
    };
    manifest.verify(raw).unwrap();
    assert!(matches!(
        manifest.verify(br#"{"records":["water","salt"]}"#),
        Err(AdapterError::ChecksumMismatch { .. })
    ));
}

#[test]
fn tainted_and_unallowlisted_fields_cannot_cross_review() {
    let policy = PromotionPolicy {
        fields: BTreeMap::from([
            (
                "canonical_name".into(),
                RuntimeFieldPolicy::new("name", ["CC0-1.0"]),
            ),
            (
                "hazard_text".into(),
                RuntimeFieldPolicy::new("safety_note", ["CC-BY-4.0"]),
            ),
        ]),
    };
    let review = review_candidate(&candidate("record-a", "water"), &policy);

    assert_eq!(
        review
            .accepted
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        vec!["name"]
    );
    assert_eq!(review.rejected.len(), 2);
    assert!(review
        .rejected
        .iter()
        .any(|rejection| matches!(rejection.reason, FieldRejectionReason::UnallowlistedField)));
    assert!(review.rejected.iter().any(|rejection| matches!(
        rejection.reason,
        FieldRejectionReason::LicenceNotAllowed { .. }
    )));
}

#[test]
fn missing_provenance_and_target_collisions_reject_every_affected_field() {
    let mut input = candidate("record-a", "water");
    input
        .fields
        .get_mut("canonical_name")
        .unwrap()
        .source_field
        .clear();
    input.fields.insert(
        "common_name".into(),
        field(json!("water"), "record.common_name", "CC0-1.0"),
    );
    let policy = PromotionPolicy {
        fields: BTreeMap::from([
            (
                "canonical_name".into(),
                RuntimeFieldPolicy::new("name", ["CC0-1.0"]),
            ),
            (
                "common_name".into(),
                RuntimeFieldPolicy::new("name", ["CC0-1.0"]),
            ),
        ]),
    };

    let review = review_candidate(&input, &policy);
    assert!(review.accepted.is_empty());
    assert_eq!(review.rejected.len(), 4);
    assert!(review
        .rejected
        .iter()
        .any(|rejection| matches!(rejection.reason, FieldRejectionReason::MissingProvenance)));
    assert!(review.rejected.iter().any(|rejection| matches!(
        rejection.reason,
        FieldRejectionReason::TargetCollision { .. }
    )));
}

#[test]
fn same_identity_from_two_records_produces_a_reviewable_conflict() {
    let conflicts = identity_conflicts(&[
        candidate("record-b", "oxidane"),
        candidate("record-a", "water"),
    ]);

    assert_eq!(conflicts.len(), 1);
    assert_eq!(
        conflicts[0]
            .records
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        vec!["record-a", "record-b"]
    );
    assert_eq!(
        conflicts[0]
            .differing_fields
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        vec!["canonical_name"]
    );
}

#[test]
fn quarantine_serialization_is_input_order_independent() {
    let a = candidate("record-a", "water");
    let b = candidate("record-b", "oxidane");
    assert_eq!(
        canonical_quarantine_bytes(vec![a.clone(), b.clone()]).unwrap(),
        canonical_quarantine_bytes(vec![b, a]).unwrap()
    );
}

#[test]
fn refresh_diff_is_stable_and_field_granular() {
    let removed = candidate("record-a", "water");
    let mut changed = candidate("record-b", "oxidane");
    let unchanged = candidate("record-c", "water");
    let mut refreshed = changed.clone();
    refreshed.identity_key = Some("REFRESHED-IDENTITY".into());
    refreshed.fields.remove("patent_count");
    refreshed.fields.insert(
        "formula".into(),
        field(json!("H2O"), "record.formula", "CC0-1.0"),
    );
    refreshed.fields.insert(
        "canonical_name".into(),
        field(json!("water"), "record.name", "CC0-1.0"),
    );
    changed.fields.insert(
        "formula".into(),
        field(json!("OH2"), "record.formula", "CC0-1.0"),
    );
    let added = candidate("record-d", "heavy water");

    let first = diff_quarantine(
        &[removed.clone(), changed.clone(), unchanged.clone()],
        &[unchanged.clone(), refreshed.clone(), added.clone()],
    )
    .unwrap();
    let reordered = diff_quarantine(
        &[unchanged, changed, removed],
        &[added, refreshed, candidate("record-c", "water")],
    )
    .unwrap();

    assert_eq!(first, reordered);
    assert_eq!(first.added_records[0].external_record_id, "record-d");
    assert_eq!(first.removed_records[0].external_record_id, "record-a");
    assert_eq!(first.changed_records.len(), 1);
    assert!(first.changed_records[0].identity_key.is_some());
    assert_eq!(first.changed_records[0].fields.len(), 3);
}

#[test]
fn duplicate_adapter_record_ids_are_refused() {
    let duplicate = candidate("record-a", "water");
    assert!(matches!(
        diff_quarantine(&[duplicate.clone(), duplicate], &[]),
        Err(AdapterError::DuplicateRecord { .. })
    ));
}

#[test]
fn batch_review_order_is_deterministic_and_keeps_identity_conflicts() {
    let policy = PromotionPolicy {
        fields: BTreeMap::from([(
            "canonical_name".into(),
            RuntimeFieldPolicy::new("name", ["CC0-1.0"]),
        )]),
    };
    let report = review_candidates(
        vec![
            candidate("record-b", "oxidane"),
            candidate("record-a", "water"),
        ],
        &policy,
    );

    assert_eq!(report.schema, ADAPTER_SCHEMA_VERSION);
    assert_eq!(report.reviews[0].external_record_id, "record-a");
    assert_eq!(report.reviews[1].external_record_id, "record-b");
    assert_eq!(report.identity_conflicts.len(), 1);
}
