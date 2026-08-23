#![cfg(all(
    feature = "engine",
    feature = "my-basic"
))]

use kerotakis_phreeqc::{databases, Phreeqc};

fn run_rate(database: &[u8], rate: &str, options: &str, expected: f64) {
    let mut engine = Phreeqc::with_database(database).unwrap();
    let input = format!(
        "SOLUTION 1\n\
             units mol/kgw\n\
             temp 25\n\
             pH 7\n\
             pe 4\n\
             Na 0.01\n\
             K 0.001\n\
             Mg 0.001\n\
             Ca 0.001\n\
             C 0.002\n\
             N(5) 0.001\n\
             S(6) 0.001\n\
             Fe(2) 1e-10\n\
             Mn(2) 1e-6\n\
         KINETICS 1\n\
             {rate}\n\
                 -formula H2O 0\n\
                 -m 0.001\n\
                 -m0 0.001\n\
                 {options}\n\
                 -steps 1 second\n\
         SELECTED_OUTPUT\n\
             -reset false\n\
             -kinetics {rate}\n\
         END\n"
    );
    engine
        .run(&input)
        .unwrap_or_else(|error| panic!("bundled rate {rate} failed:\n{error}"));
    let heading = format!("k_{rate}");
    let remaining = engine
        .last_value(&heading)
        .unwrap_or_else(|| panic!("bundled rate {rate} did not produce {heading}"));
    assert!(remaining.is_finite(), "bundled rate {rate}: {remaining}");
    assert!(
        (remaining - expected).abs() <= 5e-8,
        "bundled rate {rate}: {remaining}, legacy oracle {expected}"
    );
    eprintln!("{rate} remaining={remaining:.15e}");
}

#[test]
fn bundled_phreeqc_and_wateq4f_rate_programs_compile_and_run() {
    let rates = [
        ("Quartz", "-parms 1 1"),
        ("K-feldspar", "-parms 1 0.1"),
        ("Albite", "-parms 1 0.1"),
        ("Calcite", "-parms 1000 0.6"),
        ("Pyrite", "-parms 0 0.67 0.5 -0.11"),
        ("Organic_C", ""),
        ("Pyrolusite", ""),
    ];

    for (database_name, database) in [
        ("phreeqc.dat", databases::PHREEQC),
        ("wateq4f.dat", databases::WATEQ4F),
    ] {
        for &(rate, options) in &rates {
            eprintln!("checking {database_name} RATES/{rate}");
            // Values were originally captured from the legacy oracle
            // during development. The zero-formula harness makes
            // this a dialect/execution comparison rather than a changing
            // geochemical system.
            let expected = match (database_name, rate) {
                ("phreeqc.dat", "Pyrolusite") => 0.03957,
                ("wateq4f.dat", "Pyrolusite") => 0.038116,
                _ => 0.001,
            };
            run_rate(database, rate, options, expected);
        }
    }
}
