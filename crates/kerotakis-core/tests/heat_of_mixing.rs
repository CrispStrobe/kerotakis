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

/// The design question EXP-44 asked first, answered structurally.
///
/// `total_excess_j` used to look for water and one verified organic. That
/// shape was an accident of the first verified pair, not a claim about
/// chemistry: hᴱ is a property of a binary and nothing privileges water.
/// The cost was a wrong answer wearing an honest refusal's costume — a
/// non-aqueous binary read 0.0 J *structurally*, so acetone–chloroform
/// (the textbook exothermic pair, and one of EXP-44's stated gaps) would
/// have reported "no heat" even with a complete, verified table behind it.
///
/// The allowlist is still one pair long, and widening it is a question
/// about literature and parameters rather than about code. What this pins
/// is that the answer to "why no heat?" is now *this pair is not
/// verified*, and never *neither of these is water*.
#[test]
fn the_allowlist_is_asked_about_pairs_not_about_water() {
    use kerotakis_core::hmix::verified_pair;
    // Unordered: which one was poured first is not a thermodynamic fact.
    assert!(verified_pair("propanone", "water").is_some());
    assert!(verified_pair("water", "propanone").is_some());
    // A non-aqueous pair is refused by the allowlist — the only gate left
    // — rather than by the absence of water.
    assert!(verified_pair("propanone", "trichloromethane").is_none());
    // And water is not a free pass: it is one member of one verified pair,
    // not a qualification on its own.
    assert!(verified_pair("water", "ethanol").is_none());
}

/// A pair is a pair whichever order it was poured in.
///
/// The old code accumulated water in one variable and "the organic" in
/// another, so the two orders travelled different paths to the same
/// answer. They now travel one.
#[test]
fn the_verified_pair_gives_the_same_heat_poured_either_way() {
    let mut water_first = Bench::new();
    add(&mut water_first, "water", 3.0);
    add(&mut water_first, "propanone", 1.0);

    let mut acetone_first = Bench::new();
    add(&mut acetone_first, "propanone", 1.0);
    add(&mut acetone_first, "water", 3.0);

    let a = water_first.vessel(VesselId(0)).unwrap().excess_enthalpy_j;
    let b = acetone_first.vessel(VesselId(0)).unwrap().excess_enthalpy_j;
    assert!(a.abs() > 0.0, "the verified pair computes: {a} J");
    assert!((a - b).abs() < 1e-6, "and by either route: {a} vs {b}");
}

/// A third liquid stands the model down, and that refusal is deliberate:
/// hᴱ over a ternary is not the sum of its binaries, and this model has no
/// ternary claim to make.
#[test]
fn a_third_liquid_stands_the_model_down() {
    let mut bench = Bench::new();
    add(&mut bench, "water", 3.0);
    add(&mut bench, "propanone", 1.0);
    let before = bench.vessel(VesselId(0)).unwrap().excess_enthalpy_j;
    assert!(before.abs() > 0.0, "the binary computes first: {before} J");
    add(&mut bench, "ethanol", 1.0);
    let after = bench.vessel(VesselId(0)).unwrap().excess_enthalpy_j;
    assert_eq!(
        after, 0.0,
        "a ternary is outside the model, so the claim is withdrawn: {after} J"
    );
}
