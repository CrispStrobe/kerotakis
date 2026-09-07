#![cfg(feature = "engine")]
#[path = "../src/redox_isolation.rs"]
mod redox_isolation;
use kerotakis_phreeqc::{databases, Phreeqc};

// Unlike DbIndex's chemical formula projection, this checks the native phase
// headers even when an original colon-adduct formula cannot be indexed.
fn phase_headers(bytes: &[u8]) -> Vec<String> {
    let text = String::from_utf8_lossy(bytes);
    let mut in_phases = false;
    let mut previous = String::new();
    let mut result = Vec::new();
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let first = line.split_whitespace().next().unwrap();
        if first.len() > 3 && first.bytes().all(|b| b.is_ascii_uppercase() || b == b'_') {
            in_phases = first == "PHASES";
        } else if in_phases && line.contains('=') {
            result.push(previous.clone());
        }
        previous = line.to_string();
    }
    result
}

#[test]
fn transformed_databases_load_natively() {
    for (tag, bytes) in [
        ("wateq", databases::wateq4f()),
        ("minteq", databases::minteq_v4()),
        ("pitzer", databases::pitzer()),
    ] {
        let db = redox_isolation::transform(bytes).unwrap_or_else(|e| panic!("{tag}: {e}"));
        assert!(!db.totals.is_empty());
        assert!(!db.species.is_empty());
        let physical = kerotakis_phreeqc::dbindex::DbIndex::parse(bytes);
        let isolated = kerotakis_phreeqc::dbindex::DbIndex::parse(&db.database);
        assert_eq!(
            phase_headers(bytes),
            phase_headers(&db.database),
            "phase names must remain public: {tag}"
        );
        for (name, original) in &physical.phases {
            let changed = &isolated.phases[name];
            assert_eq!(original.log_k, changed.log_k, "phase logK: {tag}/{name}");
            assert_eq!(
                original.delta_h_kj, changed.delta_h_kj,
                "phase enthalpy: {tag}/{name}"
            );
        }
        Phreeqc::with_database(db.database).unwrap_or_else(|e| panic!("{tag}: {e}"));
    }
}

#[test]
fn isolated_totals_survive_native_batch_and_mix_at_arbitrary_pe() {
    for (tag, bytes) in [
        ("wateq", databases::wateq4f()),
        ("minteq", databases::minteq_v4()),
    ] {
        let db = redox_isolation::transform(bytes).unwrap();
        let ammonium = &db.totals["N(-3)"];
        let nitrate = &db.totals["N(5)"];
        let sulfate = &db.totals["S(6)"];
        let sulfide = &db.totals["S(-2)"];
        for scale in [1e-4, 0.01, 0.1] {
            // At the acidic charge-balanced pH, pe=-10 would demand an
            // unphysical dissolved-H2 inventory outside water stability.
            for pe in [-2, 4, 10] {
                let mut engine = Phreeqc::with_database(&db.database).unwrap();
                let program = format!("SOLUTION 1\n pH 7 charge\n pe {pe}\n units mol/kgw\n -water 1\n Na {scale}\n Cl {scale}\n {ammonium} {scale}\n {nitrate} {scale}\n {sulfate} {scale}\n {sulfide} {scale}\nSAVE solution 1\nSELECTED_OUTPUT\n -reset false\n -high_precision true\nUSER_PUNCH\n -headings ammonia nitrate sulfate sulfide\n10 PUNCH TOT(\"{ammonium}\")*TOT(\"water\"), TOT(\"{nitrate}\")*TOT(\"water\"), TOT(\"{sulfate}\")*TOT(\"water\"), TOT(\"{sulfide}\")*TOT(\"water\")\nEND\nUSE solution 1\nREACTION 1\n NaCl 1\n {scale} moles\nSAVE solution 2\nEND\nMIX 3\n 1 0.25\n 2 0.75\nEND\n");
                engine
                    .run(&program)
                    .unwrap_or_else(|e| panic!("{tag}, {scale}, pe {pe}: {e}"));
                for column in ["ammonia", "nitrate", "sulfate", "sulfide"] {
                    let actual = engine.last_value(column).unwrap();
                    assert!(
                        (actual - scale).abs() < scale * 1e-7,
                        "{tag}, {scale}, pe {pe}: {column} {actual}\n{}",
                        engine.selected_output_string()
                    );
                }
            }
        }
    }
}

#[test]
fn sulfate_mineral_feed_does_not_reduce_while_ammonia_complexes() {
    for (tag, bytes) in [
        ("wateq", databases::wateq4f()),
        ("minteq", databases::minteq_v4()),
    ] {
        let db = redox_isolation::transform(bytes).unwrap();
        let mut engine = Phreeqc::with_database(&db.database).unwrap();
        let ammonia = &db.totals["N(-3)"];
        let sulfate = &db.totals["S(6)"];
        let sulfide = &db.totals["S(-2)"];
        let program = format!("SOLUTION 1\n pH 10 charge\n units mol/kgw\n {ammonia} 0.01\n Na 0.01\n Cl 0.02\nSELECTED_OUTPUT\n -reset false\n -high_precision true\nUSER_PUNCH\n -headings ammonia sulfate sulfide copper\n10 PUNCH TOT(\"{ammonia}\")*TOT(\"water\"), TOT(\"{sulfate}\")*TOT(\"water\"), TOT(\"{sulfide}\")*TOT(\"water\"), TOT(\"Cu\")*TOT(\"water\")\nEQUILIBRIUM_PHASES 1\n Chalcanthite 0 0.0001\nEND\n");
        engine
            .run(&program)
            .unwrap_or_else(|e| panic!("{tag}: {e}"));
        assert!((engine.last_value("ammonia").unwrap() - 0.01).abs() < 1e-9);
        assert!(engine.last_value("sulfide").unwrap() < 1e-15);
        let sulfate = engine.last_value("sulfate").unwrap();
        assert!(sulfate > 1e-5, "mineral must actually dissolve");
        assert!((sulfate - engine.last_value("copper").unwrap()).abs() < 1e-10);
    }
}
