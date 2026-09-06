//! ARCH-001: Freeze current behavior.
//!
//! Snapshot the JSON contract and accepted outputs of every lesson. On
//! first run, writes golden files to tests/golden/. On subsequent runs,
//! compares against them and fails if anything changes.
//!
//! Intentionally unstable numeric fields (floating-point solver results
//! that vary with optimisation level or platform) are rounded to a
//! documented precision before comparison.

use kerotakis_core::*;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

fn lesson_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../lessons")
}

/// Round a float to N decimal places for stable comparison.
fn round(value: f64, decimals: u32) -> f64 {
    let factor = 10f64.powi(decimals as i32);
    (value * factor).round() / factor
}

/// Snapshot a vessel's observable state as a stable JSON object.
fn snapshot_vessel(v: &Vessel) -> serde_json::Value {
    let mut contents: Vec<serde_json::Value> = v
        .contents
        .iter()
        .filter(|p| p.moles.0 > 1e-12)
        .map(|p| {
            serde_json::json!({
                "species": p.species.0,
                "phase": format!("{:?}", p.phase),
                "moles": round(p.moles.0, 8),
            })
        })
        .collect();
    contents.sort_by(|a, b| {
        a["species"]
            .as_str()
            .cmp(&b["species"].as_str())
            .then_with(|| a["phase"].as_str().cmp(&b["phase"].as_str()))
    });

    let mut snap = serde_json::json!({
        "label": v.label,
        "temperature_k": round(v.temperature.0, 4),
        "contents": contents,
        "headspace": format!("{:?}", v.headspace),
    });

    if let Some(ref sol) = v.solution {
        snap["ph"] = serde_json::json!(round(sol.ph, 4));
        snap["ionic_strength"] = serde_json::json!(round(sol.ionic_strength, 6));
    }

    snap
}

/// Run a lesson and return a snapshot of each vessel + all events.
fn run_lesson(path: &Path) -> serde_json::Value {
    let content = fs::read_to_string(path).unwrap();
    let mut bench = Bench::default();
    let mut all_events = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Ok(Some(op)) = kerotakis_core::script::parse_op(line) {
            match bench.step(op) {
                Ok(events) => {
                    for e in &events {
                        all_events.push(render_event(e, Register::LV2));
                    }
                }
                Err(e) => {
                    all_events.push(format!("ERROR: {e}"));
                }
            }
        }
    }

    let vessels: Vec<serde_json::Value> = bench.vessels.iter().map(snapshot_vessel).collect();

    serde_json::json!({
        "lesson": path.file_name().unwrap().to_string_lossy(),
        "vessels": vessels,
        "event_count": all_events.len(),
        "events_sample": all_events.iter().take(20).collect::<Vec<_>>(),
    })
}

#[test]
fn lessons_produce_stable_output() {
    let golden = golden_dir();
    let lessons = lesson_dir();

    if !lessons.exists() {
        eprintln!("lessons/ not found — skipping ARCH-001 freeze test");
        return;
    }

    fs::create_dir_all(&golden).unwrap();

    let mut results = BTreeMap::new();
    for entry in fs::read_dir(&lessons).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("lab") {
            continue;
        }
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let snapshot = run_lesson(&path);
        results.insert(name, snapshot);
    }

    let snapshot_path = golden.join("lessons.json");
    // Keep the checked-in snapshot a conventional newline-terminated text file.
    let current = format!("{}\n", serde_json::to_string_pretty(&results).unwrap());

    if snapshot_path.exists() {
        let expected = fs::read_to_string(&snapshot_path).unwrap();
        // Snapshot behavior is JSON content; an editor-added terminal newline
        // is not a chemistry change and must not fail ARCH-001.
        if current.trim_end() != expected.trim_end() {
            // Write the actual for diffing
            let actual_path = golden.join("lessons.actual.json");
            fs::write(&actual_path, &current).unwrap();
            let expected_lines = expected.lines().collect::<Vec<_>>();
            let actual_lines = current.lines().collect::<Vec<_>>();
            // EVERY difference, not the first. A deliberate change to a
            // rendered sentence moves dozens of these lines at once, and a
            // report that names one of them turns updating the golden into
            // as many CI rounds as there are lines — the actual file is
            // written beside it, but nobody reading a CI log has it. The
            // cap is there so a genuinely broken run cannot bury the log.
            const MAX_REPORTED: usize = 400;
            let differences: Vec<String> = (0..expected_lines.len().max(actual_lines.len()))
                .filter(|&index| expected_lines.get(index) != actual_lines.get(index))
                .map(|index| {
                    format!(
                        "line {}:\n  expected: {}\n  actual:   {}",
                        index + 1,
                        expected_lines
                            .get(index)
                            .copied()
                            .unwrap_or("<end of file>"),
                        actual_lines.get(index).copied().unwrap_or("<end of file>"),
                    )
                })
                .collect();
            let total = differences.len();
            let first_difference = if differences.is_empty() {
                "Difference is outside line-normalized content".to_string()
            } else {
                let shown = differences.len().min(MAX_REPORTED);
                format!(
                    "{total} line(s) differ; showing {shown}:\n{}{}",
                    differences[..shown].join("\n"),
                    if total > shown {
                        format!("\n… and {} more", total - shown)
                    } else {
                        String::new()
                    }
                )
            };
            panic!(
                "ARCH-001: lesson behavior changed!\n\
                 {first_difference}\n\
                 Expected: {}\n\
                 Actual:   {}\n\
                 Run `diff {} {}` to see changes.",
                snapshot_path.display(),
                actual_path.display(),
                snapshot_path.display(),
                actual_path.display(),
            );
        }
    } else {
        // First run: write the golden file
        fs::write(&snapshot_path, &current).unwrap();
        eprintln!(
            "ARCH-001: wrote golden snapshot ({} lessons) to {}",
            results.len(),
            snapshot_path.display(),
        );
    }
}
