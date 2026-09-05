//! DATA-010's drift pin: the runtime document→species join
//! (`species_loader::parse_document`) must produce, from the SAME source
//! document this build compiled, exactly the species `build.rs`
//! generated into `REGISTRY` — every field, spectra included since
//! DATA-011 made them data.
//!
//! If build.rs and the loader ever source a field differently, this
//! fails naming the species and field.

use kerotakis_core::species::{registry, SpeciesData};

fn source_document() -> serde_json::Value {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../data/registry/registry-source-v1.json"
    );
    serde_json::from_str(&std::fs::read_to_string(path).expect("registry source readable"))
        .expect("registry source parses")
}

fn assert_species_eq(key: &str, built: &SpeciesData, loaded: &SpeciesData) {
    macro_rules! eq {
        ($field:ident) => {
            assert_eq!(
                built.$field,
                loaded.$field,
                "{key}: field '{}' differs between build.rs and the runtime loader",
                stringify!($field)
            );
        };
    }
    eq!(key);
    eq!(name);
    eq!(formula);
    eq!(inchikey);
    eq!(molar_mass);
    eq!(heat_capacity);
    eq!(density);
    eq!(standard_phase);
    eq!(appearance);
    eq!(flame_colour);
    eq!(dissolution_enthalpy_kj);
    eq!(dissolves_without_speciation);
    eq!(aqueous_solubility_g_per_100_ml);
    eq!(forms_only_above_k);
    eq!(magnetic);
    eq!(electrical_resistivity);
    eq!(provenance);
    match (&built.colour, &loaded.colour) {
        (None, None) => {}
        (Some(a), Some(b)) => {
            assert_eq!((a.r, a.g, a.b), (b.r, b.g, b.b), "{key}: colour differs");
            assert!(
                (a.strength - b.strength).abs() < 1e-12,
                "{key}: tint strength differs"
            );
        }
        _ => panic!("{key}: colour presence differs"),
    }
    // DATA-011: spectra are data now — the drift pin covers them too.
    match (built.spectrum, loaded.spectrum) {
        (None, None) => {}
        (Some(a), Some(b)) => {
            for (i, (x, y)) in a.iter().zip(b.iter()).enumerate() {
                assert!(
                    (x - y).abs() < 1e-12,
                    "{key}: spectrum band {i} differs ({x} vs {y})"
                );
            }
        }
        _ => panic!("{key}: spectrum presence differs between build.rs and the loader"),
    }
}

#[test]
fn the_runtime_loader_reproduces_the_compiled_registry() {
    let loaded =
        kerotakis_core::species_loader::parse_document(&source_document()).expect("loader parses");
    assert_eq!(
        loaded.len(),
        registry().len(),
        "loader and build.rs disagree on the species count"
    );
    for l in &loaded {
        let built = kerotakis_core::species::lookup_key(l.key)
            .unwrap_or_else(|| panic!("loader produced '{}' which REGISTRY lacks", l.key));
        assert_species_eq(l.key, built, l);
    }
}

#[test]
fn registered_loaded_species_resolve_and_never_shadow_builtins() {
    let mut doc = source_document();
    // Craft one novel species by cloning water's records under a new key
    // — and one collision (water itself) that must be skipped.
    let clone_as = |doc: &mut serde_json::Value, from: &str, to: &str| {
        for section in [
            "identities",
            "compositions",
            "phase_thermodynamics",
            "optical",
            "model_parameters",
        ] {
            let arr = doc[section].as_array().unwrap().clone();
            let mut extra: Vec<serde_json::Value> = Vec::new();
            for rec in &arr {
                let matches = rec["species_id"] == from
                    || (section == "identities" && rec["id"] == from)
                    || (rec["subject"]["id"] == from && rec["subject"]["kind"] == "species");
                if matches {
                    let mut c = rec.clone();
                    if section == "identities" {
                        c["id"] = to.into();
                        c["name"] = format!("test double of {from}").into();
                    }
                    if c.get("species_id").is_some() {
                        c["species_id"] = to.into();
                    }
                    if c["subject"]["id"] == from {
                        c["subject"]["id"] = to.into();
                    }
                    extra.push(c);
                }
            }
            doc[section].as_array_mut().unwrap().extend(extra);
        }
    };
    clone_as(&mut doc, "water", "test_pack_water");

    let loaded = kerotakis_core::species_loader::parse_document(&doc).expect("parses");
    let (added, skipped) = kerotakis_core::species::register_loaded(loaded);
    // Everything already built in is skipped; only the novel key lands.
    assert_eq!(added, 1, "exactly the novel species should register");
    assert!(skipped >= registry().len(), "built-ins must be skipped");

    let got = kerotakis_core::species::lookup_key("test_pack_water")
        .expect("novel species resolves after registration");
    assert_eq!(got.name, "test double of water");
    // The built-in was not shadowed.
    let water = kerotakis_core::species::lookup_key("water").unwrap();
    assert_eq!(water.name, "water");
    // And the shelf inventory includes it.
    assert!(kerotakis_core::species::all_species()
        .iter()
        .any(|s| s.key == "test_pack_water"));
}
