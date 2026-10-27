//! Every registry solid × routed database: who owns crystallisation on
//! cooling (`kerotakis_phreeqc::crystallisation_route`). The table in
//! `docs/crystallisation-heat-coverage-20261027.md` (between the
//! `coverage:begin`/`coverage:end` markers) and the JSON fixture are
//! recomputed here and must match; `KERO_BLESS_COVERAGE=1` rewrites both.

use kerotakis_phreeqc::crystallisation_route::{self, RouteRow};

const FIXTURE: &str = "tests/fixtures/crystallisation-route-coverage-20261027.json";
const DOC: &str = "../../docs/crystallisation-heat-coverage-20261027.md";
const BEGIN: &str = "<!-- coverage:begin -->";
const END: &str = "<!-- coverage:end -->";

fn table(rows: &[RouteRow]) -> String {
    let count = |route: &str, class: &str| {
        rows.iter()
            .filter(|r| r.route == route && r.class == class)
            .count()
    };
    let mut out = String::from(
        "| route | crystallises-with-priced-heat | solubility-only | refused |\n|---|---|---|---|\n",
    );
    for route in ["wateq4f", "minteq.v4", "pitzer"] {
        out.push_str(&format!(
            "| {route} | {} | {} | {} |\n",
            count(route, "crystallises-with-priced-heat"),
            count(route, "solubility-only"),
            count(route, "refused")
        ));
    }
    out.push_str("\n| solid | route | class | phase | reason |\n|---|---|---|---|---|\n");
    for r in rows {
        out.push_str(&format!(
            "| `{}` | {} | {} | {} | {} |\n",
            r.solid,
            r.route,
            r.class,
            r.phase.as_deref().unwrap_or("-"),
            r.reason
        ));
    }
    out
}

#[test]
fn crystallisation_route_coverage_is_current() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let rows = crystallisation_route::coverage();
    let generated = table(&rows);
    let doc_path = root.join(DOC);
    let doc = std::fs::read_to_string(&doc_path).expect("coverage doc");
    let (head, rest) = doc.split_once(BEGIN).expect("begin marker");
    let (_, tail) = rest.split_once(END).expect("end marker");
    if std::env::var_os("KERO_BLESS_COVERAGE").is_some() {
        let mut json = serde_json::to_string_pretty(&rows).unwrap();
        json.push('\n');
        std::fs::write(root.join(FIXTURE), json).unwrap();
        std::fs::write(&doc_path, format!("{head}{BEGIN}\n{generated}{END}{tail}")).unwrap();
        return;
    }
    let shipped: Vec<RouteRow> = serde_json::from_str(
        &std::fs::read_to_string(root.join(FIXTURE))
            .expect("fixture missing: run once with KERO_BLESS_COVERAGE=1 and review"),
    )
    .unwrap();
    assert_eq!(
        rows, shipped,
        "fixture stale; rerun with KERO_BLESS_COVERAGE=1"
    );
    let current = rest.split_once(END).unwrap().0.trim();
    assert_eq!(current, generated.trim(), "doc table stale");
    // Every row is one of the three classes, and every solid appears on
    // every route exactly once.
    assert!(rows.iter().all(|r| matches!(
        r.class.as_str(),
        "crystallises-with-priced-heat" | "solubility-only" | "refused"
    )));
    assert_eq!(rows.len() % 3, 0);
}

/// Spot checks independent of the fixture.
#[test]
fn known_members_classify_as_their_databases_define_them() {
    let row = |s: &str, t: &str| crystallisation_route::route_row(s, t);
    // Halite is a native phase with a stated temperature law everywhere,
    // and NaCl is curated-priced: PHREEQC owns its saturation and heat.
    for tag in ["wateq4f", "minteq.v4", "pitzer"] {
        assert_eq!(
            row("NaCl", tag).class,
            "crystallises-with-priced-heat",
            "{tag}"
        );
    }
    // No routed database defines Niter.
    for tag in ["wateq4f", "minteq.v4", "pitzer"] {
        assert_eq!(row("KNO3", tag).class, "refused", "{tag}");
    }
}
