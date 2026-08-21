#![cfg(all(
    feature = "engine",
    feature = "my-basic-preview",
    not(feature = "legacy-basic-oracle")
))]

use kerotakis_phreeqc::{databases, Phreeqc};

#[test]
fn kinetics_rate_with_arithmetic_and_save() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    engine
        .run(
            "RATES\n\
             SimpleRate\n\
             -start\n\
             10 rate = PARM(1) * M * TIME\n\
             20 IF rate > M THEN rate = M\n\
             30 SAVE rate\n\
             -end\n\
             SOLUTION 1\n\
                 pH 7\n\
             KINETICS 1\n\
                 SimpleRate\n\
                     -formula H2O 0\n\
                     -m 1\n\
                     -m0 1\n\
                     -parms 0.1\n\
                     -steps 1 second\n\
             END\n",
        )
        .unwrap();
}

#[test]
fn for_next_loop_compiles_and_runs() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    engine
        .run(
            "CALCULATE_VALUES\n\
                 ForLoop\n\
                 -start\n\
                 10 total = 0\n\
                 20 FOR i = 1 TO 5\n\
                 30   total = total + i\n\
                 40 NEXT i\n\
                 50 SAVE total\n\
                 -end\n\
             SOLUTION 1\n\
                 pH 7\n\
             SELECTED_OUTPUT\n\
                 -reset false\n\
                 -calculate_values ForLoop\n\
             END\n",
        )
        .unwrap();
    let value = engine.last_value("V_ForLoop").expect("calculated value");
    assert!((value - 15.0).abs() < 1e-10, "sum(1..5) = {value}");
}

#[test]
fn while_wend_loop_compiles_and_runs() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    engine
        .run(
            "CALCULATE_VALUES\n\
                 WhileLoop\n\
                 -start\n\
                 10 n = 1\n\
                 20 WHILE n < 100\n\
                 30   n = n * 2\n\
                 40 WEND\n\
                 50 SAVE n\n\
                 -end\n\
             SOLUTION 1\n\
                 pH 7\n\
             SELECTED_OUTPUT\n\
                 -reset false\n\
                 -calculate_values WhileLoop\n\
             END\n",
        )
        .unwrap();
    let value = engine.last_value("V_WhileLoop").expect("calculated value");
    assert!((value - 128.0).abs() < 1e-10, "2^7 = {value}");
}

#[test]
fn gosub_return_compiles_and_runs() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    engine
        .run(
            "CALCULATE_VALUES\n\
                 GosubTest\n\
                 -start\n\
                 10 x = 5\n\
                 20 GOSUB 100\n\
                 30 SAVE x\n\
                 40 GOTO 200\n\
                 100 x = x * x\n\
                 110 RETURN\n\
                 200 REM done\n\
                 -end\n\
             SOLUTION 1\n\
                 pH 7\n\
             SELECTED_OUTPUT\n\
                 -reset false\n\
                 -calculate_values GosubTest\n\
             END\n",
        )
        .unwrap();
    let value = engine.last_value("V_GosubTest").expect("calculated value");
    assert!((value - 25.0).abs() < 1e-10, "5^2 = {value}");
}

#[test]
fn if_then_else_compiles_and_runs() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    engine
        .run(
            "CALCULATE_VALUES\n\
                 IfTest\n\
                 -start\n\
                 10 x = 10\n\
                 20 IF x > 5 THEN y = 1 ELSE y = 0\n\
                 30 SAVE y\n\
                 -end\n\
             SOLUTION 1\n\
                 pH 7\n\
             SELECTED_OUTPUT\n\
                 -reset false\n\
                 -calculate_values IfTest\n\
             END\n",
        )
        .unwrap();
    let value = engine.last_value("V_IfTest").expect("calculated value");
    assert!((value - 1.0).abs() < 1e-10, "IF 10>5 THEN 1 = {value}");
}

#[test]
fn string_operations_in_punch() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    engine
        .run(
            "SOLUTION 1\n\
                 pH 7\n\
             SELECTED_OUTPUT\n\
                 -reset false\n\
             USER_PUNCH\n\
                 -headings marker\n\
                 10 a$ = \"Hello\"\n\
                 20 b$ = \" World\"\n\
                 30 PUNCH a$ + b$\n\
             END\n",
        )
        .unwrap();
    let rows = engine.selected_output();
    let values = rows.last().expect("selected-output values");
    assert_eq!(values[0], "Hello World");
}

#[test]
fn unsupported_function_fails_at_compile_time() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    let error = engine
        .run(
            "CALCULATE_VALUES\n\
                 BadFunc\n\
                 -start\n\
                 10 SAVE SURF(\"Hfo_wOH\", \"mol\")\n\
                 -end\n\
             SOLUTION 1\n\
                 pH 7\n\
             SELECTED_OUTPUT\n\
                 -reset false\n\
                 -calculate_values BadFunc\n\
             END\n",
        )
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("MY-BASIC compatibility"),
        "unregistered function should fail: {error}"
    );
}

#[test]
fn user_graph_is_rejected_explicitly() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    let error = engine
        .run(
            "SOLUTION 1\n\
                 pH 7\n\
             USER_GRAPH 1\n\
                 -start\n\
                 10 GRAPH_X 1\n\
                 20 GRAPH_Y 2\n\
                 -end\n\
             END\n",
        )
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("MY-BASIC compatibility: USER_GRAPH is not supported"),
        "{error}"
    );
}
