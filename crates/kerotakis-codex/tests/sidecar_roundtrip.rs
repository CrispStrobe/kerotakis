//! A sidecar must produce exactly what inline `_de` keys produce.
//!
//! The catalogue is moving its translations out of the English source and
//! into one file per language, so that two translators never edit the same
//! file. That is only safe if the result is identical — and "identical" has
//! to mean the parsed structure, not a diff of the text, because the whole
//! point is that the text is arranged differently.
//!
//! So: strip every `_de` key from a fixture, feed the same translations
//! back through a sidecar, and require the two parses to agree.

use kerotakis_codex::Codex;

/// One reaction with translations inline, as the catalogue has them today.
const INLINE: &str = r#"
[[reaction]]
id = "salt-in-water"
summary = "Salt dissolves."
summary_de = "Salz löst sich."
system = "aqueous"
stage = "solutions"
apparatus = ["beaker"]
concepts = ["dissolving"]
[reaction.setup]
script = "add v1 water 100mL"
[reaction.registers]
lv1 = "It disappears."
lv1_de = "Es verschwindet."
lv2 = "0.1 mol dissolves."
lv2_de = "0,1 mol lösen sich."
lv3 = "Fully dissociated."
lv3_de = "Vollständig dissoziiert."
[reaction.provenance]
source = "editorial"
[[reaction.expect.predict.diagnosis]]
option = 1
reveals = "Thinks salt vanishes."
reveals_de = "Hält Salz für verschwunden."
[reaction.expect.predict]
question = "What happens?"
question_de = "Was passiert?"
options = ["It dissolves", "It sinks"]
options_de = ["Es löst sich", "Es sinkt"]
answer = 0
"#;

/// The same reaction with no German at all.
const ENGLISH_ONLY: &str = r#"
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

/// The German, in the shape a sidecar uses: `<entry-id>.<path>`.
const SIDECAR: &str = r#"
"salt-in-water.summary" = "Salz löst sich."
"salt-in-water.registers.lv1" = "Es verschwindet."
"salt-in-water.registers.lv2" = "0,1 mol lösen sich."
"salt-in-water.registers.lv3" = "Vollständig dissoziiert."
"salt-in-water.expect.predict.question" = "Was passiert?"
"salt-in-water.expect.predict.options" = ["Es löst sich", "Es sinkt"]
"salt-in-water.expect.predict.diagnosis.0.reveals" = "Hält Salz für verschwunden."
"#;

#[test]
fn a_sidecar_reproduces_inline_translations_exactly() {
    let inline = Codex::parse(INLINE).expect("inline parses");
    let merged =
        Codex::parse_with_translations(ENGLISH_ONLY, &[("de", SIDECAR)]).expect("sidecar merges");

    let a = serde_json::to_value(&inline).expect("inline serialises");
    let b = serde_json::to_value(&merged).expect("merged serialises");
    assert_eq!(a, b, "a sidecar must produce exactly what inline keys do");
}

#[test]
fn a_key_naming_a_field_that_is_not_there_is_an_error() {
    // The failure this exists to catch: the English is reworded or removed
    // and the translation is left describing something that is gone. That
    // renders confidently and wrongly, which is worse than a gap.
    let stale = r#""salt-in-water.registers.lv4" = "Gibt es nicht.""#;
    let err = Codex::parse_with_translations(ENGLISH_ONLY, &[("de", stale)])
        .expect_err("a stale key must not be tolerated");
    let msg = err.to_string();
    assert!(msg.contains("lv4"), "the error should name the path: {msg}");
}

#[test]
fn a_key_naming_an_entry_that_is_not_there_is_an_error() {
    let stale = r#""no-such-entry.summary" = "Gibt es nicht.""#;
    let err = Codex::parse_with_translations(ENGLISH_ONLY, &[("de", stale)])
        .expect_err("a stale entry must not be tolerated");
    assert!(
        err.to_string().contains("no-such-entry"),
        "the error should name the entry: {err}"
    );
}

#[test]
fn english_alone_still_parses_and_carries_no_german() {
    let codex = Codex::parse_with_translations(ENGLISH_ONLY, &[]).expect("parses with no sidecar");
    let json = serde_json::to_string(&codex).expect("serialises");
    assert!(!json.contains("_de"), "no sidecar means no German: {json}");
}
