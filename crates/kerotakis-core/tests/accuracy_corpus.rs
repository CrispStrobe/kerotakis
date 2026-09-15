//! The accuracy corpus is a BACKLOG, and this file is the only thing that
//! checks it. Read what it does not do first.
//!
//! **It does not run the bench and it does not compare a model value against a
//! measurement.** That is deliberate and it is the owner's decision: promotion
//! to a continuous-integration gate comes only once the tolerances have
//! survived argument by someone other than their author. A gate added on the
//! day the tolerances were invented would be the third check this month that
//! read stronger than it was.
//!
//! What it does instead is hold the corpus to its own rules, which is cheap,
//! needs no engine, and catches the failure mode that actually threatens this
//! kind of file: a row that looks cited and is not. Every band must have a
//! reason, every reason must be a sentence rather than a shrug, every source
//! id must resolve, every source must declare whether anyone actually read the
//! number off the original, and the published headline must match what the
//! rows add up to.
//!
//! That last one is the anti-gaming rule made mechanical. The number this
//! corpus publishes is QUANTITIES COVERED, never rows passing, because rows
//! passing can be inflated by adding easy rows. `quantities_covered` is a
//! declared field and this file recomputes it, so the declaration cannot drift
//! away from the rows while nobody is looking.

use std::collections::BTreeSet;
use std::path::PathBuf;

fn corpus_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../validation/cases/colligative.toml")
}

fn corpus() -> toml::Value {
    let text = std::fs::read_to_string(corpus_path()).expect("the colligative accuracy corpus");
    text.parse::<toml::Value>().expect("the corpus parses")
}

fn cases(root: &toml::Value) -> Vec<&toml::value::Table> {
    root["case"]
        .as_array()
        .expect("cases")
        .iter()
        .map(|c| c.as_table().expect("a case is a table"))
        .collect()
}

fn quantities(case: &toml::value::Table) -> Vec<&toml::value::Table> {
    case["quantity"]
        .as_array()
        .expect("every case has at least one quantity")
        .iter()
        .map(|q| q.as_table().expect("a quantity is a table"))
        .collect()
}

fn is_open(q: &toml::value::Table) -> bool {
    q.get("status").and_then(|s| s.as_str()) == Some("open")
}

/// A row that cites nothing, or cites something this file cannot resolve, is
/// the whole thing the corpus exists to prevent. So every `source_id` and
/// every `primary_source_id` must name a source declared in the same file.
#[test]
fn every_row_names_a_source_that_resolves() {
    let root = corpus();
    let declared: BTreeSet<&str> = root["source"]
        .as_array()
        .expect("sources")
        .iter()
        .map(|s| s["id"].as_str().expect("a source id"))
        .collect();
    assert!(
        declared.len() >= 5,
        "a family citing fewer than five sources is probably leaning on one \
         compilation, which is the thing the provenance rule refuses"
    );

    for case in cases(&root) {
        let case_id = case["id"].as_str().unwrap();
        for q in quantities(case) {
            let name = q["name"].as_str().expect("a quantity name");
            for field in ["source_id", "primary_source_id"] {
                if let Some(id) = q.get(field).and_then(|v| v.as_str()) {
                    assert!(
                        declared.contains(id),
                        "{case_id}/{name} cites `{id}` in {field}, which is not \
                         declared in this corpus"
                    );
                }
            }
            assert!(
                q.contains_key("source_id"),
                "{case_id}/{name} has no source_id at all"
            );
        }
    }
}

/// Tolerance is argued per quantity, never a flat percentage. A `tolerance`
/// without a `tolerance_reason` is a number somebody picked, and it is
/// indistinguishable in a count from one somebody argued — which is why the
/// two live in separate fields and why widening one has to show in a diff
/// without the other moving.
#[test]
fn every_band_carries_an_argument_and_names_what_it_excludes() {
    let root = corpus();
    for case in cases(&root) {
        let case_id = case["id"].as_str().unwrap();
        for q in quantities(case) {
            let name = q["name"].as_str().unwrap();
            let reason = q
                .get("tolerance_reason")
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| panic!("{case_id}/{name} has no tolerance_reason"));
            if is_open(q) {
                continue;
            }
            assert!(
                reason.len() > 120,
                "{case_id}/{name}'s tolerance_reason is {} characters, which is \
                 too short to be an argument",
                reason.len()
            );
            // The rule the tolerance policy states: wide enough to survive an
            // improvement, narrow enough to exclude the known-wrong
            // predecessor. A band that names neither has not been argued
            // against anything.
            let lower = reason.to_ascii_lowercase();
            assert!(
                lower.contains("predecessor") || lower.contains("excludes"),
                "{case_id}/{name}'s tolerance_reason never says what the band \
                 EXCLUDES, so its width cannot be checked"
            );
            for field in ["model_tolerance", "world_tolerance"] {
                let t = q
                    .get(field)
                    .and_then(|v| v.as_float())
                    .unwrap_or_else(|| panic!("{case_id}/{name} has no {field}"));
                assert!(t > 0.0, "{case_id}/{name}'s {field} is not positive");
            }
        }
    }
}

/// The two bands must not contradict each other. This is arithmetic on the
/// file alone — no engine, no bench — but it catches a row whose recorded
/// model figure is already outside its own world band, which would mean the
/// row was written from two different states of the code.
#[test]
fn the_recorded_model_figure_sits_inside_its_own_world_band() {
    let root = corpus();
    for case in cases(&root) {
        let case_id = case["id"].as_str().unwrap();
        for q in quantities(case) {
            if is_open(q) {
                continue;
            }
            let name = q["name"].as_str().unwrap();
            let model = q["model_value"].as_float().expect("model_value");
            let world = q["world_value"].as_float().expect("world_value");
            let band = q["world_tolerance"].as_float().expect("world_tolerance");
            assert!(
                (model - world).abs() <= band,
                "{case_id}/{name}: the recorded model figure {model} is outside \
                 its own world band {world} +/- {band}. Either the band is \
                 wrong or the file was written from two different states of \
                 the engine."
            );
        }
    }
}

/// A citation whose numbers nobody read is still a citation, and it is much
/// better than the bare "by measurement" this repository was carrying. But it
/// is a WEAKER claim than a verified transcription, and the difference has to
/// be in a field rather than in a reader's assumption.
#[test]
fn every_source_says_whether_anyone_read_the_number() {
    let root = corpus();
    for s in root["source"].as_array().expect("sources") {
        let id = s["id"].as_str().unwrap();
        for field in ["citation", "kind", "access", "transcription", "licence"] {
            let v = s
                .get(field)
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| panic!("source `{id}` has no {field}"));
            assert!(!v.trim().is_empty(), "source `{id}`'s {field} is empty");
        }
        // A primary measurement is identified by its digital object
        // identifier where it has one. A book need not have one, and saying
        // it must was the 2026-09-13 mistake this corpus exists downstream of
        // — so the requirement is on papers only, and the book is required to
        // carry its edition and publisher instead.
        let kind = s["kind"].as_str().unwrap();
        let citation = s["citation"].as_str().unwrap();
        if kind == "compilation" {
            assert!(
                citation.contains("edition") && citation.contains("19"),
                "the compilation `{id}` must be cited as a book: authors, \
                 title, edition, publisher, year, table"
            );
        } else {
            let doi = s.get("doi").and_then(|v| v.as_str()).unwrap_or("");
            assert!(
                doi.starts_with("10."),
                "`{id}` is a {kind} and carries no resolvable identifier"
            );
        }
    }
}

/// The published number is quantities covered, not rows passing, and this
/// test is what stops the two drifting apart. Adding five easy rows to the
/// same quantity moves `anchored_rows` and must leave `quantities_covered`
/// exactly where it was.
#[test]
fn the_headline_is_quantities_covered_and_it_matches_the_rows() {
    let root = corpus();
    assert_eq!(
        root["headline_metric"].as_str(),
        Some("quantities_covered"),
        "the headline may not become rows passing"
    );
    assert_eq!(
        root["gate"].as_bool(),
        Some(false),
        "this corpus is a backlog; promoting it to a gate is an owner decision \
         and needs the tolerances argued by someone other than their author"
    );

    let mut anchored = 0usize;
    let mut open = 0usize;
    let mut covered: BTreeSet<&str> = BTreeSet::new();
    for case in cases(&root) {
        for q in quantities(case) {
            let name = q["name"].as_str().unwrap();
            if is_open(q) {
                open += 1;
            } else {
                anchored += 1;
                covered.insert(name);
            }
        }
    }

    assert_eq!(
        root["quantities_covered"].as_integer(),
        Some(covered.len() as i64),
        "the declared quantities_covered has drifted from the rows: {covered:?}"
    );
    assert_eq!(
        root["anchored_rows"].as_integer(),
        Some(anchored as i64),
        "the declared anchored_rows has drifted from the rows"
    );
    assert_eq!(
        root["open_rows"].as_integer(),
        Some(open as i64),
        "the declared open_rows has drifted from the rows"
    );
}

/// Agreement between two paths that share a database is not independent
/// validation of that database. Three of this family's six rows are osmotic
/// coefficients compared against the same class of measurement the bench's
/// virial coefficients are FITTED to, and a corpus that did not record that
/// would be overstating half its evidence.
#[test]
fn every_row_declares_whether_it_is_independent_of_the_path_it_tests() {
    let root = corpus();
    let mut dependent = 0usize;
    for case in cases(&root) {
        let case_id = case["id"].as_str().unwrap();
        for q in quantities(case) {
            let name = q["name"].as_str().unwrap();
            let independent = q
                .get("independent_of_path")
                .and_then(|v| v.as_bool())
                .unwrap_or_else(|| {
                    panic!("{case_id}/{name} does not say whether it is independent")
                });
            if !independent {
                dependent += 1;
                assert!(
                    q.get("independence_note")
                        .and_then(|v| v.as_str())
                        .is_some_and(|n| n.trim().len() > 40),
                    "{case_id}/{name} is not independent of the path it tests \
                     and must say what the shared ancestry is"
                );
            }
        }
    }
    assert!(
        dependent > 0,
        "a family where every row claims independence has probably not \
         checked; the osmotic-coefficient rows here share ancestry with \
         pitzer.dat's fitted coefficients and say so"
    );
}

/// The corpus has to say what a pass would NOT prove, in the file, where
/// somebody quoting a score will see it. This is the third measure this
/// project has built and the first two both read stronger than they were.
#[test]
fn the_corpus_states_the_limits_of_its_own_score() {
    let root = corpus();
    let proves = root["proves"].as_str().expect("proves");
    let not = root["does_not_prove"].as_str().expect("does_not_prove");
    assert!(
        not.len() > proves.len(),
        "the list of what a pass does not prove should be the longer of the \
         two, and if it ever is not, that is the finding"
    );
    for required in ["between the rows", "right reason", "transcri"] {
        assert!(
            not.to_ascii_lowercase().contains(required),
            "does_not_prove never mentions `{required}`"
        );
    }
}

/// A band argued against an input's uncertainty has to be arguing against the
/// input the bench actually uses.
///
/// The `registry_input` rows added on 2026-09-15 are the first thing in this
/// corpus that quotes a number out of
/// `data/registry/registry-source-v1.json`, and a quoted number is a copy
/// that can go stale. So every row is checked against the shipped registry:
/// the record must exist, its value must match, and where the corpus claims
/// an interval the registry must carry the same one. A row that says
/// `unestablished` is checked just as hard in the other direction — if
/// somebody gives that record a band and does not come back here, the corpus
/// is understating what it can now argue, which is the quieter of the two
/// failures and the one worth catching.
#[test]
fn every_declared_registry_input_still_matches_the_shipped_registry() {
    let registry: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../data/registry/registry-source-v1.json"),
        )
        .expect("the shipped registry"),
    )
    .expect("the registry parses");
    let records = registry["phase_thermodynamics"]
        .as_array()
        .expect("phase thermodynamics");

    let root = corpus();
    let inputs = root["registry_input"]
        .as_array()
        .expect("the declared registry inputs");
    assert!(
        !inputs.is_empty(),
        "a family that declares no inputs has not looked at what its model \
         bands rest on"
    );

    for input in inputs {
        let id = input["record_id"].as_str().expect("a record_id");
        let record = records
            .iter()
            .find(|record| record["id"].as_str() == Some(id))
            .unwrap_or_else(|| panic!("`{id}` is not a record in the shipped registry"));
        let value = input["value"].as_float().expect("a value");
        let shipped = record["quantity"]["value"].as_f64().expect("a value");
        assert!(
            (value - shipped).abs() < 1e-9,
            "`{id}` is {shipped} in the registry and {value} here"
        );

        let declared = input["uncertainty"].as_str().expect("an uncertainty");
        let kind = record["quantity"]["uncertainty"]["kind"]
            .as_str()
            .expect("an uncertainty kind");
        assert_eq!(
            declared, kind,
            "`{id}` carries `{kind}` in the registry and `{declared}` here"
        );
        assert!(
            input
                .get("basis")
                .and_then(|v| v.as_str())
                .is_some_and(|b| b.trim().len() > 120),
            "`{id}` gives no account of where its band comes from, or of why \
             it has none"
        );
        assert!(
            input.get("contribution").and_then(|v| v.as_str()).is_some(),
            "`{id}` does not say what it contributes to the bands below, and \
             `nothing` is an answer worth writing down"
        );

        if kind == "interval" {
            for bound in ["lower", "upper"] {
                let declared = input[bound].as_float().expect("a bound");
                let shipped = record["quantity"]["uncertainty"][bound]
                    .as_f64()
                    .expect("a bound");
                assert!(
                    (declared - shipped).abs() < 1e-9,
                    "`{id}`'s {bound} bound is {shipped} in the registry and \
                     {declared} here"
                );
            }
        }
    }
}
