//! Admission controls isolate current physical inventory from historical lots.
use kerotakis_core::clock::{Clock, ClockContext, GravityClock};
use kerotakis_core::*;

fn suspension(amount: f64) -> Vessel {
    let mut vessel = Vessel::new(VesselId(0), "particle admission control");
    vessel.deposit(SpeciesId::new("water"), Moles(5.55), Phase::Liquid);
    vessel.deposit_lot(
        SpeciesId::new("AgCl"),
        Moles(amount),
        Phase::Solid,
        Some("historical dose".into()),
        Some(1.0),
    );
    vessel
}

fn gravity(vessel: &mut Vessel) -> Vec<Event> {
    let mut events = Vec::new();
    GravityClock
        .advance(
            vessel,
            10.0,
            &ClockContext {
                settle_under_gravity: true,
                ..ClockContext::default()
            },
            &mut events,
        )
        .unwrap();
    events
}

#[test]
fn fully_dissolved_reacted_or_transferred_lots_do_not_settle_or_change() {
    for disposition in 0..3 {
        let mut vessel = suspension(0.007);
        let removed = vessel.withdraw_phase(&SpeciesId::new("AgCl"), Moles(0.007), Phase::Solid);
        match disposition {
            0 => {
                vessel.deposit(SpeciesId::new("Ag+"), Moles(0.007), Phase::Aqueous);
                vessel.deposit(SpeciesId::new("Cl-"), Moles(0.007), Phase::Aqueous);
            }
            1 => {
                // A different solid product cannot authorize the old AgCl lot.
                vessel.deposit(SpeciesId::new("Ag"), Moles(0.007), Phase::Solid);
            }
            _ => {
                let mut destination = suspension(removed.0);
                assert!(
                    !gravity(&mut destination).is_empty(),
                    "transferred solid still settles in its destination"
                );
            }
        }
        let before = format!("{vessel:?}");
        assert!(gravity(&mut vessel).is_empty());
        assert_eq!(
            format!("{vessel:?}"),
            before,
            "stale provenance must remain unchanged"
        );
    }
}

#[test]
fn same_species_aqueous_inventory_cannot_authorize_a_historical_solid_lot() {
    let mut vessel = suspension(0.007);
    vessel.withdraw_phase(&SpeciesId::new("AgCl"), Moles(0.007), Phase::Solid);
    vessel.deposit(SpeciesId::new("AgCl"), Moles(0.007), Phase::Aqueous);
    let before = format!("{vessel:?}");
    assert!(gravity(&mut vessel).is_empty());
    assert_eq!(format!("{vessel:?}"), before);
}

#[test]
fn residual_solid_and_multiple_positive_lots_still_settle_without_inventory_mutation() {
    let mut vessel = suspension(0.007);
    vessel.deposit_lot(
        SpeciesId::new("AgCl"),
        Moles(0.003),
        Phase::Solid,
        Some("second dose".into()),
        Some(2.0),
    );
    vessel.withdraw_phase(&SpeciesId::new("AgCl"), Moles(0.009), Phase::Solid);
    let contents = vessel.contents.clone();
    let lots_before = vessel.lots.clone();
    let events = gravity(&mut vessel);
    let Event::GravitySettled { separations, .. } = &events[0] else {
        panic!("missing settling event")
    };
    assert_eq!(separations.len(), 2);
    assert!(separations
        .iter()
        .all(|s| s.species.0 == "AgCl" && s.separated_fraction > 0.0));
    assert_eq!(vessel.contents, contents);
    for (before, after) in lots_before.iter().zip(&vessel.lots) {
        assert_eq!(after.moles, before.moles);
        assert_eq!(after.source, before.source);
        assert!(after.suspended_fraction.unwrap() < before.suspended_fraction.unwrap());
    }
}

#[test]
fn tiny_positive_solid_is_not_subject_to_a_display_cutoff() {
    let mut vessel = suspension(1e-20);
    let contents = vessel.contents.clone();
    assert!(!gravity(&mut vessel).is_empty());
    assert_eq!(vessel.contents, contents);
    assert!(vessel.lots[0].suspended_fraction.unwrap() < 1.0);
}

#[test]
fn zero_negative_or_nonfinite_solid_portions_do_not_authorize_settling() {
    for amount in [0.0, -0.001, f64::NAN, f64::INFINITY] {
        let mut vessel = suspension(0.007);
        vessel
            .contents
            .iter_mut()
            .find(|p| p.species.0 == "AgCl")
            .unwrap()
            .moles = Moles(amount);
        let before = format!("{vessel:?}");
        assert!(gravity(&mut vessel).is_empty());
        assert_eq!(format!("{vessel:?}"), before);
    }
}

#[test]
fn invalid_or_empty_lots_cannot_borrow_another_lots_physical_solid() {
    let mut vessel = suspension(0.007);
    let live_lot = vessel.lots[0].clone();
    for amount in [0.0, -0.001, f64::NAN, f64::INFINITY] {
        let mut stale = live_lot.clone();
        stale.moles = Moles(amount);
        vessel.lots.push(stale);
    }
    let events = gravity(&mut vessel);
    let Event::GravitySettled { separations, .. } = &events[0] else {
        panic!("missing settling event")
    };
    assert_eq!(separations.len(), 1);
    assert!(vessel.lots[0].suspended_fraction.unwrap() < 1.0);
    assert!(vessel.lots[1..]
        .iter()
        .all(|lot| lot.suspended_fraction == Some(1.0)));
}

#[test]
fn centrifuge_rejects_stale_lots_but_keeps_real_particulates() {
    let mut bench = Bench::new();
    bench.vessels[0] = suspension(0.007);
    bench.vessels[0].withdraw_phase(&SpeciesId::new("AgCl"), Moles(0.007), Phase::Solid);
    let stale = format!("{:?}", bench.vessels[0]);
    let command = || {
        kerotakis_core::script::parse_op("centrifuge v1 3000rpm 10s 8cm")
            .unwrap()
            .unwrap()
    };
    assert!(matches!(
        bench.step(command()),
        Err(BenchError::CentrifugeUnavailable(_))
    ));
    assert_eq!(format!("{:?}", bench.vessels[0]), stale);
    bench.vessels[0].deposit(SpeciesId::new("AgCl"), Moles(0.001), Phase::Solid);
    let events = bench.step(command()).unwrap();
    assert!(events
        .iter()
        .any(|e| matches!(e, Event::Centrifuged { separations, .. } if separations.len() == 1)));
}
