//! Opt-in compensated stock requirements. Default conservative ledgers keep
//! their frozen precision-refusal contracts; no ordinary script switches mode.
use kerotakis_core::stock::{StockLedger, StockUnit};

fn bottle(value: f64) -> StockLedger {
    let mut ledger = StockLedger::compensated();
    ledger.stock("water", value, StockUnit::Mole);
    ledger
}

#[test]
fn compensated_mode_is_explicit_and_default_remains_conservative() {
    assert!(!StockLedger::default().is_compensated());
    let mut default = StockLedger::default();
    default.stock("water", 1e20, StockUnit::Mole);
    assert!(default.draw("water", 0.01).is_err());
    let mut compensated = bottle(1e20);
    assert!(compensated.is_compensated());
    compensated.draw("water", 0.01).unwrap();
    assert_eq!(
        compensated.remaining_exact("water").unwrap().parts(),
        (1e20, -0.01)
    );
}

#[test]
fn projected_display_does_not_erase_authoritative_low_component() {
    let mut ledger = bottle(1.0);
    let tiny = 2.0_f64.powi(-60);
    ledger.draw("water", tiny).unwrap();
    assert_eq!(ledger.remaining("water").unwrap().amount, 1.0);
    assert_eq!(
        ledger.remaining_exact("water").unwrap().parts(),
        (1.0, -tiny)
    );
    let entries: Vec<_> = ledger.entries().collect();
    assert_eq!(entries[0].1.amount, 1.0);
    assert_eq!(entries[0].1.unit, StockUnit::Mole);
}

#[test]
fn repeated_tiny_debits_accumulate_until_scalar_level_changes() {
    let mut ledger = bottle(1.0);
    let tiny = 2.0_f64.powi(-60);
    for _ in 0..256 {
        ledger.draw("water", tiny).unwrap();
    }
    assert_eq!(
        ledger.remaining_exact("water").unwrap().parts(),
        (1.0 - f64::EPSILON, 0.0)
    );
}

#[test]
fn projected_full_balance_cannot_overdraw_hidden_debit() {
    let mut ledger = bottle(1.0);
    ledger.draw("water", 2.0_f64.powi(-60)).unwrap();
    let before = ledger.clone();
    assert!(ledger.draw("water", 1.0).is_err());
    assert_eq!(ledger, before);
}

#[test]
fn exact_binary_exhaustion_is_supported_and_empty_stays_empty() {
    let mut ledger = bottle(1.0);
    ledger.draw("water", 0.25).unwrap();
    ledger.draw("water", 0.75).unwrap();
    assert_eq!(ledger.remaining_exact("water").unwrap().parts(), (0.0, 0.0));
    let before = ledger.clone();
    assert!(ledger.draw("water", f64::from_bits(1)).is_err());
    assert_eq!(ledger, before);
}

#[test]
fn exact_subnormal_stock_and_zero_draw_are_supported() {
    let mut ledger = bottle(f64::from_bits(4));
    ledger.draw("water", f64::from_bits(1)).unwrap();
    ledger.draw("water", 0.0).unwrap();
    ledger.draw("water", f64::from_bits(3)).unwrap();
    assert_eq!(ledger.remaining_exact("water").unwrap().parts(), (0.0, 0.0));
}

#[test]
fn decimal_overdraw_is_not_clamped_in_compensated_mode() {
    let mut ledger = bottle(0.3);
    ledger.draw("water", 0.1).unwrap();
    let before = ledger.clone();
    assert!(ledger.draw("water", 0.2).is_err());
    assert_eq!(ledger, before);
}

#[test]
fn invalid_requests_are_atomic_for_tracked_and_untracked_keys() {
    for value in [-1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for key in ["water", "untracked"] {
            let mut ledger = bottle(1.0);
            let before = ledger.clone();
            assert!(ledger.draw(key, value).is_err());
            assert_eq!(ledger, before);
        }
    }
}

#[test]
fn restocking_replaces_authoritative_balance_and_unlimit_removes_it() {
    let mut ledger = bottle(1.0);
    ledger.draw("water", 2.0_f64.powi(-60)).unwrap();
    ledger.stock("water", 0.5, StockUnit::Gram);
    assert_eq!(ledger.remaining_exact("water").unwrap().parts(), (0.5, 0.0));
    assert_eq!(ledger.remaining("water").unwrap().unit, StockUnit::Gram);
    ledger.unlimit("water");
    assert_eq!(ledger.remaining_exact("water"), None);
    ledger.draw("water", 1e30).unwrap();
}

#[test]
fn cloning_stages_changes_without_mutating_original_ledger() {
    let original = bottle(1.0);
    let mut staged = original.clone();
    staged.draw("water", 2.0_f64.powi(-60)).unwrap();
    assert!(staged.draw("water", 2.0).is_err());
    assert_eq!(
        original.remaining_exact("water").unwrap().parts(),
        (1.0, 0.0)
    );
    assert_ne!(staged, original);
}

#[test]
fn new_schema_roundtrip_preserves_low_component_and_continued_debits() {
    let tiny = 2.0_f64.powi(-60);
    let mut ledger = bottle(1.0);
    ledger.draw("water", tiny).unwrap();
    let encoded = serde_json::to_value(&ledger).unwrap();
    assert_eq!(encoded["schema"], "kerotakis-stock/2");
    assert_eq!(encoded["bottles"]["water"]["amount"]["hi"], 1.0);
    assert_eq!(encoded["bottles"]["water"]["amount"]["lo"], -tiny);
    let mut restored: StockLedger = serde_json::from_value(encoded).unwrap();
    assert_eq!(restored, ledger);
    restored.draw("water", tiny).unwrap();
    assert_eq!(
        restored.remaining_exact("water").unwrap().parts(),
        (1.0, -2.0 * tiny)
    );
}

#[test]
fn new_schema_is_rejected_by_legacy_scalar_ledger_reader() {
    let value = serde_json::to_value(bottle(1.0)).unwrap();
    let old_reader = serde_json::from_value::<
        std::collections::BTreeMap<String, kerotakis_core::stock::StockAmount>,
    >(value);
    assert!(old_reader.is_err());
}

#[test]
fn legacy_scalar_json_loads_and_roundtrips_in_conservative_mode() {
    let legacy = serde_json::json!({"water":{"amount":1e20,"unit":"mole"}});
    let mut ledger: StockLedger = serde_json::from_value(legacy.clone()).unwrap();
    assert!(!ledger.is_compensated());
    assert!(ledger.draw("water", 0.01).is_err());
    assert_eq!(serde_json::to_value(ledger).unwrap(), legacy);
}

#[test]
fn malformed_new_schema_and_negative_legacy_stock_are_rejected() {
    for value in [
        serde_json::json!({"schema":"kerotakis-stock/3","bottles":{}}),
        serde_json::json!({"schema":"kerotakis-stock/2","bottles":{},"surprise":true}),
        serde_json::json!({"schema":"kerotakis-stock/2","bottles":{"water":{"amount":{"hi":1,"lo":1},"unit":"mole"}}}),
        serde_json::json!({"water":{"amount":-1,"unit":"mole"}}),
    ] {
        assert!(serde_json::from_value::<StockLedger>(value).is_err());
    }
}

#[test]
fn empty_compensated_ledger_mode_survives_bench_snapshot() {
    let mut bench = kerotakis_core::Bench::new();
    bench.stock = StockLedger::compensated();
    assert!(bench.stock.is_empty());
    let restored: kerotakis_core::Bench =
        serde_json::from_value(serde_json::to_value(bench).unwrap()).unwrap();
    assert!(restored.stock.is_compensated());
    assert!(restored.stock.is_empty());
}
