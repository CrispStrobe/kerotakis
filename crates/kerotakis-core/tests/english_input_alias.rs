//! Reviewed names are input spellings of existing identities, not new chemistry.
use kerotakis_core::script::{parse_command, parse_op};
use kerotakis_core::*;
use serde_json::{json, Value};

const NAMES: &[(&str, &str)] = &[
    ("oxygen", "O2"),
    ("nitrogen", "N2"),
    ("carbon dioxide", "CO2"),
    ("sodium chloride", "NaCl"),
];

fn add(key: &str, moles: f64) -> Operator {
    Operator::Add {
        vessel: VesselId(0),
        species: SpeciesId(key.into()),
        moles: Moles(moles),
        at: None,
    }
}

#[test]
fn reviewed_names_have_canonical_operators_and_unit_conversions() {
    for &(name, key) in NAMES {
        assert_eq!(species::lookup_key(name).unwrap().key, key);
        assert_eq!(SpeciesId::new(name), SpeciesId::new(key));
        for amount in ["1mol", "1g", "1mL"] {
            let canonical = parse_op(&format!("add v1 {key} {amount}")).unwrap();
            assert_eq!(
                parse_op(&format!("add v1 {name} {amount}")).unwrap(),
                canonical
            );
            assert_eq!(
                parse_op(&format!("stock {name} {amount}")).unwrap(),
                parse_op(&format!("stock {key} {amount}")).unwrap()
            );
        }
        for (named, canonical) in [
            (
                format!("grind v1 {name} 50um"),
                format!("grind v1 {key} 50um"),
            ),
            (
                format!("extract v1 v2 {name} 0.1mol"),
                format!("extract v1 v2 {key} 0.1mol"),
            ),
            (
                format!("titrate v1 {name} 1M 1mL until ph 7 max 1"),
                format!("titrate v1 {key} 1M 1mL until ph 7 max 1"),
            ),
        ] {
            // Parser identity parity does not claim support for using a gas as a titrant/solvent.
            assert_eq!(parse_op(&named).unwrap(), parse_op(&canonical).unwrap());
        }
    }
}

#[test]
fn english_names_log_canonical_text_in_every_shipped_locale() {
    for locale in [Locale::EN, Locale::parse("de"), Locale::parse("fr")] {
        for &(name, key) in NAMES {
            let command = parse_command(&format!("add v1 {name} 1mol"), locale).unwrap();
            assert_eq!(command.canonical, format!("add v1 {key} 1mol"));
            assert_eq!(
                command.operator,
                parse_op(&format!("add v1 {key} 1mol")).unwrap()
            );
        }
    }
    let de = parse_command("zugeben v1 Natriumchlorid 1g", Locale::parse("de")).unwrap();
    assert_eq!(de.operator, parse_op("add v1 NaCl 1g").unwrap());
}

#[test]
fn aliases_do_not_rewrite_other_command_arguments_or_infer_formulas() {
    for line in [
        "inspect sodium chloride",
        "help carbon dioxide",
        "# add v1 oxygen 1mol",
    ] {
        let command = parse_command(line, Locale::EN).unwrap();
        assert_eq!(command.canonical, line);
        assert!(command.operator.is_none());
    }
    for line in [
        "add oxygen NaCl 1mol",
        "add carbon dioxide v1 1mol",
        "look carbon dioxide",
    ] {
        assert!(parse_op(line).is_err());
    }
    for name in [
        "Oxygen",
        "o2",
        "carbon-dioxide",
        "sodium_chloride",
        "unknown oxygen",
    ] {
        assert!(parse_op(&format!("add v1 {name} 1mol")).is_err(), "{name}");
    }
    assert_ne!(SpeciesId::new("CO2"), SpeciesId::new("dry_ice"));
    assert_ne!(SpeciesId::new("CO2"), SpeciesId::new("CO2(aq)"));
    assert_ne!(SpeciesId::new("N2"), SpeciesId::new("liquid_nitrogen"));
    assert_ne!(SpeciesId::new("glucose"), SpeciesId::new("fructose"));
}

#[test]
fn direct_json_and_text_names_draw_from_one_finite_bottle() {
    for compensated in [false, true] {
        for &(name, key) in NAMES {
            let mut named = Bench::new();
            if compensated {
                named.stock = StockLedger::compensated();
            }
            let mut canonical = named.clone();
            named
                .step(Operator::StockShelf {
                    key: name.into(),
                    amount: 1.0,
                })
                .unwrap();
            canonical
                .step(Operator::StockShelf {
                    key: key.into(),
                    amount: 1.0,
                })
                .unwrap();
            let operations = [
                add(name, 0.25),
                serde_json::from_value(
                    json!({"op":"add", "vessel":0, "species":name, "moles":0.25}),
                )
                .unwrap(),
                parse_op(&format!("add v1 {name} 0.75mol"))
                    .unwrap()
                    .unwrap(),
            ];
            for (i, operation) in operations.into_iter().enumerate() {
                let events = named.step(operation).unwrap();
                let expected = canonical
                    .step(add(key, if i == 2 { 0.75 } else { 0.25 }))
                    .unwrap();
                assert_eq!(events, expected);
                assert_eq!(
                    serde_json::to_value(&named).unwrap(),
                    serde_json::to_value(&canonical).unwrap()
                );
                if i == 2 {
                    assert!(events.iter().any(
                        |e| matches!(e, Event::StockExhausted { key: actual, .. } if actual == key)
                    ));
                }
            }
            assert_eq!(named.stock.entries().count(), 1);
            assert_eq!(named.stock.remaining(name).unwrap().amount, 0.5);
            assert_eq!(named.stock.remaining(key).unwrap().amount, 0.5);
            let restored: Bench =
                serde_json::from_value(serde_json::to_value(&named).unwrap()).unwrap();
            assert_eq!(
                serde_json::to_value(restored).unwrap(),
                serde_json::to_value(named).unwrap()
            );
        }
    }
}

fn rename_bottle(mut saved: Value, name: &str, key: &str, collide: bool) -> Value {
    let bottles = if saved.get("schema").is_some() {
        saved["bottles"].as_object_mut().unwrap()
    } else {
        saved.as_object_mut().unwrap()
    };
    let amount = bottles.remove(key).unwrap();
    if collide {
        bottles.insert(key.into(), amount.clone());
    }
    bottles.insert(name.into(), amount);
    saved
}

#[test]
fn restored_stock_keeps_exact_amounts_and_refuses_alias_collisions() {
    for compensated in [false, true] {
        for &(name, key) in NAMES {
            let mut ledger = if compensated {
                StockLedger::compensated()
            } else {
                StockLedger::default()
            };
            ledger.stock(key, 1.0, StockUnit::Mole);
            if compensated {
                ledger.draw(key, 2.0_f64.powi(-60)).unwrap();
            }
            let saved = serde_json::to_value(&ledger).unwrap();
            let mut restored: StockLedger =
                serde_json::from_value(rename_bottle(saved.clone(), name, key, false)).unwrap();
            assert_eq!(
                restored.remaining_exact(name).unwrap().parts(),
                ledger.remaining_exact(key).unwrap().parts()
            );
            restored.draw(name, 0.25).unwrap();
            ledger.draw(key, 0.25).unwrap();
            assert_eq!(restored, ledger);
            assert!(
                serde_json::from_value::<StockLedger>(rename_bottle(saved, name, key, true))
                    .is_err()
            );
        }
    }
}

struct Veto<'a>(&'a str);
impl SafetyScreen for Veto<'_> {
    fn assess(&self, prospective: &Vessel) -> SafetyVerdict {
        assert!(prospective.contents.iter().all(|p| p.species.0 == self.0));
        SafetyVerdict::Veto {
            reason: "reviewed name safety control".into(),
        }
    }
}

#[test]
fn safety_sees_canonical_identity_before_mutation_and_stock_draw() {
    for &(name, key) in NAMES {
        let mut bench = Bench::new();
        bench.stock.stock(key, 1.0, StockUnit::Mole);
        let before = serde_json::to_value(&bench.vessels).unwrap();
        let events = bench
            .step_with(add(name, 0.25), &mut MixingEquilibrator, &Veto(key))
            .unwrap();
        assert!(events.iter().any(|e| matches!(e, Event::SafetyVeto { .. })));
        assert_eq!(serde_json::to_value(&bench.vessels).unwrap(), before);
        assert_eq!(bench.stock.remaining(name).unwrap().amount, 1.0);
    }
}

#[test]
fn direct_vessel_lots_queries_debits_and_restore_share_identity() {
    for &(name, key) in NAMES {
        let phase = species::lookup_key(key).unwrap().standard_phase;
        let mut vessel = Vessel::new(VesselId(0), "name receiver");
        vessel.deposit_lot(
            SpeciesId(name.into()),
            Moles(1.0),
            phase,
            Some("first".into()),
            None,
        );
        vessel.deposit_lot(
            SpeciesId(key.into()),
            Moles(2.0),
            phase,
            Some("second".into()),
            None,
        );
        assert_eq!(vessel.contents.len(), 1);
        assert_eq!(vessel.lots.len(), 2);
        assert!(vessel.lots.iter().all(|lot| lot.species.0 == key));
        assert_eq!(vessel.moles_of(&SpeciesId(name.into())).0, 3.0);
        assert_eq!(
            vessel
                .withdraw_phase(&SpeciesId(name.into()), Moles(0.5), phase)
                .0,
            0.5
        );
        let mut saved = serde_json::to_value(&vessel).unwrap();
        saved["contents"][0]["species"] = json!(name);
        saved["lots"][0]["species"] = json!(name);
        let restored: Vessel = serde_json::from_value(saved).unwrap();
        assert_eq!(restored.moles_of(&SpeciesId(name.into())).0, 2.5);
        assert_eq!(restored.lots.len(), 2);
        assert!(restored.lots.iter().all(|lot| lot.species.0 == key));
    }
}

#[test]
fn pack_species_cannot_hide_reserved_input_names() {
    for name in [
        "H2O",
        "oxygen",
        "OXYGEN",
        "nitrogen",
        "carbon dioxide",
        "carbon  dioxide",
        "sodium chloride",
    ] {
        let mut data = species::lookup_key("NaCl").unwrap().clone();
        data.key = name;
        assert_eq!(species::register_loaded(vec![data]), (0, 1));
    }
}

#[test]
fn pack_material_names_and_aliases_cannot_collide_with_species_aliases() {
    for name in [
        "H2O",
        "oxygen",
        "OXYGEN",
        "nitrogen",
        "carbon_dioxide",
        "carbon-dioxide",
        "sodium chloride",
    ] {
        for field in ["key", "name", "alias"] {
            let mut recipe = material::lookup("whole_milk", None).unwrap();
            recipe.id = "test/reviewed-input-name-collision".into();
            recipe.canonical_key = "input_name_collision_fixture".into();
            recipe.name = "Input name collision fixture".into();
            recipe.aliases.clear();
            match field {
                "key" => recipe.canonical_key = name.into(),
                "name" => recipe.name = name.into(),
                _ => {
                    recipe.aliases.insert("en".into(), vec![name.into()]);
                }
            }
            assert_eq!(material::register_loaded(vec![recipe]), (0, 1));
        }
    }
}

#[test]
fn raw_solvent_grind_and_titrant_names_match_canonical_execution() {
    for &(name, key) in NAMES {
        for line in [
            format!("extract v1 v2 {key} 0.1mol"),
            format!("grind v1 {key} 50um"),
            format!("titrate v1 {key} 1M 1mL until ph 7 max 1"),
        ] {
            let canonical = parse_op(&line).unwrap().unwrap();
            let mut named = canonical.clone();
            match &mut named {
                Operator::Extract { solvent, .. } => *solvent = SpeciesId(name.into()),
                Operator::Grind { species, .. } => *species = SpeciesId(name.into()),
                Operator::Titrate { titrant, .. } => *titrant = SpeciesId(name.into()),
                _ => unreachable!(),
            }
            let mut a = Bench::new();
            a.step(add("water", 2.0)).unwrap();
            a.step(Operator::NewVessel { kind: None }).unwrap();
            let mut b = a.clone();
            let outcome = |bench: &mut Bench, op| {
                bench
                    .step(op)
                    .map(|events| serde_json::to_value(events).unwrap())
                    .map_err(|error| error.to_string())
            };
            assert_eq!(outcome(&mut a, canonical), outcome(&mut b, named));
            assert_eq!(
                serde_json::to_value(a).unwrap(),
                serde_json::to_value(b).unwrap()
            );
        }
    }
}

#[test]
fn optional_pack_admission_still_accepts_noncolliding_names() {
    let mut data = species::lookup_key("NaCl").unwrap().clone();
    data.key = "input_alias_noncollision_species_fixture";
    assert_eq!(species::register_loaded(vec![data]), (1, 0));
    assert!(species::lookup_key("input_alias_noncollision_species_fixture").is_some());
    let mut recipe = material::lookup("whole_milk", None).unwrap();
    recipe.id = "test/reviewed-input-name-noncollision".into();
    recipe.canonical_key = "input_alias_noncollision_material_fixture".into();
    recipe.name = "Input alias noncollision material fixture".into();
    recipe.aliases.clear();
    assert_eq!(material::register_loaded(vec![recipe]), (1, 0));
    assert!(material::lookup("input_alias_noncollision_material_fixture", None).is_some());
}
