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
    add(&mut bench, "ethanol", 1.7);
    let events = add(&mut bench, "CuSO4", 0.01);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::NotYetModeled { .. })),
        "no curated row for CuSO4/ethanol yet — the apology stands until the data lands"
    );
}

#[test]
fn permanganate_is_deliberately_not_tabled() {
    // KMnO4 in ethanol REACTS (the oxidation the safety screen warns
    // about); tabulating it as soluble or inert would be a lie. Until
    // the curated-reaction rung lands, the honest apology stands.
    assert!(!kerotakis_core::nonaqueous::verdict_exists(
        &SpeciesId::new("KMnO4"),
        "ethanol"
    ));
}
