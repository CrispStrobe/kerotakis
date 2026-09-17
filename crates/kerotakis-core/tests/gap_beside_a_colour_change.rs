//! GUI-106: the note that said nothing happened, one line above what did.
//!
//! Running `lessons/starch-iodine-test.lab` produced, in order:
//!
//! ```text
//! You add cornstarch to v2.
//! Hmm — nothing visible happens in v2 (this part of the lab isn't awake yet).
//! You look closely at v2. The liquid is blue-black and so cloudy you cannot see through it.
//! ```
//!
//! The gap the middle line carries is real — no wired solver speciates
//! starch — but the sentence carrying it makes a second claim that is not
//! the event's to make, and in the lesson whose entire point is the
//! colour change it is the opposite of true.
//!
//! The rule under test: the bench reads the liquid's colour word on both
//! sides of the step, and where it MOVED the lv1 line stops claiming the
//! vessel did nothing. The note itself is never dropped — at lv2 and lv3
//! it is word for word what it was, at lv1 it is still said — so this
//! cannot hide a gap, which is the half the second test exists to pin.
//!
//! The solver here is a fixture that raises a gap note and touches
//! nothing, and that is the right fixture rather than a shortcut: the
//! thing under test is what the BENCH does with a note beside a colour,
//! not which solver happened to raise it.

use kerotakis_core::*;

/// A solver that looks, changes nothing, and says it cannot model this.
struct AlwaysAGap;

impl Equilibrator for AlwaysAGap {
    fn name(&self) -> &'static str {
        "always-a-gap"
    }

    fn chemistry_applies(&self, _vessel: &Vessel) -> bool {
        false
    }

    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        Ok(vec![Event::not_modeled(
            vessel.id,
            kerotakis_core::ops::NotModelledCause::NoSolver,
            kerotakis_core::phrase::Phrase::bare(
                "not-modeled.test-only",
                "a fixture gap, raised on every step",
            ),
        )])
    }
}

fn stack() -> SolverStack {
    SolverStack::new(vec![Box::new(MixingEquilibrator), Box::new(AlwaysAGap)])
}

/// `add` one species to `v0`, over a vessel prepared by `prepare`.
fn add_to(prepare: impl FnOnce(&mut Vessel), key: &str, moles: f64) -> Vec<Event> {
    let mut bench = Bench::new();
    prepare(&mut bench.vessels[0]);
    let mut stack = stack();
    bench
        .step_with(
            Operator::Add {
                vessel: VesselId(0),
                species: SpeciesId::new(key),
                moles: Moles(moles),
                at: None,
            },
            &mut stack,
            &PermissiveScreen,
        )
        .expect("add")
}

fn lv1(events: &[Event]) -> Vec<String> {
    render_events(events, Register::LV1)
}

fn lv2(events: &[Event]) -> Vec<String> {
    render_events(events, Register::LV2)
}

/// The lesson's own step: cornstarch into dilute Lugol solution.
#[test]
fn a_gap_beside_a_colour_change_stops_saying_nothing_happened() {
    let events = add_to(
        |v| {
            v.deposit(SpeciesId::new("water"), Moles(5.55), Phase::Liquid);
            v.deposit(SpeciesId::new("KI"), Moles(0.000_245), Phase::Aqueous);
            v.deposit(SpeciesId::new("I2"), Moles(0.000_080), Phase::Aqueous);
        },
        "starch",
        0.006_1,
    );

    // The flag is set, and set for the right reason: the liquid moved.
    assert!(
        events.iter().any(|event| matches!(
            event,
            Event::NotYetModeled {
                beside_a_visible_change: true,
                ..
            }
        )),
        "the bench saw the colour move: {events:#?}"
    );

    let lines = lv1(&events);
    assert!(
        !lines.iter().any(|line| line.contains("nothing visible")),
        "the vessel went blue-black; nothing may say it did nothing:\n{lines:#?}"
    );
    assert!(
        lines
            .iter()
            .any(|line| line.contains("Something did change")),
        "the gap is still said, in a sentence that is true:\n{lines:#?}"
    );

    // And the gap itself is untouched where it can be stated precisely.
    assert!(
        lv2(&events)
            .iter()
            .any(|line| line.contains("a fixture gap")),
        "lv2 still carries the whole reason: {:#?}",
        lv2(&events)
    );
}

/// The half that matters more: a gap that really is a gap still says so.
///
/// Iron in water is K17, the experiment the lv1 sentence was written for.
/// Dropping the lump in changes what is IN the beaker and changes nothing
/// about how the water looks, so the colour word does not move and the
/// sentence stands exactly as it did.
#[test]
fn a_gap_with_nothing_to_see_still_says_nothing_happened() {
    let events = add_to(
        |v| v.deposit(SpeciesId::new("water"), Moles(5.55), Phase::Liquid),
        "Fe",
        0.01,
    );
    let lines = lv1(&events);
    assert!(
        lines
            .iter()
            .any(|line| line.contains("nothing visible happens")),
        "an unmodelled gap over an unchanged liquid keeps its sentence:\n{lines:#?}"
    );
    assert!(
        !events.iter().any(|event| matches!(
            event,
            Event::NotYetModeled {
                beside_a_visible_change: true,
                ..
            }
        )),
        "nothing about this vessel looks different: {events:#?}"
    );
}

/// A colour change in ONE vessel does not excuse a note about another.
///
/// The flag is per vessel, and the cheapest way to get this wrong would
/// be to set it for the step.
#[test]
fn the_flag_belongs_to_the_vessel_and_not_to_the_step() {
    let mut bench = Bench::new();
    bench.vessels[0].deposit(SpeciesId::new("water"), Moles(5.55), Phase::Liquid);
    bench.vessels[0].deposit(SpeciesId::new("KI"), Moles(0.000_245), Phase::Aqueous);
    bench.vessels[0].deposit(SpeciesId::new("I2"), Moles(0.000_080), Phase::Aqueous);
    let mut stack = stack();
    bench
        .step_with(
            Operator::NewVessel { kind: None },
            &mut stack,
            &PermissiveScreen,
        )
        .expect("new vessel");
    let second = bench.vessels[1].id;
    bench
        .step_with(
            Operator::Add {
                vessel: second,
                species: SpeciesId::new("water"),
                moles: Moles(5.55),
                at: None,
            },
            &mut stack,
            &PermissiveScreen,
        )
        .expect("water");

    let events = bench
        .step_with(
            Operator::Add {
                vessel: VesselId(0),
                species: SpeciesId::new("starch"),
                moles: Moles(0.006_1),
                at: None,
            },
            &mut stack,
            &PermissiveScreen,
        )
        .expect("starch");

    for event in &events {
        if let Event::NotYetModeled {
            vessel,
            beside_a_visible_change,
            ..
        } = event
        {
            if *vessel != VesselId(0) {
                assert!(
                    !beside_a_visible_change,
                    "v{} did not change colour: {events:#?}",
                    vessel.0
                );
            }
        }
    }
}
