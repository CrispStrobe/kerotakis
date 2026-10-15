//! Formula input resolves to one inventory and finite stock identity.
use kerotakis_core::script::parse_op;
use kerotakis_core::*;
use serde_json::{json, Value};

fn add(key: &str, moles: f64) -> Operator {
    Operator::Add {
        vessel: VesselId(0),
        // Deliberately bypass the constructor, as direct API callers can.
        species: SpeciesId(key.into()),
        moles: Moles(moles),
        at: None,
    }
}

#[test]
fn formula_and_canonical_text_have_equal_quantities_and_snapshots() {
    for amount in ["1mol", "18g", "10mL"] {
        let canonical = parse_op(&format!("add v1 water {amount}"))
            .unwrap()
            .unwrap();
        let alias = parse_op(&format!("add v1 H2O {amount}")).unwrap().unwrap();
        assert_eq!(alias, canonical);
        let mut a = Bench::new();
        let mut b = Bench::new();
        a.step(canonical).unwrap();
        b.step(alias).unwrap();
        assert_eq!(
            serde_json::to_value(a).unwrap(),
            serde_json::to_value(b).unwrap()
        );
    }
    for key in ["h2o", "H2o", "unknown-water-key"] {
        assert!(parse_op(&format!("add v1 {key} 1mol")).is_err());
    }
}

#[test]
fn identity_bearing_commands_and_shelf_json_are_canonical() {
    for line in [
        "stock water 18g",
        "extract v1 v2 water 0.1mol",
        "grind v1 water 50um",
        "titrate v1 water 1M 1mL until ph 7 max 1",
    ] {
        assert_eq!(
            parse_op(line).unwrap(),
            parse_op(&line.replace("water", "H2O")).unwrap()
        );
    }
    let shelf = Operator::StockShelf {
        key: "H2O".into(),
        amount: 1.0,
    };
    assert_eq!(serde_json::to_value(shelf).unwrap()["key"], "water");
    let restored: Operator = serde_json::from_value(json!({
        "op": "stock_shelf", "key": "H2O", "amount": 1.0
    }))
    .unwrap();
    assert!(matches!(restored, Operator::StockShelf { key, .. } if key == "water"));
}

#[test]
fn direct_solvent_grinding_and_titrant_inputs_match_canonical_behavior() {
    for line in [
        "extract v1 v2 water 0.1mol",
        "grind v1 water 50um",
        "titrate v1 water 1M 1mL until ph 7 max 1",
    ] {
        let canonical = parse_op(line).unwrap().unwrap();
        let mut alias = canonical.clone();
        match &mut alias {
            Operator::Extract { solvent, .. } => *solvent = SpeciesId("H2O".into()),
            Operator::Grind { species, .. } => *species = SpeciesId("H2O".into()),
            Operator::Titrate { titrant, .. } => *titrant = SpeciesId("H2O".into()),
            _ => unreachable!(),
        }
        let mut a = Bench::new();
        a.step(add("water", 2.0)).unwrap();
        a.step(Operator::NewVessel { kind: None }).unwrap();
        let mut b = a.clone();
        let outcome = |bench: &mut Bench, op| {
            bench
                .step(op)
                .map(|e| serde_json::to_value(e).unwrap())
                .map_err(|e| e.to_string())
        };
        assert_eq!(outcome(&mut a, canonical), outcome(&mut b, alias));
        assert_eq!(
            serde_json::to_value(a).unwrap(),
            serde_json::to_value(b).unwrap()
        );
    }
}

#[test]
fn direct_and_json_operators_share_a_finite_bottle_and_canonical_log() {
    for compensated in [false, true] {
        let mut bench = Bench::new();
        if compensated {
            bench.stock = StockLedger::compensated();
        }
        bench
            .step(Operator::StockShelf {
                key: "H2O".into(),
                amount: 1.0,
            })
            .unwrap();
        bench.step(add("H2O", 0.25)).unwrap();
        let json_add: Operator = serde_json::from_value(json!({
            "op": "add", "vessel": 0, "species": "H2O", "moles": 0.25
        }))
        .unwrap();
        bench.step(json_add).unwrap();
        assert_eq!(bench.stock.entries().count(), 1);
        assert_eq!(bench.stock.remaining("water"), bench.stock.remaining("H2O"));
        assert_eq!(bench.stock.remaining("H2O").unwrap().amount, 0.5);
        let before = serde_json::to_value(bench.vessel(VesselId(0)).unwrap()).unwrap();
        let events = bench.step(add("H2O", 0.75)).unwrap();
        assert!(events
            .iter()
            .any(|e| matches!(e, Event::StockExhausted { key, .. } if key == "water")));
        assert_eq!(
            before,
            serde_json::to_value(bench.vessel(VesselId(0)).unwrap()).unwrap()
        );
        let saved = serde_json::to_value(&bench).unwrap();
        assert_eq!(saved["log"][0]["operator"]["key"], "water");
        for entry in saved["log"].as_array().unwrap().iter().skip(1) {
            assert_eq!(entry["operator"]["species"], "water");
        }
        bench.stock.unlimit("H2O");
        assert!(bench.stock.remaining("water").is_none());
    }
}

fn alias_stock(mut saved: Value, collision: bool) -> Value {
    let bottles = if saved.get("schema").is_some() {
        saved["bottles"].as_object_mut().unwrap()
    } else {
        saved.as_object_mut().unwrap()
    };
    let amount = bottles.remove("water").unwrap();
    if collision {
        bottles.insert("water".into(), amount.clone());
    }
    bottles.insert("H2O".into(), amount);
    saved
}

#[test]
fn both_stock_formats_normalize_single_alias_and_refuse_collisions() {
    for compensated in [false, true] {
        let mut ledger = if compensated {
            StockLedger::compensated()
        } else {
            StockLedger::default()
        };
        ledger.stock("water", 1.0, StockUnit::Mole);
        if compensated {
            ledger.draw("water", 2.0_f64.powi(-60)).unwrap();
        }
        let saved = serde_json::to_value(&ledger).unwrap();
        let mut restored: StockLedger =
            serde_json::from_value(alias_stock(saved.clone(), false)).unwrap();
        assert_eq!(restored, ledger);
        assert_eq!(
            restored.remaining_exact("H2O").unwrap().parts(),
            ledger.remaining_exact("water").unwrap().parts()
        );
        restored.draw("H2O", 0.25).unwrap();
        ledger.draw("water", 0.25).unwrap();
        assert_eq!(restored, ledger);
        assert!(serde_json::from_value::<StockLedger>(alias_stock(saved, true)).is_err());
    }
}

#[test]
fn direct_vessel_inputs_queries_and_debits_use_one_species_and_preserve_lots() {
    let mut vessel = Vessel::new(VesselId(0), "alias receiver");
    vessel.deposit_lot(
        SpeciesId("H2O".into()),
        Moles(1.0),
        Phase::Liquid,
        Some("first".into()),
        None,
    );
    vessel.deposit_lot(
        SpeciesId::new("water"),
        Moles(2.0),
        Phase::Liquid,
        Some("second".into()),
        None,
    );
    assert_eq!(vessel.contents.len(), 1);
    assert_eq!(vessel.lots.len(), 2);
    assert!(vessel.lots.iter().all(|lot| lot.species.0 == "water"));
    assert_eq!(vessel.moles_of(&SpeciesId("H2O".into())).0, 3.0);
    assert_eq!(vessel.withdraw(&SpeciesId("H2O".into()), Moles(0.5)).0, 0.5);
    assert_eq!(
        vessel
            .withdraw_phase(&SpeciesId("H2O".into()), Moles(0.5), Phase::Liquid)
            .0,
        0.5
    );
    assert_eq!(vessel.moles_of(&SpeciesId::new("water")).0, 2.0);
}

#[test]
fn restored_alias_portions_keep_their_quantities_and_provenance() {
    let mut vessel = Vessel::new(VesselId(0), "restore receiver");
    vessel.deposit_lot(
        SpeciesId::new("water"),
        Moles(1.0),
        Phase::Liquid,
        Some("first".into()),
        None,
    );
    vessel.deposit_lot(
        SpeciesId::new("water"),
        Moles(2.0),
        Phase::Liquid,
        Some("second".into()),
        None,
    );
    let mut saved = serde_json::to_value(&vessel).unwrap();
    saved["contents"][0]["species"] = json!("H2O");
    saved["lots"][0]["species"] = json!("H2O");
    let restored: Vessel = serde_json::from_value(saved).unwrap();
    assert_eq!(restored.moles_of(&SpeciesId::new("water")).0, 3.0);
    assert_eq!(restored.lots.len(), 2);
    assert_eq!(restored.lots[0].moles.0, 1.0);
    assert_eq!(restored.lots[1].moles.0, 2.0);
    assert!(restored.lots.iter().all(|lot| lot.species.0 == "water"));
}

#[test]
fn alias_water_transfer_and_snapshot_replay_conserve_inventory() {
    let mut bench = Bench::new();
    bench.step(add("H2O", 2.0)).unwrap();
    bench.step(Operator::NewVessel { kind: None }).unwrap();
    bench
        .step(Operator::Decant {
            from: VesselId(0),
            to: VesselId(1),
            fraction: 0.5,
        })
        .unwrap();
    let restored: Bench = serde_json::from_value(serde_json::to_value(&bench).unwrap()).unwrap();
    let total: f64 = restored
        .vessels
        .iter()
        .map(|v| v.moles_of(&SpeciesId("H2O".into())).0)
        .sum();
    assert!((total - 2.0).abs() < 1e-12);
    assert!(restored
        .vessels
        .iter()
        .flat_map(|v| &v.contents)
        .all(|p| p.species.0 == "water"));
}

struct Veto;
impl SafetyScreen for Veto {
    fn assess(&self, prospective: &Vessel) -> SafetyVerdict {
        assert!(prospective.contents.iter().all(|p| p.species.0 == "water"));
        SafetyVerdict::Veto {
            reason: "alias safety control".into(),
        }
    }
}

#[test]
fn alias_safety_veto_preserves_inventory_and_stock() {
    let mut bench = Bench::new();
    bench.stock.stock("water", 1.0, StockUnit::Mole);
    let before = serde_json::to_value(&bench.vessels).unwrap();
    let events = bench
        .step_with(add("H2O", 0.25), &mut MixingEquilibrator, &Veto)
        .unwrap();
    assert!(events.iter().any(|e| matches!(e, Event::SafetyVeto { .. })));
    assert_eq!(serde_json::to_value(&bench.vessels).unwrap(), before);
    assert_eq!(bench.stock.remaining("H2O").unwrap().amount, 1.0);
}
