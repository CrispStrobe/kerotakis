//! Constructed label/identity contracts, not aqueous or optical calibration.
use kerotakis_core::*;
fn term(section: &str, key: &str, locale: Locale) -> String {
    Phrase::new(
        "test.optical-species-label",
        "{value}",
        vec![("value".into(), Slot::term(section, key))],
    )
    .render(locale)
}
fn water() -> Vessel {
    let mut v = Vessel::new(VesselId(0), "labels");
    v.deposit(SpeciesId::new("water"), Moles(5.55), Phase::Liquid);
    v
}
fn indicator(id: &str) -> Vessel {
    let mut v = water();
    v.deposit(SpeciesId::new(id), Moles(1e-6), Phase::Aqueous);
    v
}
fn labels() -> [(&'static str, [&'static str; 3]); 2] {
    [
        (
            "methyl_orange",
            ["methyl orange", "Methylorange", "orangé de méthyle"],
        ),
        (
            "bromothymol_blue",
            ["bromothymol blue", "Bromthymolblau", "bleu de bromothymol"],
        ),
    ]
}
fn locales() -> [Locale; 3] {
    [Locale::EN, Locale::parse("de"), Locale::parse("fr")]
}
#[test]
fn canonical_species_terms_have_declared_localized_labels() {
    for (id, expected) in labels() {
        for (locale, label) in locales().into_iter().zip(expected) {
            assert_eq!(term("species", id, locale), label);
        }
    }
}
#[test]
fn direct_english_species_terms_keep_existing_translations() {
    for (locale, expected) in
        locales()
            .into_iter()
            .zip(["methyl orange", "Methylorange", "orangé de méthyle"])
    {
        assert_eq!(term("species", "methyl orange", locale), expected);
    }
}
#[test]
fn unknown_and_native_complex_terms_keep_exact_identity() {
    for key in ["Cu(NH3)4+2", "unknown_species_label"] {
        for locale in locales() {
            assert_eq!(term("species", key, locale), key);
        }
    }
}
#[test]
fn non_species_sections_do_not_resolve_registry_ids() {
    for locale in locales() {
        assert_eq!(
            term("not-a-species-section", "methyl_orange", locale),
            "methyl_orange"
        );
    }
}
#[test]
fn actual_appearance_uses_localized_species_without_changing_slots() {
    for (id, expected) in labels() {
        let v = indicator(id);
        let before = serde_json::to_value(&v).unwrap();
        let seen = appearance::observe(&v);
        let note = seen
            .notes
            .iter()
            .find(|p| p.key == "look.indicator-optics-missing-ph")
            .unwrap();
        let serialized = serde_json::to_value(note).unwrap();
        assert!(serialized.to_string().contains(id));
        for (locale, label) in locales().into_iter().zip(expected) {
            let text = note.render(locale);
            assert!(text.contains(label), "{text}");
            assert!(!text.contains(id), "{text}");
        }
        assert_eq!(serde_json::to_value(note).unwrap(), serialized);
        assert_eq!(serde_json::to_value(&v).unwrap(), before);
    }
}
#[test]
fn actual_bench_refusal_uses_localized_species_without_mutation() {
    for (id, expected) in labels() {
        let mut b = Bench::new();
        b.vessels[0] = indicator(id);
        let before = serde_json::to_value(&b.vessels[0]).unwrap();
        let events = b
            .step_with(
                Operator::Measure {
                    vessel: VesselId(0),
                    instrument: Instrument::Spectrophotometer,
                },
                &mut SolverStack::new(vec![]),
                &PermissiveScreen,
            )
            .unwrap();
        assert!(!events.iter().any(|e| matches!(e, Event::Measured { .. })));
        let reason = events
            .iter()
            .find_map(|e| match e {
                Event::NotYetModeled {
                    reason: Some(p), ..
                } if p.key == "not-modeled.indicator-optics-missing-ph" => Some(p),
                _ => None,
            })
            .unwrap();
        assert!(serde_json::to_value(reason)
            .unwrap()
            .to_string()
            .contains(id));
        for (locale, label) in locales().into_iter().zip(expected) {
            let text = reason.render(locale);
            assert!(text.contains(label), "{text}");
            assert!(!text.contains(id), "{text}");
        }
        assert_eq!(serde_json::to_value(&b.vessels[0]).unwrap(), before);
    }
}
#[test]
fn scene_names_known_ph_gap_ids_without_rewriting_them() {
    for (id, english) in [
        ("methyl_orange", "methyl orange"),
        ("bromothymol_blue", "bromothymol blue"),
    ] {
        let v = indicator(id);
        let before = serde_json::to_value(&v).unwrap();
        let scene = serde_json::to_value(scene_vessel(&v)).unwrap();
        assert_eq!(scene["liquid"]["optical_ph_gaps"], serde_json::json!([id]));
        assert_eq!(scene["liquid"]["optical_species_names"][id], english);
        assert_eq!(
            scene["liquid"]["optical_species_names"]
                .as_object()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(serde_json::to_value(&v).unwrap(), before);
    }
}
#[test]
fn scene_name_map_covers_both_gap_kinds_and_omits_unknown_native_ids() {
    let mut v = indicator("methyl_orange");
    v.solution = Some(SolutionInfo {
        scope: Default::default(),
        solvent_kg: Some(0.1),
        particle_molality: None,
        pe: None,
        redox: Vec::new(),
        ph: f64::NAN,
        ionic_strength: 0.0,
        species: vec![
            SpeciesDetail {
                name: "CuSO4".into(),
                molality: 0.001,
                activity: 0.001,
            },
            SpeciesDetail {
                name: "Cu(NH3)4+2".into(),
                molality: 0.001,
                activity: 0.001,
            },
        ],
        provenance: None,
        solvent_activity: None,
    });
    let before = serde_json::to_value(&v).unwrap();
    let s = serde_json::to_value(scene_vessel(&v)).unwrap();
    let liquid = &s["liquid"];
    assert_eq!(
        liquid["optical_ph_gaps"],
        serde_json::json!(["methyl_orange"])
    );
    let spectral = liquid["spectral_gaps"].as_array().unwrap();
    assert!(spectral.contains(&serde_json::json!("CuSO4")));
    assert!(spectral.contains(&serde_json::json!("Cu(NH3)4+2")));
    assert_eq!(liquid["optical_species_names"]["CuSO4"], "copper sulfate");
    assert_eq!(
        liquid["optical_species_names"]["methyl_orange"],
        "methyl orange"
    );
    assert!(liquid["optical_species_names"].get("Cu(NH3)4+2").is_none());
    assert_eq!(
        liquid["optical_species_names"].as_object().unwrap().len(),
        2
    );
    assert_eq!(serde_json::to_value(&v).unwrap(), before);
}
#[test]
fn blank_and_older_scene_payloads_need_no_name_map() {
    let mut value = serde_json::to_value(scene_vessel(&water())).unwrap();
    assert!(value["liquid"].get("optical_species_names").is_none());
    value["liquid"]
        .as_object_mut()
        .unwrap()
        .remove("optical_species_names");
    let old: kerotakis_core::scene::SceneVessel = serde_json::from_value(value).unwrap();
    assert!(serde_json::to_value(old).unwrap()["liquid"]
        .get("optical_species_names")
        .is_none());
}
