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
progress = "starter"
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

/// The models half of the catalogue, which was dark for a year.
///
/// `models.toml` reported 100% German while its `name`, `power`, `explains`
/// and `fails_at` were all English. Half of the reason was here: `Model` had
/// no `_de` fields and no catch-all, so a sidecar key for one of them was
/// injected into the TOML document correctly and then **dropped by serde**
/// on the way into the struct, because no field claimed it. The parse
/// succeeded, the export was English, and nothing anywhere said so.
///
/// The other half was that the coverage lint did not count those fields, so
/// the gap could not be reported either. `tools/tests/test_codex_locale_lint.py`
/// holds that half; this holds this one.
const MODEL_ENGLISH: &str = r#"
[[model]]
id = "particle-model"
name = "Particle model"
power = "Lets you predict that matter is conserved when it appears to vanish."
explains = ["dissolving without any loss of mass"]
fails_at = ["Cannot say WHY sodium and chlorine react, only that they do."]
[model.registers]
lv1 = "Everything is made of pieces too small to see."
lv2 = "One substance in three arrangements."
lv3 = "A continuum is the large-number limit."
[model.provenance]
source = "editorial"
"#;

const MODEL_GERMAN: &str = r#"
"particle-model.name" = "Teilchenmodell"
"particle-model.power" = "Lässt vorhersagen, dass Materie erhalten bleibt, wenn sie zu verschwinden scheint."
"particle-model.explains" = ["Lösen ohne jeden Masseverlust"]
"particle-model.fails_at" = ["Kann nicht sagen, WARUM Natrium und Chlor reagieren, nur dass sie es tun."]
"#;

const MODEL_FRENCH: &str = r#"
"particle-model.name" = "Modèle particulaire"
"particle-model.fails_at" = ["Ne peut pas dire POURQUOI le sodium et le chlore réagissent."]
"#;

#[test]
fn a_models_german_is_not_dropped_on_the_way_into_the_struct() {
    let codex =
        Codex::parse_with_translations(MODEL_ENGLISH, &[("de", MODEL_GERMAN)]).expect("parses");
    let model = &codex.models[0];
    assert_eq!(model.name_de.as_deref(), Some("Teilchenmodell"));
    assert!(
        model.power_de.is_some(),
        "power_de was accepted and then lost"
    );
    assert_eq!(
        model.explains_de,
        Some(vec!["Lösen ohne jeden Masseverlust".to_string()]),
    );
    assert_eq!(
        model.fails_at_de,
        Some(vec![
            "Kann nicht sagen, WARUM Natrium und Chlor reagieren, nur dass sie es tun.".to_string()
        ]),
        "fails_at is the field a model exists for; it must survive"
    );
}

/// The same claim for a language the types do NOT name, which is the one
/// that proves the catch-all rather than the four named fields.
#[test]
fn a_models_french_reaches_the_json_the_web_reads() {
    let codex =
        Codex::parse_with_translations(MODEL_ENGLISH, &[("fr", MODEL_FRENCH)]).expect("parses");
    let json = serde_json::to_string(&codex.models[0]).expect("serialises");
    assert!(
        json.contains("Modèle particulaire"),
        "the French model name was accepted by the parser and then lost.\n\
         serialised model: {json}"
    );
    assert!(
        json.contains("Ne peut pas dire POURQUOI"),
        "the French boundary was accepted by the parser and then lost.\n\
         serialised model: {json}"
    );
}
