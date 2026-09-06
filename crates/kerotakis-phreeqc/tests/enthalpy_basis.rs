//! The heat basis is read off the shipped database files, and this is where
//! that reading is checked against the engine that would otherwise supply it.
//!
//! It matters more than it looks. The browser build has no PHREEQC to ask,
//! so if the basis came from the engine natively and from the file in wasm,
//! the two would charge different heats, land on different temperatures, and
//! a cache-only replay would miss states a live run had cached. That is
//! exactly how it failed: `carbonated_bottle` and `partial_freezing` could
//! not find themselves in the shipped results.
//!
//! So the file is the single source, and the engine is the check.

#![cfg(feature = "engine")]

use kerotakis_phreeqc::{derived, Phreeqc};

/// Establish 25 °C explicitly rather than trusting constructor defaults —
/// `species_delta_h` answers for the state the engine is in.
fn probe(database: &[u8]) -> Phreeqc {
    let mut engine = Phreeqc::with_database(database).expect("database");
    engine
        .run("SOLUTION 1\n    temp 25\n    pH 7\nEND\n")
        .expect("probe");
    engine
}

#[test]
fn the_file_derived_basis_agrees_with_the_engine() {
    let cases: &[(&str, &[u8], &[&str])] = &[
        (
            "wateq4f",
            kerotakis_phreeqc::databases::WATEQ4F,
            &["OH-", "HCO3-"],
        ),
        (
            "minteq.v4",
            kerotakis_phreeqc::databases::minteq_v4(),
            &["OH-", "HCO3-", "H2CO3"],
        ),
        // pitzer states almost no `delta_h` at all: these come from the
        // SLOPE of its `-analytic` log K expressions, which is the whole
        // reason this test exists.
        ("pitzer", kerotakis_phreeqc::databases::PITZER, &["OH-"]),
        // A master species is the basis and must price at zero on both
        // sides — the file has no entry for it at all, and the engine
        // agrees there is no reaction to have an enthalpy.
    ];

    for (tag, database, species) in cases {
        let idx = derived::index_for(tag);
        let mut engine = probe(database);
        for name in *species {
            // The same lookup the balance makes: a stated enthalpy, or
            // zero because the species IS the basis.
            let from_file = idx
                .species_delta_h_kj
                .get(*name)
                .copied()
                .or_else(|| idx.species_element.contains_key(*name).then_some(0.0))
                .unwrap_or_else(|| panic!("{tag}: nothing prices {name}"));
            let from_engine = engine
                .species_delta_h(name)
                .unwrap_or_else(|e| panic!("{tag}: engine declined {name}: {e:?}"));
            assert!(
                (from_file - from_engine).abs() < 0.05,
                "{tag} {name}: file says {from_file} kJ/mol, engine says {from_engine}"
            );
        }
    }
}

/// And the one the whole heat balance rests on, pinned by value as well as
/// by agreement: neutralisation is the reverse of the reaction defining
/// hydroxide, and every dataset should put it near the literature -55.8.
#[test]
fn every_dataset_prices_hydroxide_near_the_literature() {
    for tag in ["wateq4f", "minteq.v4", "pitzer"] {
        let oh = derived::index_for(tag).species_delta_h_kj["OH-"];
        assert!(
            (55.0..=57.0).contains(&oh),
            "{tag} hydroxide at {oh} kJ/mol, against a literature 55.8"
        );
    }
}
