use kerotakis_core::ops::Event;
use kerotakis_core::script::parse_op;
use kerotakis_core::{Bench, Moles, SpeciesId};

fn run(bench: &mut Bench, command: &str) -> Vec<Event> {
    bench
        .step(parse_op(command).expect("valid command").expect("operator"))
        .expect("step succeeds")
}

fn prepared(yeast: Option<&str>) -> Bench {
    let mut bench = Bench::new();
    run(&mut bench, "add v1 water 100mL");
    run(&mut bench, "add v1 table_sugar 10g");
    if let Some(yeast) = yeast {
        run(&mut bench, &format!("add v1 {yeast}"));
    }
    bench
}

#[test]
fn hydrated_yeast_converts_finite_sucrose_with_balanced_stoichiometry() {
    let mut bench = prepared(Some("dry_yeast 1g"));
    let sucrose_before = bench.vessels[0].moles_of(&SpeciesId::new("sucrose")).0;
    let events = run(&mut bench, "wait 600s");
    let step = events.iter().find_map(|event| match event {
        Event::Fermented {
            sucrose_moles,
            ethanol_moles,
            carbon_dioxide_moles,
            active_yeast_grams,
            ..
        } => Some((
            sucrose_moles.0,
            ethanol_moles.0,
            carbon_dioxide_moles.0,
            *active_yeast_grams,
        )),
        _ => None,
    });
    let (sugar, ethanol, carbon_dioxide, active_yeast) = step.expect("fermentation event");
    assert!(sugar > 1e-4);
    assert!((ethanol - 4.0 * sugar).abs() < 1e-12);
    assert!((carbon_dioxide - 4.0 * sugar).abs() < 1e-12);
    assert!(active_yeast > 0.5 && active_yeast <= 1.0);
    assert!(
        (bench.vessels[0].moles_of(&SpeciesId::new("sucrose")).0 - (sucrose_before - sugar)).abs()
            < 1e-12
    );
    assert!((bench.vessels[0].moles_of(&SpeciesId::new("ethanol")).0 - ethanol).abs() < 1e-12);
    assert!(events.iter().any(|event| matches!(
        event,
        Event::GasProduced { reaction, species, moles, .. }
            if reaction == "yeast-sucrose-fermentation"
                && species.0 == "CO2"
                && (moles.0 - carbon_dioxide).abs() < 1e-12
    )));
}

#[test]
fn sugar_water_without_yeast_does_not_ferment() {
    let mut bench = prepared(None);
    let before = bench.vessels[0].moles_of(&SpeciesId::new("sucrose"));
    let events = run(&mut bench, "wait 600s");
    assert!(!events
        .iter()
        .any(|event| matches!(event, Event::Fermented { .. })));
    assert_eq!(
        bench.vessels[0].moles_of(&SpeciesId::new("sucrose")),
        before
    );
    assert_eq!(
        bench.vessels[0].moles_of(&SpeciesId::new("ethanol")),
        Moles(0.0)
    );
}

#[test]
fn already_hydrated_fresh_yeast_starts_faster_than_dry_yeast() {
    let produced = |yeast: &str| {
        let mut bench = prepared(Some(yeast));
        run(&mut bench, "wait 2s")
            .into_iter()
            .find_map(|event| match event {
                Event::Fermented { sucrose_moles, .. } => Some(sucrose_moles.0),
                _ => None,
            })
            .unwrap_or(0.0)
    };
    let dry = produced("dry_yeast 1g");
    let fresh = produced("fresh_yeast 3.333g");
    assert!(fresh > dry * 2.0, "dry={dry}, fresh={fresh}");
}

// ── The three cultures beside the yeast ──────────────────────────────

fn lactic(bench: &Bench) -> f64 {
    bench.vessels[0].moles_of(&SpeciesId::new("lactic_acid")).0
}

fn milk_solids(bench: &Bench) -> f64 {
    bench.vessels[0]
        .unresolved_materials
        .iter()
        .filter(|portion| portion.recipe_id == "household/whole-milk-surrogate")
        .map(|portion| portion.amount)
        .sum()
}

fn yoghurt_at(celsius: Option<&str>) -> Bench {
    let mut bench = Bench::new();
    match celsius {
        Some(at) => run(&mut bench, &format!("add v1 milk 100mL @ {at}")),
        None => run(&mut bench, "add v1 milk 100mL"),
    };
    run(&mut bench, "add v1 yoghurt_culture 1g");
    run(&mut bench, "wait 8h");
    bench
}

#[test]
fn a_lactic_culture_turns_milk_sugar_into_acid_and_makes_no_gas() {
    let mut bench = Bench::new();
    run(&mut bench, "add v1 milk 100mL");
    let solids_before = milk_solids(&bench);
    let mass_before = bench.vessels[0].mass().0;
    run(&mut bench, "add v1 yoghurt_culture 1g");
    let mass_with_culture = bench.vessels[0].mass().0;
    let events = run(&mut bench, "wait 8h");
    assert!(lactic(&bench) > 0.0, "no lactic acid: {events:?}");
    // The lactose came out of conserved unresolved milk solids, and the
    // vessel's mass did not move: acid mass in, solids and water out.
    assert!(milk_solids(&bench) < solids_before);
    assert!(
        (bench.vessels[0].mass().0 - mass_with_culture).abs() < 1e-9,
        "mass moved: {} -> {}",
        mass_with_culture,
        bench.vessels[0].mass().0
    );
    assert!(mass_with_culture > mass_before);
    // Homolactic means no gas. A yoghurt pot does not rise.
    assert_eq!(
        bench.vessels[0].moles_of(&SpeciesId::new("CO2")),
        Moles(0.0)
    );
    assert_eq!(
        bench.vessels[0].moles_of(&SpeciesId::new("ethanol")),
        Moles(0.0)
    );
    assert!(!events
        .iter()
        .any(|event| matches!(event, Event::Fermented { .. })));
}

#[test]
fn refrigeration_slows_the_yoghurt_culture_down() {
    let warm = lactic(&yoghurt_at(None));
    let cold = lactic(&yoghurt_at(Some("5C")));
    assert!(warm > 0.0);
    assert!(cold >= 0.0);
    assert!(
        cold * 10.0 < warm,
        "a refrigerator must slow it by much more than a tenth: warm={warm}, cold={cold}"
    );
}

#[test]
fn acetic_bacteria_oxidise_ethanol_to_vinegar_and_stop_without_air() {
    let vinegar = |with_oxygen: bool| {
        let mut bench = Bench::new();
        run(&mut bench, "add v1 water 100mL");
        run(&mut bench, "add v1 ethanol 0.1mol");
        run(&mut bench, "add v1 acetobacter 1g");
        if with_oxygen {
            run(&mut bench, "add v1 O2 0.1mol");
        }
        let before = bench.vessels[0].mass().0;
        run(&mut bench, "wait 24h");
        let acid = bench.vessels[0].moles_of(&SpeciesId::new("CH3COOH")).0;
        (acid, (bench.vessels[0].mass().0 - before).abs())
    };
    let (aerated, mass_drift) = vinegar(true);
    let (sealed_off, _) = vinegar(false);
    assert!(aerated > 0.0, "no vinegar was made");
    assert!(mass_drift < 1e-9, "mass moved by {mass_drift}");
    assert_eq!(
        sealed_off, 0.0,
        "an oxidation with no oxygen in the vessel must do nothing"
    );
}

#[test]
fn sourdough_makes_acid_and_gas_from_the_same_sugar() {
    let mut bench = Bench::new();
    run(&mut bench, "add v1 sourdough_starter 50g");
    run(&mut bench, "add v1 flour 50g");
    run(&mut bench, "add v1 water 50mL");
    let before = bench.vessels[0].mass().0;
    let events = run(&mut bench, "wait 8h");
    assert!(lactic(&bench) > 0.0, "no acid: {events:?}");
    assert!(
        bench.vessels[0].moles_of(&SpeciesId::new("CO2")).0 > 0.0,
        "no gas: {events:?}"
    );
    assert!(bench.vessels[0].moles_of(&SpeciesId::new("ethanol")).0 > 0.0);
    // The heterolactic equation balances exactly on paper; the residue
    // here is the registry's own sucrose molar mass, 342.2965 against the
    // 342.2970 the same atomic weights add up to. The alcoholic route has
    // carried that 0.0005 g/mol since it was written, so the tolerance
    // names the rounding rather than hiding it.
    let drift = (bench.vessels[0].mass().0 - before).abs();
    assert!(drift < 1e-5, "mass drifted by {drift} g");
    // The gas is announced rather than deposited silently.
    assert!(events.iter().any(|event| matches!(
        event,
        Event::GasProduced { species, .. } if species.0 == "CO2"
    )));
    // Acid and gas in the ratio the balanced heterolactic equation gives.
    let acid = lactic(&bench);
    let gas = bench.vessels[0].moles_of(&SpeciesId::new("CO2")).0;
    assert!((acid - gas).abs() < 1e-12, "acid={acid}, gas={gas}");
}

#[test]
fn a_sourdough_starter_in_milk_ferments_nothing_rather_than_making_silent_gas() {
    let mut bench = Bench::new();
    run(&mut bench, "add v1 milk 100mL");
    run(&mut bench, "add v1 sourdough_starter 20g");
    run(&mut bench, "wait 8h");
    // The heterolactic route deliberately reads dissolved sucrose only,
    // because the gas it makes is announced through the sucrose count and
    // the milk's lactose has no such count. The starter carries a little
    // sugar of its own, so that much ferments and the milk's does not.
    assert_eq!(milk_solids(&bench), {
        let mut control = Bench::new();
        run(&mut control, "add v1 milk 100mL");
        milk_solids(&control)
    });
}

#[test]
fn baker_s_yeast_still_runs_the_route_it_always_did() {
    let mut bench = prepared(Some("dry_yeast 1g"));
    let events = run(&mut bench, "wait 600s");
    assert!(events
        .iter()
        .any(|event| matches!(event, Event::Fermented { .. })));
    assert_eq!(
        bench.vessels[0].moles_of(&SpeciesId::new("lactic_acid")),
        Moles(0.0)
    );
}
