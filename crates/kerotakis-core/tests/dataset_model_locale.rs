//! `Provenance.dataset` and `.model` — which dataset answered and which
//! model it applies — in the reader's language.
//!
//! The fifth and last instance of the family `Inert.why` (#626),
//! `NotYetModeled.what` (#632), `scene_vessel` (#628) and `routing`
//! (#642) closed one field at a time. `kero explain` made this one plain
//! the moment it spoke German (#648):
//!
//! ```text
//! Modell:  WATEQ Debye-Hückel extension (reliable to about I = 1 mol/kgw)
//! ... mit wateq4f.dat plus USBM IC 9429 reference-temperature complexes,
//!     with the reviewed Sander HBr gas-uptake slice
//! ```
//!
//! `wateq4f.dat` is a NAME and must never be translated. What is welded
//! to it is a SENTENCE — the reliability range in a parenthesis, the
//! "plus …" and "with the reviewed …" clauses saying what this lab added
//! to the vendored file and why — and a German reader met all of it in
//! English.
//!
//! Same bargain #642 struck for the routing, and this is the gate over
//! both halves of it:
//!
//! 1. the English field is the recipe rendered in the SOURCE language, so
//!    nothing that reads the string can tell the change happened — and
//!    for `model` that is not a nicety, because `solve.rs` routes the
//!    colligative answer on `model.starts_with("Pitzer")`;
//! 2. a reader who does not read English gets the catalogue's sentence
//!    with the names still in it, wherever a host renders the event.
//!
//! The recipes here are built by hand rather than imported, because the
//! two composers live in `kerotakis-phreeqc` and `kerotakis-cea`, which
//! depend on this crate rather than the other way round. Each composer
//! pins its own English beside itself; this pins the plumbing they share.

use kerotakis_core::phrase::{Phrase, Slot};
use kerotakis_core::render::{render_event_in, Register};
use kerotakis_core::vessel::Provenance;
use kerotakis_core::{Locale, VesselId};

fn de() -> Locale {
    Locale::parse("de")
}

/// The dataset claim as the aqueous router composes it: a file name
/// nested through two clauses.
fn wateq4f() -> Phrase {
    Phrase::new(
        "provenance.dataset.with-gas-uptake-slice",
        "{file}, with the reviewed Sander HBr gas-uptake slice",
        vec![(
            Provenance::DATASET_FILE_SLOT.to_string(),
            Slot::phrase(Phrase::new(
                "provenance.dataset.plus-reference-complexes",
                "{file} plus USBM IC 9429 reference-temperature complexes",
                vec![(
                    Provenance::DATASET_FILE_SLOT.to_string(),
                    Slot::text("wateq4f.dat"),
                )],
            )),
        )],
    )
}

/// The model claim for the same dataset, with its reliability bound as a
/// number rather than as four characters of English.
fn wateq_model() -> Phrase {
    Phrase::new(
        "provenance.model.wateq-debye-huckel",
        "{name} extension (reliable to about I = {ionic_strength} mol/kgw)",
        vec![
            ("name".to_string(), Slot::text("WATEQ Debye-Hückel")),
            ("ionic_strength".to_string(), Slot::number("1")),
        ],
    )
}

fn routed() -> Provenance {
    Provenance::new(
        "PHREEQC (IPhreeqc, USGS)",
        wateq4f(),
        wateq_model(),
        Vec::new(),
        Phrase::bare(
            "routing.default-inorganic",
            "the default inorganic aqueous dataset",
        ),
    )
}

/// The English field is the recipe rendered in English, and not a second
/// copy of the sentence.
///
/// This is the property every consumer that is not a reader rests on:
/// `tools/chemistry-audit` files the string, the provenance tests assert
/// it with `assert_eq!`, and `solve.rs` branches on it.
#[test]
fn the_english_is_the_recipe_rendered_in_english() {
    let provenance = routed();
    assert_eq!(
        provenance.dataset,
        "wateq4f.dat plus USBM IC 9429 reference-temperature complexes, with the reviewed Sander HBr gas-uptake slice"
    );
    assert_eq!(
        provenance.model,
        "WATEQ Debye-Hückel extension (reliable to about I = 1 mol/kgw)"
    );
    assert_eq!(provenance.dataset_in(Locale::EN), provenance.dataset);
    assert_eq!(provenance.model_in(Locale::EN), provenance.model);
}

/// The branch a reword would have broken silently.
///
/// `solve.rs::solvent_activity_of` asks whether `Provenance.model` starts
/// with `states::ION_INTERACTION_MODEL_PREFIX` to choose between the
/// ion-interaction and the ideal colligative route, and one molal salt
/// water freezes at −3.4 °C on one and −3.61 °C on the other. The prefix
/// is a NAME in a text slot, so it is in the English AND in the German —
/// and the English is the half that decides.
#[test]
fn the_model_field_still_carries_the_prefix_the_solver_matches_on() {
    let pitzer = Provenance::new(
        "PHREEQC (IPhreeqc, USGS)",
        "pitzer.dat",
        Phrase::new(
            "provenance.model.ion-interaction",
            "{name} specific-ion-interaction model (valid at high ionic strength)",
            vec![("name".to_string(), Slot::text("Pitzer"))],
        ),
        Vec::new(),
        Phrase::bare("routing.default-inorganic", "x"),
    );
    assert!(
        pitzer
            .model
            .starts_with(kerotakis_core::states::ION_INTERACTION_MODEL_PREFIX),
        "{}",
        pitzer.model
    );
    // And in German too, because a person's name is not translated.
    assert!(
        pitzer.model_in(de()).contains("Pitzer"),
        "{}",
        pitzer.model_in(de())
    );
    // The German is not the English.
    assert!(
        !pitzer
            .model_in(de())
            .contains("valid at high ionic strength"),
        "{}",
        pitzer.model_in(de())
    );
}

/// German gets the clauses and keeps every name.
#[test]
fn german_translates_the_sentence_and_not_the_names() {
    let provenance = routed();
    let dataset = provenance.dataset_in(de());
    assert!(dataset.contains("wateq4f.dat"), "{dataset}");
    assert!(dataset.contains("USBM IC 9429"), "{dataset}");
    assert!(
        !dataset.contains("plus USBM IC 9429 reference"),
        "{dataset}"
    );
    assert!(!dataset.contains("with the reviewed"), "{dataset}");

    let model = provenance.model_in(de());
    assert!(model.contains("WATEQ Debye-Hückel"), "{model}");
    assert!(!model.contains("reliable to about"), "{model}");
}

/// The FILE comes back out of the sentence because it was PUT there.
///
/// `dataset_file` was a whitespace scan while `dataset` was a finished
/// string (#653) — the best guess available, and one that only worked
/// because English puts the noun first. It reads the named slot now, and
/// descends through the nesting to find it.
#[test]
fn the_file_name_is_read_from_the_recipe_and_not_cut_off_the_front() {
    assert_eq!(routed().dataset_file(), "wateq4f.dat");
    // A name the composer wrote AFTER two English words — the shape the
    // token scan got wrong, and the reason this is not a token scan.
    let vendored = Provenance::new(
        "Kerotakis analytic equilibrium evaluator",
        Phrase::new(
            "provenance.dataset.vendored-usgs",
            "vendored USGS {file}",
            vec![(
                Provenance::DATASET_FILE_SLOT.to_string(),
                Slot::text("phreeqc.dat"),
            )],
        ),
        "model",
        Vec::new(),
        Phrase::bare("routing.default-inorganic", "x"),
    );
    assert_eq!(vendored.dataset, "vendored USGS phreeqc.dat");
    assert_eq!(vendored.dataset_file(), "phreeqc.dat");
}

/// A dataset that is a NAME and nothing else carries no recipe, and the
/// scan still answers for it.
#[test]
fn a_bare_name_needs_no_recipe_and_comes_back_whole() {
    let combustion = Provenance::new(
        "curated combustion (Kerotakis)",
        "kerotakis:combustion:curated-fuels-v1",
        "model",
        Vec::new(),
        Phrase::bare("routing.default-inorganic", "x"),
    );
    assert_eq!(combustion.dataset_phrase, None);
    assert_eq!(
        combustion.dataset_file(),
        "kerotakis:combustion:curated-fuels-v1"
    );
    // A name renders as itself in every language, which is the point of
    // not giving it a catalogue row.
    assert_eq!(
        combustion.dataset_in(de()),
        "kerotakis:combustion:curated-fuels-v1"
    );
}

/// A session saved before the recipes existed still loads, and still
/// answers — in English, which is the language it was written in.
///
/// Both fields are `skip_serializing_if = "Option::is_none"`, so this is
/// exactly the JSON an older engine wrote.
#[test]
fn a_save_from_before_the_recipe_still_answers() {
    let json = r#"{
        "engine": "PHREEQC (IPhreeqc, USGS)",
        "dataset": "wateq4f.dat plus USBM IC 9429 reference-temperature complexes",
        "model": "Pitzer specific-ion-interaction model",
        "dataset_sources": [],
        "routing": "written before there was a recipe"
    }"#;
    let provenance: Provenance = serde_json::from_str(json).expect("an older save still loads");
    assert_eq!(provenance.dataset_phrase, None);
    assert_eq!(provenance.model_phrase, None);
    assert_eq!(provenance.dataset_in(de()), provenance.dataset);
    assert_eq!(provenance.model_in(de()), provenance.model);
    // The token scan is what it has, and it is what it always had.
    assert_eq!(provenance.dataset_file(), "wateq4f.dat");
    // And the solver's branch is unmoved by any of it.
    assert!(provenance
        .model
        .starts_with(kerotakis_core::states::ION_INTERACTION_MODEL_PREFIX));
}

/// The surface this reaches: a host that reads the EVENT rather than the
/// rendered line.
///
/// `provenance.ts` lifts `event.provenance` off any event and
/// `ProvenanceDrawer.svelte` prints `dataset` and `model` verbatim beside
/// the numbers, so both have to be German by the time the event leaves
/// the engine — exactly as `routing` already was.
#[test]
fn the_event_a_host_reads_carries_the_translated_claims() {
    let event = kerotakis_core::Event::SolutionRouted {
        vessel: VesselId(0),
        provenance: routed(),
    };
    let localized = kerotakis_core::localize_events(std::slice::from_ref(&event), de());
    let kerotakis_core::Event::SolutionRouted { provenance, .. } = &localized[0] else {
        panic!("the event survives localization as itself");
    };
    assert!(
        provenance.dataset.contains("wateq4f.dat"),
        "{}",
        provenance.dataset
    );
    assert!(
        !provenance.dataset.contains("with the reviewed"),
        "no English left in it: {}",
        provenance.dataset
    );
    assert!(
        !provenance.model.contains("reliable to about"),
        "no English left in it: {}",
        provenance.model
    );
    // The recipes are the SOURCE and stay in the source language: a host
    // that composes the sentence itself must not be handed one already
    // translated, or it would be translated twice.
    assert_eq!(
        provenance.dataset_phrase.as_ref().map(|p| p.key.as_str()),
        Some("provenance.dataset.with-gas-uptake-slice")
    );
    assert_eq!(
        provenance.model_phrase.as_ref().map(|p| p.key.as_str()),
        Some("provenance.model.wateq-debye-huckel")
    );
    // English is the no-op it has always been.
    assert_eq!(
        kerotakis_core::localize_events(std::slice::from_ref(&event), Locale::EN)[0],
        event
    );
}

/// The lines a reader actually sees, at every register, with no English
/// left in them.
///
/// LV1 names the FILE and nothing else, which is the register a
/// nine-year-old reads; LV2 and LV3 carry the sentence, and LV3 the model
/// as well.
#[test]
fn every_register_is_german_all_the_way_through() {
    let event = kerotakis_core::Event::SolutionRouted {
        vessel: VesselId(0),
        provenance: routed(),
    };
    let lv1 = render_event_in(&event, Register::LV1, de());
    assert!(lv1.contains("wateq4f.dat"), "{lv1}");
    assert!(
        !lv1.contains("plus USBM"),
        "the clause is not in lv1: {lv1}"
    );

    for register in [Register::LV2, Register::LV3] {
        let line = render_event_in(&event, register, de());
        assert!(line.contains("wateq4f.dat"), "{register:?}: {line}");
        assert!(
            !line.contains("with the reviewed") && !line.contains("reliable to about"),
            "{register:?} is German, not the English source: {line}"
        );
    }
}
