//! The honesty pass says a thing once, not once per step.
//!
//! `Event::Inert` and the `NotYetModeled` twin beside it answer the same
//! question every step — *is this solid doing anything?* — and answered it
//! out loud every step. A live transcript (2026-09-18, the peroxide/chalk
//! vessel) carried the same forty-word German sentence about chalk's
//! solubility more than thirty times, and *"Silbernitrat ist mit einer
//! Flüssigkeit in Kontakt"* ten times.
//!
//! The mechanism is #653's, deliberately not a second one: compare
//! [`kerotakis_core::phrase::Phrase::shape`] against a `#[serde(skip)]`
//! field on the vessel, exactly as `Vessel::aqueous_routing_said` does for
//! `Event::SolutionRouted`.
//!
//! `render_events` already collapsed identical lines, but only inside ONE
//! batch and only at lv1 — which is why thirty steps of the same sentence
//! survived it at every register.

use kerotakis_core::render::{render_events_in, Register};
use kerotakis_core::*;

fn stack() -> SolverStack {
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(HonestyEquilibrator),
    ])
}

fn add(
    bench: &mut Bench,
    solver: &mut SolverStack,
    v: VesselId,
    key: &str,
    moles: f64,
) -> Vec<Event> {
    bench
        .step_with(
            Operator::Add {
                vessel: v,
                species: SpeciesId::new(key),
                moles: Moles(moles),
                at: None,
            },
            solver,
            &PermissiveScreen,
        )
        .expect("add")
}

fn stir(bench: &mut Bench, solver: &mut SolverStack, v: VesselId) -> Vec<Event> {
    bench
        .step_with(
            Operator::Stir {
                vessel: v,
                rpm: 60.0,
                seconds: 1.0,
            },
            solver,
            &PermissiveScreen,
        )
        .expect("stir")
}

fn inert_lines(events: &[Event]) -> Vec<&Event> {
    events
        .iter()
        .filter(|e| matches!(e, Event::Inert { .. }))
        .collect()
}

fn apologies(events: &[Event]) -> Vec<&Event> {
    events
        .iter()
        .filter(|e| matches!(e, Event::NotYetModeled { .. }))
        .collect()
}

/// Chalk under water: said once, and then it stands.
#[test]
fn an_inert_solid_is_named_once_and_not_once_per_step() {
    let mut bench = Bench::new();
    let mut solver = stack();
    let v = VesselId(0);
    add(&mut bench, &mut solver, v, "water", 5.5);
    let introduced = add(&mut bench, &mut solver, v, "CaCO3", 0.01);
    assert_eq!(
        inert_lines(&introduced).len(),
        1,
        "the reader is told once, when it becomes true: {introduced:#?}"
    );

    for round in 0..8 {
        let more = stir(&mut bench, &mut solver, v);
        assert!(
            inert_lines(&more).is_empty(),
            "round {round}: nothing about the chalk has changed, so there is \
             nothing to say: {:#?}",
            inert_lines(&more)
        );
    }
}

/// A solid the bench has no dissolution route for gets the same
/// treatment, and needs it: neither of its two recipes carries a
/// measurement, so every repeat is a repeat of the whole sentence.
#[test]
fn the_unmodelled_dissolution_apology_is_also_said_once() {
    let mut bench = Bench::new();
    let mut solver = stack();
    let v = VesselId(0);
    add(&mut bench, &mut solver, v, "water", 5.5);
    let introduced = add(&mut bench, &mut solver, v, "Mg(OH)2", 0.02);
    assert_eq!(
        apologies(&introduced)
            .iter()
            .filter(|e| matches!(e, Event::NotYetModeled { what, .. } if what.contains("no wired solver models")))
            .count(),
        1,
        "{introduced:#?}"
    );

    for round in 0..5 {
        let more = stir(&mut bench, &mut solver, v);
        assert!(
            !apologies(&more)
                .iter()
                .any(|e| matches!(e, Event::NotYetModeled { what, .. } if what.contains("no wired solver models"))),
            "round {round}: {:#?}",
            apologies(&more)
        );
    }
}

/// Two solids are two sentences. The shape carries the species as a
/// catalogue TERM, so one standing sentence cannot silence another's.
#[test]
fn a_second_solid_gets_its_own_sentence() {
    let mut bench = Bench::new();
    let mut solver = stack();
    let v = VesselId(0);
    add(&mut bench, &mut solver, v, "water", 5.5);
    let chalk = add(&mut bench, &mut solver, v, "CaCO3", 0.01);
    assert_eq!(inert_lines(&chalk).len(), 1, "{chalk:#?}");
    let sand = add(&mut bench, &mut solver, v, "SiO2", 0.01);
    let named = inert_lines(&sand);
    assert_eq!(
        named.len(),
        1,
        "the sand is news even though the chalk's sentence still stands: {sand:#?}"
    );
    assert!(
        matches!(named[0], Event::Inert { species, .. } if species.0 == "SiO2"),
        "and it is the sand that is named: {named:#?}"
    );
}

/// A solid that stops being inert and is inert again is announced again.
/// The vessel holds what STANDS, not everything ever said.
#[test]
fn a_sentence_that_stopped_standing_is_said_again() {
    let mut bench = Bench::new();
    let mut solver = stack();
    let v = VesselId(0);
    add(&mut bench, &mut solver, v, "water", 5.5);
    assert_eq!(
        inert_lines(&add(&mut bench, &mut solver, v, "CaCO3", 0.01)).len(),
        1
    );
    assert!(inert_lines(&stir(&mut bench, &mut solver, v)).is_empty());

    // Take the chalk out — filtered off, in bench terms. The state is
    // edited directly because this test is about the RECORD rather than
    // about any one verb: what matters is that the sentence stops being
    // true. (Pouring the WATER off would not do it: an aqueous portion of
    // the chalk stays behind, the solid is still in contact with a
    // liquid, and the sentence still stands — correctly.)
    let held = bench
        .vessels
        .iter_mut()
        .find(|vessel| vessel.id == v)
        .expect("the vessel");
    held.contents
        .retain(|portion| portion.species != SpeciesId::new("CaCO3"));
    assert!(
        inert_lines(&stir(&mut bench, &mut solver, v)).is_empty(),
        "there is no chalk to say anything about"
    );

    // Put chalk back and the reader is told again, because they are being
    // told about a state that has just become true again.
    let again = add(&mut bench, &mut solver, v, "CaCO3", 0.01);
    assert_eq!(
        inert_lines(&again).len(),
        1,
        "it became true again, so it is news again: {again:#?}"
    );
}

/// The suppression is a property of the sentence, not of English.
///
/// This is why the comparison is `Phrase::shape` and not the rendered
/// string: a shape is locale-free by construction, so a German reader is
/// told exactly as often as an English one — no more, and no fewer.
#[test]
fn a_german_reader_is_told_exactly_as_often() {
    fn line_counts(locale: Locale) -> Vec<usize> {
        let mut bench = Bench::new();
        let mut solver = stack();
        let v = VesselId(0);
        add(&mut bench, &mut solver, v, "water", 5.5);
        let mut counts = vec![render_events_in(
            &add(&mut bench, &mut solver, v, "CaCO3", 0.01),
            Register::LV2,
            locale,
        )
        .len()];
        for _ in 0..4 {
            counts.push(
                render_events_in(&stir(&mut bench, &mut solver, v), Register::LV2, locale).len(),
            );
        }
        counts
    }
    assert_eq!(
        line_counts(Locale::EN),
        line_counts(Locale::parse("de")),
        "the same beaker produces the same NUMBER of lines in both languages"
    );
}
