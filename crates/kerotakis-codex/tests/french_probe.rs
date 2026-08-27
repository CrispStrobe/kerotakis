//! Can a second language be added without touching Rust?
//!
//! The claim this catalogue makes is that a new language is a data change:
//! drop in `codex/i18n/<code>.toml` and it renders. German proves nothing
//! about that on its own, because German is the language the types were
//! written for — `summary_de`, `question_de`, `reveals_de` are fields, not
//! entries in a map.
//!
//! So this feeds French through the exact path a translator would use and
//! asks whether the French comes back out. Nothing here is French-specific;
//! it is the cheapest available language that is not the one the types name.

use kerotakis_codex::Codex;

const ENGLISH: &str = r#"
[[reaction]]
id = "salt-in-water"
summary = "Salt dissolves."
system = "aqueous"
stage = "solutions"
apparatus = ["beaker"]
concepts = ["dissolving"]
[reaction.setup]
script = "add v1 water 100mL"
[reaction.registers]
lv1 = "It disappears."
lv2 = "0.1 mol dissolves."
lv3 = "Fully dissociated."
[reaction.provenance]
source = "editorial"
[[reaction.expect.predict.diagnosis]]
option = 1
reveals = "Thinks salt vanishes."
[reaction.expect.predict]
question = "What happens?"
options = ["It dissolves", "It sinks"]
answer = 0
"#;

/// Exactly what a translator would write, in the sidecar format.
const FRENCH: &str = r#"
"salt-in-water.summary" = "Le sel se dissout."
"salt-in-water.registers.lv1" = "Il disparaît."
"#;

const GERMAN: &str = r#"
"salt-in-water.summary" = "Salz löst sich."
"salt-in-water.registers.lv1" = "Es verschwindet."
"#;

#[test]
fn german_reaches_the_reader() {
    let codex = Codex::parse_with_translations(ENGLISH, &[("de", GERMAN)]).expect("parses");
    let entry = &codex.reactions[0];
    assert_eq!(entry.summary_de.as_deref(), Some("Salz löst sich."));
    assert_eq!(entry.registers.get_in(1, "de"), Some("Es verschwindet."));
}

/// The register map is keyed by string, so it carries any language.
#[test]
fn french_registers_survive() {
    let codex = Codex::parse_with_translations(ENGLISH, &[("fr", FRENCH)]).expect("parses");
    let entry = &codex.reactions[0];
    assert_eq!(
        entry.registers.get_in(1, "fr"),
        Some("Il disparaît."),
        "a map-shaped field should carry any locale"
    );
}

/// The typed fields are named `_de`. This asks whether that is a real
/// limit or only a naming habit, and it asks end to end: the web reads
/// serialised JSON, so if the French survives to there it survives to the
/// screen. If it does not, the sidecar was accepted and then discarded in
/// silence — which is the failure this whole translation has been prone to.
#[test]
fn french_summary_reaches_the_json_the_web_reads() {
    let codex = Codex::parse_with_translations(ENGLISH, &[("fr", FRENCH)]).expect("parses");
    let json = serde_json::to_string(&codex.reactions[0]).expect("serialises");
    assert!(
        json.contains("Le sel se dissout."),
        "the French summary was accepted by the parser and then lost.\n\
         serialised entry: {json}"
    );
}

/// The whole claim, end to end: a language arrives as one file in a
/// directory and nothing else changes.
///
/// `load_dir` is the path the CLI and the export both take, so this is
/// what a translator would actually experience. It is a test rather than
/// a thing checked once by hand because the failure mode is silence — the
/// French would simply not appear, and every other gate would stay green.
#[test]
fn a_new_language_is_one_file_in_a_directory() {
    let dir = std::env::temp_dir().join("kerotakis-french-probe");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("i18n")).expect("temp dir");
    std::fs::write(dir.join("salt.toml"), ENGLISH).expect("english");
    std::fs::write(dir.join("i18n/fr.toml"), FRENCH).expect("french");

    let codex = Codex::load_dir(&dir).expect("loads");
    let json = serde_json::to_string(&codex.reactions[0]).expect("serialises");

    assert!(
        json.contains("Le sel se dissout."),
        "the summary did not survive load_dir.\nserialised: {json}"
    );
    assert!(
        json.contains("Il disparaît."),
        "the register did not survive load_dir.\nserialised: {json}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}
