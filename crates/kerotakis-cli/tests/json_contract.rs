//! The `--json` output is the API contract shared with future wasm/mobile
//! clients — every line must be a well-formed step object with a stable
//! shape (PLAN.md, "CLI first").

use std::io::Write;
use std::process::Command;

#[test]
fn json_output_is_the_api_contract() {
    let dir = std::env::temp_dir().join(format!("kero-json-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let lab = dir.join("contract.lab");
    let mut f = std::fs::File::create(&lab).unwrap();
    writeln!(
        f,
        "add v1 water 100mL\nadd v1 NaCl 1g\nmeasure v1 ph\nmeasure v1 balance"
    )
    .unwrap();

    let out = Command::new(env!("CARGO_BIN_EXE_kero"))
        .args(["run", lab.to_str().unwrap(), "--json"])
        .output()
        .expect("kero runs");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let stdout = String::from_utf8(out.stdout).unwrap();
    let steps: Vec<serde_json::Value> = stdout
        .lines()
        .map(|l| serde_json::from_str(l).expect("every line is JSON"))
        .collect();
    assert_eq!(steps.len(), 4, "one JSON object per operator");

    for (i, step) in steps.iter().enumerate() {
        assert_eq!(step["step"], i, "steps are numbered");
        assert!(step["operator"]["op"].is_string(), "operator is tagged");
        assert!(step["events"].is_array(), "events are an array");
        assert!(step["bench"]["vessels"].is_array(), "bench state included");
    }

    // The salt add must have produced a computed dissolution.
    let salt_events = steps[1]["events"].as_array().unwrap();
    assert!(
        salt_events.iter().any(|e| e["event"] == "dissolved"),
        "expected a 'dissolved' event, got {salt_events:?}"
    );
    // The pH measurement reads the characterised solution.
    let ph_events = steps[2]["events"].as_array().unwrap();
    assert!(
        ph_events
            .iter()
            .any(|e| e["event"] == "measured" && e["unit"] == "pH"),
        "expected a pH measurement, got {ph_events:?}"
    );

    std::fs::remove_dir_all(&dir).ok();
}
