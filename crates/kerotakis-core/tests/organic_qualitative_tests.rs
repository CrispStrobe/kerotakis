//! Qualitative organic/biochemical tests: biuret colour, chemical protein
//! denaturation, catalase heat inactivation and the reducing-sugar route
//! through the curated stage.

use kerotakis_core::script::parse_op;
use kerotakis_core::species::Phase;
use kerotakis_core::{appearance, biuret, protein, Bench, Moles, SpeciesId};

fn step(bench: &mut Bench, command: &str) {
    bench
        .step(parse_op(command).expect("parse").expect("operator"))
        .expect(command);
}

fn alkaline_copper(bench: &mut Bench) {
    let v = &mut bench.vessels[0];
    v.deposit(SpeciesId::new("Na+"), Moles(0.005), Phase::Aqueous);
    v.deposit(
        SpeciesId::new("base_equivalents"),
        Moles(0.005),
        Phase::Aqueous,
    );
    v.deposit(SpeciesId::new("Cu(OH)2"), Moles(5e-5), Phase::Solid);
}

#[test]
fn egg_white_in_alkaline_copper_is_violet_and_blank_is_not() {
    let mut bench = Bench::new();
    step(&mut bench, "add v1 water 10mL");
    step(&mut bench, "add v1 egg_white 1g");
    alkaline_copper(&mut bench);
    let v = &bench.vessels[0];
    assert!(biuret::complex_moles(v) > 0.0);
    let look = appearance::observe(v);
    assert!(
        look.notes.iter().any(|n| n.key == "look.biuret-surrogate"),
        "{}",
        look.words
    );
    assert!(
        !look.words.contains("copper(II) hydroxide"),
        "{}",
        look.words
    );

    let mut blank = Bench::new();
    step(&mut blank, "add v1 water 10mL");
    alkaline_copper(&mut blank);
    assert_eq!(biuret::complex_moles(&blank.vessels[0]), 0.0);
    let look = appearance::observe(&blank.vessels[0]);
    assert!(!look.notes.iter().any(|n| n.key == "look.biuret-surrogate"));
}

#[test]
fn ethanol_precipitates_egg_white_but_a_splash_does_not() {
    let mut bench = Bench::new();
    step(&mut bench, "add v1 water 10mL");
    step(&mut bench, "add v1 egg_white 1g");
    step(&mut bench, "add v1 ethanol 5mL");
    assert!(protein::observe(&bench.vessels[0])[0].coagulated);

    let mut splash = Bench::new();
    step(&mut splash, "add v1 water 10mL");
    step(&mut splash, "add v1 egg_white 1g");
    step(&mut splash, "add v1 ethanol 0.2mL");
    assert!(!protein::observe(&splash.vessels[0])[0].coagulated);
}

#[test]
fn whole_milk_carries_biuret_reactive_protein() {
    let mut bench = Bench::new();
    step(&mut bench, "add v1 whole_milk 5mL");
    let grams: f64 = protein::observe(&bench.vessels[0])
        .iter()
        .map(|p| p.protein_mass_g)
        .sum();
    // 5 mL at 1.03 g/mL, 3.15 g protein per 100 g.
    assert!((grams - 5.0 * 1.03 * 0.0315).abs() < 1e-6, "{grams}");
}

#[test]
fn boiled_dry_yeast_lot_is_marked_denatured_and_unboiled_is_not() {
    let mut boiled = Bench::new();
    step(&mut boiled, "add v1 water 10mL");
    step(&mut boiled, "add v1 dry_yeast 0.5g");
    step(&mut boiled, "heat v1 3kJ");
    step(&mut boiled, "cool v1 2.5kJ");
    assert!(boiled.vessels[0]
        .lots
        .iter()
        .any(|lot| lot.species.0 == "catalase" && lot.heat_denatured));

    let mut fresh = Bench::new();
    step(&mut fresh, "add v1 water 10mL");
    step(&mut fresh, "add v1 dry_yeast 0.5g");
    assert!(fresh.vessels[0].lots.iter().all(|lot| !lot.heat_denatured));
}

#[test]
fn boiled_yeast_no_longer_decomposes_peroxide() {
    let oxygen = |boil: bool| {
        let mut bench = Bench::new();
        step(&mut bench, "add v1 water 10mL");
        step(&mut bench, "add v1 dry_yeast 0.5g");
        if boil {
            step(&mut bench, "heat v1 3kJ");
            step(&mut bench, "cool v1 2.5kJ");
        } else {
            step(&mut bench, "wait 30s");
        }
        step(&mut bench, "add v1 hydrogen_peroxide_3_percent 5mL");
        step(&mut bench, "wait 60s");
        let v = &bench.vessels[0];
        v.moles_of(&SpeciesId::new("H2O2")).0
    };
    // 5.05 g of 3 % peroxide.
    let initial = 5.05 * 0.03 / 34.0147;
    let fresh_left = oxygen(false);
    let boiled_left = oxygen(true);
    assert!(
        fresh_left < 0.8 * initial,
        "fresh {fresh_left} of {initial}"
    );
    assert!(
        boiled_left > 0.99 * initial,
        "boiled {boiled_left} of {initial}"
    );
}

#[test]
fn reducing_sugar_route_fires_from_the_curated_stage() {
    use kerotakis_core::solve::Equilibrator;
    let make = |sugar: &str| {
        let mut v = kerotakis_core::Vessel::new(kerotakis_core::VesselId(0), "test tube");
        v.deposit(SpeciesId::new("water"), Moles(0.5), Phase::Liquid);
        v.deposit(SpeciesId::new(sugar), Moles(0.001), Phase::Aqueous);
        v.deposit(SpeciesId::new("Na+"), Moles(0.003), Phase::Aqueous);
        v.deposit(
            SpeciesId::new("base_equivalents"),
            Moles(0.002),
            Phase::Aqueous,
        );
        v.deposit(SpeciesId::new("SO4-2"), Moles(0.0005), Phase::Aqueous);
        v.deposit(SpeciesId::new("CuO"), Moles(0.0005), Phase::Solid);
        v.temperature = kerotakis_core::Kelvin(368.15);
        v
    };
    let mut curated = kerotakis_core::curated::CuratedEquilibrator;
    let mut glucose = make("glucose");
    assert!(curated.applies(&glucose));
    curated.equilibrate(&mut glucose).unwrap();
    assert!(glucose.moles_of(&SpeciesId::new("Cu2O")).0 > 2.4e-4);
    let mut sucrose = make("sucrose");
    curated.equilibrate(&mut sucrose).unwrap();
    assert_eq!(sucrose.moles_of(&SpeciesId::new("Cu2O")).0, 0.0);
}
