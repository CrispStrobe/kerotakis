#![cfg(feature = "engine")]

use kerotakis_phreeqc::{databases, Phreeqc, PhreeqcError};

unsafe extern "C" {
    fn GetSpeciesDeltaH(id: i32, name: *const std::os::raw::c_char, delta_h: *mut f64) -> i32;
}

#[test]
fn raw_bridge_rejects_an_invalid_instance_without_writing_output() {
    let name = c"OH-";
    let mut value = 1234.5;
    let status = unsafe { GetSpeciesDeltaH(i32::MAX, name.as_ptr(), &mut value) };

    assert_eq!(status, -6, "expected IPQ_BADINSTANCE");
    assert_eq!(value, 1234.5, "a failed call must not fabricate a value");
}

#[test]
fn safe_bridge_rejects_unknown_species_and_returns_finite_values() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    engine
        .run("SOLUTION 1\n    temp 25\n    pH 7\nEND\n")
        .unwrap();

    let error = engine.species_delta_h("DefinitelyNotASpecies").unwrap_err();
    assert!(matches!(error, PhreeqcError::UnknownSpecies(_)));

    let value = engine.species_delta_h("OH-").unwrap();
    assert!(value.is_finite());
    assert!(value > 0.0);
}

#[test]
#[cfg(feature = "legacy-basic-oracle")]
fn native_delta_h_matches_the_legacy_basic_oracle_for_embedded_databases() {
    let cases = [
        ("phreeqc", databases::PHREEQC),
        ("wateq4f", databases::WATEQ4F),
        ("minteq.v4", databases::MINTEQ_V4),
        ("pitzer", databases::PITZER),
    ];

    for (name, database) in cases {
        let mut engine = Phreeqc::with_database(database).unwrap();
        engine
            .run("SOLUTION 1\n    temp 25\n    pH 7\nEND\n")
            .unwrap();
        let native = engine.species_delta_h("OH-").unwrap();

        engine
            .run(
                "SOLUTION 1\n    temp 25\n    pH 7\n\
                 SELECTED_OUTPUT\n    -reset false\n\
                 USER_PUNCH\n    -headings dh\n\
                 10 PUNCH DELTA_H_SPECIES(\"OH-\")\nEND\n",
            )
            .unwrap();
        let basic = engine.last_value("dh").unwrap();

        assert!(
            // Stock selected output prints three decimals here, so its
            // half-unit-in-last-place is the tightest meaningful oracle.
            (native - basic).abs() <= 5e-4,
            "{name}: native {native} differs from BASIC oracle {basic}"
        );
    }
}

#[test]
fn native_delta_h_uses_the_current_solution_temperature() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    engine
        .run("SOLUTION 1\n    temp 25\n    pH 7\nEND\n")
        .unwrap();
    let at_25_c = engine.species_delta_h("OH-").unwrap();

    engine
        .run("SOLUTION 1\n    temp 50\n    pH 7\nEND\n")
        .unwrap();
    let at_50_c = engine.species_delta_h("OH-").unwrap();

    assert!(
        (at_25_c - at_50_c).abs() > 1e-3,
        "reaction enthalpy should track the current temperature: {at_25_c} vs {at_50_c}"
    );
}
