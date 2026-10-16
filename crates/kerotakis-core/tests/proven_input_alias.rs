//! Exact reviewed input identities; no new substance, phase or chemical model.
use kerotakis_core::script::{parse_command, parse_op};
use kerotakis_core::*;
use serde_json::{json, Value};

const ALIASES: &[(&str, &str)] = &[("CuSO4_anhydrous", "CuSO4"), ("acetate", "CH3COO-")];

fn add(key: &str, moles: f64) -> Operator {
    Operator::Add {
        vessel: VesselId(0),
        species: SpeciesId(key.into()),
        moles: Moles(moles),
        at: None,
    }
}

#[test]
fn reviewed_aliases_have_canonical_parser_direct_and_json_identity() {
    for &(alias, key) in ALIASES {
        assert_eq!(species::lookup_key(alias).unwrap().key, key);
        assert_eq!(SpeciesId::new(alias), SpeciesId::new(key));
        let canonical = parse_op(&format!("add v1 {key} 0.001mol"))
            .unwrap()
            .unwrap();
        let parsed = parse_op(&format!("add v1 {alias} 0.001mol"))
            .unwrap()
            .unwrap();
        let direct = add(alias, 0.001);
        let decoded: Operator =
            serde_json::from_value(json!({"op":"add", "vessel":0, "species":alias, "moles":0.001}))
                .unwrap();
        assert_eq!(parsed, canonical);
        assert_eq!(decoded, canonical);
        assert_eq!(
            serde_json::to_value(direct).unwrap(),
            serde_json::to_value(&canonical).unwrap()
        );
        for locale in [Locale::EN, Locale::parse("de"), Locale::parse("fr")] {
            let command = parse_command(&format!("add v1 {alias} 0.001mol"), locale).unwrap();
            assert_eq!(command.canonical, format!("add v1 {key} 0.001mol"));
            assert_eq!(command.operator, Some(canonical.clone()));
        }
    }
}

#[test]
fn mixed_spellings_draw_one_finite_molar_bottle_and_refuse_overdraw() {
    for compensated in [false, true] {
        for &(alias, key) in ALIASES {
            let mut named = Bench::new();
            if compensated {
                named.stock = StockLedger::compensated();
            }
            // Species stock is explicitly in moles; no density/unit assumption.
            named.stock.stock(alias, 1.0, StockUnit::Mole);
            let mut canonical = named.clone();
            let operations = [
                add(alias, 0.25),
                serde_json::from_value(json!({"op":"add", "vessel":0,
                    "species":alias, "moles":0.25}))
                .unwrap(),
                parse_op(&format!("add v1 {alias} 0.25mol"))
                    .unwrap()
                    .unwrap(),
            ];
            for operation in operations {
                let events = named
                    .step_with(operation, &mut MixingEquilibrator, &PermissiveScreen)
                    .unwrap();
                let expected = canonical
                    .step_with(add(key, 0.25), &mut MixingEquilibrator, &PermissiveScreen)
                    .unwrap();
                assert_eq!(events, expected);
                assert_eq!(
                    serde_json::to_value(&named).unwrap(),
                    serde_json::to_value(&canonical).unwrap()
                );
            }
            assert_eq!(named.stock.entries().count(), 1);
            assert_eq!(named.stock.remaining(alias).unwrap().amount, 0.25);
            assert_eq!(named.stock.remaining(key).unwrap().amount, 0.25);
            let before_vessels = serde_json::to_value(&named.vessels).unwrap();
            let before_stock = named.stock.clone();
            let events = named
                .step_with(add(alias, 0.5), &mut MixingEquilibrator, &PermissiveScreen)
                .unwrap();
            assert!(events.iter().any(|event| matches!(event,
                Event::StockExhausted { key: actual, .. } if actual == key)));
            assert_eq!(
                serde_json::to_value(&named.vessels).unwrap(),
                before_vessels
            );
            assert_eq!(named.stock, before_stock);
            let restored: Bench =
                serde_json::from_value(serde_json::to_value(&named).unwrap()).unwrap();
            assert_eq!(
                serde_json::to_value(restored).unwrap(),
                serde_json::to_value(named).unwrap()
            );
        }
    }
}

fn renamed_bottle(mut saved: Value, alias: &str, key: &str, collision: bool) -> Value {
    let bottles = if saved.get("schema").is_some() {
        saved["bottles"].as_object_mut().unwrap()
    } else {
        saved.as_object_mut().unwrap()
    };
    let amount = bottles.remove(key).unwrap();
    if collision {
        bottles.insert(key.into(), amount.clone());
    }
    bottles.insert(alias.into(), amount);
    saved
}

#[test]
fn restored_alias_bottles_preserve_exact_amounts_and_reject_duplicate_identity() {
    for compensated in [false, true] {
        for &(alias, key) in ALIASES {
            let mut stock = if compensated {
                StockLedger::compensated()
            } else {
                StockLedger::default()
            };
            stock.stock(key, 1.0, StockUnit::Mole);
            if compensated {
                stock.draw(key, 2.0_f64.powi(-60)).unwrap();
            }
            let saved = serde_json::to_value(&stock).unwrap();
            let mut restored: StockLedger =
                serde_json::from_value(renamed_bottle(saved.clone(), alias, key, false)).unwrap();
            assert_eq!(
                restored.remaining_exact(alias).unwrap().parts(),
                stock.remaining_exact(key).unwrap().parts()
            );
            restored.draw(alias, 0.25).unwrap();
            stock.draw(key, 0.25).unwrap();
            assert_eq!(restored, stock);
            assert!(
                serde_json::from_value::<StockLedger>(renamed_bottle(saved, alias, key, true))
                    .is_err()
            );
        }
    }
}

#[test]
fn legacy_vessel_alias_restores_canonically_and_merges_only_the_same_identity() {
    for &(alias, key) in ALIASES {
        let phase = species::lookup_key(key).unwrap().standard_phase;
        let mut vessel = Vessel::new(VesselId(0), "reviewed alias receiver");
        vessel.deposit(SpeciesId(alias.into()), Moles(0.25), phase);
        vessel.deposit(SpeciesId::new(key), Moles(0.5), phase);
        assert_eq!(vessel.contents.len(), 1);
        assert_eq!(vessel.moles_of(&SpeciesId(alias.into())).0, 0.75);
        let mut saved = serde_json::to_value(&vessel).unwrap();
        saved["contents"][0]["species"] = json!(alias);
        let restored: Vessel = serde_json::from_value(saved).unwrap();
        assert_eq!(restored.contents[0].species.0, key);
        assert_eq!(restored.contents[0].phase, phase);
        assert_eq!(restored.moles_of(&SpeciesId(alias.into())).0, 0.75);
    }
}

#[test]
fn charge_hydrate_acid_and_counterion_distinctions_are_preserved() {
    let dry = species::lookup_key("CuSO4_anhydrous").unwrap();
    let hydrate = species::lookup_key("chalcanthite").unwrap();
    assert_eq!(dry.formula, "CuSO4");
    assert_eq!(dry.standard_phase, Phase::Solid);
    assert_eq!(hydrate.formula, "CuSO4·5H2O");
    assert_ne!(dry.key, hydrate.key);
    assert_ne!(dry.molar_mass, hydrate.molar_mass);
    let acetate = species::lookup_key("acetate").unwrap();
    let formula = stoich::parse_formula(acetate.formula).unwrap();
    assert_eq!(formula.charge, -1.0);
    assert_eq!(formula.counts.get("C"), Some(&2.0));
    assert_eq!(formula.counts.get("H"), Some(&3.0));
    assert_eq!(formula.counts.get("O"), Some(&2.0));
    assert_eq!(acetate.standard_phase, Phase::Aqueous);
    for distinct in ["CH3COOH", "NaOAc", "ethyl_acetate"] {
        assert_ne!(SpeciesId::new("acetate"), SpeciesId::new(distinct));
        assert_ne!(
            acetate.formula,
            species::lookup_key(distinct).unwrap().formula
        );
    }
    assert!(species::lookup_key("Na").is_none());
    assert!(parse_op("add v1 Na 0.001mol").is_err());
    assert_ne!(SpeciesId::new("Na"), SpeciesId::new("Na+"));
    assert_eq!(
        stoich::parse_formula(species::lookup_key("Na+").unwrap().formula)
            .unwrap()
            .charge,
        1.0
    );
}

#[test]
fn loaded_species_and_materials_cannot_shadow_the_reviewed_aliases() {
    for &(alias, _) in ALIASES {
        let mut data = species::lookup_key("NaCl").unwrap().clone();
        data.key = alias;
        assert_eq!(species::register_loaded(vec![data]), (0, 1));
        let mut recipe = material::lookup("whole_milk", None).unwrap();
        recipe.id = "test/proven-input-alias-collision".into();
        recipe.canonical_key = "proven_input_alias_collision".into();
        recipe.aliases.clear();
        recipe.aliases.insert("en".into(), vec![alias.into()]);
        assert_eq!(material::register_loaded(vec![recipe]), (0, 1));
    }
}
