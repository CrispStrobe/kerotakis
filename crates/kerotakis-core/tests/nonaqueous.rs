//! CAP-23 rung 1: the single-solvent organic bench answers with
//! handbook numbers where it used to apologise. The acceptance is the
//! transcript that motivated the task: salts and a metal in ethanol.

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
fn the_transcript_scenario_answers_instead_of_apologising() {
    let mut bench = Bench::new();
    add(&mut bench, "AgCl", 0.0070);
    add(&mut bench, "AgNO3", 0.0059);
    add(&mut bench, "NaCl", 0.0171);
    let events = add(&mut bench, "ethanol", 1.7126);
    let verdicts: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            Event::DissolvedInSolvent {
                species,
                dissolved,
                undissolved,
                ..
            } => Some((species.0.as_str(), dissolved.0, undissolved.0)),
            _ => None,
        })
        .collect();
    // AgCl: insoluble — everything stays solid.
    let agcl = verdicts
        .iter()
        .find(|(s, ..)| *s == "AgCl")
        .expect("AgCl verdict");
    assert!(agcl.1 == 0.0 && (agcl.2 - 0.0070).abs() < 1e-12);
    // AgNO3: 2.1 g/100 mL — 0.0059 mol in ~100 mL is under the limit,
    // so it all dissolves.
    let agno3 = verdicts
        .iter()
        .find(|(s, ..)| *s == "AgNO3")
        .expect("AgNO3 verdict");
    assert!(agno3.2 == 0.0 && (agno3.1 - 0.0059).abs() < 1e-12);
    // NaCl: 0.065 g/100 mL ≈ 1.1 mmol dissolves, the rest sits.
    let nacl = verdicts
        .iter()
        .find(|(s, ..)| *s == "NaCl")
        .expect("NaCl verdict");
    assert!(
        nacl.1 > 0.0008 && nacl.1 < 0.0015,
        "limit-bound: {}",
        nacl.1
    );
    assert!((nacl.1 + nacl.2 - 0.0171).abs() < 1e-12, "conservation");
    // No apology for any covered species.
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::NotYetModeled { .. })),
        "covered pairs must not also apologise: {events:?}"
    );

    // Zinc: computed inertness, with the reason.
    let events = add(&mut bench, "Zn", 0.0153);
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::InertInSolvent { species, why, .. }
                if species.0 == "Zn" && why.contains("proton source")
        )),
        "zinc gets a computed no-reaction: {events:?}"
    );
}

#[test]
fn a_settled_species_is_not_reverdicted_every_step() {
    let mut bench = Bench::new();
    add(&mut bench, "ethanol", 1.7);
    add(&mut bench, "NaCl", 0.017);
    // NaCl settled at its limit; adding zinc later must not re-report
    // sodium chloride's zero-progress dissolution.
    let events = add(&mut bench, "Zn", 0.01);
    assert!(
        !events.iter().any(|e| matches!(
            e,
            Event::DissolvedInSolvent { species, .. } if species.0 == "NaCl"
        )),
        "settled NaCl stays quiet: {events:?}"
    );
}

#[test]
fn repeated_sulfur_doses_accumulate_and_report_the_current_total() {
    let mut bench = Bench::new();
    add(&mut bench, "ethanol", 1.7126);
    add(&mut bench, "S", 0.0312);
    let events = add(&mut bench, "S", 0.0312);

    let total = bench
        .vessel(VesselId(0))
        .unwrap()
        .moles_of(&SpeciesId::new("S"));
    assert!(
        (total.0 - 0.0624).abs() < 1e-12,
        "every dose remains in inventory"
    );
    let added = events
        .iter()
        .find_map(|event| match event {
            Event::Added {
                species,
                total_after,
                ..
            } if species.0 == "S" => *total_after,
            _ => None,
        })
        .expect("added event carries the post-dose total");
    assert!((added.0 - 0.0624).abs() < 1e-12);
    let rendered = render_events(&events, Register::LV2);
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("0.0624 mol now in vessel")),
        "{rendered:?}"
    );
}

#[test]
fn water_present_means_this_rung_stands_aside() {
    let mut bench = Bench::new();
    add(&mut bench, "water", 5.0);
    add(&mut bench, "ethanol", 1.0);
    let events = add(&mut bench, "NaCl", 0.01);
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::DissolvedInSolvent { .. })),
        "mixed water/organic is the aqueous stack's problem, honestly"
    );
}

#[test]
fn an_uncovered_pair_still_gets_the_honest_apology() {
    let mut bench = Bench::new();
    add(&mut bench, "propanone", 1.7);
    let events = add(&mut bench, "gypsum", 0.01);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::NotYetModeled { .. })),
        "no curated row for gypsum/propanone — the apology stands until the data lands"
    );
}

#[test]
fn permanganate_is_deliberately_not_tabled() {
    assert!(!kerotakis_core::nonaqueous::verdict_exists(
        &SpeciesId::new("KMnO4"),
        "ethanol"
    ));
}

#[test]
fn calcium_chloride_is_very_soluble_in_ethanol() {
    let mut bench = Bench::new();
    add(&mut bench, "ethanol", 1.7126);
    let events = add(&mut bench, "CaCl2", 0.005);
    let verdict = events
        .iter()
        .find_map(|e| match e {
            Event::DissolvedInSolvent {
                species,
                dissolved,
                undissolved,
                ..
            } if species.0 == "CaCl2" => Some((dissolved.0, undissolved.0)),
            _ => None,
        })
        .expect("CaCl2 verdict");
    assert!(
        verdict.0 > 0.0 && verdict.1 == 0.0,
        "CaCl2 at 25.8 g/100 mL should dissolve completely at 0.005 mol in ~100 mL ethanol"
    );
}

#[test]
fn sodium_carbonate_is_insoluble_in_ethanol() {
    let mut bench = Bench::new();
    add(&mut bench, "ethanol", 1.7126);
    let events = add(&mut bench, "Na2CO3", 0.01);
    let verdict = events
        .iter()
        .find_map(|e| match e {
            Event::DissolvedInSolvent {
                species,
                dissolved,
                undissolved,
                ..
            } if species.0 == "Na2CO3" => Some((dissolved.0, undissolved.0)),
            _ => None,
        })
        .expect("Na2CO3 verdict");
    assert!(
        verdict.0 == 0.0 && (verdict.1 - 0.01).abs() < 1e-12,
        "Na2CO3 is insoluble in ethanol — all stays solid"
    );
}

#[test]
fn sulfur_dissolves_well_in_ethyl_acetate() {
    let mut bench = Bench::new();
    add(&mut bench, "ethyl_acetate", 1.5);
    let events = add(&mut bench, "S", 0.001);
    let verdict = events
        .iter()
        .find_map(|e| match e {
            Event::DissolvedInSolvent {
                species,
                dissolved,
                undissolved,
                ..
            } if species.0 == "S" => Some((dissolved.0, undissolved.0)),
            _ => None,
        })
        .expect("S verdict");
    assert!(
        verdict.0 > 0.0 && verdict.1 == 0.0,
        "sulfur at 1.8 g/100 mL in ethyl acetate: 0.001 mol should dissolve completely"
    );
}

#[test]
fn zinc_is_inert_in_hexane() {
    let mut bench = Bench::new();
    add(&mut bench, "hexane", 1.5);
    let events = add(&mut bench, "Zn", 0.01);
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::InertInSolvent { species, .. } if species.0 == "Zn"
        )),
        "zinc gets computed no-reaction in hexane: {events:?}"
    );
}
