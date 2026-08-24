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
    // The mask holds in every rendered line while the quest runs: the
    // learner never reads the real name from the bench.
    let before_answer = out.split("quest answer").next().unwrap();
    assert!(
        !before_answer.contains("sodium chloride") && !before_answer.contains("NaCl"),
        "sealed until answered: {before_answer}"
    );
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
