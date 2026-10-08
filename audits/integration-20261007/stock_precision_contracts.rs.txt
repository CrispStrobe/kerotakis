//! Stock donor arithmetic must not invent stock, erase it, or debit inaccurately.
use kerotakis_core::stock::*;
fn ledger(n: f64) -> StockLedger {
    let mut l = StockLedger::default();
    l.stock("water", n, StockUnit::Mole);
    l
}
#[test]
fn an_empty_bottle_refuses_every_positive_draw() {
    for n in [1e-12, f64::from_bits(1)] {
        let mut l = ledger(0.0);
        let before = l.clone();
        assert!(l.draw("water", n).is_err());
        assert_eq!(l, before);
    }
}
#[test]
fn microscopic_relative_overdraw_is_not_float_slack() {
    let mut l = ledger(1e-12);
    let before = l.clone();
    assert!(l.draw("water", 2e-12).is_err());
    assert_eq!(l, before);
}
#[test]
fn invalid_requests_never_mutate_stock_even_when_untracked() {
    for n in [-1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for tracked in [true, false] {
            let mut l = if tracked {
                ledger(1.0)
            } else {
                StockLedger::default()
            };
            let before = l.clone();
            assert!(l.draw("water", n).is_err());
            assert_eq!(l, before);
        }
    }
}
#[test]
fn swallowed_stock_debit_refuses() {
    let mut l = ledger(1e20);
    let before = l.clone();
    assert!(l.draw("water", 0.01).is_err());
    assert_eq!(l, before);
}
#[test]
fn inaccurate_nonzero_stock_debit_refuses() {
    let mut l = ledger(1e12);
    let before = l.clone();
    assert!(l.draw("water", 0.001).is_err());
    assert_eq!(l, before);
}
#[test]
fn ordinary_draw_exact_exhaustion_and_zero_draw_remain_supported() {
    let mut l = ledger(0.5);
    l.draw("water", 0.25).unwrap();
    assert_eq!(l.remaining("water").unwrap().amount, 0.25);
    l.draw("water", 0.0).unwrap();
    l.draw("water", 0.25).unwrap();
    assert_eq!(l.remaining("water").unwrap().amount, 0.0);
}
#[test]
fn exact_subnormal_stock_debit_and_json_roundtrip_remain_supported() {
    let mut l = ledger(f64::from_bits(4));
    l.draw("water", f64::from_bits(1)).unwrap();
    assert_eq!(l.remaining("water").unwrap().amount, f64::from_bits(3));
    let restored: StockLedger = serde_json::from_value(serde_json::to_value(&l).unwrap()).unwrap();
    assert_eq!(restored, l);
}
#[test]
fn relative_last_bit_rounding_remains_supported() {
    let mut l = ledger(0.3);
    l.draw("water", 0.1).unwrap();
    l.draw("water", 0.2).unwrap();
    assert_eq!(l.remaining("water").unwrap().amount, 0.0);
}
