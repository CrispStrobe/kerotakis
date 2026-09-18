//! Nothing is in contact with a liquid in a vessel that has none.
//!
//! From the same 2026-09-18 transcript, and the third place in it where
//! two parts of one answer contradict each other:
//!
//! ```text
//! v1 NICHT MODELLIERT: das letzte Wasser ist fort …
//! v1 NICHT MODELLIERT: Silbernitrat ist mit einer Flüssigkeit in Kontakt;
//!    kein angebundener Löser modelliert diese Auflösung oder Reaktion
//! ```
//!
//! The predicate was `any(Phase::Liquid | Phase::Aqueous)`. It was not
//! reading a stale field or a pre-evaporation snapshot: it was reading the
//! aqueous compartment, which SURVIVES the solvent. Taking a beaker to
//! dryness removes the water portion and leaves the solutes filed aqueous
//! — `solve::stranded_solutes` is the sentence the bench says about
//! exactly that state — so `any(…)` went on answering *there is a liquid
//! here* off matter dissolved in a solvent that had gone. It disagreed
//! with `solvent_state`, three hundred lines above it in the same file,
//! which has always measured the solvent against `OBSERVABLE_MOLES`.
//!
//! These tests assert the RELATIONSHIP: where no liquid is present,
//! nothing may be described as in contact with one.

use kerotakis_core::render::{render_events_in, Register};
use kerotakis_core::solve::{liquid_medium_present, solvent_state, SolventState};
use kerotakis_core::*;

/// Every sentence the honesty pass's per-solid loop can say. All four are
/// claims about a solid meeting a liquid, and all four are gated by the
/// one reading.
const SAID_OF_A_SOLID_MEETING_A_LIQUID: [&str; 4] = [
    "inert.insoluble-in-water",
    "inert.insoluble-in-water-trace-in-solution",
    "not-modeled.dissolves-unspeciated",
    "not-modeled.no-dissolution-solver",
];

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

fn boil_dry(bench: &mut Bench, solver: &mut SolverStack) -> Vec<Event> {
    bench
        .step_with(
            Operator::Evaporate {
                vessel: VesselId(0),
                fraction: 1.0,
            },
            solver,
            &PermissiveScreen,
        )
        .expect("evaporate")
}

fn held(bench: &Bench) -> &Vessel {
    bench
        .vessels
        .iter()
        .find(|vessel| vessel.id == VesselId(0))
        .expect("the vessel")
}

/// Every `Phrase` key a batch carries a reason under.
fn reason_keys(events: &[Event]) -> Vec<String> {
    events
        .iter()
        .filter_map(|event| match event {
            Event::Inert {
                reason: Some(reason),
                ..
            }
            | Event::NotYetModeled {
                reason: Some(reason),
                ..
            } => Some(reason.key.clone()),
            _ => None,
        })
        .collect()
}

/// The invariant, against the beaker rather than against a string.
fn assert_nothing_claims_a_liquid(bench: &Bench, events: &[Event]) {
    if liquid_medium_present(held(bench)) {
        return;
    }
    let keys = reason_keys(events);
    for said in SAID_OF_A_SOLID_MEETING_A_LIQUID {
        assert!(
            !keys.iter().any(|key| key.as_str() == said),
            "there is no liquid in this vessel and `{said}` was said about \
             a solid in it: {events:#?}"
        );
    }
}

/// The transcript, reduced: a beaker taken to dryness over a solid.
///
/// The second solid is added AFTER the water has gone, so nothing here
/// rests on #667's repetition suppression — that sentence has never been
/// said about silver nitrate in this vessel, and under the old predicate
/// it would have been news.
#[test]
fn a_solid_is_not_in_contact_with_a_liquid_once_the_water_has_gone() {
    let mut bench = Bench::new();
    let mut solver = stack();
    add(&mut bench, &mut solver, "water", 5.5);
    add(&mut bench, &mut solver, "CaCO3", 0.01);
    let dried = boil_dry(&mut bench, &mut solver);
    assert!(
        dried.iter().any(|event| matches!(
            event,
            Event::NotYetModeled { what, .. } if what.contains("the last of the water is gone")
        )),
        "the bench has to have said the water left, or this is a different \
         beaker from the transcript's: {dried:#?}"
    );

    // The state the old predicate was reading, spelled out: an aqueous
    // portion and no liquid under it.
    let vessel = held(&bench);
    assert!(
        vessel
            .contents
            .iter()
            .any(|portion| portion.phase == Phase::Aqueous),
        "the stranded solute is what made `any(Liquid | Aqueous)` true: {:#?}",
        vessel.contents
    );
    assert!(
        !vessel
            .contents
            .iter()
            .any(|portion| portion.phase == Phase::Liquid),
        "and there is no liquid at all: {:#?}",
        vessel.contents
    );
    assert!(!liquid_medium_present(vessel), "{:#?}", vessel.contents);

    let silver = add(&mut bench, &mut solver, "AgNO3", 0.01);
    assert_nothing_claims_a_liquid(&bench, &silver);
    assert_nothing_claims_a_liquid(&bench, &dried);
}

/// …and in the words a German reader meets, since that is the line the
/// owner read.
#[test]
fn the_german_reader_is_not_told_about_a_liquid_that_has_gone() {
    let mut bench = Bench::new();
    let mut solver = stack();
    add(&mut bench, &mut solver, "water", 5.5);
    add(&mut bench, &mut solver, "CaCO3", 0.01);
    boil_dry(&mut bench, &mut solver);
    let silver = add(&mut bench, &mut solver, "AgNO3", 0.01);
    let de = Locale::parse("de");
    let lines = render_events_in(&silver, Register::LV2, de);
    assert!(
        !lines
            .iter()
            .any(|line| line.contains("mit einer Flüssigkeit in Kontakt")),
        "{lines:#?}"
    );
}

/// The over-correction guard. A solid in real water still gets its
/// apology, because the gap it names is real.
#[test]
fn a_solid_in_real_water_still_gets_its_apology() {
    let mut bench = Bench::new();
    let mut solver = stack();
    add(&mut bench, &mut solver, "water", 5.5);
    let silver = add(&mut bench, &mut solver, "AgNO3", 0.01);
    assert!(
        reason_keys(&silver)
            .iter()
            .any(|key| key.as_str() == "not-modeled.no-dissolution-solver"),
        "{silver:#?}"
    );
}

/// The reading itself, and the agreement it was missing.
///
/// `solvent_state` and the honesty pass describe the same beaker, so they
/// have to measure it the same way. They did not: one used
/// `OBSERVABLE_MOLES`, the other mere presence, and a vessel the first
/// called `Absent` was one the second called wet.
#[test]
fn a_liquid_is_a_liquid_and_an_aqueous_portion_is_not_one() {
    let mut stranded = Vessel::new(VesselId(0), "boiled dry over chalk");
    stranded.deposit(SpeciesId::new("CaCO3"), Moles(0.01), Phase::Solid);
    stranded.deposit(SpeciesId::new("CaCO3"), Moles(1.29e-5), Phase::Aqueous);
    assert!(
        !liquid_medium_present(&stranded),
        "a solute filed aqueous is not a solvent"
    );
    assert_eq!(solvent_state(&stranded), SolventState::Absent);

    let mut wet = Vessel::new(VesselId(0), "a beaker of water");
    wet.deposit(SpeciesId::new("water"), Moles(5.5), Phase::Liquid);
    assert!(liquid_medium_present(&wet));

    let mut organic = Vessel::new(VesselId(0), "a beaker of ethanol");
    organic.deposit(SpeciesId::new("ethanol"), Moles(1.0), Phase::Liquid);
    assert!(
        liquid_medium_present(&organic),
        "the reading is about a LIQUID, not about water: a solid standing \
         in ethanol is in contact with one"
    );

    // A milligram of water is the transcript's own residue. Below what a
    // beaker could show, the two readings have to agree that it is gone.
    let mut residue = Vessel::new(VesselId(0), "a residue");
    residue.deposit(SpeciesId::new("water"), Moles(1e-9), Phase::Liquid);
    assert!(!liquid_medium_present(&residue));
    assert_eq!(solvent_state(&residue), SolventState::Absent);
}
