//! Registry-derived catalogue coverage, including every serializable gate kind.
use kerotakis_core::family::*;

#[test]
fn shipped_family_terms_and_all_gate_labels_have_catalogue_entries() {
    let records = load_records(include_str!("../../../data/families/families-v1.toml")).unwrap();
    // Deliberately exhaustive: adding a GateSet field requires updating this
    // inventory, rather than silently giving a new gate an untranslated label.
    let all = GateSet {
        medium: Some(MediumGate::Any),
        temperature_k: Some(KelvinWindow {
            min: Some(300.0),
            max: None,
        }),
        ph: Some(PhWindow {
            min: Some(3.0),
            max: None,
        }),
        catalyst: Some(CatalystGate::Species { key: "H+".into() }),
        light: Some(LightGate {
            max_wavelength_nm: 400.0,
        }),
        required_groups: vec!["ester".into()],
        forbidden_groups: vec!["alcohol".into()],
    };
    let gate_fields = serde_json::to_value(all).unwrap();
    for catalogue in [
        include_str!("../i18n/de.toml"),
        include_str!("../i18n/fr.toml"),
    ] {
        let values: toml::Value = toml::from_str(catalogue).unwrap();
        let has = |section: &str, key: &str| {
            assert!(
                values
                    .get(section)
                    .and_then(|s| s.get(key))
                    .and_then(|v| v.as_str())
                    .is_some_and(|v| !v.trim().is_empty()),
                "missing {section}.{key}"
            );
        };
        for gate in gate_fields.as_object().unwrap().keys() {
            has("family-gate-label", gate);
        }
        for record in &records {
            for group in record
                .gates
                .required_groups
                .iter()
                .chain(&record.gates.forbidden_groups)
            {
                has("family-group", group);
            }
            if let Some(catalyst) = &record.gates.catalyst {
                match catalyst {
                    CatalystGate::Species { key } => has("family-species", key),
                    CatalystGate::AnyOf { keys } => {
                        for key in keys {
                            has("family-species", key);
                        }
                    }
                }
            }
        }
    }
}
