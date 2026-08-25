//! EXP-44 on the bench: mixing heat as a state function, applied only
//! for verified pairs, withheld where the parameters would lie.

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
fn acetone_into_water_warms_the_beaker() {
    let mut bench = Bench::new();
    add(&mut bench, "water", 3.0);
    let t0 = bench.vessel(VesselId(0)).unwrap().temperature.0;
    let events = add(&mut bench, "propanone", 1.0);
    let q = events
        .iter()
        .find_map(|e| match e {
            Event::HeatOfMixing { joules, .. } => Some(*joules),
            _ => None,
        })
        .expect("mixing heat is an event");
    assert!(q > 0.0, "exothermic for the verified pair: {q} J");
    let t1 = bench.vessel(VesselId(0)).unwrap().temperature.0;
    assert!(t1 > t0, "the glass grows warm: {t0} → {t1}");
    assert!(t1 - t0 < 15.0, "and only a little: ΔT = {}", t1 - t0);
}

#[test]
fn the_pour_path_cannot_change_the_answer() {
    // Hᴱ is a state function: one pour or five, same final state.
    let mut one = Bench::new();
    add(&mut one, "water", 3.0);
    add(&mut one, "propanone", 1.0);

    let mut five = Bench::new();
    add(&mut five, "water", 3.0);
    for _ in 0..5 {
        add(&mut five, "propanone", 0.2);
    }
    let t_one = one.vessel(VesselId(0)).unwrap().temperature.0;
    let t_five = five.vessel(VesselId(0)).unwrap().temperature.0;
    assert!(
        (t_one - t_five).abs() < 1e-6,
        "path independence: {t_one} vs {t_five}"
    );
    let h_one = one.vessel(VesselId(0)).unwrap().excess_enthalpy_j;
    let h_five = five.vessel(VesselId(0)).unwrap().excess_enthalpy_j;
    assert!((h_one - h_five).abs() < 1e-6, "stored Hᴱ agrees too");
}

#[test]
fn the_unverified_pair_is_withheld_not_guessed() {
    // Ethanol–water mixing warms a real beaker — but this parameter
    // set inverts the dilute-end sign, so the bench withholds the
    // number rather than teaching a wrong one. The thermo crate pins
    // the deviation; if parameters improve, the allowlist reopens.
    let mut bench = Bench::new();
    add(&mut bench, "water", 3.0);
    let events = add(&mut bench, "ethanol", 1.0);
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::HeatOfMixing { .. })),
        "withheld: {events:?}"
    );
}

#[test]
fn layered_liquids_get_no_mixing_heat() {
    let mut bench = Bench::new();
    add(&mut bench, "water", 2.0);
    let events = add(&mut bench, "hexane", 1.0);
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::HeatOfMixing { .. })),
        "two phases do not mix, so nothing is released: {events:?}"
    );
}
