//! JSON representation contracts frozen before the inventory serde repair.
//! Canonical keys are registered element symbol + positive u32 mass number,
//! with optional lowercase m. This validates identity syntax, not the existence
//! or evaluated nuclear properties of every syntactically admissible isotope.
use kerotakis_core::nuclide::{Nuclide, NuclideLedger};
use kerotakis_core::{Vessel, VesselId};
use serde_json::json;

#[test]
fn empty_inventory_preserves_existing_json_object() {
    assert_eq!(
        serde_json::to_value(NuclideLedger::default()).unwrap(),
        json!({"inventory": {}})
    );
    let ledger: NuclideLedger = serde_json::from_value(json!({"inventory": {}})).unwrap();
    assert!(ledger.inventory.is_empty());
}

#[test]
fn ground_state_inventory_uses_canonical_key_and_roundtrips() {
    let mut ledger = NuclideLedger::default();
    ledger.deposit(Nuclide::new("C", 14), 1e-12);
    assert_eq!(
        serde_json::to_value(&ledger).unwrap(),
        json!({"inventory": {"C-14": 1e-12}})
    );
    let copy: NuclideLedger =
        serde_json::from_str(&serde_json::to_string(&ledger).unwrap()).unwrap();
    assert_eq!(copy.inventory, ledger.inventory);
}

#[test]
fn metastable_and_ground_state_remain_distinct() {
    let mut ledger = NuclideLedger::default();
    ledger.deposit(Nuclide::parse("Tc-99").unwrap(), 0.1);
    ledger.deposit(Nuclide::parse("Tc-99m").unwrap(), 0.2);
    let value = serde_json::to_value(&ledger).unwrap();
    assert_eq!(value, json!({"inventory": {"Tc-99": 0.1, "Tc-99m": 0.2}}));
    let copy: NuclideLedger = serde_json::from_value(value).unwrap();
    assert_eq!(copy.inventory, ledger.inventory);
}

#[test]
fn vessel_with_owned_nuclides_has_json_roundtrip() {
    let mut vessel = Vessel::new(VesselId(0), "tracer");
    vessel.nuclides.deposit(Nuclide::new("C", 14), 1e-12);
    let value = serde_json::to_value(&vessel).unwrap();
    let copy: Vessel = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(serde_json::to_value(copy).unwrap(), value);
}

#[test]
fn ordinary_nuclide_object_representation_is_unchanged() {
    let isotope = Nuclide::parse("Tc-99m").unwrap();
    assert_eq!(
        serde_json::to_value(&isotope).unwrap(),
        json!({"element":"Tc", "mass_number":99, "metastable":true})
    );
    let ground: Nuclide = serde_json::from_value(json!({"element":"C", "mass_number":14})).unwrap();
    assert_eq!(ground, Nuclide::new("C", 14));
}

#[test]
fn malformed_or_noncanonical_inventory_keys_are_rejected() {
    for key in [
        "",
        "C",
        "C14",
        "C-",
        "C-0",
        "C--14",
        "C-14M",
        "C-14mm",
        "c-14",
        "Xx-14",
        "C-+14",
        "C-014",
        " C-14",
        "C-14 ",
        "C-4294967296",
    ] {
        assert!(
            serde_json::from_value::<NuclideLedger>(json!({"inventory": {key: 1.0}})).is_err(),
            "accepted {key}"
        );
    }
}

#[test]
fn malformed_owned_keys_cannot_be_serialized_as_valid_inventory() {
    for isotope in [
        Nuclide::new("Xx", 14),
        Nuclide::new("c", 14),
        Nuclide::new("C", 0),
        Nuclide::new("C-14", 1),
    ] {
        let mut ledger = NuclideLedger::default();
        ledger.deposit(isotope, 1.0);
        assert!(serde_json::to_string(&ledger).is_err());
    }
}

#[test]
fn nonfinite_and_negative_inventory_values_refuse_serialization() {
    for amount in [
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        -1.0,
        -f64::from_bits(1),
    ] {
        let mut ledger = NuclideLedger::default();
        ledger.deposit(Nuclide::new("C", 14), amount);
        assert!(serde_json::to_value(&ledger).is_err(), "accepted {amount}");
    }
}

#[test]
fn null_nonfinite_overflow_and_negative_json_values_are_rejected() {
    for raw in [
        "null", "-1", "-5e-324", "1e400", "NaN", "Infinity", "\"1.0\"",
    ] {
        let text = format!("{{\"inventory\":{{\"C-14\":{raw}}}}}");
        assert!(
            serde_json::from_str::<NuclideLedger>(&text).is_err(),
            "accepted {raw}"
        );
    }
}

#[test]
fn duplicate_and_alias_collisions_cannot_silently_overwrite() {
    for text in [
        r#"{"inventory":{"C-14":1,"C-14":2}}"#,
        r#"{"inventory":{"C-14":1,"C-014":2}}"#,
        r#"{"inventory":{"C-+14":1,"C-14":2}}"#,
    ] {
        assert!(serde_json::from_str::<NuclideLedger>(text).is_err());
    }
}

#[test]
fn zero_and_smallest_positive_amounts_roundtrip_without_pruning() {
    let mut ledger = NuclideLedger::default();
    ledger.deposit(Nuclide::new("C", 12), 0.0);
    ledger.deposit(Nuclide::new("C", 14), f64::from_bits(1));
    let text = serde_json::to_string(&ledger).unwrap();
    let copy: NuclideLedger = serde_json::from_str(&text).unwrap();
    assert_eq!(copy.inventory, ledger.inventory);
    assert_eq!(copy.inventory.len(), 2);
}
