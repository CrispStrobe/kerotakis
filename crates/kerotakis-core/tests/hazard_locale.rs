//! The safety notes reach the reader in their language.
//!
//! These are the strings that say why the thing would hurt you on a real
//! bench. They are also the ones the shell reads straight off the event
//! rather than through `render_event_in`, so they stayed English under an
//! otherwise German session — a German frame around English prose, in the
//! one place where not being read is dangerous rather than untidy.

use kerotakis_core::render::{localize_event, localize_events};
use kerotakis_core::solve::Severity;
use kerotakis_core::{Event, Locale};

fn warn(hazard: &str, real_world: &str) -> Event {
    Event::HazardWarning {
        severity: Severity::Caution,
        hazard: hazard.to_string(),
        real_world: real_world.to_string(),
    }
}

fn parts(event: &Event) -> (String, String) {
    match event {
        Event::HazardWarning {
            hazard, real_world, ..
        } => (hazard.clone(), real_world.clone()),
        other => panic!("not a hazard warning: {other:?}"),
    }
}

#[test]
fn a_static_hazard_and_its_reason_are_translated() {
    let event = warn(
        "mixing bleach with ammonia makes chloramine, a toxic gas",
        "People are hospitalised every year from mixing these two household cleaners.",
    );
    let (hazard, real_world) = parts(&localize_event(&event, Locale::parse("de")));
    assert!(
        hazard.contains("Chloramin"),
        "hazard not translated: {hazard}"
    );
    assert!(
        real_world.contains("Krankenhaus"),
        "the reason not translated: {real_world}"
    );
}

/// Every hazardous substance has a German sentence.
///
/// The fallback is silent by construction — an untranslated hazard renders
/// in English and nothing reports it — so the only way this stays true is
/// to walk the table the sentences are built from and check each one.
/// Adding a hazardous odour without German fails here rather than shipping.
#[test]
fn hazardous_vapours_are_all_translated() {
    let de = Locale::parse("de");
    let mut missing = Vec::new();
    for odor in kerotakis_core::senses::ODORS {
        if !odor.hazardous {
            continue;
        }
        let english = format!("{} vapour is hazardous to inhale", odor.species);
        let (hazard, _) = parts(&localize_event(&warn(&english, "…"), de));
        if hazard == english {
            missing.push(english);
        }
    }
    assert!(missing.is_empty(), "no German for: {missing:#?}");
}

/// German hyphenates a formula onto a noun, and calls a gas a gas.
#[test]
fn the_vapour_sentences_are_idiomatic() {
    let de = Locale::parse("de");
    let (ammonia, _) = parts(&localize_event(
        &warn("NH3 vapour is hazardous to inhale", "…"),
        de,
    ));
    assert!(
        ammonia.contains("NH3-Dampf"),
        "a formula compounds with a hyphen: {ammonia}"
    );
    let (chlorine, _) = parts(&localize_event(
        &warn("Cl2 vapour is hazardous to inhale", "…"),
        de,
    ));
    assert!(
        chlorine.contains("Gas"),
        "chlorine is already a gas, not a vapour: {chlorine}"
    );
}

/// An untranslated hazard renders in English rather than disappearing.
#[test]
fn an_unknown_hazard_survives_untranslated() {
    let event = warn("something nobody has written German for", "nor this");
    let (hazard, real_world) = parts(&localize_event(&event, Locale::parse("de")));
    assert_eq!(hazard, "something nobody has written German for");
    assert_eq!(real_world, "nor this");
}

#[test]
fn english_is_left_exactly_as_it_was() {
    let event = warn(
        "mixing bleach with acid releases chlorine, a toxic gas",
        "Chlorine gas was used as a chemical weapon; even small amounts damage lungs.",
    );
    let (hazard, real_world) = parts(&localize_event(&event, Locale::EN));
    assert_eq!(
        hazard,
        "mixing bleach with acid releases chlorine, a toxic gas"
    );
    assert!(real_world.starts_with("Chlorine gas"));
}

/// Events that are not hazards pass through untouched.
#[test]
fn other_events_are_not_disturbed() {
    let events = vec![warn("radioactive source: ionising radiation", "…")];
    let out = localize_events(&events, Locale::parse("de"));
    assert_eq!(out.len(), 1);
    let (hazard, _) = parts(&out[0]);
    assert!(hazard.contains("Strahlung"), "{hazard}");
}
