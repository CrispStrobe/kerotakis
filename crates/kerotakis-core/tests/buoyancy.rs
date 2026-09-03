//! KID-13: the dancing raisin, and the density arithmetic under it.

use kerotakis_core::buoyancy;
use kerotakis_core::ops::Event;
use kerotakis_core::script::parse_op;
use kerotakis_core::species::SpeciesId;
use kerotakis_core::Bench;

fn bench_with(commands: &[&str]) -> Bench {
    let mut bench = Bench::new();
    for command in commands {
        let op = parse_op(command)
            .unwrap_or_else(|error| panic!("parse {command}: {error}"))
            .expect("operator");
        bench
            .step(op)
            .unwrap_or_else(|error| panic!("{command}: {error}"));
    }
    bench
}

/// A raisin is an object, and the recipe says so. Its sugars are
/// installed species and are deliberately left inside it: dissolving them
/// would delete the thing the experiment is about.
#[test]
fn a_raisin_is_conserved_whole_and_is_still_a_visible_object() {
    let recipe = kerotakis_core::material::lookup("Rosine", Some("de")).expect("localized raisin");
    assert_eq!(recipe.canonical_key, "raisin");
    let expansion = recipe.expand(5.0, 0).expect("fixed expansion");
    assert!(
        expansion.components.is_empty(),
        "a raisin does not dissolve into its sugars over a demonstration"
    );
    assert!((expansion.unresolved_amount - 5.0).abs() < 1e-12);
    assert!(
        recipe
            .lot_assumptions
            .iter()
            .any(|assumption| assumption.contains("installed species")),
        "the reason must say that the sugars ARE installed and are still not resolved"
    );

    // KID-12 resolved the last two recipes carrying the conserved
    // unresolved solid role. The raisin is its user again, which is what
    // keeps that contract exercised rather than merely supported.
    let bench = bench_with(&["add v1 Rosine 5g"]);
    let vessel = &bench.vessels[0];
    let solids = kerotakis_core::material::conserved_unresolved_solids(vessel);
    assert_eq!(solids.len(), 1);
    assert_eq!(solids[0].colour_word, "dark brown");
    let observed = kerotakis_core::appearance::observe(vessel);
    assert!(observed.words.contains("raisin"), "{}", observed.words);
}

/// The number the experiment is about: a raisin at 1.35 g/mL in water
/// needs attached bubbles worth about a third of its own volume.
#[test]
fn a_raisin_in_water_needs_a_third_of_its_volume_in_attached_gas() {
    let bench = bench_with(&["add v1 Rosine 5g", "add v1 water 200mL"]);
    let ride = buoyancy::observe(&bench.vessels[0]).expect("a raisin in a liquid");
    assert!((ride.object_density_g_per_ml - 1.35).abs() < 1e-9);
    assert!((ride.liquid_density_g_per_ml - 1.0).abs() < 0.01);
    assert!(
        (ride.lift_gas_fraction - 0.35).abs() < 0.01,
        "{}",
        ride.lift_gas_fraction
    );
}

/// A dissolved solid has a volume, and forgetting it is not a rounding
/// error. `Vessel::liquid_volume` excludes solute volume by design — the
/// solution's volume is carried by its solvent — and reading a density
/// straight off that gave **2.33 g/mL** for a sugar syrup, denser than
/// anything that has ever been poured. The solute's own density puts the
/// volume back.
#[test]
fn a_sugar_syrup_reads_the_density_a_hydrometer_would() {
    let bench = bench_with(&[
        "add v1 water 100mL",
        "add v1 sucrose 200g",
        "add v1 Rosine 5g",
    ]);
    let vessel = &bench.vessels[0];
    assert!(
        vessel.moles_of(&SpeciesId::new("sucrose")).0 > 0.0,
        "the sugar is in the vessel"
    );
    let density = buoyancy::liquid_density_g_per_ml(vessel).expect("a liquid");
    assert!(
        (1.2..1.4).contains(&density),
        "a two-thirds sugar syrup is about 1.3 g/mL, not {density}"
    );

    // And the raisin is far closer to floating in it than in water.
    let ride = buoyancy::observe(vessel).expect("a raisin in a liquid");
    assert!(
        ride.lift_gas_fraction < 0.15,
        "the syrup does most of the lifting: {}",
        ride.lift_gas_fraction
    );
}

/// The event is triggered by gas actually leaving the liquid, not by the
/// raisin being present. Still water offers nothing to ride, and the
/// bench must not narrate a dance that is not happening.
#[test]
fn still_water_says_nothing_about_riding_bubbles() {
    let mut bench = Bench::new();
    let mut events = Vec::new();
    for command in ["add v1 Rosine 5g", "add v1 water 200mL"] {
        let op = parse_op(command).expect("valid").expect("operator");
        events.extend(bench.step(op).expect("step"));
    }
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::BubbleRide { .. })),
        "no gas, no ride"
    );
}

/// Nothing but the raisin rides. A stone is also denser than water and
/// gas does not lift it: the effect needs a surface bubbles stick to, and
/// no recipe here describes surface texture — so the table is curated,
/// and this test is what stops it being inferred from density.
#[test]
fn an_ordinary_dense_solid_is_not_a_bubble_rider() {
    let bench = bench_with(&["add v1 chalk_stick 5g", "add v1 water 200mL"]);
    assert!(buoyancy::observe(&bench.vessels[0]).is_none());
}
