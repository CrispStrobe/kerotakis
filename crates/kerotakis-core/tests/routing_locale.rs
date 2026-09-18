//! `Provenance.routing` — why one dataset answered and not another — in
//! the reader's language.
//!
//! The routing line is the third place a solver welded a finished English
//! paragraph shut with `format!`, after `Inert.why` (#626) and
//! `NotYetModeled.what` (#632), and it is the one that is said BESIDE the
//! numbers rather than instead of them. It is also the one with consumers
//! that are not readers: `tools/chemistry-audit/analyse.py` files it as a
//! `routing_claim`, `kero explain` prints it, and three tests in
//! `kerotakis-phreeqc` assert on what it says. So the field keeps the
//! English and gains a recipe beside it, and this is the gate over both
//! halves of that bargain:
//!
//! 1. the English is the recipe rendered in the SOURCE language, so
//!    nothing that reads the string can tell the change happened;
//! 2. a reader who does not read English gets the catalogue's sentence,
//!    holes and all, wherever a host renders the event.

use kerotakis_core::phrase::{Phrase, Slot};
use kerotakis_core::render::{render_event_in, Register};
use kerotakis_core::vessel::Provenance;
use kerotakis_core::{Kelvin, Locale, VesselId};

fn de() -> Locale {
    Locale::parse("de")
}

/// `Provenance::new` fills the English from the recipe rather than from a
/// second copy of the sentence. Two copies is how a product starts
/// reading like two.
#[test]
fn the_english_is_the_recipe_rendered_in_english() {
    let routing = Phrase::new(
        "routing.concentrated-ion-interaction",
        "chosen because the solution is concentrated (~{molality} mol/kgw), where the ion-interaction model is the valid one",
        vec![("molality".to_string(), Slot::number("16.0"))],
    );
    let provenance = Provenance::new("engine", "dataset", "model", Vec::new(), routing);
    assert_eq!(
        provenance.routing,
        "chosen because the solution is concentrated (~16.0 mol/kgw), where the ion-interaction model is the valid one"
    );
    assert_eq!(provenance.routing_in(Locale::EN), provenance.routing);
}

/// …and the German is the catalogue's, with the measurement given the
/// reader's decimal separator on the way through. `16,0` is the point of
/// the typed slot: a `format!` had already baked the point in.
#[test]
fn german_reads_the_catalogue_and_the_readers_separator() {
    let provenance = Provenance::new(
        "engine",
        "dataset",
        "model",
        Vec::new(),
        Phrase::new(
            "routing.concentrated-ion-interaction",
            "chosen because the solution is concentrated (~{molality} mol/kgw), where the ion-interaction model is the valid one",
            vec![("molality".to_string(), Slot::number("16.0"))],
        ),
    );
    let german = provenance.routing_in(de());
    assert!(
        german.contains("konzentriert") && german.contains("16,0 mol/kgw"),
        "{german}"
    );
    assert!(
        !german.contains("chosen because"),
        "no English left in it: {german}"
    );
}

/// The nesting the electrode pass needs: one solver's routing inside
/// another's clause, translated as one sentence rather than two glued
/// together. A `push_str` could not have been translated at all.
#[test]
fn a_nested_routing_is_one_translated_sentence() {
    let mut provenance = Provenance::new(
        "engine",
        "dataset",
        "model",
        Vec::new(),
        Phrase::bare(
            "routing.default-inorganic",
            "the default inorganic aqueous dataset",
        ),
    );
    let outer = Phrase::new(
        "routing.with-redox-note",
        "{routing}. {note}",
        vec![
            ("routing".to_string(), provenance.routing_slot()),
            (
                "note".to_string(),
                Slot::phrase(Phrase::bare(
                    "routing.slow-couples-held-as-added",
                    "some elements here keep the oxidation state they were added in: only the couples that equilibrate on a bench timescale exchange electrons, and the slow ones — sulfate, nitrate, carbonate — are held as added",
                )),
            ),
        ],
    );
    provenance.say_routing(outer);
    assert_eq!(
        provenance.routing,
        "the default inorganic aqueous dataset. some elements here keep the oxidation state they were added in: only the couples that equilibrate on a bench timescale exchange electrons, and the slow ones — sulfate, nitrate, carbonate — are held as added"
    );
    let german = provenance.routing_in(de());
    assert!(
        german.starts_with("der voreingestellte anorganische wässrige Datensatz. manche Elemente"),
        "both halves German, and joined by the catalogue: {german}"
    );
}

/// A provenance read back from a session saved before the recipe existed
/// carries only the English, and is handed on rather than mangled: the
/// engine cannot translate a sentence it did not compose.
#[test]
fn a_provenance_without_a_recipe_keeps_its_english() {
    let provenance = Provenance {
        engine: "engine".to_string(),
        dataset: "dataset".to_string(),
        model: "model".to_string(),
        dataset_sources: Vec::new(),
        routing: "written before there was a recipe".to_string(),
        routing_phrase: None,
        dataset_phrase: None,
        model_phrase: None,
    };
    assert_eq!(
        provenance.routing_in(de()),
        "written before there was a recipe"
    );
}

/// The surface this actually reaches: `ThermalEquilibrium` is the one
/// event that carries a `Provenance`, the provenance drawer prints
/// `provenance.routing` verbatim, and `localize_events` is where the
/// engine learns who is reading.
#[test]
fn the_event_a_host_reads_carries_the_translated_routing() {
    let event = kerotakis_core::Event::ThermalEquilibrium {
        vessel: VesselId(0),
        temperature: Kelvin(2769.0),
        reaction_energy_j: None,
        holds_nothing: false,
        provenance: Provenance::new(
            "curated combustion (Kerotakis)",
            "kerotakis:combustion:curated-fuels-v1",
            "model",
            Vec::new(),
            Phrase::bare(
                "routing.curated-fuel-table",
                "NASA CEA carries no thermochemistry for this fuel, so the curated table answered instead of the vessel reaching the model boundary",
            ),
        ),
    };
    let localized = kerotakis_core::localize_events(std::slice::from_ref(&event), de());
    let kerotakis_core::Event::ThermalEquilibrium { provenance, .. } = &localized[0] else {
        panic!("the event survives localization as itself");
    };
    assert!(
        provenance.routing.starts_with("NASA CEA führt"),
        "{}",
        provenance.routing
    );
    // The recipe is the source and stays in the source language: a host
    // that composes the sentence itself must not be handed one that has
    // already been translated once.
    assert_eq!(
        provenance.routing_phrase.as_ref().map(|p| p.key.as_str()),
        Some("routing.curated-fuel-table")
    );
    // English is the no-op it has always been.
    let untouched = kerotakis_core::localize_events(std::slice::from_ref(&event), Locale::EN);
    assert_eq!(untouched[0], event);
}

/// The aqueous half, which is why `Event::SolutionRouted` exists.
///
/// `ProvenanceDrawer.svelte` prints `source.routing` verbatim off whatever
/// event carries a provenance, so the routing has to be German by the time
/// the event leaves the engine — exactly as the combustion one above is.
/// The reader in `provenance.ts` is written against the FIELD and not
/// against a list of event names, so this arm is the whole of the wiring.
#[test]
fn the_aqueous_routing_reaches_a_host_in_the_readers_language() {
    let event = kerotakis_core::Event::SolutionRouted {
        vessel: VesselId(0),
        provenance: Provenance::new(
            "PHREEQC (IPhreeqc, USGS)",
            "pitzer.dat",
            "Pitzer specific-ion-interaction",
            Vec::new(),
            Phrase::new(
                "routing.concentrated-ion-interaction",
                "chosen because the solution is concentrated (~{molality} mol/kgw), where the ion-interaction model is the valid one",
                vec![("molality".to_string(), Slot::number("16.0"))],
            ),
        ),
    };
    let localized = kerotakis_core::localize_events(std::slice::from_ref(&event), de());
    let kerotakis_core::Event::SolutionRouted { provenance, .. } = &localized[0] else {
        panic!("the event survives localization as itself");
    };
    assert!(
        provenance.routing.contains("konzentriert") && provenance.routing.contains("16,0 mol/kgw"),
        "{}",
        provenance.routing
    );
    assert!(
        !provenance.routing.contains("chosen because"),
        "no English left in it: {}",
        provenance.routing
    );
    // The dataset is a FILE NAME and is never translated.
    assert_eq!(provenance.dataset, "pitzer.dat");
    // The recipe stays in the source language, as the combustion one does.
    assert_eq!(
        provenance.routing_phrase.as_ref().map(|p| p.key.as_str()),
        Some("routing.concentrated-ion-interaction")
    );
    assert_eq!(
        kerotakis_core::localize_events(std::slice::from_ref(&event), Locale::EN)[0],
        event
    );
}

/// The line a reader actually sees, in German, at every register.
///
/// `localize_event` renders the event; this is the gate that the three
/// catalogue rows exist and that no English survives into the German
/// sentence. A key with no German row would render its English source and
/// pass every other test in this file.
#[test]
fn the_rendered_line_is_german_at_every_register() {
    let event = kerotakis_core::Event::SolutionRouted {
        vessel: VesselId(0),
        provenance: Provenance::new(
            "PHREEQC (IPhreeqc, USGS)",
            "pitzer.dat",
            "Pitzer specific-ion-interaction",
            Vec::new(),
            Phrase::bare(
                "routing.default-inorganic",
                "the default inorganic aqueous dataset",
            ),
        ),
    };
    for register in [Register::LV1, Register::LV2, Register::LV3] {
        let line = render_event_in(&event, register, de());
        assert!(
            line.contains("pitzer.dat"),
            "{register} names the file: {line}"
        );
        assert!(
            !line.contains("is being worked out")
                && !line.contains("answered by")
                && !line.contains("routing →"),
            "{register} is German, not the English source: {line}"
        );
    }
}

/// LV1 gets the dataset's FILE, not the English clause welded to it.
///
/// `dataset` is a name with a sentence on the end of it — *wateq4f.dat
/// plus USBM IC 9429 reference-temperature complexes, with the reviewed
/// Sander HBr gas-uptake slice* — and LV1 is the register a nine-year-old
/// reads. An English clause in the middle of a German line is the defect
/// this whole family of changes exists to remove, and putting a new one
/// there would be adding to it.
#[test]
fn lv1_names_the_file_and_not_the_clause_welded_to_it() {
    let provenance = Provenance::new(
        "PHREEQC (IPhreeqc, USGS)",
        "wateq4f.dat plus USBM IC 9429 reference-temperature complexes, with the reviewed Sander HBr gas-uptake slice",
        "WATEQ Debye-Hückel extension (reliable to about I = 1 mol/kgw)",
        Vec::new(),
        Phrase::bare(
            "routing.default-inorganic",
            "the default inorganic aqueous dataset",
        ),
    );
    assert_eq!(provenance.dataset_file(), "wateq4f.dat");
    // A dataset id with no space in it comes back whole.
    let combustion = Provenance::new(
        "curated combustion (Kerotakis)",
        "kerotakis:combustion:curated-fuels-v1",
        "model",
        Vec::new(),
        Phrase::bare("routing.default-inorganic", "x"),
    );
    assert_eq!(
        combustion.dataset_file(),
        "kerotakis:combustion:curated-fuels-v1"
    );

    let event = kerotakis_core::Event::SolutionRouted {
        vessel: VesselId(0),
        provenance,
    };
    let lv1 = render_event_in(&event, Register::LV1, de());
    assert!(lv1.contains("wateq4f.dat"), "{lv1}");
    assert!(
        !lv1.contains("USBM") && !lv1.contains("with the reviewed"),
        "no English clause in a German lv1 line: {lv1}"
    );
    // LV2 keeps the whole field, which is what `explain` and the drawer
    // already print, and is where the reader asking for it is.
    let lv2 = render_event_in(&event, Register::LV2, de());
    assert!(lv2.contains("USBM IC 9429"), "{lv2}");
}

/// `Phrase::shape` is what `Event::SolutionRouted` compares, so it has to
/// hold two properties that a rendered comparison does not: a measurement
/// moving is the same shape, and a different NAME is not.
#[test]
fn a_routings_shape_ignores_its_measurements_and_keeps_its_names() {
    let at = |molality: &str| {
        Phrase::new(
            "routing.concentrated-ion-interaction",
            "chosen because the solution is concentrated (~{molality} mol/kgw), where the ion-interaction model is the valid one",
            vec![("molality".to_string(), Slot::number(molality))],
        )
    };
    assert_eq!(at("16.0").shape(), at("16.2").shape());
    assert_ne!(
        at("16.0").shape(),
        Phrase::bare(
            "routing.default-inorganic",
            "the default inorganic aqueous dataset"
        )
        .shape(),
        "a different reason is a different shape"
    );

    // A dataset NAME lives in a text slot, and swapping it is a different
    // answer however alike the two sentences read.
    let second = |dataset: &str| {
        Phrase::new(
            "routing.second-speciation-for-solvent",
            "{routing}; the solvent's activity is from {dataset}",
            vec![
                (
                    "routing".to_string(),
                    Slot::phrase(Phrase::bare("routing.default-inorganic", "the default")),
                ),
                ("dataset".to_string(), Slot::text(dataset)),
            ],
        )
    };
    assert_ne!(second("pitzer.dat").shape(), second("wateq4f.dat").shape());
    assert_eq!(second("pitzer.dat").shape(), second("pitzer.dat").shape());
}
