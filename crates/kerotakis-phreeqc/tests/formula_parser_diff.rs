//! OPT-8's proof and its permanent residue. Two independent formula
//! parsers used to exist; the pre-unification differential over all 641
//! formulas in the shipped databases found zero numeric disagreements
//! and exactly one dialect difference — PHREEQC's pseudo-element master
//! species (Cyanide, Edta, X, the DOM fragments…), which the textbook
//! dialect rightly refuses and the database dialect rightly accepts.
//! The second parser is now an adapter; this test keeps the corpus as a
//! regression harness and holds the dialect boundary in both
//! directions.

use kerotakis_core::stoich;
use kerotakis_phreeqc::databases;
use kerotakis_phreeqc::dbindex::{canon_element, parse_formula as db_parse, split_hydrate};
use std::collections::BTreeMap;

fn corpus() -> Vec<String> {
    let mut formulas = Vec::new();
    for db in [
        databases::PHREEQC,
        databases::WATEQ4F,
        databases::MINTEQ_V4,
        databases::PITZER,
    ] {
        let text = String::from_utf8_lossy(db);
        for line in text.lines() {
            let line = line.trim();
            // Phase-equation shape: "Formula = products…" with the
            // formula as the first LHS term — the same harvest
            // dbindex::parse_phase_equation performs.
            let Some((lhs, _)) = line.split_once('=') else {
                continue;
            };
            let Some(first) = lhs.split(" + ").next() else {
                continue;
            };
            let token = first.trim();
            if token.is_empty() || token.contains(' ') || token.starts_with('#') {
                continue;
            }
            let (base, _waters) = split_hydrate(token);
            formulas.push(base);
        }
    }
    formulas.sort();
    formulas.dedup();
    formulas
}

#[test]
fn the_dialects_agree_numerically_and_differ_only_on_pseudo_elements() {
    let mut compared = 0usize;
    let mut disagreements: Vec<String> = Vec::new();
    for f in corpus() {
        // The working set is what dbindex accepts today; formulas it
        // rejects never reach its composition path.
        let Some(db_map) = db_parse(&f) else {
            continue;
        };
        match stoich::parse_formula_with(&f, stoich::FormulaDialect::PhreeqcMaster) {
            Ok(st) => {
                let st_map: BTreeMap<String, f64> = st
                    .counts
                    .iter()
                    .map(|(k, v)| (canon_element(k), *v))
                    .fold(BTreeMap::new(), |mut m, (k, v)| {
                        *m.entry(k).or_insert(0.0) += v;
                        m
                    });
                let db_canon: BTreeMap<String, f64> = db_map
                    .iter()
                    .map(|(k, v)| (canon_element(k), *v))
                    .fold(BTreeMap::new(), |mut m, (k, v)| {
                        *m.entry(k).or_insert(0.0) += v;
                        m
                    });
                if st_map != db_canon {
                    disagreements.push(format!("{f}: stoich {st_map:?} vs dbindex {db_canon:?}"));
                }
            }
            Err(e) => disagreements.push(format!("{f}: stoich refuses ({e}); dbindex {db_map:?}")),
        }
        compared += 1;
    }
    // The boundary holds from the other side too: the textbook dialect
    // must go on refusing the pseudo-elements a learner should never
    // meet as chemistry.
    for pseudo in ["Cyanide-", "Edta-4", "Butylamine", "X-", "Hdg"] {
        assert!(
            stoich::parse_formula(pseudo).is_err(),
            "textbook dialect must refuse pseudo-element '{pseudo}'"
        );
    }
    eprintln!("compared {compared} database formulas");
    assert!(compared > 300, "the corpus should be large; got {compared}");
    assert!(
        disagreements.is_empty(),
        "{} disagreements:\n{}",
        disagreements.len(),
        disagreements.join("\n")
    );
}
