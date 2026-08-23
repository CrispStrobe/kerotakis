#![cfg(all(feature = "engine", feature = "my-basic",))]

use kerotakis_phreeqc::Phreeqc;
use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn every_official_phreeqc_example_runs() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor/iphreeqc");
    let examples = root.join("phreeqc3-examples");
    let databases = root.join("database");
    let scratch =
        std::env::temp_dir().join(format!("kerotakis-phreeqc-examples-{}", std::process::id()));
    let _ = fs::remove_dir_all(&scratch);
    fs::create_dir_all(&scratch).unwrap();
    for entry in fs::read_dir(&examples).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_file() {
            fs::copy(entry.path(), scratch.join(entry.file_name())).unwrap();
        }
    }
    // The stock example generates `ex21_Hto_rad.tsv`, while its bundled input
    // file is named `ex21_HTO_rad.tsv`. Supply the spelling the example asks
    // for so the official corpus is portable to case-sensitive runners.
    fs::copy(
        scratch.join("ex21_HTO_rad.tsv"),
        scratch.join("ex21_Hto_rad.tsv"),
    )
    .unwrap();

    let cases: [(&str, PathBuf); 32] = [
        ("ex1", databases.join("phreeqc.dat")),
        ("ex2", databases.join("phreeqc.dat")),
        ("ex2b", databases.join("phreeqc.dat")),
        ("ex3", databases.join("phreeqc.dat")),
        ("ex4", databases.join("phreeqc.dat")),
        ("ex5", databases.join("phreeqc.dat")),
        ("ex6", databases.join("phreeqc.dat")),
        ("ex7", databases.join("phreeqc.dat")),
        ("ex8", databases.join("phreeqc.dat")),
        ("ex9", databases.join("phreeqc.dat")),
        ("ex10", databases.join("phreeqc.dat")),
        ("ex11", databases.join("phreeqc.dat")),
        ("ex12", databases.join("phreeqc.dat")),
        ("ex12a", databases.join("phreeqc.dat")),
        ("ex13a", databases.join("phreeqc.dat")),
        ("ex13b", databases.join("phreeqc.dat")),
        ("ex13c", databases.join("phreeqc.dat")),
        ("ex13ac", databases.join("phreeqc.dat")),
        ("ex14", databases.join("phreeqc.dat")),
        ("ex15", examples.join("ex15.dat")),
        ("ex15a", examples.join("ex15.dat")),
        ("ex15b", examples.join("ex15.dat")),
        ("ex16", databases.join("phreeqc.dat")),
        ("ex17", databases.join("pitzer.dat")),
        ("ex17b", databases.join("pitzer.dat")),
        ("ex18", databases.join("phreeqc.dat")),
        ("ex19", databases.join("phreeqc.dat")),
        ("ex19b", databases.join("phreeqc.dat")),
        ("ex20a", databases.join("iso.dat")),
        ("ex20b", databases.join("iso.dat")),
        ("ex21", databases.join("phreeqc.dat")),
        ("ex22", databases.join("phreeqc.dat")),
    ];

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&scratch).unwrap();
    let mut failures = Vec::new();
    for (name, database_path) in cases {
        if let Ok(filter) = std::env::var("EXAMPLE_PROBE_FILTER") {
            if name != filter {
                continue;
            }
        }
        let database = fs::read(&database_path).unwrap();
        let input = fs::read_to_string(scratch.join(name)).unwrap();
        let status = match Phreeqc::with_database(&database) {
            Ok(mut engine) => match if name == "ex20b" {
                let marker = "INCLUDE$ ex20_open";
                let split = input.find(marker).expect("ex20b include marker");
                engine.run(&input[..split]).and_then(|()| {
                    fs::write(scratch.join("ex20_open"), engine.selected_output_string())
                        .expect("write generated ex20 input");
                    engine.run(&input[split..])
                })
            } else {
                engine.run(&input)
            } {
                Ok(()) => format!("ok selected_rows={}", engine.selected_output().len()),
                Err(error) => format!("run_error={}", error.to_string().replace('\n', " | ")),
            },
            Err(error) => format!("database_error={}", error.to_string().replace('\n', " | ")),
        };
        eprintln!("EXAMPLE_COMPLETION name={name} {status}");
        if !status.starts_with("ok ") {
            failures.push(format!("{name}: {status}"));
        }
    }
    std::env::set_current_dir(original_dir).unwrap();
    fs::remove_dir_all(scratch).unwrap();
    assert!(
        failures.is_empty(),
        "{} official examples failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
