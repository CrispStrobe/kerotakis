//! Typed cation-exchange bookkeeping that does not require a chemistry engine.

use kerotakis_core::*;

fn sodium_resin() -> ExchangeSites {
    ExchangeSites {
        label: "sodium softener resin".to_string(),
        dry_mass: Grams(2.0),
        capacity: Moles(2e-3),
        occupancy: vec![ExchangeOccupancy {
            ion: ExchangeIon::Sodium,
            moles: Moles(2e-3),
        }],
    }
}

#[test]
fn exchange_capacity_is_counted_in_charge_equivalents() {
    let mut resin = sodium_resin();
    assert_eq!(resin.occupied_equivalents(), Moles(2e-3));
    assert!(resin.has_valid_capacity());

    resin.occupancy = vec![
        ExchangeOccupancy {
            ion: ExchangeIon::Calcium,
            moles: Moles(0.75e-3),
        },
        ExchangeOccupancy {
            ion: ExchangeIon::Magnesium,
            moles: Moles(0.25e-3),
        },
    ];
    assert_eq!(resin.occupied_equivalents(), Moles(2e-3));
    assert_eq!(resin.bound(ExchangeIon::Calcium), Moles(0.75e-3));
    assert_eq!(resin.bound(ExchangeIon::Magnesium), Moles(0.25e-3));
    assert!(resin.has_valid_capacity());

    resin.occupancy[0].moles = Moles(0.8e-3);
    assert!(!resin.has_valid_capacity());
}

#[test]
fn vessel_mass_includes_support_and_bound_cations() {
    let mut vessel = Vessel::new(VesselId(0), "softener batch");
    vessel.exchanges.push(sodium_resin());

    let sodium_mass = species::lookup_key("Na+").unwrap().molar_mass * 2e-3;
    assert!((vessel.mass().0 - (2.0 + sodium_mass)).abs() < 1e-12);
    assert!(!vessel.is_empty());
}

#[test]
fn legacy_vessels_deserialise_without_exchange_sites() {
    let json = r#"{
        "elapsed_seconds": 0.0,
        "id": 0,
        "label": "beaker",
        "contents": [],
        "temperature": 298.15,
        "pressure": 101325.0,
        "thermal_mode": "adiabatic",
        "headspace": { "boundary": "open" },
        "surfaces": [],
        "solute_charge": 0.0,
        "solution": null
    }"#;
    let vessel: Vessel = serde_json::from_str(json).expect("old vessel JSON remains readable");
    assert!(vessel.exchanges.is_empty());
}
