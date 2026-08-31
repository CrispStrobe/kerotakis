//! BRD-002: the shelf holds finite bottles, and running out is said out loud.

use kerotakis_core::render::{render_event, Register};
use kerotakis_core::script::parse_op;
use kerotakis_core::stock::StockUnit;
use kerotakis_core::*;

fn water(bench: &mut Bench, moles: f64) {
    bench
        .step(Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new("water"),
            moles: Moles(moles),
            at: None,
        })
        .expect("water is not stocked, so it never runs out");
}

fn salt(moles: f64) -> Operator {
    Operator::Add {
        vessel: VesselId(0),
        species: SpeciesId::new("NaCl"),
        moles: Moles(moles),
        at: None,
    }
}

#[test]
fn an_unstocked_bottle_is_an_unlimited_supply() {
    let mut bench = Bench::new();
    water(&mut bench, 10.0);
    for _ in 0..20 {
        bench.step(salt(0.1)).expect("nothing limits an open shelf");
    }
    assert!(bench.stock.is_empty(), "no bottle was ever stocked");
    assert!(
        (bench
            .vessel(VesselId(0))
            .unwrap()
            .moles_of(&SpeciesId::new("NaCl"))
            .0
            - 2.0)
            .abs()
            < 1e-9
    );
}

#[test]
fn a_stocked_bottle_empties_and_then_refuses_the_next_dispense() {
    let mut bench = Bench::new();
    water(&mut bench, 10.0);
    bench
        .step(Operator::StockShelf {
            key: "NaCl".into(),
            amount: 0.5,
        })
        .expect("NaCl is a registry species");

    bench.step(salt(0.3)).expect("0.3 of 0.5 fits");
    assert!((bench.stock.remaining("NaCl").unwrap().amount - 0.2).abs() < 1e-12);

    let events = bench
        .step(salt(0.3))
        .expect("a refusal is not an engine error");
    let refusal = events
        .iter()
        .find_map(|event| match event {
            Event::StockExhausted {
                key,
                requested,
                remaining,
                unit,
            } => Some((key.clone(), *requested, *remaining, *unit)),
            _ => None,
        })
        .expect("running out is an event, not silence");
    assert_eq!(refusal.0, "NaCl");
    assert!((refusal.1 - 0.3).abs() < 1e-12, "the amount asked for");
    assert!((refusal.2 - 0.2).abs() < 1e-12, "what is actually left");
    assert_eq!(refusal.3, StockUnit::Mole);

    // Nothing moved: not the bottle, and not the beaker.
    assert!((bench.stock.remaining("NaCl").unwrap().amount - 0.2).abs() < 1e-12);
    assert!(
        (bench
            .vessel(VesselId(0))
            .unwrap()
            .moles_of(&SpeciesId::new("NaCl"))
            .0
            - 0.3)
            .abs()
            < 1e-9,
        "the refused dispense deposited nothing"
    );
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::Added { .. })),
        "a refusal must not also claim the addition happened"
    );
}

#[test]
fn the_refusal_speaks_in_all_three_registers_and_names_both_numbers() {
    let event = Event::StockExhausted {
        key: "NaCl".into(),
        requested: 0.3,
        remaining: 0.2,
        unit: StockUnit::Mole,
    };
    for register in [Register::LV1, Register::LV2, Register::LV3] {
        let line = render_event(&event, register);
        assert!(!line.is_empty(), "{register} has prose");
        assert!(
            line.contains("NaCl"),
            "{register} names the substance: {line}"
        );
        assert!(line.contains("0.2"), "{register} says what is left: {line}");
        assert!(
            line.contains("0.3"),
            "{register} says what was asked: {line}"
        );
    }
    // The three voices are three sentences, not one repeated.
    let lv1 = render_event(&event, Register::LV1);
    let lv3 = render_event(&event, Register::LV3);
    assert_ne!(lv1, lv3);
}

#[test]
fn a_material_bottle_is_counted_in_the_recipes_own_basis() {
    let mut bench = Bench::new();
    bench
        .step(Operator::StockShelf {
            key: "white_vinegar_5_percent".into(),
            amount: 100.0,
        })
        .expect("a household recipe is stockable");
    assert_eq!(
        bench
            .stock
            .remaining("white_vinegar_5_percent")
            .unwrap()
            .unit,
        StockUnit::Gram,
        "a mass-fraction recipe empties in grams, not in moles of acetic acid"
    );

    let pour = |amount: f64| match parse_op(&format!("add v1 white_vinegar_5_percent {amount}g"))
        .expect("the grammar accepts a material by key")
    {
        Some(op) => op,
        None => panic!("that line is an operator"),
    };

    bench.step(pour(60.0)).expect("60 g of 100 g fits");
    assert!(
        (bench
            .stock
            .remaining("white_vinegar_5_percent")
            .unwrap()
            .amount
            - 40.0)
            .abs()
            < 1e-9
    );

    let events = bench
        .step(pour(60.0))
        .expect("a refusal is not an engine error");
    assert!(
        events.iter().any(
            |event| matches!(event, Event::StockExhausted { unit, .. } if *unit == StockUnit::Gram)
        ),
        "the second 60 g is refused in grams"
    );
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::MaterialAdded { .. })),
        "and nothing was expanded into the vessel"
    );
}

#[test]
fn the_snapshot_token_round_trips_the_shelf_stock() {
    let mut bench = Bench::new();
    water(&mut bench, 10.0);
    bench
        .step(Operator::StockShelf {
            key: "NaCl".into(),
            amount: 0.5,
        })
        .unwrap();
    bench.step(salt(0.3)).unwrap();

    // This is exactly what the protocol's `snapshot`/`restore` pair does —
    // the token is opaque `Bench` serde, so if the ledger lives on the
    // bench, undo gets it for free. This test is the proof of that claim.
    let token = serde_json::to_string(&bench).expect("the bench serialises");
    let restored: Bench = serde_json::from_str(&token).expect("and parses back");
    assert_eq!(restored.stock, bench.stock);
    assert!((restored.stock.remaining("NaCl").unwrap().amount - 0.2).abs() < 1e-12);

    // And the restored bench refuses the same dispense the original would.
    let mut restored = restored;
    let events = restored.step(salt(0.3)).unwrap();
    assert!(events
        .iter()
        .any(|event| matches!(event, Event::StockExhausted { .. })));
}

#[test]
fn a_snapshot_written_before_the_ledger_existed_restores_as_an_open_shelf() {
    // Old hosts and old saved sessions must not break: a token with no
    // `stock` key is a bench whose bottles are bottomless, which is what it
    // meant when it was written.
    let legacy = r#"{"vessels":[],"log":[]}"#;
    let bench: Bench = serde_json::from_str(legacy).expect("an older token still parses");
    assert!(bench.stock.is_empty());
}

#[test]
fn stocking_something_the_shelf_does_not_have_is_refused_by_name() {
    let mut bench = Bench::new();
    let error = bench
        .step(Operator::StockShelf {
            key: "unobtainium".into(),
            amount: 1.0,
        })
        .expect_err("there is no such bottle to fill");
    assert!(matches!(error, BenchError::UnstockableKey(key) if key == "unobtainium"));
}

#[test]
fn the_stock_line_parses_and_pins_the_canonical_key() {
    let op = parse_op("stock NaCl 0.5mol").unwrap().expect("an operator");
    assert!(matches!(&op, Operator::StockShelf { key, amount }
        if key == "NaCl" && (*amount - 0.5).abs() < 1e-12));

    // A material stated in millilitres is converted by the same reviewed
    // bulk density `add` uses, and stored in the recipe's own basis.
    let op = parse_op("stock white_vinegar_5_percent 100mL")
        .unwrap()
        .expect("an operator");
    let Operator::StockShelf { key, amount } = op else {
        panic!("stock parses to StockShelf");
    };
    assert_eq!(key, "white_vinegar_5_percent");
    assert!(
        amount > 90.0 && amount < 120.0,
        "100 mL of a watery liquid is ~100 g, got {amount}"
    );
}
