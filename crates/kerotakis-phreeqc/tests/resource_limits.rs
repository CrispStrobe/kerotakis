#![cfg(all(
    feature = "engine",
    feature = "my-basic-preview",
    not(feature = "legacy-basic-oracle")
))]

use kerotakis_phreeqc::{databases, Phreeqc};

#[test]
fn infinite_loop_hits_statement_budget() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    let error = engine
        .run(
            "RATES\n\
             InfLoop\n\
             -start\n\
             10 GOTO 10\n\
             -end\n\
             SOLUTION 1\n\
                 pH 7\n\
             KINETICS 1\n\
                 InfLoop\n\
                     -formula H2O 0\n\
                     -m 1\n\
                     -steps 1 second\n\
             END\n",
        )
        .unwrap_err()
        .to_string();
    assert!(error.contains("statement budget"), "{error}");
}

#[test]
fn recursion_depth_limit_prevents_stack_overflow() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    let error = engine
        .run(
            "CALCULATE_VALUES\n\
                 Infinite\n\
                 -start\n\
                 10 SAVE CALC_VALUE(\"Infinite\") + 1\n\
                 -end\n\
             SOLUTION 1\n\
                 pH 7\n\
             SELECTED_OUTPUT\n\
                 -reset false\n\
                 -calculate_values Infinite\n\
             END\n",
        )
        .unwrap_err()
        .to_string();
    assert!(error.contains("recursion budget"), "{error}");
}

#[test]
fn output_flood_hits_output_budget() {
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

#[test]
fn engine_remains_usable_after_budget_exhaustion() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    let _ = engine.run(
        "RATES\n\
         Spin\n\
         -start\n\
         10 GOTO 10\n\
         -end\n\
         SOLUTION 1\n\
             pH 7\n\
         KINETICS 1\n\
             Spin\n\
                 -formula H2O 0\n\
                 -m 1\n\
                 -steps 1 second\n\
         END\n",
    );
    engine
        .run("SOLUTION 2\n    pH 7\nEND\n")
        .expect("engine usable after failure");
}

#[test]
fn malformed_input_does_not_corrupt_state() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    let _ = engine.run(
        "RATES\n\
         Bad\n\
         -start\n\
         10 IF THEN\n\
         -end\n\
         SOLUTION 1\n\
             pH 7\n\
         KINETICS 1\n\
             Bad\n\
                 -formula H2O 0\n\
                 -m 1\n\
                 -steps 1 second\n\
         END\n",
    );
    engine
        .run(
            "SOLUTION 1\n\
                 pH 7\n\
                 Na 0.01\n\
             END\n",
        )
        .expect("engine works after malformed input");
}
