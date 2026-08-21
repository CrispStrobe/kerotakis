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
                 -headings h_activity sodium marker delta_h calcite_formula\n\
             10 PUNCH ACT(\"H+\"), TOT(\"Na\"), \"ok\", DELTA_H_SPECIES(\"OH-\"), PHASE_FORMULA$(\"Calcite\")\n\
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
    assert_eq!(value("calcite_formula"), "CaCO3");
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

#[test]
fn extended_numeric_callbacks_match_phreeqc_state() {
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
                 -headings log_h gfw_water sum_na\n\
                 10 PUNCH LA(\"H+\"), GFW(\"H2O\"), SUM_SPECIES(\"Na*\")\n\
             END\n",
        )
        .unwrap();

    let log_h = engine.last_value("log_h").expect("LA output");
    let gfw_water = engine.last_value("gfw_water").expect("GFW output");
    let sum_na = engine.last_value("sum_na").expect("SUM_SPECIES output");
    assert!((log_h + 7.0).abs() < 1e-8, "LA(H+)={log_h}");
    assert!((gfw_water - 18.01528).abs() < 1e-3, "GFW(H2O)={gfw_water}");
    assert!((sum_na - 0.01).abs() < 1e-8, "SUM_SPECIES(Na*)={sum_na}");
}

#[test]
fn assemblage_callbacks_return_finite_moles() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    engine
        .run(
            "SOLUTION 1\n\
                 units mol/kgw\n\
                 pH 7\n\
                 Ca 0.001\n\
                 C 0.002\n\
             EQUILIBRIUM_PHASES 1\n\
                 Calcite 0 0.001\n\
             GAS_PHASE 1\n\
                 -fixed_volume\n\
                 -volume 1\n\
                 CO2(g) -3.5\n\
             SELECTED_OUTPUT\n\
                 -reset false\n\
                 -high_precision true\n\
             USER_PUNCH\n\
                 -headings calcite_moles co2_moles sum_co2\n\
                 10 PUNCH EQUI(\"Calcite\"), GAS(\"CO2(g)\"), SUM_GAS(\"CO2*\")\n\
             END\n",
        )
        .unwrap();

    let calcite = engine.last_value("calcite_moles").expect("EQUI output");
    let co2 = engine.last_value("co2_moles").expect("GAS output");
    let sum_co2 = engine.last_value("sum_co2").expect("SUM_GAS output");
    assert!(
        (calcite - 8.388311650956e-4).abs() <= 5e-9,
        "EQUI(Calcite)={calcite}"
    );
    assert!((co2 - 1.082666032533e-4).abs() <= 5e-9, "GAS(CO2(g))={co2}");
    assert!((sum_co2 - co2).abs() <= 5e-9, "SUM_GAS(CO2*)={sum_co2}");
}

#[test]
fn runtime_state_variables_match_the_legacy_backend() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    engine
        .run(
            "SOLUTION 7\n\
                 temp 25\n\
                 pH 7\n\
             SELECTED_OUTPUT\n\
                 -reset false\n\
                 -high_precision true\n\
             USER_PUNCH\n\
                 -headings basic_tc basic_tk basic_cell solution_volume basic_sim_time basic_total_time\n\
                 10 PUNCH TC, TK, CELL_NO, SOLN_VOL, SIM_TIME, TOTAL_TIME\n\
             END\n",
        )
        .unwrap();

    let value = |heading| engine.last_value(heading).unwrap();
    assert!((value("basic_tc") - 25.0).abs() < 1e-10);
    assert!((value("basic_tk") - 298.15).abs() < 1e-10);
    assert!((value("basic_cell") - 7.0).abs() < 1e-10);
    assert!((value("solution_volume") - 1.00296575755).abs() < 1e-10);
    assert!(value("basic_sim_time").abs() < 1e-10);
    assert!(value("basic_total_time").abs() < 1e-10);
}

#[test]
fn phreeqc_string_function_spellings_match_the_legacy_backend() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    engine
        .run(
            "CALCULATE_VALUES\n\
                 StringFunctions\n\
                 -start\n\
                 10 s$ = CHR$(65) + MID$(\"xyz\", 2, 2)\n\
                 20 SAVE ASC(MID$(s$, 1, 1)) + VAL(STR$(LEN(s$))) + ASC(STR$(LEN(s$))) + ASC(STR$(-1))\n\
                 -end\n\
             SOLUTION 1\n\
                 pH 7\n\
             SELECTED_OUTPUT\n\
                 -reset false\n\
                 -calculate_values StringFunctions\n\
             END\n",
        )
        .unwrap();
    // PHREEQC STR$ reserves a sign column for positive values (ASCII space)
    // and starts negative values with '-'. ex21 relies on that field separator.
    assert_eq!(engine.last_value("V_StringFunctions"), Some(145.0));
}

#[test]
fn kinetics_moles_callback_matches_selected_kinetics() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    engine
        .run(
            "RATES\n\
                 KeroProbe\n\
                 -start\n\
                 10 SAVE 0\n\
                 -end\n\
             SOLUTION 1\n\
                 pH 7\n\
             KINETICS 1\n\
                 KeroProbe\n\
                     -formula H2O 0\n\
                     -m 0.123\n\
                     -m0 0.123\n\
                     -steps 1 second\n\
             SELECTED_OUTPUT\n\
                 -reset false\n\
                 -high_precision true\n\
             USER_PUNCH\n\
                 -headings probe_moles\n\
                 10 PUNCH KIN(\"KeroProbe\")\n\
             END\n",
        )
        .unwrap();
    assert!((engine.last_value("probe_moles").unwrap() - 0.123).abs() < 1e-10);
}

#[test]
fn phreeqc_math_aliases_match_the_legacy_backend() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    engine
        .run(
            "CALCULATE_VALUES\n\
                 MathAliases\n\
                 -start\n\
                 10 SAVE SQRT(9) + ARCTAN(1)\n\
                 -end\n\
             SOLUTION 1\n\
                 pH 7\n\
             SELECTED_OUTPUT\n\
                 -reset false\n\
                 -high_precision true\n\
                 -calculate_values MathAliases\n\
             END\n",
        )
        .unwrap();
    let expected = 3.0 + std::f64::consts::FRAC_PI_4;
    assert!((engine.last_value("V_MathAliases").unwrap() - expected).abs() < 1e-10);
}
