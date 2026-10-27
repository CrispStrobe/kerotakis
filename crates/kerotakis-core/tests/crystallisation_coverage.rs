//! The crystallisation-heat coverage table cannot go stale: every registry
//! solid is re-enumerated by `crystallisation::coverage()` and compared with
//! the checked-in fixture and with the table in
//! `docs/crystallisation-heat-coverage-20261027.md`.
//! `KERO_BLESS_COVERAGE=1` rewrites the fixture (the doc table is then
//! regenerated from it by hand or by the same bless run's printed rows).

use kerotakis_core::crystallisation;

const FIXTURE: &str = "tests/fixtures/crystallisation-heat-coverage-20261027.json";
const DOC: &str = "../../docs/crystallisation-heat-coverage-20261027.md";

#[test]
fn crystallisation_coverage_table_is_current() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let rows = crystallisation::coverage();
    let generated = serde_json::to_value(&rows).expect("rows serialise");
    if std::env::var_os("KERO_BLESS_COVERAGE").is_some() {
        let mut text = serde_json::to_string_pretty(&generated).unwrap();
        text.push('\n');
        std::fs::write(root.join(FIXTURE), text).unwrap();
        for row in &rows {
            println!(
                "| `{}` | {} | {} |",
                row.solid, row.crystallisation, row.detail
            );
        }
        return;
    }
    let shipped: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(root.join(FIXTURE)).unwrap()).unwrap();
    assert_eq!(
        generated, shipped,
        "coverage fixture is stale; rerun with KERO_BLESS_COVERAGE=1"
    );
    let doc = std::fs::read_to_string(root.join(DOC)).unwrap();
    for row in &rows {
        let line = format!(
            "| `{}` | {} | {} |",
            row.solid, row.crystallisation, row.detail
        );
        assert!(doc.contains(&line), "doc table is missing or stale: {line}");
    }
    // Every curve is structurally sound, and every curve salt is a row.
    assert!(crystallisation::audit_curves().is_empty());
    for curve in crystallisation::CURVES {
        assert!(rows.iter().any(|r| r.solid == curve.salt
            && (r.crystallisation == "owned" || r.crystallisation == "curve-unpriced")));
    }
}
