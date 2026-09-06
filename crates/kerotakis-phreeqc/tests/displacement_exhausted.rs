//! The other half of the series grid: a metal that has finished.
//!
//! `bystanders` writes an `Inert` for a metal nothing is happening to, and
//! for a long time it wrote the same sentence in two opposite situations.
//! Silver in copper sulfate does nothing because the couple runs uphill —
//! "silver sits above copper … the electrons would have to flow uphill" —
//! and that is right. Iron left over in a beaker whose copper has all
//! plated onto it got that sentence too, in the same step as the events
//! saying the iron had just displaced the copper:
//!
//! ```text
//! Fe + Cu+2 → Fe+2 + Cu · 0.0199 mol iron consumed · 0.0199 mol copper plated out onto iron
//! v1: iron does not react — iron sits above copper in the activity series
//!     (E° -0.447 V against +0.342 V), so the electrons would have to flow
//!     uphill: the less reactive metal does not displace the more reactive one
//! ```
//!
//! Iron is the MORE reactive metal there and E° says so on the same line.
//! What is true is that there is no copper left to displace, which is a
//! different sentence and now the one that gets written.

#![cfg(feature = "engine")]

use kerotakis_core::displacement::SERIES;
use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn stack() -> SolverStack {
    let aqueous = PhreeqcEquilibrator::new().expect("engine");
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(DisplacementEquilibrator::wrapping(Box::new(aqueous))),
        Box::new(StateEquilibrator),
        Box::new(HonestyEquilibrator),
    ])
}

fn add(bench: &mut Bench, stack: &mut SolverStack, key: &str, moles: f64) -> Vec<Event> {
    bench
        .step_with(
            Operator::Add {
                vessel: VesselId(0),
                species: SpeciesId::new(key),
                moles: Moles(moles),
                at: None,
            },
            stack,
            &PermissiveScreen,
        )
        .expect("step")
}

/// 100 mL of water, then each addition in turn; the last step's events.
fn run(additions: &[(&str, f64)]) -> (Bench, Vec<Event>) {
    let mut stack = stack();
    let mut bench = Bench::new();
    add(&mut bench, &mut stack, "water", 5.55);
    let mut last = Vec::new();
    for (key, moles) in additions {
        last = add(&mut bench, &mut stack, key, *moles);
    }
    (bench, last)
}

fn e0(key: &str) -> f64 {
    SERIES
        .iter()
        .find(|c| c.reduced == key)
        .expect("couple")
        .e0_volts
}

/// The transcript that started this. Iron into copper sulfate, with iron
/// left over: the copper plates, and what the bench then says about the
/// spare iron must not contradict the line above it.
#[test]
fn leftover_iron_is_told_the_copper_ran_out_not_that_it_cannot_displace_it() {
    let (bench, events) = run(&[("CuSO4", 0.02), ("Fe", 0.05)]);

    // The displacement happened.
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::Plated { species, onto, .. } if species.0 == "Cu" && onto.0 == "Fe"
        )),
        "iron displaces copper: {events:?}"
    );
    // And there is iron left over to have an opinion about.
    let iron_left: f64 = bench
        .vessel(VesselId(0))
        .expect("vessel")
        .contents
        .iter()
        .filter(|p| p.species.0 == "Fe" && p.phase == Phase::Solid)
        .map(|p| p.moles.0)
        .sum();
    assert!(iron_left > 0.02, "iron in excess: {iron_left}");

    let iron_verdicts: Vec<&Event> = events
        .iter()
        .filter(|e| matches!(e, Event::Inert { species, .. } if species.0 == "Fe"))
        .collect();
    assert!(
        !iron_verdicts.is_empty(),
        "a verdict about the iron: {events:?}"
    );

    // The defect, named: never the uphill sentence when E° says downhill.
    // Asserted over EVERY verdict about the iron, not one of them — the
    // point is that the sentence cannot be written, not that some other
    // sentence was written too.
    assert!(
        e0("Fe") < e0("Cu"),
        "the premise: iron is the more reactive metal"
    );
    for verdict in &iron_verdicts {
        let Event::Inert { why, .. } = verdict else {
            unreachable!()
        };
        assert!(
            !why.contains("uphill") && !why.contains("sits above copper"),
            "iron does not sit above copper: {why}"
        );
    }

    // The beaker also has an acid in it, and iron in a weak acid is
    // kinetically blocked — a true sentence about a different question,
    // and the one the displacement solver writes first. The verdict this
    // test is about is the one that names the ion that ran out.
    let spent_verdict = iron_verdicts
        .iter()
        .copied()
        .find(|e| matches!(e, Event::Inert { spent: Some(_), .. }))
        .unwrap_or_else(|| panic!("a verdict about the copper running out: {events:?}"));
    let Event::Inert { why, spent, .. } = spent_verdict else {
        unreachable!()
    };
    assert!(
        why.contains("plated out") && why.contains("nothing left to displace"),
        "the reason is that the copper ran out: {why}"
    );
    assert_eq!(
        spent.as_ref().map(|s| s.0.as_str()),
        Some("Cu+2"),
        "and it names the ion that was used up"
    );

    // What a nine-year-old is told, which is where the contradiction was
    // loudest: the lv1 sentence used to be "the iron does not swap places
    // with anything dissolved here".
    let lv1 = render_event(spent_verdict, Register::LV1);
    assert!(
        lv1.contains("copper") && lv1.contains("nothing left to do"),
        "lv1: {lv1}"
    );
    assert!(
        !lv1.contains("does not swap places"),
        "not the generic refusal: {lv1}"
    );
}

/// The uphill cell of the grid is untouched: silver in copper sulfate is
/// still told, in those words, that it sits above copper.
#[test]
fn the_uphill_verdict_is_unchanged_where_it_was_right() {
    let (_, events) = run(&[("CuSO4", 0.01), ("Ag", 0.02)]);
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::Inert {
                species, why, spent, ..
            }
                if species.0 == "Ag"
                    && why.contains("above copper")
                    && why.contains("uphill")
                    && spent.is_none()
        )),
        "{events:?}"
    );
}

/// Magnesium is the same shape as the iron, three couples further down,
/// and `lessons/spannungsreihe.lab` is where a reader met it: the
/// magnesium takes every copper ion in the glass and is then told it
/// "does not swap places with anything dissolved here".
#[test]
fn spare_magnesium_over_stripped_copper_sulfate_says_the_copper_is_gone() {
    let (_, events) = run(&[("CuSO4", 0.01), ("Mg", 0.03)]);
    let magnesium = events
        .iter()
        .find_map(|e| match e {
            Event::Inert {
                species,
                why,
                spent,
                ..
            } if species.0 == "Mg" => Some((why, spent)),
            _ => None,
        })
        .unwrap_or_else(|| panic!("a verdict about the magnesium: {events:?}"));
    assert!(
        !magnesium.0.contains("uphill"),
        "magnesium displaced the copper: {}",
        magnesium.0
    );
    assert!(magnesium.1.is_some(), "{}", magnesium.0);
}
