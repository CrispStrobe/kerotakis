//! PLAN P3s, the last two items: every liquid the registry knows has a
//! plateau of its own, and a vessel whose solvent is not a settled liquid
//! says so instead of quietly answering anyway.
//!
//! What was wrong before is easy to state and was easy to miss. The bench
//! had a complete phase model — plateaus, latent heat, colligative shifts,
//! a superheat correction — and it was wired to exactly three substances:
//! water, dry ice and liquid nitrogen. Ethanol over a burner did not boil.
//! It got hotter, and hotter, and stayed liquid at any temperature you
//! liked, because `VAPORISATION_ENTHALPIES` had one row in it and the
//! comment beside that row said keeping it to one was the point.
//!
//! These tests are the arithmetic that says it is no longer one row.

use kerotakis_core::phase_route::PhaseRouteEquilibrator;
use kerotakis_core::solve::{solvent_state, SolventState};
use kerotakis_core::species::Phase;
use kerotakis_core::*;

fn stack() -> SolverStack {
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(StateEquilibrator),
        Box::new(PhaseRouteEquilibrator),
        Box::new(HonestyEquilibrator),
    ])
}

fn v(bench: &Bench) -> &vessel::Vessel {
    bench.vessel(VesselId(0)).unwrap()
}

fn moles(bench: &Bench, key: &str, phase: Phase) -> f64 {
    v(bench)
        .contents
        .iter()
        .filter(|p| p.species.0 == key && p.phase == phase)
        .map(|p| p.moles.0)
        .sum()
}

fn add(bench: &mut Bench, stack: &mut SolverStack, key: &str, n: f64) -> Vec<Event> {
    bench
        .step_with(
            Operator::Add {
                vessel: VesselId(0),
                species: SpeciesId::new(key),
                moles: Moles(n),
                at: None,
            },
            stack,
            &PermissiveScreen,
        )
        .expect("add")
}

fn heat(bench: &mut Bench, stack: &mut SolverStack, joules: f64) -> Vec<Event> {
    bench
        .step_with(
            Operator::Heat {
                vessel: VesselId(0),
                energy: Joules(joules),
                source: None,
            },
            stack,
            &PermissiveScreen,
        )
        .expect("heat")
}

fn cool(bench: &mut Bench, stack: &mut SolverStack, joules: f64) -> Vec<Event> {
    bench
        .step_with(
            Operator::Cool {
                vessel: VesselId(0),
                energy: Joules(joules),
            },
            stack,
            &PermissiveScreen,
        )
        .expect("cool")
}

/// The one the PLAN item names: a spirit burner's fuel boils at 78.4 °C
/// and the thermometer holds there while it does.
///
/// 1 mol of ethanol is 46 g, about 58 mL. Warming it from 25 °C to its
/// boiling point costs 112.3 × 53.24 ≈ 6.0 kJ; boiling all of it would cost
/// a further 38.6 kJ. 20 kJ therefore lands squarely on the plateau with
/// most of the ethanol still in the flask, which is the observation.
#[test]
fn pure_ethanol_boils_at_its_own_point_and_the_thermometer_holds_there() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "ethanol", 1.0);
    let events = heat(&mut bench, &mut s, 20_000.0);

    let t_c = v(&bench).temperature.to_celsius();
    let boiling_c = species::lookup(&SpeciesId::new("ethanol"))
        .and_then(|d| d.transitions)
        .and_then(|t| t.boiling_k)
        .expect("the registry carries ethanol's boiling point")
        - 273.15;
    assert!(
        (boiling_c - 78.24).abs() < 0.2,
        "the registry's ethanol boiling point moved: {boiling_c} C"
    );
    assert!(
        (t_c - boiling_c).abs() < 0.5,
        "the burner should be held at ethanol's plateau; thermometer reads {t_c} C \
         against a boiling point of {boiling_c} C. Events: {events:?}"
    );

    let left = moles(&bench, "ethanol", Phase::Liquid);
    assert!(
        left > 0.6 && left < 0.8,
        "20 kJ pays about 6.0 kJ of sensible heat and then buys ~0.36 mol of \
         vapour at 38.56 kJ/mol; {left} mol of liquid ethanol left"
    );
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::StateChanged { from: Phase::Liquid, to: Phase::Gas, species, .. }
                if species.0 == "ethanol"
        )),
        "the boil has to be announced, not merely accounted for: {events:?}"
    );
}

/// Superheat is not a state a liquid can be in, and the correction that
/// already caught it for cryogens now catches it for anything with a
/// vaporisation row: dumping far more heat than the boil costs empties the
/// flask rather than reporting ethanol at 400 °C.
#[test]
fn no_ordinary_liquid_is_left_standing_above_its_boiling_point() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "propanone", 0.5);
    heat(&mut bench, &mut s, 60_000.0);
    assert!(
        moles(&bench, "propanone", Phase::Liquid) < 1e-6,
        "0.5 mol of propanone costs about 3 kJ of sensible heat and 14.6 kJ to \
         boil; 60 kJ must empty the flask, and {} mol is still liquid at {} C",
        moles(&bench, "propanone", Phase::Liquid),
        v(&bench).temperature.to_celsius()
    );
}

/// The cooling-curve experiment school actually runs. Naphthalene melts at
/// 80.2 °C, and the plateau on the way back down is the whole point of the
/// graph a learner draws.
#[test]
fn naphthalene_melts_and_freezes_on_its_own_plateau() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "naphthalene", 0.2);
    // 0.2 mol from 25 °C to 80.2 °C is about 0.2 × 165 × 55.2 = 1.8 kJ, and
    // melting it all costs 0.2 × 19.01 = 3.8 kJ. 3 kJ lands on the plateau.
    heat(&mut bench, &mut s, 3_000.0);
    let t_c = v(&bench).temperature.to_celsius();
    assert!(
        (t_c - 80.25).abs() < 1.0,
        "the melt should hold it at 80.2 C, thermometer reads {t_c} C; \
         solid {} mol, liquid {} mol",
        moles(&bench, "naphthalene", Phase::Solid),
        moles(&bench, "naphthalene", Phase::Liquid)
    );
    assert!(
        moles(&bench, "naphthalene", Phase::Liquid) > 1e-6
            && moles(&bench, "naphthalene", Phase::Solid) > 1e-6,
        "a plateau means BOTH phases are in the tube at once: solid {}, liquid {}",
        moles(&bench, "naphthalene", Phase::Solid),
        moles(&bench, "naphthalene", Phase::Liquid)
    );

    // And back down again: the same latent heat has to be given up before
    // the thermometer will move.
    cool(&mut bench, &mut s, 3_000.0);
    assert!(
        moles(&bench, "naphthalene", Phase::Solid) > 0.19,
        "cooled through the same plateau it is solid again: {} mol solid",
        moles(&bench, "naphthalene", Phase::Solid)
    );
}

/// A metal melts at its own point and pays for it. Lead at 327.5 °C is the
/// one a soldering iron and a fishing-weight mould both live on.
#[test]
fn lead_melts_at_its_own_point_and_the_latent_heat_is_charged() {
    let mut bench = Bench::new();
    let mut s = stack();
    // 1 mol of lead is 207 g. 25 °C to 327.5 °C costs 26.65 × 302.5 ≈ 8.1 kJ;
    // melting it costs 4.77 kJ more.
    add(&mut bench, &mut s, "Pb", 1.0);
    heat(&mut bench, &mut s, 10_000.0);
    let t_c = v(&bench).temperature.to_celsius();
    assert!(
        (t_c - 327.46).abs() < 1.5,
        "lead should be held on its melting plateau, thermometer reads {t_c} C"
    );
    let liquid = moles(&bench, "Pb", Phase::Liquid);
    assert!(
        liquid > 0.2 && liquid < 0.6,
        "about 1.9 kJ is left over the plateau, which buys ~0.4 mol at \
         4.77 kJ/mol; got {liquid} mol molten"
    );

    // Iron is the control: its melting point is above what a Bunsen can
    // reach, so the same generosity must not melt it.
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "Fe", 1.0);
    heat(&mut bench, &mut s, 10_000.0);
    assert!(
        moles(&bench, "Fe", Phase::Liquid) < 1e-9,
        "10 kJ is nowhere near iron's 1538 C melting point"
    );
}

/// Glacial acetic acid is called glacial because it freezes at 16.6 °C, and
/// a cold laboratory really does solidify the bottle.
#[test]
fn acetic_acid_freezes_where_the_word_glacial_comes_from() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "CH3COOH", 1.0);
    // 25 °C down to 16.6 °C costs 123.1 × 8.4 ≈ 1.0 kJ; freezing it all
    // gives up 11.73 kJ.
    cool(&mut bench, &mut s, 6_000.0);
    let t_c = v(&bench).temperature.to_celsius();
    assert!(
        (t_c - 16.6).abs() < 1.0,
        "the freeze holds it at 16.6 C, thermometer reads {t_c} C"
    );
    let solid = moles(&bench, "CH3COOH", Phase::Solid);
    assert!(
        solid > 0.3 && solid < 0.6,
        "about 5 kJ past the plateau freezes ~0.43 mol at 11.73 kJ/mol; got {solid} mol"
    );
}

// ── the honest boundary ────────────────────────────────────────────

/// A block of ice is not a solution, and the bench says which of those two
/// things it is holding.
#[test]
fn a_frozen_vessel_says_the_water_is_ice() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "water", 5.55);
    add(&mut bench, &mut s, "Na+", 0.05);
    add(&mut bench, &mut s, "Cl-", 0.05);
    let events = cool(&mut bench, &mut s, 200_000.0);

    assert_eq!(
        solvent_state(v(&bench)),
        SolventState::Frozen,
        "200 kJ out of 100 mL freezes it solid; contents {:?} at {} C",
        v(&bench).contents,
        v(&bench).temperature.to_celsius()
    );
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::NotYetModeled { what, .. } if what.contains("ice is not a solution")
        )),
        "the withdrawal has to be spoken, not merely done: {events:?}"
    );
    assert!(
        v(&bench).solution.is_none(),
        "and the pH meter has nothing to read"
    );
}

/// Pure water does not get apologised to. The kettle boils, the transition
/// is announced, and nothing is withheld because nothing was dissolved.
#[test]
fn a_kettle_of_pure_water_is_not_nagged_about_its_missing_ph() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "water", 5.55);
    let events = heat(&mut bench, &mut s, 40_000.0);
    assert_eq!(
        solvent_state(v(&bench)),
        SolventState::Pure,
        "boiling, but with nothing dissolved to withhold"
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::NotYetModeled { .. })),
        "an apology for the pH of nothing is noise dressed as honesty: {events:?}"
    );
}

/// The state the honesty pass exists for, in one line: a vessel that
/// reaches dryness by BOILING gets the same sentence as one that reaches it
/// by evaporating, because it is the same impossible state.
#[test]
fn boiling_a_beaker_dry_strands_its_ions_out_loud() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "water", 0.5);
    add(&mut bench, &mut s, "Na+", 0.01);
    add(&mut bench, &mut s, "Cl-", 0.01);
    // 0.5 mol of water needs ~2.8 kJ to reach 100 C and 20.3 kJ to boil.
    let events = heat(&mut bench, &mut s, 60_000.0);
    assert!(
        moles(&bench, "water", Phase::Liquid) < 1e-9,
        "60 kJ boils half a mole of water away entirely; {} mol left",
        moles(&bench, "water", Phase::Liquid)
    );
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::NotYetModeled { what, .. }
                if what.contains("not a state a beaker can be in")
        )),
        "ions with no solvent must be named: {events:?}"
    );
}
