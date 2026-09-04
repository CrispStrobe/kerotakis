//! CAP-25 slice 1: the bench gains a nose and glass gains a limit.

use kerotakis_core::*;

fn add(bench: &mut Bench, key: &str, moles: f64) -> Vec<Event> {
    bench
        .step(Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new(key),
            moles: Moles(moles),
            at: None,
        })
        .expect("add")
}

#[test]
fn the_waft_reports_curated_odours_and_warns() {
    let mut bench = Bench::new();
    add(&mut bench, "water", 2.0);
    add(&mut bench, "NH3", 0.05);
    let events = bench
        .step(script::parse_op("smell v1").unwrap().unwrap())
        .unwrap();
    let notes = events
        .iter()
        .find_map(|e| match e {
            Event::Smelled { notes, .. } => Some(notes.clone()),
            _ => None,
        })
        .expect("a Smelled event");
    assert!(
        notes
            .iter()
            .any(|(sp, d)| sp.0 == "NH3" && d.contains("pungent")),
        "ammonia announces itself: {notes:?}"
    );
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::HazardWarning { hazard, .. } if hazard.contains("NH3")
        )),
        "hazardous vapours carry the warning: {events:?}"
    );
}

#[test]
fn odourless_is_data_not_absence() {
    let mut bench = Bench::new();
    add(&mut bench, "water", 2.0);
    let events = bench
        .step(script::parse_op("waft v1").unwrap().unwrap())
        .unwrap();
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::Smelled { notes, .. } if notes.is_empty()
        )),
        "plain water: the empty answer is spoken: {events:?}"
    );
}

#[test]
fn sealed_glass_has_a_limit_and_the_ledger_survives_the_bang() {
    let mut bench = Bench::new();
    bench
        .step(Operator::Seal {
            vessel: VesselId(0),
            headspace_volume: Liters(0.001),
        })
        .unwrap();
    // CO2's standard phase is gas: pumping 0.02 mol into a sealed
    // millilitre is ~5e7 Pa — far past what glass holds.
    let events = add(&mut bench, "CO2", 0.02);
    let burst = events
        .iter()
        .find_map(|e| match e {
            Event::Burst {
                at_pa, rating_pa, ..
            } => Some((*at_pa, *rating_pa)),
            _ => None,
        })
        .expect("the vessel bursts");
    assert!(burst.0 > burst.1, "failure above the rating");
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::HazardWarning { severity, .. }
                if matches!(severity, kerotakis_core::Severity::Danger)
        )),
        "the bang is a Danger line: {events:?}"
    );
    let v = bench.vessel(VesselId(0)).unwrap();
    assert!(
        matches!(v.headspace, kerotakis_core::Headspace::Open),
        "the seal is gone"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::GasEvolved { species, .. } if species.0 == "CO2")),
        "the gas vented as events — the ledger is exact through the failure"
    );
}

#[test]
fn an_open_vessel_never_bursts() {
    let mut bench = Bench::new();
    let events = add(&mut bench, "CO2", 0.5);
    assert!(
        !events.iter().any(|e| matches!(e, Event::Burst { .. })),
        "no seal, no burst — gas leaves an open vessel"
    );
}

/// KID-10b: an odour is a question of how much, not only of what.
///
/// `waft` used to match odour rows by Brønsted family, in both directions,
/// so **sodium acetate smelled of vinegar** and ammonium chloride smelled
/// of ammonia. Both are salts of the odorous thing rather than the odorous
/// thing, and both were reported with the same confidence as the real
/// bottle. A peer session found it; the fallback existed to fix a real bug
/// in one direction — vinegar poured into water leaves acetate in the
/// ledger — and the relation it tested has no direction.
///
/// The fallback is gone. It has no job left: the aqueous engine now keeps
/// both members of a Brønsted pair in the ledger, so the odorous molecule
/// is there under its own key when it is genuinely present.
#[test]
fn vinegar_smells_and_its_salt_does_not() {
    use kerotakis_core::script::parse_op;
    use kerotakis_core::Bench;

    let smells = |commands: &[&str]| {
        let mut bench = Bench::new();
        for command in commands {
            let op = parse_op(command)
                .unwrap_or_else(|error| panic!("parse {command}: {error}"))
                .expect("operator");
            bench.step(op).expect("step");
        }
        kerotakis_core::senses::waft(&bench.vessels[0])
            .iter()
            .map(|odor| odor.species)
            .collect::<Vec<_>>()
    };

    // Neat acetic acid is 17 mol/L and unmistakable.
    assert!(smells(&["add v1 CH3COOH 20mL"]).contains(&"CH3COOH"));
    // A trace of it in a lot of water is not. 1e-5 mol in 100 mL is
    // 1e-4 mol/L, an order below the floor.
    assert!(smells(&["add v1 water 100mL", "add v1 CH3COOH 0.00001mol"]).is_empty());
}

/// The floors differ by three orders of magnitude between rows, and that
/// is the fact rather than a fudge: you smell ammonia far below the
/// concentration at which you smell vinegar, and hydrogen peroxide barely
/// at all even neat.
#[test]
fn the_detection_floor_belongs_to_the_substance() {
    use kerotakis_core::senses::{odor_of, ODORS};
    use kerotakis_core::SpeciesId;
    let floor = |key: &str| odor_of(&SpeciesId::new(key)).expect(key).detect_molar;
    assert!(floor("NH3") < floor("CH3COOH"), "ammonia is smelled lower");
    assert!(floor("CH3COOH") < floor("H2O2"), "peroxide barely smells");
    // Every row carries one: a missing floor would silently mean "any
    // trace at all", which is the behaviour this test exists to prevent.
    for odor in ODORS {
        assert!(
            odor.detect_molar > 0.0,
            "{} has no detection floor",
            odor.species
        );
    }
}

/// A gas in the headspace is not gated: it has already reached the nose.
#[test]
fn a_gas_in_the_headspace_is_smelled_however_little_of_it_there_is() {
    use kerotakis_core::species::Phase;
    use kerotakis_core::units::Moles;
    use kerotakis_core::{SpeciesId, Vessel, VesselId};
    let mut vessel = Vessel::new(VesselId(0), "beaker");
    vessel.deposit(SpeciesId::new("Cl2"), Moles(1e-9), Phase::Gas);
    assert_eq!(
        kerotakis_core::senses::waft(&vessel)
            .iter()
            .map(|o| o.species)
            .collect::<Vec<_>>(),
        vec!["Cl2"]
    );
}
