#![cfg(all(
    feature = "engine",
    feature = "my-basic-preview",
    not(feature = "legacy-basic-oracle")
))]

use kerotakis_phreeqc::{databases, Phreeqc};

const SIMPLE_RATE: &str = "RATES\n\
                          KeroNoOp\n\
                          -start\n\
                          10 moles = M * TIME\n\
                          20 SAVE moles\n\
                          -end\n\
                          SOLUTION 1\n\
                              pH 7\n\
                          KINETICS 1\n\
                              KeroNoOp\n\
                                  -formula H2O 0\n\
                                  -m 1\n\
                                  -m0 1\n\
                                  -steps 0.25 seconds\n\
                          END\n";

#[test]
fn numbered_rate_program_receives_runtime_values_and_saves_moles() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    engine
        .run(
            "RATES\n\
             KeroNoOp\n\
             -start\n\
             10 note$ = \"semicolon; stays in string\"\n\
             20 REM GOTO 999 stays in comment\n\
             30 moles = -1\n\
             40 GOTO 60\n\
             50 moles = -2\n\
             60 rate_value = M * TIME\n\
             70 SAVE rate_value\n\
             -end\n\
             SOLUTION 1\n\
                 pH 7\n\
             KINETICS 1\n\
                 KeroNoOp\n\
                     -formula H2O 0\n\
                     -m 1\n\
                     -m0 1\n\
                     -steps 0.25 seconds\n\
             END\n",
        )
        .unwrap();
}

#[test]
fn malformed_program_reports_my_basic_error_instead_of_falling_back() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    let error = engine
        .run(
            "RATES\n\
             Broken\n\
             -start\n\
             10 IF THEN\n\
             -end\n\
             SOLUTION 1\n\
                 pH 7\n\
             KINETICS 1\n\
                 Broken\n\
                     -formula H2O 0\n\
                     -m 1\n\
                     -steps 1 second\n\
             END\n",
        )
        .unwrap_err()
        .to_string();
    assert!(error.contains("MY-BASIC compatibility"), "{error}");
}

#[test]
fn independent_instances_compile_run_and_clean_up_safely() {
    let handles: Vec<_> = (0..4)
        .map(|_| {
            std::thread::spawn(|| {
                let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
                engine.run(SIMPLE_RATE).unwrap();
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn native_chemistry_callbacks_and_parm_execute_in_a_rate_program() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    engine
        .run(
            "RATES\n\
             NativeChemistry\n\
             -start\n\
             10 callback_sum = ACT(\"H+\") + MOL(\"H+\") + TOT(\"Na\")\n\
             20 callback_sum = callback_sum + SI(\"Calcite\") + SR(\"Calcite\")\n\
             30 callback_sum = callback_sum + LM(\"H+\") + DELTA_H_SPECIES(\"OH-\")\n\
             40 SAVE callback_sum * 0 + PARM(1) * TIME\n\
             -end\n\
             SOLUTION 1\n\
                 pH 7\n\
                 Na 0.01\n\
                 Ca 0.001\n\
                 C 0.001 as HCO3\n\
             KINETICS 1\n\
                 NativeChemistry\n\
                     -formula H2O 0\n\
                     -m 1\n\
                     -m0 1\n\
                     -parms 0.5\n\
                     -steps 0.25 seconds\n\
             SELECTED_OUTPUT\n\
                 -reset false\n\
                 -kinetics NativeChemistry\n\
             END\n",
        )
        .unwrap();
    let remaining = engine
        .last_value("k_NativeChemistry")
        .expect("kinetics selected output");
    assert!(
        (remaining - 0.875).abs() < 1e-10,
        "remaining moles: {remaining}"
    );
}

#[test]
fn runaway_program_stops_at_the_statement_budget() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    let error = engine
        .run(
            "RATES\n\
             Runaway\n\
             -start\n\
             10 GOTO 10\n\
             -end\n\
             SOLUTION 1\n\
                 pH 7\n\
             KINETICS 1\n\
                 Runaway\n\
                     -formula H2O 0\n\
                     -m 1\n\
                     -steps 1 second\n\
             END\n",
        )
        .unwrap_err()
        .to_string();
    assert!(error.contains("statement budget exceeded"), "{error}");
    engine
        .run("SOLUTION 2\n    pH 7\nEND\n")
        .expect("engine remains usable after budget cancellation");
}

#[test]
fn log10_and_floor_evaluate_correctly() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    engine
        .run(
            "CALCULATE_VALUES\n\
                 LogFloorTest\n\
                 -start\n\
                 10 x = LOG10(1000)\n\
                 20 y = FLOOR(3.7)\n\
                 30 SAVE x + y\n\
                 -end\n\
             SOLUTION 1\n\
                 pH 7\n\
             SELECTED_OUTPUT\n\
                 -reset false\n\
                 -calculate_values LogFloorTest\n\
             END\n",
        )
        .unwrap();
    let value = engine
        .last_value("V_LogFloorTest")
        .expect("calculated value");
    assert!(
        (value - 6.0).abs() < 1e-10,
        "LOG10(1000)+FLOOR(3.7) = {value}"
    );
}

#[test]
fn calc_value_recursive_invocation_succeeds() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    engine
        .run(
            "CALCULATE_VALUES\n\
                 DoubleNa\n\
                 -start\n\
                 10 SAVE TOT(\"Na\") * 2\n\
                 -end\n\
             RATES\n\
                 CalcUser\n\
                 -start\n\
                 10 SAVE CALC_VALUE(\"DoubleNa\") * TIME\n\
                 -end\n\
             SOLUTION 1\n\
                 units mol/kgw\n\
                 pH 7\n\
                 Na 0.01\n\
             KINETICS 1\n\
                 CalcUser\n\
                     -formula H2O 0\n\
                     -m 1\n\
                     -m0 1\n\
                     -steps 0.5 seconds\n\
             SELECTED_OUTPUT\n\
                 -reset false\n\
                 -kinetics CalcUser\n\
             END\n",
        )
        .unwrap();
    let value = engine.last_value("k_CalcUser").expect("kinetics output");
    let expected = 1.0 - 0.02 * 0.5;
    assert!(
        (value - expected).abs() < 1e-6,
        "remaining after CALC_VALUE(DoubleNa)*TIME = {value}, expected {expected}"
    );
}

#[test]
fn calc_value_circular_reference_hits_recursion_budget() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    let error = engine
        .run(
            "CALCULATE_VALUES\n\
                 Circular\n\
                 -start\n\
                 10 SAVE CALC_VALUE(\"Circular\") + 1\n\
                 -end\n\
             SOLUTION 1\n\
                 pH 7\n\
             SELECTED_OUTPUT\n\
                 -reset false\n\
                 -calculate_values Circular\n\
             END\n",
        )
        .unwrap_err()
        .to_string();
    assert!(error.contains("recursion budget"), "{error}");
}

#[test]
fn data_read_restore_pattern_compiles_and_runs() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    engine
        .run(
            "CALCULATE_VALUES\n\
                 DataTest\n\
                 -start\n\
                 10 DATA 2.5, 3.5, 4.0\n\
                 20 RESTORE 10\n\
                 30 READ a, b, c\n\
                 40 REM GOTO 10 is not control flow\n\
                 50 note$ = \"GOTO 10\"\n\
                 60 SAVE a + b + c\n\
                 -end\n\
             SOLUTION 1\n\
                 pH 7\n\
             SELECTED_OUTPUT\n\
                 -reset false\n\
                 -calculate_values DataTest\n\
             END\n",
        )
        .unwrap();
    let value = engine.last_value("V_DataTest").expect("calculated value");
    assert!((value - 10.0).abs() < 1e-10, "DATA sum = {value}");
}

#[test]
fn data_cursor_can_be_restored_inside_control_flow() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    engine
        .run(
            "CALCULATE_VALUES\n\
                 DynamicData\n\
                 -start\n\
                 10 DATA 1, 2\n\
                 20 total = 0\n\
                 30 FOR i = 1 TO 2\n\
                 40 RESTORE 10\n\
                 50 READ a, b\n\
                 60 total = total + a + b\n\
                 70 NEXT i\n\
                 80 SAVE total\n\
                 -end\n\
             SOLUTION 1\n\
                 pH 7\n\
             SELECTED_OUTPUT\n\
                 -reset false\n\
                 -calculate_values DynamicData\n\
             END\n",
        )
        .unwrap();
    assert_eq!(engine.last_value("V_DynamicData"), Some(6.0));
}

#[test]
fn gfw_executes_in_a_rate_program() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    engine
        .run(
            "CALCULATE_VALUES\n\
                 GfwTest\n\
                 -start\n\
                 10 SAVE GFW(\"H2O\")\n\
                 -end\n\
             SOLUTION 1\n\
                 pH 7\n\
             SELECTED_OUTPUT\n\
                 -reset false\n\
                 -calculate_values GfwTest\n\
             END\n",
        )
        .unwrap();
    let value = engine.last_value("V_GfwTest").expect("calculated value");
    assert!((value - 18.015).abs() < 0.01, "GFW(H2O) = {value}");
}

#[test]
fn output_budget_stops_excessive_print() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    let error = engine
        .run(
            "SOLUTION 1\n\
                 pH 7\n\
             PRINT\n\
                 -user_print true\n\
             USER_PRINT\n\
                 10 FOR i = 1 TO 100000\n\
                 20 PRINT \"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\"\n\
                 30 NEXT i\n\
             END\n",
        )
        .unwrap_err()
        .to_string();
    assert!(error.contains("output budget"), "{error}");
}
