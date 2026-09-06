//! DIAGNOSTIC — never merged. Runs a dozen experiments through the real
//! CLI and prints every transcript by failing, so the transcripts appear in
//! the CI log where the VPS (which may not compile) can read them.

use std::process::Command;

fn lessons_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../lessons")
}

fn run_file(path: &std::path::Path) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_kero"))
        .args(["run", path.to_str().unwrap()])
        .output()
        .expect("kero runs");
    format!(
        "{}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

fn run_script(name: &str, lines: &[&str]) -> String {
    let dir = std::env::temp_dir().join("kero-diag");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("{name}.lab"));
    let mut text = String::from("register lv2\n");
    for l in lines {
        text.push_str(l);
        text.push('\n');
    }
    std::fs::write(&path, text).unwrap();
    run_file(&path)
}

#[test]
fn print_a_dozen_transcripts() {
    let mut report = String::new();
    let inline: &[(&str, &[&str])] = &[
        (
            "chalk-40kJ",
            &["add v1 CaCO3 0.1mol", "heat v1 40kJ", "inspect v1", "measure v1 thermometer"],
        ),
        (
            "neutralisation",
            &[
                "add v1 water 100mL",
                "add v1 HCl 0.05mol",
                "measure v1 ph",
                "add v1 NaOH 0.05mol",
                "measure v1 thermometer",
                "measure v1 ph",
                "inspect v1",
            ],
        ),
        (
            "mg-in-acid",
            &[
                "add v1 water 100mL",
                "add v1 HCl 0.1mol",
                "add v1 Mg 0.02mol",
                "measure v1 thermometer",
                "inspect v1",
                "wait 10min",
                "inspect v1",
            ],
        ),
        (
            "iron-in-copper-sulfate",
            &[
                "add v1 water 100mL",
                "add v1 CuSO4 0.02mol",
                "look v1",
                "add v1 Fe 0.05mol",
                "wait 1h",
                "inspect v1",
                "look v1",
            ],
        ),
        (
            "freeze-water",
            &["add v1 water 100mL", "cool v1 60kJ", "measure v1 thermometer", "inspect v1"],
        ),
        (
            "boil-sugar-water",
            &[
                "add v1 water 100mL",
                "add v1 sucrose 20g",
                "heat v1 60kJ",
                "measure v1 thermometer",
                "inspect v1",
            ],
        ),
        (
            "permanganate",
            &["add v1 water 100mL", "add v1 KMnO4 0.001mol", "look v1", "measure v1 ph", "inspect v1"],
        ),
        (
            "dry-ice",
            &["add v1 dry_ice 10g", "measure v1 thermometer", "inspect v1", "wait 10min", "inspect v1"],
        ),
        (
            "candle-burn-ethanol",
            &["add v1 ethanol 10mL", "measure v1 balance", "ignite v1", "measure v1 balance", "inspect v1"],
        ),
        (
            "salt-water-heat",
            &[
                "add v1 water 100mL",
                "add v1 NaCl 10g",
                "measure v1 conductivity",
                "heat v1 20kJ",
                "measure v1 thermometer",
                "evaporate v1 0.5",
                "inspect v1",
            ],
        ),
    ];
    for (name, lines) in inline {
        report.push_str(&format!("\n===== {name} =====\n{}\n", run_script(name, lines)));
    }
    for lesson in [
        "fizz.lab",
        "fire.lab",
        "electrolysis.lab",
        "calorimetry.lab",
        "spannungsreihe.lab",
        "yeast-fermentation.lab",
        "sealed-gas.lab",
        "spirit-still.lab",
    ] {
        report.push_str(&format!(
            "\n===== lesson {lesson} =====\n{}\n",
            run_file(&lessons_dir().join(lesson))
        ));
    }
    panic!("DIAGNOSTIC TRANSCRIPTS\n{report}");
}
