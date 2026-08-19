//! Freezing, melting, and the plateau in between.
//!
//! These pin the two claims that make the states model worth having: the
//! temperature stops at the transition while the phase change is under way,
//! and the transition itself moves with how many particles are dissolved.

use kerotakis_core::*;

fn stack() -> SolverStack {
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(StateEquilibrator),
        Box::new(HonestyEquilibrator),
    ])
}

fn water_bench(moles: f64) -> Bench {
    let mut bench = Bench::new();
    let mut s = stack();
    bench
        .step_with(
            Operator::Add {
                vessel: VesselId(0),
                species: SpeciesId::new("water"),
                moles: Moles(moles),
                at: None,
            },
            &mut s,
            &PermissiveScreen,
        )
        .expect("add");
    bench
}

fn cool(bench: &mut Bench, joules: f64) -> Vec<Event> {
    bench
        .step_with(
            Operator::Cool {
                vessel: VesselId(0),
                energy: Joules(joules),
            },
            &mut stack(),
            &PermissiveScreen,
        )
        .expect("cool")
}

#[test]
fn water_does_not_go_below_zero_while_it_is_still_freezing() {
    // The bug this whole module exists for: the bench used to report
    // liquid water at -71 C because nothing reconsidered the phase.
    let mut bench = water_bench(5.5343);
    let events = cool(&mut bench, 40_000.0);
    let v = bench.vessel(VesselId(0)).unwrap();
    assert!(
        (v.temperature.to_celsius() - 0.0).abs() < 0.05,
        "the plateau holds it at the freezing point, got {} C",
        v.temperature.to_celsius()
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::StateChanged { .. })),
        "{events:?}"
    );
    // Partly frozen: both phases present.
    let ice: f64 = v
        .contents
        .iter()
        .filter(|p| p.phase == Phase::Solid)
        .map(|p| p.moles.0)
        .sum();
    let liquid: f64 = v
        .contents
        .iter()
        .filter(|p| p.phase == Phase::Liquid)
        .map(|p| p.moles.0)
        .sum();
    assert!(ice > 0.0 && liquid > 0.0, "ice {ice}, liquid {liquid}");
    // Energy check: cooling to 0 C then freezing must account for the 40 kJ.
    let sensible = 5.5343 * 75.3 * 25.0;
    let expected_ice = (40_000.0 - sensible) / kerotakis_core::states::WATER_H_FUS;
    assert!(
        (ice - expected_ice).abs() < 0.05,
        "expected {expected_ice:.3} mol of ice, got {ice:.3}"
    );
}

#[test]
fn no_temperature_is_announced_that_the_vessel_never_reached() {
    let mut bench = water_bench(5.5343);
    let events = cool(&mut bench, 40_000.0);
    let actual = bench.vessel(VesselId(0)).unwrap().temperature;
    for e in &events {
        if let Event::TemperatureChanged { to, .. } = e {
            assert!(
                (to.0 - actual.0).abs() < 0.05,
                "announced {} K but the vessel is at {} K",
                to.0,
                actual.0
            );
        }
    }
}

#[test]
fn enough_cooling_freezes_it_solid_and_then_chills_the_ice() {
    let mut bench = water_bench(5.5343);
    cool(&mut bench, 200_000.0);
    let v = bench.vessel(VesselId(0)).unwrap();
    assert!(
        v.temperature.to_celsius() < -5.0,
        "past the plateau it gets colder again: {} C",
        v.temperature.to_celsius()
    );
    assert!(
        v.contents.iter().all(|p| p.phase != Phase::Liquid),
        "nothing liquid left: {:?}",
        v.contents
    );
}

#[test]
fn ice_melts_again_when_warmed() {
    let mut bench = water_bench(5.5343);
    cool(&mut bench, 200_000.0);
    let events = bench
        .step_with(
            Operator::Heat {
                vessel: VesselId(0),
                energy: Joules(200_000.0),
            },
            &mut stack(),
            &PermissiveScreen,
        )
        .expect("heat");
    let v = bench.vessel(VesselId(0)).unwrap();
    assert!(
        v.contents.iter().any(|p| p.phase == Phase::Liquid),
        "it melted back: {:?} / {events:?}",
        v.contents
    );
}

#[test]
fn a_frozen_vessel_has_no_ph() {
    // Ice is not a solution. Continuing to report a pH beside a block of
    // ice was the original complaint.
    let mut bench = water_bench(5.5343);
    cool(&mut bench, 200_000.0);
    assert!(bench.vessel(VesselId(0)).unwrap().solution.is_none());
}
