//! BRD-002 remainder: finding what is on the shelf, and saying where a
//! vessel's contents came from when they came from a recipe.

use std::io::Write;
use std::process::{Command, Stdio};

/// `find` and `species` live in the REPL loop rather than in `exec_line`,
/// because they ask about the catalogue rather than doing anything to the
/// bench. So the tests drive the REPL, as the quest tests do.
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

/// The gap this closes: `species` printed several hundred rows and no
/// materials at all, so a learner looking for "the vinegar" had no way to
/// discover that `vinegar` is a name the bench takes.
#[test]
fn find_reaches_a_material_a_species_listing_never_showed() {
    let out = run_repl("find vinegar\n");
    assert!(
        out.contains("white_vinegar_5_percent"),
        "the recipe is findable by its familiar name: {out}"
    );
    assert!(out.contains("material"), "and says which kind it is: {out}");
    // The key is the thing to type, so the answer says how to use it.
    assert!(out.contains("add v1"), "and how to ask for it: {out}");
}

/// A German bench is only usable if the German names find anything.
#[test]
fn find_matches_an_alias_and_says_which_one() {
    let out = run_repl("find Essig\n");
    assert!(
        out.contains("white_vinegar_5_percent"),
        "the German alias reaches the recipe: {out}"
    );
    assert!(
        out.contains("de: Essig"),
        "and the answer explains itself rather than looking like a coincidence: {out}"
    );
    // "Essig" is also inside "Fluessigseife" (hand soap). An exact hit
    // drops the coincidences rather than ranking them below.
    assert!(
        !out.contains("liquid_hand_soap"),
        "an exact alias hit suppresses substring noise: {out}"
    );
}

/// The half-remembered name, which is what the verb is for.
#[test]
fn find_takes_a_fragment_when_nothing_matches_exactly() {
    let out = run_repl("find chlor\n");
    for expected in ["NaCl", "KCl", "HCl"] {
        assert!(out.contains(expected), "'{expected}' missing from: {out}");
    }
}

/// What a search says about a bottle must be what the ledger enforces, or
/// a learner is told to stock in one unit and refused in another.
#[test]
fn find_reports_the_shelf_level_it_will_be_refused_against() {
    let out = run_repl("find NaCl\nstock NaCl 0.5mol\nfind NaCl\n");
    assert!(
        out.contains("unstocked (unlimited)"),
        "an unstocked key is an unlimited supply, said out loud: {out}"
    );
    assert!(
        out.contains("0.5000 mol left"),
        "and a stocked one reports what is left, in the unit it is counted in: {out}"
    );
}

#[test]
fn an_empty_search_says_so_rather_than_listing_the_catalogue() {
    let out = run_repl("find zzzznotathing\n");
    assert!(
        out.contains("nothing on the shelf matches"),
        "a miss is a sentence, not silence: {out}"
    );
}

/// `explain` reported the solver's provenance and said nothing about the
/// fact that half the answer rested on a *recipe* — a reviewed estimate of
/// a composition, with a confidence and lot assumptions behind it.
///
/// The recipe leaves no trace in the contents when it expands cleanly: its
/// acetic acid is indistinguishable from acetic acid poured from a bottle.
/// That is exactly the case worth reporting, and it is why this reads the
/// bench log rather than the vessel.
#[test]
fn explain_names_the_recipe_a_vessel_was_built_from() {
    let out = run_repl("add v1 white_vinegar_5_percent 100mL\nexplain v1\n");
    assert!(
        out.contains("white_vinegar_5_percent@v1"),
        "the recipe is named with its version, which is what a replay pins: {out}"
    );
    assert!(out.contains("CH3COOH"), "and what it expanded into: {out}");
    // The confidence is a claim about how far to trust the composition, so
    // it is a sentence rather than an enum variant name.
    assert!(
        out.contains("surrogate —"),
        "the confidence is spelled out: {out}"
    );
    assert!(
        out.contains("assumes:"),
        "and the lot assumptions travel with it: {out}"
    );
}

/// A vessel built from bottles says nothing about recipes — the section
/// appears because there is something to say, not on every explain.
#[test]
fn explain_is_silent_about_recipes_when_none_were_used() {
    let out = run_repl("add v1 water 250mL\nadd v1 NaCl 0.1mol\nexplain v1\n");
    assert!(
        !out.contains("came from the recipe"),
        "no recipe, no recipe section: {out}"
    );
}
