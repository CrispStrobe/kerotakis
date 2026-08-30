use kerotakis_core::{
    element_coverage_json, element_coverage_report, element_coverage_report_with_lessons,
    element_coverage_report_with_routes, ElementCapability, InstalledLessonRoute,
    InstalledRunnableRoute, ShelfItemKind, ELEMENT_SYMBOLS,
};

#[test]
fn report_has_all_identities_and_real_shelf_provenance() {
    let report = element_coverage_report().expect("valid generated coverage");
    assert_eq!(report.elements.len(), 118);
    assert_eq!(
        report
            .elements
            .iter()
            .map(|entry| entry.symbol.as_str())
            .collect::<Vec<_>>(),
        ELEMENT_SYMBOLS
    );
    assert!(report
        .elements
        .iter()
        .any(|entry| entry.capability == ElementCapability::IdentityOnly));

    let oxygen = report
        .elements
        .iter()
        .find(|entry| entry.symbol == "O")
        .expect("oxygen identity");
    assert!(oxygen.examples.iter().any(|item| item.shelf_key == "water"));
    let peroxide = oxygen
        .examples
        .iter()
        .find(|item| item.shelf_key == "hydrogen_peroxide_3_percent")
        .expect("expanded household recipe reaches oxygen");
    assert_eq!(peroxide.kind, ShelfItemKind::MaterialRecipe);
    assert!(peroxide
        .formula_species_keys
        .iter()
        .any(|key| key == "H2O2"));

    for entry in &report.elements {
        for item in &entry.examples {
            assert!(
                kerotakis_core::species::lookup_key(&item.shelf_key).is_some()
                    || kerotakis_core::material::lookup(&item.shelf_key, None).is_some(),
                "{} points at missing shelf key {}",
                entry.symbol,
                item.shelf_key
            );
        }
        for route in &entry.routes {
            for key in &route.required_shelf_keys {
                assert!(
                    kerotakis_core::species::lookup_key(key).is_some()
                        || kerotakis_core::material::lookup(key, None).is_some(),
                    "route {} points at missing shelf key {key}",
                    route.key
                );
            }
        }
    }
}

#[test]
fn report_and_portable_json_are_deterministic() {
    let first = element_coverage_json().expect("coverage JSON");
    let second = element_coverage_json().expect("coverage JSON replay");
    assert_eq!(first, second);
    let decoded: serde_json::Value = serde_json::from_str(&first).expect("portable JSON");
    assert_eq!(decoded["schema"], 1);
    assert_eq!(decoded["elements"].as_array().unwrap().len(), 118);
}

#[test]
fn lesson_routes_are_validated_and_raise_capability() {
    let lesson = InstalledLessonRoute {
        key: "lesson/fizz".into(),
        label: "Familiar carbonate fizz".into(),
        required_shelf_keys: vec!["NaHCO3".into(), "CH3COOH".into()],
    };
    let report = element_coverage_report_with_lessons(&[lesson]).expect("installed lesson");
    assert!(report.elements.iter().any(|entry| {
        entry.capability == ElementCapability::LessonBacked
            && entry.routes.iter().any(|route| route.key == "lesson/fizz")
    }));

    let bad = InstalledLessonRoute {
        key: "lesson/missing".into(),
        label: "Cannot run".into(),
        required_shelf_keys: vec!["not-on-the-shelf".into()],
    };
    assert!(element_coverage_report_with_lessons(&[bad]).is_err());

    let reaction = InstalledRunnableRoute {
        key: "reaction/fizz".into(),
        label: "Carbonate fizz".into(),
        required_shelf_keys: vec!["NaHCO3".into(), "CH3COOH".into()],
        lesson: false,
    };
    let report = element_coverage_report_with_routes(&[reaction]).expect("replay-proved route");
    assert!(report
        .elements
        .iter()
        .any(|entry| entry.capability == ElementCapability::Reacting));
}

#[test]
fn regression_summary_matches_reviewed_fixture() {
    let report = element_coverage_report().expect("coverage");
    let selected = ["H", "O", "Na", "Mn", "Fe", "Cu", "Zn", "Ag", "Pb"]
        .into_iter()
        .map(|symbol| {
            let entry = report
                .elements
                .iter()
                .find(|entry| entry.symbol == symbol)
                .unwrap();
            (symbol.to_string(), serde_json::json!(entry.examples.len()))
        })
        .collect::<serde_json::Map<_, _>>();
    let summary = serde_json::json!({
        "schema": report.schema,
        "elements": report.elements.len(),
        "identity_only": report.elements.iter().filter(|entry| entry.capability == ElementCapability::IdentityOnly).count(),
        "property_backed": report.elements.iter().filter(|entry| entry.capability == ElementCapability::PropertyBacked).count(),
        "selected_example_counts": selected,
    });
    let actual = serde_json::to_value(summary).unwrap();
    let expected: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/element-coverage-v1.json")).unwrap();
    assert_eq!(actual, expected);
}
