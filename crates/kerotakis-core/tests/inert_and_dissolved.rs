//! A solid may not be told it dissolved and, on the next line, that it is
//! untouched.
//!
//! From a live transcript (2026-09-18, the peroxide/chalk vessel), in the
//! order a reader met them:
//!
//! ```text
//! v1 0,000001 mol Kreide (Calciumcarbonat) gelöst
//! v1 Kreide (Calciumcarbonat) inert: … Es ist noch vollständig vorhanden
//! ```
//!
//! Two subsystems describing one lump of chalk and disagreeing about it.
//! [`MixingEquilibrator`] moves a micromole into the aqueous compartment on
//! the registry's own reviewed limit, and the honesty pass then read that
//! same limit out of the same table and appended *it is still all there* —
//! a claim about the remaining SOLID, asserted from a solubility rather
//! than derived from what was left.
//!
//! Both facts are true and both are worth saying. What was false was the
//! last clause, so the last clause is now the vessel's to decide.
//!
//! **These tests pin the relationship, not the sentence.** A test that
//! pinned the new wording would pass again the day the wording and the
//! state came apart, which is the defect itself.

use kerotakis_core::phrase::Phrase;
use kerotakis_core::render::{render_events_in, Register};
use kerotakis_core::*;

/// The sentence that claims the solid is undiminished.
const WHOLLY_INTACT: &str = "inert.insoluble-in-water";
/// …and the one that admits a trace of it has gone.
const TRACE_IN_SOLUTION: &str = "inert.insoluble-in-water-trace-in-solution";

fn stack() -> SolverStack {
    SolverStack::new(vec![
        Box::new(MixingEquilibrator) as Box<dyn Equilibrator>,
        Box::new(HonestyEquilibrator),
    ])
}

fn add(bench: &mut Bench, solver: &mut SolverStack, key: &str, moles: f64) -> Vec<Event> {
    bench
        .step_with(
            Operator::Add {
                vessel: VesselId(0),
                species: SpeciesId::new(key),
                moles: Moles(moles),
                at: None,
            },
            solver,
            &PermissiveScreen,
        )
        .expect("add")
}

fn inert_reasons(events: &[Event]) -> Vec<(&SpeciesId, &Phrase)> {
    events
        .iter()
        .filter_map(|event| match event {
            Event::Inert {
                species,
                reason: Some(reason),
                ..
            } => Some((species, reason)),
            _ => None,
        })
        .collect()
}

/// How much of `species` the vessel holds anywhere but in the solid.
fn outside_the_solid(bench: &Bench, species: &SpeciesId) -> f64 {
    bench
        .vessels
        .iter()
        .find(|vessel| vessel.id == VesselId(0))
        .expect("the vessel")
        .contents
        .iter()
        .filter(|portion| &portion.species == species && portion.phase != Phase::Solid)
        .map(|portion| portion.moles.0)
        .sum()
}

/// The invariant, stated against the beaker rather than against a string:
/// the wholly-intact verdict is entitled to stand only where none of that
/// solid is standing in any other phase.
fn assert_verdicts_match_the_vessel(bench: &Bench, events: &[Event]) {
    for (species, reason) in inert_reasons(events) {
        let elsewhere = outside_the_solid(bench, species);
        match reason.key.as_str() {
            WHOLLY_INTACT => assert_eq!(
                elsewhere, 0.0,
                "{} is described as wholly intact while {elsewhere} mol of it \
                 stands outside the solid",
                species.0
            ),
            TRACE_IN_SOLUTION => assert!(
                elsewhere > 0.0,
                "{} is described as partly gone with nothing of it outside \
                 the solid",
                species.0
            ),
            _ => {}
        }
    }
}

/// The transcript, reduced: one batch that dissolves chalk and describes
/// the chalk.
#[test]
fn chalk_that_has_just_dissolved_is_not_also_called_untouched() {
    let mut bench = Bench::new();
    let mut solver = stack();
    add(&mut bench, &mut solver, "water", 5.5);
    let batch = add(&mut bench, &mut solver, "CaCO3", 0.01);

    let chalk = SpeciesId::new("CaCO3");
    let dissolved: f64 = batch
        .iter()
        .filter_map(|event| match event {
            Event::Dissolved { species, moles, .. } if *species == chalk => Some(moles.0),
            _ => None,
        })
        .sum();
    assert!(
        dissolved > 0.0,
        "the reproduction needs the dissolution that starts the contradiction: {batch:#?}"
    );

    let verdicts = inert_reasons(&batch);
    assert_eq!(verdicts.len(), 1, "one verdict about the chalk: {batch:#?}");
    assert_ne!(
        verdicts[0].1.key, WHOLLY_INTACT,
        "{dissolved} mol went into solution in this very batch, so nothing here \
         may say the lump is undiminished: {:#?}",
        verdicts[0].1
    );
    assert_verdicts_match_the_vessel(&bench, &batch);
}

/// …and the words a reader actually meets differ, in both languages. The
/// point of the repair is that the two states read differently; a repair
/// that renamed the key and left the sentence identical would be no repair
/// at all.
#[test]
fn the_two_states_do_not_read_the_same() {
    fn rendered(locale: Locale, key: &str) -> String {
        let phrase = match key {
            WHOLLY_INTACT => Phrase::new(
                WHOLLY_INTACT,
                "{name} does not dissolve in water: its reviewed solubility is {limit} g per 100 mL, which is below anything a beaker would show. It is still all there",
                vec![
                    ("name".to_string(), Slot::term("species", "chalk")),
                    ("limit".to_string(), Slot::number("0.0013")),
                ],
            ),
            _ => Phrase::new(
                TRACE_IN_SOLUTION,
                "{name} hardly dissolves in water: its reviewed solubility is {limit} g per 100 mL, which is below anything a beaker would show. A trace of it is in solution; the rest is still there",
                vec![
                    ("name".to_string(), Slot::term("species", "chalk")),
                    ("limit".to_string(), Slot::number("0.0013")),
                ],
            ),
        };
        phrase.render(locale)
    }
    for locale in [Locale::EN, Locale::parse("de")] {
        assert_ne!(
            rendered(locale, WHOLLY_INTACT),
            rendered(locale, TRACE_IN_SOLUTION),
            "the two verdicts have to be two sentences, not two keys"
        );
    }
    // And the German is a translation rather than the English falling
    // through, which is what an untranslated new key would look like.
    assert_ne!(
        rendered(Locale::parse("de"), TRACE_IN_SOLUTION),
        rendered(Locale::EN, TRACE_IN_SOLUTION),
        "the new verdict needs its own catalogue row"
    );
}

/// Both branches, and the #667 interaction, asked of the honesty pass
/// directly.
///
/// The pass is run over a hand-built vessel rather than through the bench
/// because the first branch describes a beaker that exists for one
/// instant: chalk has landed in water and nothing has dissolved yet. Put
/// the same beaker through [`MixingEquilibrator`] and the reviewed limit
/// has already taken its micromole, which is the whole of this defect.
///
/// `Vessel::honesty_said` (#667) holds what STANDS and compares
/// `Phrase::shape()`. Two keys are two shapes, so the moment a trace goes
/// into solution is a change in the standing verdict and is announced, and
/// every step after it is an echo and is not. That is the deliberate
/// interaction: the repair alters the shape of exactly the sentence #667
/// de-duplicates.
#[test]
fn the_verdict_follows_the_vessel_and_is_still_said_once() {
    let mut honesty = HonestyEquilibrator;
    let mut vessel = Vessel::new(VesselId(0), "chalk, just landed");
    vessel.deposit(SpeciesId::new("water"), Moles(5.5), Phase::Liquid);
    vessel.deposit(SpeciesId::new("CaCO3"), Moles(0.01), Phase::Solid);

    let first = honesty.equilibrate(&mut vessel).expect("honesty");
    let verdicts = inert_reasons(&first);
    assert_eq!(verdicts.len(), 1, "one verdict about the chalk: {first:#?}");
    assert_eq!(
        verdicts[0].1.key, WHOLLY_INTACT,
        "nothing of it stands outside the solid, so the clause is true — and \
         now checked rather than asserted: {first:#?}"
    );
    assert!(
        inert_reasons(&honesty.equilibrate(&mut vessel).expect("honesty")).is_empty(),
        "and it is said once, not once per pass"
    );

    // A trace goes into solution — the micromole the reviewed limit takes.
    vessel.deposit(SpeciesId::new("CaCO3"), Moles(1.29e-5), Phase::Aqueous);
    let changed = honesty.equilibrate(&mut vessel).expect("honesty");
    let verdicts = inert_reasons(&changed);
    assert_eq!(
        verdicts.len(),
        1,
        "the standing verdict stopped being true, which is news: {changed:#?}"
    );
    assert_eq!(verdicts[0].1.key, TRACE_IN_SOLUTION, "{changed:#?}");
    assert!(
        inert_reasons(&honesty.equilibrate(&mut vessel).expect("honesty")).is_empty(),
        "and the new verdict stands in its turn"
    );
}

/// The whole batch a reader sees, in German, read as lines rather than as
/// events: no line may claim the chalk is wholly there while another says
/// it dissolved.
#[test]
fn the_german_reader_is_not_told_both() {
    let mut bench = Bench::new();
    let mut solver = stack();
    add(&mut bench, &mut solver, "water", 5.5);
    let batch = add(&mut bench, &mut solver, "CaCO3", 0.01);
    let de = Locale::parse("de");
    let lines = render_events_in(&batch, Register::LV2, de);
    let dissolved = lines.iter().any(|line| line.contains("gelöst"));
    assert!(
        dissolved,
        "the dissolution line is what makes this a pair: {lines:#?}"
    );
    // "vollständig vorhanden" is the German of the clause that was false.
    // Pinned here on purpose and only here: this one test is about the
    // sentence a reader met in the transcript.
    assert!(
        !lines
            .iter()
            .any(|line| line.contains("vollständig vorhanden")),
        "{lines:#?}"
    );
    assert_verdicts_match_the_vessel(&bench, &batch);
}
