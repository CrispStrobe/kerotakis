//! Additional frozen controls for the finite precision of two components.
use kerotakis_core::stock::{StockLedger, StockUnit};

#[test]
fn debit_below_both_components_resolution_is_refused_atomically() {
    let mut ledger = StockLedger::compensated();
    ledger.stock("water", 1e300, StockUnit::Mole);
    ledger.draw("water", 0.01).unwrap();
    let before = ledger.clone();
    assert!(ledger.draw("water", f64::from_bits(1)).is_err());
    assert_eq!(ledger, before);
}

#[test]
fn changing_low_component_with_inaccurate_debit_is_refused_atomically() {
    let mut ledger = StockLedger::compensated();
    ledger.stock("water", 1e300, StockUnit::Mole);
    ledger.draw("water", 1e12).unwrap();
    let before = ledger.clone();
    // The low component's scalar subtraction changes by 0.0009765625,
    // which is not an accurate ledger debit for the requested 0.001.
    assert!(ledger.draw("water", 0.001).is_err());
    assert_eq!(ledger, before);
}
