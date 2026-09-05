use kerotakis_core::enzyme::EnzymeFamily;
use kerotakis_core::script::parse_op;
use kerotakis_core::units::Kelvin;
use kerotakis_core::{Bench, Event};

fn run(lines: &[&str]) -> (Bench, Vec<Event>) {
    let mut bench = Bench::new();
    let mut events = Vec::new();
    for line in lines {
        let op = parse_op(line).expect("parse").expect("operator");
        events.extend(bench.step(op).expect("step"));
    }
    (bench, events)
}

fn hydrolysed(events: &[Event], family: EnzymeFamily) -> f64 {
    events
        .iter()
        .filter_map(|event| match event {
            Event::EnzymeHydrolysed {
                family: seen,
                hydrolysed_mass_g,
                ..
            } if *seen == family => Some(*hydrolysed_mass_g),
            _ => None,
        })
        .sum()
}

#[test]
fn each_family_acts_only_on_its_reviewed_food_substrate() {
    let cases = [
        ("whole_milk", "lactase", EnzymeFamily::Lactase),
        ("gelatin", "protease", EnzymeFamily::Protease),
        ("vegetable_oil", "lipase", EnzymeFamily::Lipase),
    ];
    for (material, enzyme, family) in cases {
        let (_, active) = run(&[
            "add v1 water 50mL",
            &format!("add v1 {material} 10g"),
            &format!("add v1 {enzyme} 0.1g"),
            "wait 1h",
        ]);
        assert!(hydrolysed(&active, family) > 0.0, "{material}: {active:?}");

        let (_, control) = run(&[
            "add v1 water 50mL",
            &format!("add v1 {material} 10g"),
            "wait 1h",
        ]);
        assert_eq!(hydrolysed(&control, family), 0.0, "{material}: {control:?}");
    }
}

#[test]
fn wrong_enzyme_is_a_silent_negative_control() {
    let (_, events) = run(&[
        "add v1 water 50mL",
        "add v1 whole_milk 10g",
        "add v1 lipase 0.1g",
        "wait 1h",
    ]);
    assert!(!events
        .iter()
        .any(|event| matches!(event, Event::EnzymeHydrolysed { .. })));
}

#[test]
fn progress_is_bounded_dose_ordered_and_mass_conserving() {
    let (low, low_events) = run(&[
        "add v1 water 50mL",
        "add v1 gelatin 10g",
        "add v1 protease 0.01g",
        "wait 1h",
    ]);
    let (high, high_events) = run(&[
        "add v1 water 50mL",
        "add v1 gelatin 10g",
        "add v1 protease 0.1g",
        "wait 1h",
    ]);
    let (low_before, _) = run(&[
        "add v1 water 50mL",
        "add v1 gelatin 10g",
        "add v1 protease 0.01g",
    ]);
    let (high_before, _) = run(&[
        "add v1 water 50mL",
        "add v1 gelatin 10g",
        "add v1 protease 0.1g",
    ]);
    assert!(
        hydrolysed(&high_events, EnzymeFamily::Protease)
            > hydrolysed(&low_events, EnzymeFamily::Protease)
    );
    assert!((low.vessels[0].mass().0 - low_before.vessels[0].mass().0).abs() < 1e-9);
    assert!((high.vessels[0].mass().0 - high_before.vessels[0].mass().0).abs() < 1e-9);
    let state = high.vessels[0]
        .unresolved_materials
        .iter()
        .find_map(|portion| portion.enzyme_hydrolysis.as_ref())
        .expect("progress state");
    assert!(state.converted_fraction > 0.0 && state.converted_fraction <= 1.0);
}

#[test]
fn split_wait_matches_one_interval() {
    let (_, one) = run(&[
        "add v1 water 50mL",
        "add v1 whole_milk 10g",
        "add v1 lactase 0.1g",
        "wait 1h",
    ]);
    let (_, two) = run(&[
        "add v1 water 50mL",
        "add v1 whole_milk 10g",
        "add v1 lactase 0.1g",
        "wait 30min",
        "wait 30min",
    ]);
    assert!(
        (hydrolysed(&one, EnzymeFamily::Lactase) - hydrolysed(&two, EnzymeFamily::Lactase)).abs()
            < 1e-9
    );
}

#[test]
fn activity_is_reduced_away_from_the_temperature_envelope() {
    let activity_at = |kelvin| {
        let (mut bench, _) = run(&[
            "add v1 water 50mL",
            "add v1 whole_milk 10g",
            "add v1 lactase 0.1g",
        ]);
        bench.vessels[0].temperature = Kelvin(kelvin);
        let events = bench
            .step(parse_op("wait 10min").unwrap().unwrap())
            .expect("wait");
        hydrolysed(&events, EnzymeFamily::Lactase)
    };
    let near_optimum = activity_at(310.15);
    assert!(near_optimum > activity_at(275.15));
    assert!(near_optimum > activity_at(345.15));
}

#[test]
fn decant_carries_progress_and_catalyst_proportionally() {
    let (mut bench, _) = run(&[
        "add v1 whole_milk 20g",
        "add v1 lactase 0.1g",
        "wait 10min",
        "new beaker",
        "decant v1 v2 0.5",
    ]);
    let states: Vec<_> = bench
        .vessels
        .iter()
        .map(|vessel| {
            vessel.unresolved_materials[0]
                .enzyme_hydrolysis
                .as_ref()
                .expect("transferred progress")
                .converted_fraction
        })
        .collect();
    assert!((states[0] - states[1]).abs() < 1e-12);
    let mass_before: f64 = bench.vessels.iter().map(|vessel| vessel.mass().0).sum();
    let events = bench
        .step(parse_op("wait 10min").unwrap().unwrap())
        .expect("wait");
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(event, Event::EnzymeHydrolysed { .. }))
            .count(),
        2
    );
    let mass_after: f64 = bench.vessels.iter().map(|vessel| vessel.mass().0).sum();
    assert!((mass_after - mass_before).abs() < 1e-9);
}

// ── The acidity window (bio-049, bio-050) ───────────────────────────

/// The core-only solver stack characterises no solution, so there is no pH
/// to read; the corpus runs on the aqueous tail, which does. Setting the
/// solved acidity by hand is how a core test exercises the same term.
fn at_ph(bench: &mut Bench, ph: f64) {
    bench.vessels[0].solution = Some(kerotakis_core::vessel::SolutionInfo {
        ph,
        pe: None,
        redox: Vec::new(),
        ionic_strength: 0.1,
        species: Vec::new(),
        provenance: None,
    });
}

fn pepsin_on_albumin_at(ph: f64) -> f64 {
    let (mut bench, _) = run(&[
        "add v1 protein 1g",
        "add v1 water 100mL",
        "add v1 pepsin 0.001mol",
    ]);
    at_ph(&mut bench, ph);
    let events = bench
        .step(parse_op("wait 1h").unwrap().unwrap())
        .expect("wait");
    hydrolysed(&events, EnzymeFamily::Pepsin)
}

#[test]
fn pepsin_digests_protein_in_acid_and_stops_in_base() {
    let stomach = pepsin_on_albumin_at(1.5);
    let neutral = pepsin_on_albumin_at(7.0);
    let lye = pepsin_on_albumin_at(13.0);
    assert!(stomach > 0.0, "pepsin did nothing in acid");
    assert!(
        stomach > neutral,
        "stomach={stomach}, neutral={neutral}: pepsin must prefer acid"
    );
    assert!(
        lye < 1e-9,
        "pepsin digested {lye} g of protein in a strong base"
    );
}

#[test]
fn the_generic_protease_is_not_switched_off_by_a_neutral_beaker() {
    let (mut bench, _) = run(&[
        "add v1 protein 1g",
        "add v1 water 100mL",
        "add v1 protease 0.001mol",
    ]);
    at_ph(&mut bench, 7.0);
    let events = bench
        .step(parse_op("wait 1h").unwrap().unwrap())
        .expect("wait");
    assert!(hydrolysed(&events, EnzymeFamily::Protease) > 0.0);
}

#[test]
fn an_uncharacterised_beaker_keeps_the_activity_it_always_had() {
    // No aqueous solver has looked, so there is no acidity to read and the
    // pH term must not invent a neutral one and silently halve every rate.
    let (_, events) = run(&[
        "add v1 water 50mL",
        "add v1 gelatin 10g",
        "add v1 protease 0.1g",
        "wait 1h",
    ]);
    assert!(hydrolysed(&events, EnzymeFamily::Protease) > 0.0);
}

// ── A food that carries its own enzyme (bio-052, bio-053) ───────────

#[test]
fn fresh_pineapple_cuts_gelatine_without_an_enzyme_being_weighed_out() {
    let (_, events) = run(&["add v1 gelatin 10g", "add v1 pineapple 20g", "wait 1h"]);
    let cut = hydrolysed(&events, EnzymeFamily::Bromelain);
    assert!(cut > 0.0, "{events:?}");
    let (_, control) = run(&["add v1 gelatin 10g", "add v1 water 20mL", "wait 1h"]);
    assert_eq!(hydrolysed(&control, EnzymeFamily::Bromelain), 0.0);
}

#[test]
fn cooked_pineapple_no_longer_cuts_gelatine_even_after_it_cools() {
    let (mut bench, _) = run(&["add v1 pineapple 20g", "heat v1 20kJ", "add v1 gelatin 10g"]);
    assert!(
        bench.vessels[0].temperature.0 > 343.15,
        "the heat step must actually cook it"
    );
    let hot = bench
        .step(parse_op("wait 1min").unwrap().unwrap())
        .expect("wait");
    assert_eq!(hydrolysed(&hot, EnzymeFamily::Bromelain), 0.0);
    // Irreversibility is the whole point: cooling does not revive it.
    bench.vessels[0].temperature = Kelvin(298.15);
    let cooled = bench
        .step(parse_op("wait 1h").unwrap().unwrap())
        .expect("wait");
    assert_eq!(
        hydrolysed(&cooled, EnzymeFamily::Bromelain),
        0.0,
        "{cooled:?}"
    );
    assert!(bench.vessels[0]
        .unresolved_materials
        .iter()
        .any(|portion| portion
            .enzyme_hydrolysis
            .as_ref()
            .is_some_and(|state| state.carried_enzyme_denatured)));
}

#[test]
fn a_carried_enzyme_and_a_weighed_one_share_the_same_substrate_pool() {
    let alone = run(&["add v1 gelatin 10g", "add v1 pineapple 20g", "wait 10min"]).1;
    let together = run(&[
        "add v1 gelatin 10g",
        "add v1 pineapple 20g",
        "add v1 protease 0.1g",
        "wait 10min",
    ])
    .1;
    let sum = |events: &[Event]| {
        hydrolysed(events, EnzymeFamily::Bromelain) + hydrolysed(events, EnzymeFamily::Protease)
    };
    assert!(sum(&together) > sum(&alone));
    // One pool, one progress record, one event — not one per catalyst.
    assert_eq!(
        together
            .iter()
            .filter(|event| matches!(event, Event::EnzymeHydrolysed { .. }))
            .count(),
        1
    );
}
