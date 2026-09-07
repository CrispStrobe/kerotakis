//! The bench's Cp(T) curves are NASA's file, not a retyping of it.
//!
//! `kerotakis-core` cannot depend on `kerotakis-cea` — the edge runs the
//! other way — so the curves in the registry are transcribed coefficients
//! rather than a call into `nasa9::db()`. That is exactly the arrangement
//! that rots quietly: a transposed digit in a registry row would change
//! every heat ledger on the bench and nothing would say so.
//!
//! This test is the join. It reads the same `thermo.inp` the equilibrium
//! solver reads, finds the record each registry interval names in its own
//! `reference` line, and demands the coefficients be bit-for-bit the ones
//! the parser found. It lives here because this is the only crate that can
//! see both sides.

use std::collections::BTreeMap;

use kerotakis_cea::nasa9;
use kerotakis_core::heat_capacity::CpForm;
use kerotakis_core::species::REGISTRY;

/// The record name a registry interval cites, e.g. `CaCO3(cr)` out of
/// `"CaCO3(cr): Hexagonal Gurvich,1996a pt1 p483 pt2 p376."`.
fn cited_record(reference: &str) -> Option<&str> {
    reference.split_once(": ").map(|(name, _)| name)
}

#[test]
fn every_registry_interval_names_a_record_in_the_vendored_file() {
    let db = nasa9::db();
    let mut checked = 0usize;
    for species in REGISTRY {
        for curve in species.heat_capacity_polys {
            for interval in curve.intervals {
                let name = cited_record(interval.reference).unwrap_or_else(|| {
                    panic!(
                        "{}: interval reference {:?} does not name a thermo.inp record",
                        species.key, interval.reference
                    )
                });
                assert!(
                    db.species.contains_key(name) || db.reactants.contains_key(name),
                    "{}: cites thermo.inp record {name:?}, which the parser did not find",
                    species.key
                );
                checked += 1;
            }
        }
    }
    assert!(checked > 60, "only {checked} intervals were checked");
}

#[test]
fn every_transcribed_coefficient_is_the_one_in_the_file() {
    // `ThermoDb` keys by name, so where thermo.inp carries two records under
    // one name — Fe(a) either side of its lambda transition, Fe2O3(cr)
    // either side of the Curie point — only the last survives the parse.
    // Those intervals cannot be checked here and are counted rather than
    // waved through: if the count of unmatched intervals ever grows, a
    // registry row has stopped naming a real interval.
    let db = nasa9::db();
    let mut matched = 0usize;
    let mut unmatched: BTreeMap<&str, Vec<(f64, f64)>> = BTreeMap::new();
    for species in REGISTRY {
        for curve in species.heat_capacity_polys {
            for interval in curve.intervals {
                assert_eq!(
                    interval.form,
                    CpForm::Nasa9,
                    "{}: this test only knows how to check NASA-9 rows; a Shomate \
                     row needs its own source check rather than this one",
                    species.key
                );
                let name = cited_record(interval.reference).expect("a cited record");
                let record = db
                    .species
                    .get(name)
                    .or_else(|| db.reactants.get(name))
                    .expect("the record exists");
                let found = record.intervals.iter().find(|i| {
                    (i.t_min - interval.t_min).abs() < 1e-9
                        && (i.t_max - interval.t_max).abs() < 1e-9
                });
                let Some(found) = found else {
                    unmatched
                        .entry(species.key)
                        .or_default()
                        .push((interval.t_min, interval.t_max));
                    continue;
                };
                for (position, (registry, file)) in interval
                    .coefficients
                    .iter()
                    .zip(found.coeffs.iter())
                    .enumerate()
                {
                    assert_eq!(
                        registry,
                        file,
                        "{} {name} {}-{} K: coefficient a{} is {registry} in the \
                         registry and {file} in vendor/nasa-cea/thermo.inp",
                        species.key,
                        interval.t_min,
                        interval.t_max,
                        position + 1
                    );
                }
                matched += 1;
            }
        }
    }
    assert!(
        matched > 55,
        "only {matched} intervals could be matched against the file"
    );
    let unmatched_count: usize = unmatched.values().map(Vec::len).sum();
    assert!(
        unmatched_count <= 5,
        "{unmatched_count} registry intervals name a record whose parsed copy \
         does not carry them: {unmatched:?}. Two are expected (Fe and Fe2O3 \
         are each written twice in thermo.inp under one name and the parser \
         keeps the last); more than that means a registry row has drifted"
    );
}

#[test]
fn the_two_evaluators_agree_where_they_can_both_answer() {
    // The registry has its own evaluator because core cannot call this one.
    // Two implementations of the same polynomial is exactly the sort of
    // duplication that diverges, so pin them to each other across the range
    // each interval actually covers.
    let db = nasa9::db();
    let mut compared = 0usize;
    for species in REGISTRY {
        for curve in species.heat_capacity_polys {
            for interval in curve.intervals {
                let name = cited_record(interval.reference).expect("a cited record");
                let record = db
                    .species
                    .get(name)
                    .or_else(|| db.reactants.get(name))
                    .expect("the record exists");
                let shared = record.intervals.iter().any(|i| {
                    (i.t_min - interval.t_min).abs() < 1e-9
                        && (i.t_max - interval.t_max).abs() < 1e-9
                });
                if !shared {
                    continue;
                }
                // Strictly inside: at a shared endpoint the two evaluators
                // legitimately pick different intervals, and NASA's fits are
                // continuous there but not identical to the last bit.
                for step in 1..4 {
                    let t =
                        interval.t_min + (interval.t_max - interval.t_min) * f64::from(step) / 4.0;
                    let ours = interval.cp(t);
                    let theirs = record.cp(t).expect("the record evaluates");
                    assert!(
                        (ours - theirs).abs() < 1e-9 * theirs.abs().max(1.0),
                        "{} {name} at {t} K: the registry evaluator says {ours} and \
                         kerotakis-cea says {theirs}",
                        species.key
                    );
                    compared += 1;
                }
            }
        }
    }
    assert!(compared > 120, "only {compared} points were compared");
}
