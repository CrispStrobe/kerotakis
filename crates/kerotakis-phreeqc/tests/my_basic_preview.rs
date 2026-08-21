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
