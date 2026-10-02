use kerotakis_data::*;
use std::collections::BTreeMap;

#[test]
fn reviewed_olive_density_promotion_keeps_units_reference_and_missing_uncertainty() {
    let raw = include_bytes!("../../../provenance/odeh-2015-olive-oil-transcription.json");
    let manifest: SnapshotManifest = serde_json::from_str(include_str!(
        "../../../provenance/odeh-2015-olive-oil-manifest.json"
    ))
    .unwrap();
    let transcription: serde_json::Value = serde_json::from_slice(raw).unwrap();
    let licence = "LicenseRef-Odeh-2015-Attribution-Grant";
    let candidate = QuarantinedCandidate {
        adapter_id: manifest.adapter_id.clone(),
        source_record_id: "Table1/S1a".into(),
        external_record_id: "olive_oil".into(),
        identity_key: None,
        fields: BTreeMap::from([(
            "mass_density".into(),
            CandidateField::new(
                transcription["mass_density"].clone(),
                "Table1.S1a.mass_density",
                licence,
            )
            .with_unit("g/cm3"),
        )]),
    };
    let policy = PromotionPolicy {
        fields: BTreeMap::from([(
            "mass_density".into(),
            RuntimeFieldPolicy::new("bulk_density", [licence])
                .with_dimension(Dimension::MassDensity),
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
            external_record_id: "olive_oil".into(),
            fields: vec!["mass_density".into()],
        }],
    });
    assert!(!report.refuses(), "{report:?}");
    let promoted = review_candidate(&candidate, &policy);
    assert!(promoted.rejected.is_empty());
    assert_eq!(
        promoted.accepted["bulk_density"].value.as_f64().unwrap(),
        0.9161
    );
    let registry: RegistryDocument = serde_json::from_str(include_str!(
        "../../../data/registry/registry-source-v1.json"
    ))
    .unwrap();
    let rows: Vec<_> = registry
        .material_recipes
        .iter()
        .filter(|recipe| {
            recipe
                .bulk_density
                .as_ref()
                .is_some_and(|datum| datum.source_id == manifest.source_id)
        })
        .collect();
    assert_eq!(rows.len(), 1, "the grant approves one density scalar only");
    let row = rows[0];
    assert_eq!(row.canonical_key, "olive_oil");
    let datum = row.bulk_density.as_ref().unwrap();
    assert_eq!(datum.value, 0.9161);
    assert_eq!(datum.uncertainty, Uncertainty::NotReported);
    assert_eq!(datum.unit.dimension, Dimension::MassDensity);
    assert_eq!(
        datum.conditions.temperature.as_ref().unwrap().lower,
        transcription["temperature_C"].as_f64().unwrap() + 273.15
    );
    assert_eq!(row.unresolved_fraction.unwrap().lower, 1.0);
    assert!(row.components.is_empty());
    assert_eq!(row.confidence, MaterialConfidence::Surrogate);
}
