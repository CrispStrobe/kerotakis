//! CAP-2's acceptance: a titration study over lessons/titration.lab
//! reproduces the equivalence point the codex states — at equivalence
//! the amounts of acid and base are equal — and the runs are
//! byte-deterministic under the rayon pool.

use std::process::Command;

fn run_study(extra: &[&str]) -> String {
    let lessons = concat!(env!("CARGO_MANIFEST_DIR"), "/../../lessons/titration.lab");
    let mut args = vec![
        "study",
        lessons,
        "--vary",
        "add:v1:HCl=0.005..0.02:4",
        "--collect",
        "ph@v1,titrant_volume@v1",
    ];
    args.extend_from_slice(extra);
    let out = Command::new(env!("CARGO_BIN_EXE_kero"))
        .args(&args)
        .output()
        .expect("kero study runs");
    assert!(
        out.status.success(),
        "study exited {:?}: {}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("utf8")
}

#[test]
fn titration_study_reproduces_the_equivalence_point() {
    let ndjson = run_study(&[]);
    let rows: Vec<serde_json::Value> = ndjson
        .lines()
        .map(|l| serde_json::from_str(l).expect("one JSON object per line"))
        .collect();
    assert_eq!(rows.len(), 4, "one row per varied value");
    for (i, row) in rows.iter().enumerate() {
        assert_eq!(
            row["run"], i,
            "rows arrive in run order, not completion order"
        );
        let acid_moles = row["value"].as_f64().unwrap();
        let volume = row["probes"]["titrant_volume@v1"]["value"]
            .as_f64()
            .expect("the titration delivered a volume");
        // 1 mol/L standard: delivered base moles equal acid moles at the
        // crossing, within the one-step (1 mL = 0.001 mol) resolution.
        let base_moles = volume * 1.0;
        assert!(
            base_moles >= acid_moles - 1e-12 && base_moles - acid_moles <= 0.001 + 1e-12,
            "equivalence: {base_moles} mol NaOH for {acid_moles} mol HCl"
        );
        let ph = row["probes"]["ph@v1"]["value"].as_f64().unwrap();
        assert!(ph >= 7.0, "the crossing step ends past neutral, got {ph}");
        assert!(
            row["provenance"]
                .as_str()
                .unwrap()
                .contains("titration.lab"),
            "every row carries its provenance"
        );
    }
}

#[test]
fn a_study_is_byte_deterministic() {
    assert_eq!(run_study(&[]), run_study(&[]), "same study, same bytes");
}

#[test]
fn csv_output_has_one_row_per_run() {
    let csv = run_study(&["--csv"]);
    let lines: Vec<&str> = csv.lines().collect();
    assert_eq!(lines[0], "run,value,ph@v1,titrant_volume@v1");
    assert_eq!(lines.len(), 6, "header + 4 rows + provenance comment");
    assert!(lines[5].starts_with("# computed replay"));
}

#[test]
fn an_ambiguous_selector_refuses_with_the_candidates() {
    let lessons = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../lessons/titration-manual.lab"
    );
    let out = Command::new(env!("CARGO_BIN_EXE_kero"))
        .args([
            "study",
            lessons,
            "--vary",
            "add:v1:NaOH=0.001..0.01:3",
            "--collect",
            "ph@v1",
        ])
        .output()
        .expect("runs");
    assert!(
        !out.status.success(),
        "four NaOH adds cannot be one selector"
    );
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("use line:"),
        "the refusal points at the line:<N> escape hatch, got: {err}"
    );
}
