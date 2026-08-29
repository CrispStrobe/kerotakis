//! BRD-003: one synthetic failure per refusal class. Every importer that
//! BRD-003 gates (BRD-010, BRD-011, BRD-013, BRD-060) calls the same function,
//! so each of these is a promotion that must not happen.

use std::collections::{BTreeMap, BTreeSet};

use kerotakis_data::*;
use serde_json::json;

const RAW: &[u8] = br#"{"records":[{"id":"water","name":"water","density":0.997}]}"#;

fn manifest() -> SnapshotManifest {
    SnapshotManifest {
        schema: ADAPTER_SCHEMA_VERSION,
        adapter_id: "synthetic-v1".into(),
        source_id: "synthetic".into(),
        source_revision: "fixture-1".into(),
        retrieved: "2026-08-29".into(),
        raw_artifact: "raw/snapshot.json".into(),
        record_count: 1,
        sha256: snapshot_sha256(RAW),
    }
}

fn candidate() -> QuarantinedCandidate {
    QuarantinedCandidate {
        adapter_id: "synthetic-v1".into(),
        source_record_id: "snapshot/water".into(),
        external_record_id: "water".into(),
        identity_key: Some("XLYOFNOQVPJJNP-UHFFFAOYSA-N".into()),
        fields: BTreeMap::from([
            (
                "canonical_name".to_string(),
                CandidateField::new(json!("water"), "records[0].name", "CC0-1.0"),
            ),
            (
                "density".to_string(),
                CandidateField::new(json!(0.997), "records[0].density", "CC0-1.0")
                    .with_unit("g/cm3"),
            ),
        ]),
    }
}

fn policy() -> PromotionPolicy {
    PromotionPolicy {
        fields: BTreeMap::from([
            (
                "canonical_name".to_string(),
                RuntimeFieldPolicy::new("name", ["CC0-1.0"]),
            ),
            (
                "density".to_string(),
                RuntimeFieldPolicy::new("mass_density", ["CC0-1.0"])
                    .with_dimension(Dimension::MassDensity),
            ),
        ]),
    }
}

fn eligible() -> Vec<EligibleFieldList> {
    vec![EligibleFieldList {
        adapter_id: "synthetic-v1".into(),
        external_record_id: "water".into(),
        fields: vec!["canonical_name".into(), "density".into()],
    }]
}

/// Run the lint over a flow a caller has tampered with.
fn lint(
    tamper: impl FnOnce(
        &mut SnapshotManifest,
        &mut Vec<u8>,
        &mut Vec<QuarantinedCandidate>,
        &mut PromotionPolicy,
        &mut Vec<EligibleFieldList>,
    ),
) -> ProvenanceLintReport {
    let mut manifest = manifest();
    let mut raw = RAW.to_vec();
    let mut candidates = vec![candidate()];
    let mut policy = policy();
    let mut eligible = eligible();
    tamper(
        &mut manifest,
        &mut raw,
        &mut candidates,
        &mut policy,
        &mut eligible,
    );
    let allowed = default_runtime_data_licences();
    lint_promotion(&PromotionLintInput {
        manifest: &manifest,
        raw_snapshot: &raw,
        candidates: &candidates,
        policy: &policy,
        allowed_runtime_licences: &allowed,
        eligible_fields: &eligible,
    })
}

fn violations(report: &ProvenanceLintReport) -> Vec<String> {
    report
        .violations
        .iter()
        .map(|violation| {
            serde_json::to_value(violation).unwrap()["violation"]
                .as_str()
                .unwrap()
                .to_owned()
        })
        .collect()
}

#[test]
fn a_clean_flow_is_accepted_and_counts_what_it_checked() {
    let report = lint(|_, _, _, _, _| {});
    assert!(!report.refuses(), "{:?}", report.violations);
    assert_eq!(report.checked_records, 1);
    assert_eq!(report.checked_fields, 2);
    assert_eq!(report.adapter_id, "synthetic-v1");
    assert!(report.clone().into_result().is_ok());
}

#[test]
fn a_field_without_a_source_or_a_licence_cannot_be_promoted() {
    let report = lint(|_, _, candidates, _, _| {
        candidates[0]
            .fields
            .get_mut("canonical_name")
            .unwrap()
            .source_field
            .clear();
        candidates[0]
            .fields
            .get_mut("density")
            .unwrap()
            .licence
            .clear();
    });
    assert!(report.refuses());
    assert_eq!(
        violations(&report),
        vec!["missing_field_provenance", "missing_field_provenance"]
    );
    assert!(report.violations.iter().any(|violation| matches!(
        violation,
        ProvenanceViolation::MissingFieldProvenance { missing, field, .. }
            if missing == "source_field" && field == "canonical_name"
    )));
    assert!(report.violations.iter().any(|violation| matches!(
        violation,
        ProvenanceViolation::MissingFieldProvenance { missing, field, .. }
            if missing == "licence" && field == "density"
    )));
}

#[test]
fn a_record_with_no_source_record_id_taints_every_field_it_carries() {
    let report = lint(|_, _, candidates, _, _| candidates[0].source_record_id.clear());
    assert_eq!(
        violations(&report),
        vec!["missing_field_provenance", "missing_field_provenance"]
    );
}

#[test]
fn a_licence_outside_the_runtime_lane_cannot_reach_a_runtime_pack() {
    let report = lint(|_, _, candidates, _, _| {
        candidates[0]
            .fields
            .get_mut("canonical_name")
            .unwrap()
            .licence = "CC-BY-SA-4.0".into();
    });
    assert_eq!(violations(&report), vec!["licence_not_allowed_for_runtime"]);
    assert!(matches!(
        &report.violations[0],
        ProvenanceViolation::LicenceNotAllowedForRuntime { licence, .. } if licence == "CC-BY-SA-4.0"
    ));
}

#[test]
fn a_policy_that_would_admit_a_share_alike_licence_is_itself_refused() {
    let report = lint(|_, _, _, policy, _| {
        policy
            .fields
            .get_mut("canonical_name")
            .unwrap()
            .allowed_licences
            .insert("CC-BY-SA-4.0".into());
    });
    assert_eq!(
        violations(&report),
        vec!["policy_admits_non_runtime_licence"]
    );
}

#[test]
fn a_caller_may_narrow_the_runtime_licence_lane() {
    let manifest = manifest();
    let candidates = vec![candidate()];
    let policy = policy();
    let eligible = eligible();
    let narrower = BTreeSet::from(["CC-BY-4.0".to_string()]);
    let report = lint_promotion(&PromotionLintInput {
        manifest: &manifest,
        raw_snapshot: RAW,
        candidates: &candidates,
        policy: &policy,
        allowed_runtime_licences: &narrower,
        eligible_fields: &eligible,
    });
    assert!(report.refuses());
    assert!(report.violations.iter().any(|violation| matches!(
        violation,
        ProvenanceViolation::LicenceNotAllowedForRuntime { .. }
    )));
}

#[test]
fn raw_bytes_that_no_longer_hash_to_the_manifest_stop_the_flow() {
    let report = lint(|_, raw, _, _, _| raw.push(b'\n'));
    assert_eq!(violations(&report), vec!["snapshot_checksum_mismatch"]);
    let ProvenanceViolation::SnapshotChecksumMismatch { expected, actual } = &report.violations[0]
    else {
        panic!("expected a checksum refusal");
    };
    assert_ne!(expected, actual);
    assert_eq!(*expected, snapshot_sha256(RAW));
}

#[test]
fn an_unusable_manifest_is_refused_before_any_field_is_read() {
    let report = lint(|manifest, _, _, _, _| manifest.source_revision.clear());
    assert_eq!(violations(&report), vec!["snapshot_manifest_rejected"]);
}

#[test]
fn candidates_cannot_claim_a_snapshot_they_did_not_come_from() {
    let report = lint(|_, _, candidates, _, _| {
        candidates[0].adapter_id = "some-other-adapter".into();
    });
    // The eligible list still names the original adapter, so the record it
    // points at is now missing too — both are real problems.
    assert!(violations(&report).contains(&"snapshot_adapter_mismatch".to_string()));
}

#[test]
fn more_candidates_than_the_snapshot_holds_is_impossible() {
    let report = lint(|_, _, candidates, _, _| {
        let mut second = candidate();
        second.external_record_id = "heavy-water".into();
        candidates.push(second);
    });
    assert!(violations(&report).contains(&"candidates_exceed_snapshot".to_string()));
}

#[test]
fn two_candidates_sharing_one_record_key_are_refused() {
    let report = lint(|manifest, _, candidates, _, _| {
        manifest.record_count = 4;
        candidates.push(candidate());
    });
    assert!(violations(&report).contains(&"duplicate_candidate".to_string()));
}

#[test]
fn an_eligible_list_naming_a_field_the_record_lacks_is_refused() {
    let report = lint(|_, _, _, _, eligible| {
        eligible[0].fields.push("melting_point".into());
    });
    assert_eq!(violations(&report), vec!["eligible_field_not_on_record"]);
    assert!(matches!(
        &report.violations[0],
        ProvenanceViolation::EligibleFieldNotOnRecord { field, .. } if field == "melting_point"
    ));
}

#[test]
fn an_eligible_list_naming_an_unknown_record_is_refused() {
    let report = lint(|_, _, _, _, eligible| {
        eligible[0].external_record_id = "heavy-water".into();
    });
    assert_eq!(
        violations(&report),
        vec!["eligible_record_not_in_quarantine"]
    );
}

#[test]
fn an_eligible_field_the_policy_never_allowlisted_is_refused() {
    let report = lint(|_, _, candidates, _, eligible| {
        candidates[0].fields.insert(
            "patent_count".into(),
            CandidateField::new(json!(12), "records[0].patents", "CC0-1.0"),
        );
        eligible[0].fields.push("patent_count".into());
    });
    assert_eq!(violations(&report), vec!["eligible_field_not_allowlisted"]);
}

#[test]
fn a_repeated_eligible_field_is_refused_rather_than_deduplicated() {
    let report = lint(|_, _, _, _, eligible| eligible[0].fields.push("density".into()));
    assert_eq!(violations(&report), vec!["eligible_field_repeated"]);
}

#[test]
fn a_quantity_whose_unit_is_not_reviewed_cannot_be_promoted() {
    let report = lint(|_, _, candidates, _, _| {
        candidates[0].fields.get_mut("density").unwrap().unit = Some("furlongs per barn".into());
    });
    assert_eq!(violations(&report), vec!["unit_not_normalized"]);
    let ProvenanceViolation::UnitNotNormalized { unit, .. } = &report.violations[0] else {
        panic!("expected a unit refusal");
    };
    assert_eq!(unit, "furlongs per barn", "the original spelling survives");
}

#[test]
fn a_quantity_field_that_arrives_without_a_unit_cannot_be_promoted() {
    let report = lint(|_, _, candidates, _, _| {
        candidates[0].fields.get_mut("density").unwrap().unit = None;
    });
    assert_eq!(violations(&report), vec!["unit_not_normalized"]);
}

#[test]
fn a_unit_measuring_the_wrong_quantity_cannot_be_promoted() {
    let report = lint(|_, _, candidates, _, _| {
        candidates[0].fields.get_mut("density").unwrap().unit = Some("kJ/mol".into());
    });
    assert_eq!(violations(&report), vec!["unit_not_normalized"]);
}

#[test]
fn a_field_with_no_runtime_target_is_not_the_lints_business() {
    let report = lint(|_, _, candidates, _, _| {
        candidates[0].fields.insert(
            "depositor_comment".into(),
            CandidateField::new(json!("looks blue to me"), "", "LicenseRef-Restricted"),
        );
    });
    assert!(!report.refuses(), "{:?}", report.violations);
    assert_eq!(report.checked_fields, 2);
}

#[test]
fn the_report_is_deterministic_for_one_input() {
    let first = lint(|_, _, candidates, _, _| {
        candidates[0].fields.get_mut("density").unwrap().licence = "GPL-3.0-only".into();
    });
    let second = lint(|_, _, candidates, _, _| {
        candidates[0].fields.get_mut("density").unwrap().licence = "GPL-3.0-only".into();
    });
    assert_eq!(first, second);
    assert_eq!(
        serde_json::to_string(&first).unwrap(),
        serde_json::to_string(&second).unwrap()
    );
}
