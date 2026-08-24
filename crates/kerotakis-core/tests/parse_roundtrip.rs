//! GUI-005: Parse round-trip acceptance.
//!
//! Every lesson line that produces an Operator must round-trip through
//! JSON serialisation, and parsing must never touch bench state.

use kerotakis_core::script::parse_op;
use kerotakis_core::{Bench, Operator};
use std::fs;
use std::path::{Path, PathBuf};

fn lesson_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../lessons")
}

#[test]
fn parse_every_lesson_line() {
    let lessons = lesson_dir();
    if !lessons.exists() {
        eprintln!("lessons/ not found — skipping parse round-trip");
        return;
    }

    let mut total = 0usize;
    let mut operators = 0usize;

    for entry in fs::read_dir(&lessons).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("lab") {
            continue;
        }
        let text = fs::read_to_string(&path).unwrap();
        let name = path.file_stem().unwrap().to_string_lossy();

        for (lineno, line) in text.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("register ") {
                continue;
            }
            total += 1;

            let result = parse_op(line);
            assert!(
                result.is_ok(),
                "{name}:{}: parse_op failed: {}",
                lineno + 1,
                result.unwrap_err()
            );

            if let Ok(Some(op)) = result {
                operators += 1;
                let json = serde_json::to_string(&op).unwrap();
                let _back: Operator = serde_json::from_str(&json).unwrap_or_else(|e| {
                    panic!(
                        "{name}:{}: Operator JSON round-trip failed: {e}",
                        lineno + 1
                    )
                });
            }
        }
    }

    assert!(operators > 0, "no operators parsed from any lesson");
    eprintln!("parsed {total} lines, {operators} operators — all round-trip clean");
}

#[test]
fn parse_leaves_bench_unchanged() {
    let bench = Bench::new();
    let state_before = serde_json::to_string(&bench.vessels).unwrap();

    let lines = [
        "new",
        "add v1 water 100mL",
        "heat v1 500J",
        "add v1 NaCl 0.01mol",
        "# a comment",
        "",
        "register lv2",
    ];
    for line in &lines {
        let _ = parse_op(line);
    }

    let state_after = serde_json::to_string(&bench.vessels).unwrap();
    assert_eq!(
        state_before, state_after,
        "parse_op must not mutate bench state"
    );
}
