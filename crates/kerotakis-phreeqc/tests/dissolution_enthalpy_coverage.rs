//! The dissolution/hydration/reaction-heat ownership table cannot go stale.
//!
//! Recomputes, from the shipped registry and the vendored databases:
//! * the `llnl.dat` rows carried as `data/llnl-dissolution-references-20261027.json`;
//! * the ownership route of EVERY registry solid on EVERY routed database;
//! * the reaction-heat rows of `kerotakis_core::reaction_heat::audit`;
//!
//! and compares them with the checked-in fixture and
//! `docs/dissolution-enthalpy-coverage-20261027.md`. `KERO_BLESS_COVERAGE=1`
//! rewrites the JSON fixtures instead of failing (the Markdown table is then
//! printed for pasting).

use std::collections::BTreeMap;

use kerotakis_core::species::{self, Phase};
use kerotakis_phreeqc::derived::DB_TAGS;
use kerotakis_phreeqc::dissolution_reference::{self, SourceTable};
use kerotakis_phreeqc::enthalpy::{solid_route, SolidRoute};

const LLNL: &[u8] = include_bytes!("../../../vendor/iphreeqc/database/llnl.dat");
const COVERAGE: &str = include_str!("fixtures/dissolution-enthalpy-coverage-20261027.json");
const DOC: &str = include_str!("../../../docs/dissolution-enthalpy-coverage-20261027.md");

fn manifest(path: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn bless() -> bool {
    std::env::var_os("KERO_BLESS_COVERAGE").is_some()
}

fn close(a: Option<f64>, b: Option<f64>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(x), Some(y)) => (x - y).abs() <= 1e-9 * x.abs().max(1.0),
        _ => false,
    }
}

fn same_terms(a: &[(String, f64)], b: &[(String, f64)]) -> bool {
    a.len() == b.len()
        && a.iter()
            .zip(b)
            .all(|((s, n), (t, m))| s == t && (n - m).abs() < 1e-12)
}

fn same_table(a: &SourceTable, b: &SourceTable) -> Result<(), String> {
    let names = |t: &SourceTable| t.phases.iter().map(|p| p.phase.clone()).collect::<Vec<_>>();
    if names(a) != names(b) {
        return Err(format!("phases differ: {:?} vs {:?}", names(a), names(b)));
    }
    for (p, q) in a.phases.iter().zip(&b.phases) {
        if p.formula != q.formula
            || !same_terms(&p.lhs, &q.lhs)
            || !same_terms(&p.rhs, &q.rhs)
            || !close(p.log_k, q.log_k)
            || !close(p.delta_h_kj, q.delta_h_kj)
        {
            return Err(format!("phase {} differs: {p:?} vs {q:?}", p.phase));
        }
    }
    let keys = |t: &SourceTable| t.species.keys().cloned().collect::<Vec<_>>();
    if keys(a) != keys(b) {
        return Err(format!("species differ: {:?} vs {:?}", keys(a), keys(b)));
    }
    for (name, r) in &a.species {
        let s = &b.species[name];
        if !same_terms(&r.lhs, &s.lhs)
            || !same_terms(&r.rhs, &s.rhs)
            || !close(r.log_k, s.log_k)
            || !close(r.delta_h_kj, s.delta_h_kj)
        {
            return Err(format!("species {name} differs: {r:?} vs {s:?}"));
        }
    }
    Ok(())
}

#[test]
fn the_llnl_rows_are_reextracted_from_the_vendored_file() {
    let text = String::from_utf8_lossy(LLNL);
    let fresh = dissolution_reference::reduce_for_fixture(&dissolution_reference::parse_source(
        "llnl.dat", &text,
    ));
    let shipped = dissolution_reference::table("llnl.dat").expect("fixture");
    if let Err(diff) = same_table(&fresh, shipped) {
        if bless() {
            std::fs::write(
                manifest("data/llnl-dissolution-references-20261027.json"),
                serde_json::to_string_pretty(&fresh).unwrap() + "\n",
            )
            .unwrap();
        } else {
            panic!("llnl fixture is stale ({diff}); rerun with KERO_BLESS_COVERAGE=1");
        }
    }
    // The rows this lane exists for are present.
    for phase in ["Niter", "Periclase", "Lime", "Chalcanthite", "MgSO4"] {
        assert!(fresh.phases.iter().any(|p| p.phase == phase), "{phase}");
    }
}

fn status(key: &str, db: &str) -> String {
    match solid_route(key, db) {
        Ok(priced) => match priced.route {
            SolidRoute::Curated => "curated".to_string(),
            SolidRoute::Hydration { hydroxide, source } => {
                format!("hydration:{source}:{hydroxide}")
            }
            SolidRoute::RoutedMineral => "routed".to_string(),
            SolidRoute::Reference { source, phase } => format!("reference:{source}:{phase}"),
        },
        Err(e) => format!("refused: {}", e.why),
    }
}

fn coverage() -> BTreeMap<String, BTreeMap<String, String>> {
    species::REGISTRY
        .iter()
        .filter(|s| s.standard_phase == Phase::Solid)
        .map(|s| {
            (
                s.key.to_string(),
                DB_TAGS
                    .iter()
                    .map(|db| (db.to_string(), status(s.key, db)))
                    .collect(),
            )
        })
        .collect()
}

#[test]
fn every_registry_solid_has_a_current_ownership_row() {
    let fresh = coverage();
    let shipped: BTreeMap<String, BTreeMap<String, String>> =
        serde_json::from_str(COVERAGE).expect("coverage fixture parses");
    let mut table = String::new();
    for (key, row) in &fresh {
        table.push_str(&format!(
            "| `{key}` | {} | {} | {} |\n",
            row["wateq4f"], row["minteq.v4"], row["pitzer"]
        ));
    }
    if fresh != shipped {
        if bless() {
            std::fs::write(
                manifest("tests/fixtures/dissolution-enthalpy-coverage-20261027.json"),
                serde_json::to_string_pretty(&fresh).unwrap() + "\n",
            )
            .unwrap();
            println!("{table}");
        } else {
            let changed: Vec<_> = fresh
                .iter()
                .filter(|(k, v)| shipped.get(*k) != Some(v))
                .collect();
            panic!(
                "coverage fixture is stale; changed rows: {changed:#?}\nregenerated table:\n{table}"
            );
        }
    }
    for line in table.lines() {
        assert!(DOC.contains(line), "docs table is stale at: {line}");
    }
    let total = fresh.len();
    let counts: Vec<String> = DB_TAGS
        .iter()
        .map(|db| {
            let priced = fresh
                .values()
                .filter(|row| !row[*db].starts_with("refused"))
                .count();
            format!("{db} {priced}/{total}")
        })
        .collect();
    let line = format!("Priced after this change: {}.", counts.join(", "));
    assert!(DOC.contains(&line), "docs counts are stale: {line}");
}

#[test]
fn every_reaction_heat_row_is_listed_and_per_extent() {
    let rows = kerotakis_core::reaction_heat::audit();
    for row in &rows {
        assert!(row.per_extent_j.is_finite(), "{row:?}");
        let listed = if let Some(pack) = row.family.strip_prefix("gas-pack:") {
            DOC.contains(&format!("gas-pack:{pack}"))
        } else {
            DOC.contains(&format!("`{}`", row.id))
        };
        assert!(listed, "reaction heat row missing from docs: {row:?}");
    }
}
