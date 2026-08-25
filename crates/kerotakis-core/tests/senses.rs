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
