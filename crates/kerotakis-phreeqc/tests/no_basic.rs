#![cfg(all(feature = "engine", not(feature = "my-basic")))]

use kerotakis_phreeqc::{databases, Phreeqc};

const DISABLED: &str = "PHREEQC BASIC capability is disabled";

fn assert_disabled(input: &str, context: &str) {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    let error = engine.run(input).unwrap_err().to_string();
    assert!(
        error.contains(DISABLED),
        "{context}: expected a stable capability error, got:\n{error}"
    );
}

#[test]
fn dormant_rate_and_calculated_value_definitions_remain_loadable() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    engine
        .run(
            "RATES\nNoOp\n-start\n10 SAVE 0\n-end\n\
             CALCULATE_VALUES\nNoOpValue\n-start\n10 SAVE 0\n-end\n\
             SOLUTION 1\n    pH 7\nEND\n",
        )
        .unwrap();
}

#[test]
fn user_punch_is_rejected_before_program_execution() {
    assert_disabled(
        "SOLUTION 1\n    pH 7\n\
         SELECTED_OUTPUT\n    -reset false\n\
         USER_PUNCH\n    -headings impossible\n\
         10 PUNCH 12345\nEND\n",
        "USER_PUNCH",
    );
}

#[test]
fn user_print_is_rejected_before_program_execution() {
    assert_disabled(
        "SOLUTION 1\n    pH 7\n\
         PRINT\n    -reset false\n    -user_print true\n\
         USER_PRINT\n10 PRINT \"must not print\"\nEND\n",
        "USER_PRINT",
    );
}

#[test]
fn kinetics_is_rejected_before_the_rate_program_executes() {
    assert_disabled(
        "RATES\nNoOp\n-start\n10 SAVE 0\n-end\n\
         SOLUTION 1\n    pH 7\n\
         KINETICS 1\n    NoOp\n        -m 1\n        -steps 1\nEND\n",
        "RATES/KINETICS",
    );
}

#[test]
fn user_graph_is_rejected_before_program_execution() {
    assert_disabled(
        "SOLUTION 1\n    pH 7\n\
         USER_GRAPH 1\n-start\n10 GRAPH_X 12345\n-end\nEND\n",
        "USER_GRAPH",
    );
}
