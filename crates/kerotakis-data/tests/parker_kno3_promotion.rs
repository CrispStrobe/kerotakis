//! One reviewed government-report datum, with no global licence relaxation.
use kerotakis_data::*;
use serde_json::json;
use std::collections::BTreeMap;
#[test]
fn parker_kno3_promotion_pins_source_units_and_runtime_value() {
    let raw = include_bytes!("../../../provenance/parker-1965-kno3-transcription.json");
    let manifest: SnapshotManifest = serde_json::from_str(include_str!(
        "../../../provenance/parker-1965-kno3-manifest.json"
    ))
    .unwrap();
    let transcription: serde_json::Value = serde_json::from_slice(raw).unwrap();
    let licence = "LicenseRef-US-Public-Domain";
    let candidate = QuarantinedCandidate {
        adapter_id: manifest.adapter_id.clone(),
        source_record_id: "NSRDS-NBS 2/p30/KNO3".into(),
        external_record_id: "KNO3".into(),
        identity_key: None,
        fields: BTreeMap::from([(
            "heat_of_solution".into(),
            CandidateField::new(
                transcription["heat_of_solution"].clone(),
                "p30.KNO3.best_value",
                licence,
            )
            .with_unit("cal/mol"),
        )]),
    };
    let policy = PromotionPolicy {
        fields: BTreeMap::from([(
            "heat_of_solution".into(),
            RuntimeFieldPolicy::new("enthalpy_of_dissolution", [licence])
                .with_dimension(Dimension::MolarEnergy),
        )]),
    };
    let allowed = [licence.to_owned()].into_iter().collect();
    let report = lint_promotion(&PromotionLintInput {
        manifest: &manifest,
        raw_snapshot: raw,
        candidates: std::slice::from_ref(&candidate),
        policy: &policy,
        allowed_runtime_licences: &allowed,
        eligible_fields: &[EligibleFieldList {
            adapter_id: manifest.adapter_id.clone(),
            external_record_id: "KNO3".into(),
            fields: vec!["heat_of_solution".into()],
        }],
    });
    assert!(!report.refuses(), "{report:?}");
    let reviewed = review_candidate(&candidate, &policy);
    assert!(reviewed.rejected.is_empty());
    let promoted = reviewed.accepted["enthalpy_of_dissolution"]
        .value
        .as_f64()
        .unwrap();
    assert!((promoted - 34.89456).abs() < 1e-10);
    let registry: serde_json::Value = serde_json::from_str(include_str!(
        "../../../data/registry/registry-source-v1.json"
    ))
    .unwrap();
    let row = registry["phase_thermodynamics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| {
            r["species_id"] == json!("KNO3") && r["property"] == json!("enthalpy_of_dissolution")
        })
        .unwrap();
    assert_eq!(row["quantity"]["source_id"], json!(manifest.source_id));
    assert_eq!(row["quantity"]["value"], json!(promoted));
}
