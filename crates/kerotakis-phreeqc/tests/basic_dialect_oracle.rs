#![cfg(all(
    feature = "engine",
    any(feature = "legacy-basic-oracle", feature = "my-basic-preview")
))]

use kerotakis_phreeqc::{databases, Phreeqc};

#[test]
fn user_punch_values_match_native_phreeqc_results() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    engine
        .run(
            "SOLUTION 1\n\
                 units mol/kgw\n\
                 pH 7\n\
                 Na 0.01\n\
             SELECTED_OUTPUT\n\
                 -reset false\n\
                 -high_precision true\n\
             USER_PUNCH\n\
                 -headings h_activity sodium marker delta_h\n\
             10 PUNCH ACT(\"H+\"), TOT(\"Na\"), \"ok\", DELTA_H_SPECIES(\"OH-\")\n\
             END\n",
        )
        .unwrap();

    let expected_delta_h = engine.species_delta_h("OH-").unwrap();
    let rows = engine.selected_output();
    let headings = rows.first().expect("selected-output headings");
    let values = rows.last().expect("selected-output values");
    let value = |name: &str| {
        let index = headings.iter().position(|heading| heading == name).unwrap();
        values[index].as_str()
    };
    assert!(value("h_activity").parse::<f64>().unwrap() > 0.0);
    assert!((value("sodium").parse::<f64>().unwrap() - 0.01).abs() < 1e-8);
    assert_eq!(value("marker"), "ok");
    let punched_delta_h = value("delta_h").parse::<f64>().unwrap();
    assert!(
        // PHREEQC's USER_PUNCH path currently formats this field to three
        // decimal places even when selected output requests high precision.
        (punched_delta_h - expected_delta_h).abs() <= 5e-4,
        "PUNCH {punched_delta_h}, native {expected_delta_h}"
    );
}

#[test]
fn calculate_values_saves_a_native_chemistry_result() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    engine
        .run(
            "CALCULATE_VALUES\n\
                 DoubleNa\n\
                 -start\n\
                 10 SAVE TOT(\"Na\") * 2\n\
                 -end\n\
             SOLUTION 1\n\
                 units mol/kgw\n\
                 pH 7\n\
                 Na 0.01\n\
             SELECTED_OUTPUT\n\
                 -reset false\n\
                 -calculate_values DoubleNa\n\
             END\n",
        )
        .unwrap();

    let value = engine.last_value("V_DoubleNa").expect("calculated value");
    assert!((value - 0.02).abs() < 1e-8, "calculated value: {value}");
}

#[test]
fn user_print_is_routed_to_the_phreeqc_output_sink() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    engine
        .run(
            "SOLUTION 1\n\
                 pH 7\n\
             PRINT\n\
                 -reset false\n\
                 -user_print true\n\
             USER_PRINT\n\
                 10 PRINT \"KEROTAKIS_BASIC_MARKER\"\n\
             END\n",
        )
        .unwrap();

    assert!(engine.output_string().contains("KEROTAKIS_BASIC_MARKER"));
}
