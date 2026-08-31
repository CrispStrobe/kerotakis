//! EXP-0's acceptance, bench half: the demo quest completes through
//! the real REPL with the full solver stack — sealed unknown masked in
//! every rendered line, identified only by name at the end, the pH
//! value-claim read from the actual solved state, and two command
//! orders both reaching the star.

use std::io::Write;
use std::process::{Command, Stdio};

fn run_repl(script: &str) -> String {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let mut child = Command::new(env!("CARGO_BIN_EXE_kero"))
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("kero repl spawns");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(script.as_bytes())
        .unwrap();
    let out = child.wait_with_output().expect("repl exits");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

const ORDER_A: &str = "quest start the-white-unknown\n\
    add v1 water 250mL\n\
    add v1 unknown-a 1g\n\
    inspect\n\
    particles v1\n\
    explain v1\n\
    measure v1 ph\n\
    quest answer unknown-a NaCl\n\
    quest status\n";

const ORDER_B: &str = "quest start the-white-unknown\n\
    add v1 water 250mL\n\
    quest answer unknown-a NaCl\n\
    add v1 unknown-a 1g\n\
    measure v1 thermometer\n\
    measure v1 ph\n\
    quest status\n";

#[test]
fn order_a_completes_with_the_unknown_sealed() {
    let out = run_repl(ORDER_A);
    assert!(out.contains("quest started"), "start line: {out}");
    assert!(
        out.contains("★ quest complete"),
        "the star at the end: {out}"
    );
    // The mask holds in every rendered line while the quest runs — the
    // x-ray views included: `inspect`, `particles` and `explain` once
    // printed the vessel's truth unmasked, and a census row reading
    // `Na+` answers the quest no less than one reading `NaCl`.
    let before_answer = out.split("quest answer").next().unwrap();
    for leak in ["sodium", "chloride", "NaCl", "Na+", "Cl-"] {
        assert!(
            !before_answer.contains(leak),
            "sealed until answered, but '{leak}' printed: {before_answer}"
        );
    }
    assert!(
        before_answer.contains("unknown-a"),
        "the alias speaks instead: {before_answer}"
    );
    assert!(out.contains("COMPLETE"), "status agrees: {out}");
}

#[test]
fn order_b_also_completes() {
    let out = run_repl(ORDER_B);
    assert!(
        out.contains("★ quest complete"),
        "a different order reaches the same star: {out}"
    );
}

#[test]
fn the_learners_own_bottle_is_not_masked_but_poured_unknown_travels_sealed() {
    // Two vessels: v1 holds the sealed sample, v2 the learner's own NaCl
    // — same substance, different knowledge. The covers follow the alias,
    // not the species: v2's census names its ions openly, and only after
    // matter is poured out of the sealed vessel does its destination go
    // behind the covers too.
    let script = "quest start the-white-unknown\n\
        add v1 water 250mL\n\
        add v1 unknown-a 1g\n\
        new\n\
        add v2 water 100mL\n\
        add v2 NaCl 1g\n\
        particles v2\n\
        new\n\
        add v3 water 100mL\n\
        decant v1 v3 0.5\n\
        particles v3\n\
        register lv1\n\
        particles v1\n\
        register lv2\n";
    let out = run_repl(script);
    // v2 — the learner's own bottle — speaks plainly.
    assert!(
        out.contains("Na+"),
        "the learner's own NaCl census is not masked: {out}"
    );
    // v1's and v3's censuses wear the alias.
    let v2_census = out
        .split("v2 — what the particles are doing:")
        .nth(1)
        .unwrap()
        .split("particles")
        .next()
        .unwrap();
    let sealed_censuses = out
        .split("v3 — what the particles are doing:")
        .nth(1)
        .unwrap();
    // v1's census runs at lv1, where the renderer speaks in plain
    // registry names — the mask must cover that spelling too.
    for leak in ["Na+", "Cl-", "sodium", "chloride"] {
        assert!(
            !sealed_censuses.contains(leak),
            "poured unknown stays sealed in v3 and v1, but '{leak}' printed: {sealed_censuses}"
        );
    }
    assert!(
        sealed_censuses.contains("unknown-a"),
        "the alias speaks in the sealed censuses: {sealed_censuses}"
    );
    assert!(
        v2_census.contains("Na+"),
        "v2's own census names its ions: {v2_census}"
    );
}

#[test]
fn a_wrong_answer_is_spoken_and_nothing_locks() {
    let script = "quest start the-white-unknown\n\
        add v1 water 250mL\n\
        add v1 unknown-a 1g\n\
        quest answer unknown-a CuSO4\n\
        measure v1 ph\n\
        quest answer unknown-a NaCl\n";
    let out = run_repl(script);
    assert!(
        out.contains("is not CuSO4"),
        "the refusal names the guess: {out}"
    );
    assert!(
        out.contains("★ quest complete"),
        "the right answer still completes afterwards: {out}"
    );
}
