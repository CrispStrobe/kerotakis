//! The 2026-11-28 sweep's iodoform and indicator scripts through the real
//! binary and the full solver stack (S4/S5 in
//! docs/reaction-sweep-gaps-20261120.md).

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static CASE: AtomicUsize = AtomicUsize::new(0);

fn run_with(script: &str, json: bool) -> String {
    let dir = std::env::temp_dir().join(format!(
        "kero-haloform-{}-{}",
        std::process::id(),
        CASE.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let lab = dir.join("case.lab");
    std::fs::write(&lab, script).unwrap();
    let mut args = vec!["run", lab.to_str().unwrap()];
    if json {
        args.push("--json");
    }
    let out = Command::new(env!("CARGO_BIN_EXE_kero"))
        .args(&args)
        .output()
        .expect("kero runs");
    assert!(
        out.status.success(),
        "script failed:\n{script}\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    std::fs::remove_dir_all(&dir).ok();
    String::from_utf8(out.stdout).unwrap()
}

fn last_contents(script: &str) -> Vec<serde_json::Value> {
    let text = run_with(script, true);
    let last: serde_json::Value =
        serde_json::from_str(text.lines().last().unwrap()).expect("JSON");
    last["bench"]["vessels"].as_array().unwrap()[0]["contents"]
        .as_array()
        .cloned()
        .unwrap()
}

fn moles(contents: &[serde_json::Value], species: &str) -> f64 {
    contents
        .iter()
        .filter(|p| p["species"] == species)
        .map(|p| p["moles"].as_f64().unwrap())
        .sum()
}

fn iodoform_script(substrate: &str) -> String {
    // The sweep's t14 script, substrate varied.
    format!(
        "add v1 water 10mL\nadd v1 {substrate} 0.002mol\nadd v1 I2 0.003mol\nadd v1 KI 0.006mol\nadd v1 NaOH 0.01mol\nheat v1 1kJ\nlook v1\ninspect v1\n"
    )
}

#[test]
fn warm_alkaline_iodine_gives_yellow_iodoform_from_ethanol() {
    let contents = last_contents(&iodoform_script("ethanol"));
    let chi3 = moles(&contents, "CHI3");
    // Iodine limits: 3 mmol I2 over four per ethanol.
    assert!((chi3 - 0.00075).abs() < 1e-6, "{contents:#?}");
    let text = run_with(&iodoform_script("ethanol"), false);
    assert!(text.contains("iodoform"), "{text}");
    assert!(text.contains("yellow"), "{text}");
}

#[test]
fn methanol_gives_no_iodoform() {
    let contents = last_contents(&iodoform_script("methanol"));
    assert_eq!(moles(&contents, "CHI3"), 0.0, "{contents:#?}");
}

fn liquid_word(indicator: &str, ph_reagent: &str) -> String {
    let script = format!(
        "add v1 water 50mL\nadd v1 {indicator} 0.00001mol\n{ph_reagent}look v1\n"
    );
    let text = run_with(&script, false);
    text.lines()
        .rev()
        .find_map(|l| l.split("The liquid is ").nth(1))
        .unwrap_or_else(|| panic!("no look line:\n{text}"))
        .split(' ')
        .next()
        .unwrap()
        .to_string()
}

/// pH ≈ 3 (1 mM HCl), ≈ 7 (HCl neutralised by the same NaOH), ≈ 11
/// (1 mM NaOH).
const ACID: &str = "add v1 HCl 0.00005mol\n";
const NEUTRAL: &str = "add v1 HCl 0.00005mol\nadd v1 NaOH 0.00005mol\n";
const BASE: &str = "add v1 NaOH 0.00005mol\n";

#[test]
fn methyl_orange_is_red_in_acid_and_yellow_at_7_and_11() {
    assert_eq!(liquid_word("methyl_orange", ACID), "red");
    assert_eq!(liquid_word("methyl_orange", NEUTRAL), "yellow");
    assert_eq!(liquid_word("methyl_orange", BASE), "yellow");
}

#[test]
fn bromothymol_blue_is_yellow_in_acid_and_blue_in_base() {
    assert_eq!(liquid_word("bromothymol_blue", ACID), "yellow");
    assert_eq!(liquid_word("bromothymol_blue", BASE), "blue");
    // At pH ~7 it is inside its sourced 6.0–7.6 band: the word is the
    // computed mixed hue, never the acid form's drifted "orange".
    let mid = liquid_word("bromothymol_blue", NEUTRAL);
    assert_ne!(mid, "orange", "{mid}");
    assert_ne!(mid, "yellow", "{mid}");
}
