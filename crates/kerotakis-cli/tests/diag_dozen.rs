//! DIAGNOSTIC — never merged. Runs a second sweep of experiments through the
//! real CLI after the day's engine changes (heat-source ceiling, per-phase
//! Cp, the room, vented burn products, displacement verdicts, µmol
//! formatting, conductivity fit) and prints every transcript by failing, so
//! they appear in the CI log where the VPS (which does not compile) can read
//! them.

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
    let dir = std::env::temp_dir().join("kero-diag2");
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
fn print_the_second_sweep() {
    let mut report = String::new();
    let inline: &[(&str, &[&str])] = &[
        (
            "chalk-40kJ-burner",
            &[
                "add v1 CaCO3 0.1mol",
                "heat v1 40kJ",
                "inspect v1",
                "wait 30min",
                "measure v1 thermometer",
                "inspect v1",
            ],
        ),
        (
            "chalk-40kJ-candle",
            &[
                "add v1 CaCO3 0.1mol",
                "heat v1 40kJ on candle",
                "inspect v1",
            ],
        ),
        (
            "water-on-hotplate",
            &[
                "add v1 water 100mL",
                "heat v1 50kJ on hotplate",
                "measure v1 thermometer",
                "inspect v1",
                "heat v1 50kJ",
                "measure v1 thermometer",
                "inspect v1",
            ],
        ),
        (
            "freeze-then-room",
            &[
                "add v1 water 100mL",
                "cool v1 60kJ",
                "measure v1 thermometer",
                "wait 30min",
                "measure v1 thermometer",
                "inspect v1",
            ],
        ),
        (
            "dry-ice-in-room",
            &[
                "add v1 dry_ice 10g",
                "measure v1 thermometer",
                "wait 5min",
                "inspect v1",
                "wait 30min",
                "measure v1 thermometer",
                "inspect v1",
            ],
        ),
        (
            "dry-ice-sealed",
            &[
                "add v1 dry_ice 5g",
                "seal v1 500mL",
                "wait 10min",
                "measure v1 pressure",
                "inspect v1",
            ],
        ),
        (
            "ethanol-burn-then-cool",
            &[
                "add v1 ethanol 10mL",
                "ignite v1",
                "measure v1 thermometer",
                "wait 1h",
                "measure v1 thermometer",
                "inspect v1",
            ],
        ),
        (
            "iron-in-copper-sulfate",
            &[
                "add v1 water 100mL",
                "add v1 CuSO4 0.02mol",
                "add v1 Fe 0.05mol",
                "wait 1h",
                "look v1",
                "inspect v1",
            ],
        ),
        (
            "electrolysis-of-brine",
            &[
                "add v1 water 100mL",
                "add v1 NaCl 5g",
                "measure v1 conductivity",
                "electrolyse v1 0.5A 10min",
                "inspect v1",
            ],
        ),
        (
            "sealed-soda-vinegar-cooling",
            &[
                "add v1 white_vinegar_5_percent 50mL",
                "seal v1 200mL",
                "add v1 baking_soda 3g",
                "measure v1 pressure",
                "measure v1 balance",
                "wait 20min",
                "measure v1 pressure",
                "measure v1 balance",
                "measure v1 thermometer",
            ],
        ),
        (
            "german-verbs",
            &[
                "zugeben v1 Wasser 100mL",
                "zugeben v1 NaCl 2g",
                "messen v1 Leitfähigkeit",
                "erhitzen v1 5kJ",
                "messen v1 Thermometer",
            ],
        ),
        (
            "decant-pour",
            &[
                "add v1 water 100mL",
                "add v1 CuSO4 0.01mol",
                "new",
                "decant v1 v2 0.5",
                "inspect v1",
                "inspect v2",
                "look v2",
            ],
        ),
        (
            "sugar-boil-hold",
            &[
                "add v1 water 100mL",
                "add v1 sucrose 20g",
                "heat v1 60kJ",
                "measure v1 thermometer",
                "wait 10min",
                "measure v1 thermometer",
                "inspect v1",
            ],
        ),
        (
            "mg-ribbon-burn",
            &[
                "add v1 Mg 1.2g",
                "ignite v1",
                "inspect v1",
                "wait 30min",
                "measure v1 thermometer",
            ],
        ),
    ];
    for (name, lines) in inline {
        report.push_str(&format!(
            "\n===== {name} =====\n{}\n",
            run_script(name, lines)
        ));
    }
    for lesson in [
        "sealed-mass-conservation.lab",
        "rates.lab",
        "there-and-back.lab",
        "two-roads.lab",
    ] {
        report.push_str(&format!(
            "\n===== lesson {lesson} =====\n{}\n",
            run_file(&lessons_dir().join(lesson))
        ));
    }
    panic!("DIAGNOSTIC TRANSCRIPTS\n{report}");
}
