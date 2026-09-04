//! KID-19b: what is lighter than the liquid is at the top of it.

use kerotakis_core::script::parse_op;
use kerotakis_core::{appearance, Bench};

fn words(commands: &[&str]) -> String {
    let mut bench = Bench::new();
    for command in commands {
        let op = parse_op(command)
            .unwrap_or_else(|error| panic!("parse {command}: {error}"))
            .expect("operator");
        bench
            .step(op)
            .unwrap_or_else(|error| panic!("{command}: {error}"));
    }
    appearance::observe(&bench.vessels[0]).words
}

/// K32 was a silent miss: four polymers with reviewed densities, all four
/// sitting as undifferentiated solids, and `look` saying only that the
/// water was cloudy. Nothing was missing but the comparison.
#[test]
fn polypropylene_floats_and_the_denser_plastics_do_not() {
    let floating = words(&["add v1 water 200mL", "add v1 PP 5g"]);
    assert!(
        floating.contains("polypropylene floats on top"),
        "0.90 g/mL is lighter than water: {floating}"
    );
    for sinker in ["PS", "PET"] {
        let sunk = words(&[
            "add v1 water 200mL",
            &format!("add v1 {sinker} 5g"),
            "wait 2min",
        ]);
        assert!(
            !sunk.contains("floats on top"),
            "{sinker} is denser than water: {sunk}"
        );
        assert!(
            sunk.contains("at the bottom"),
            "and it must still be named where it went: {sunk}"
        );
    }
}

/// Something sitting on the surface is not a suspension. Five grams of
/// floating polystyrene made the water "so cloudy you cannot see through
/// it", which is the same defect the plated-metal branch beside it exists
/// to prevent.
#[test]
fn a_floating_solid_does_not_cloud_the_water() {
    let floating = words(&["add v1 water 200mL", "add v1 PP 5g"]);
    assert!(
        floating.contains("colourless and clear"),
        "the water is clear and the plastic is on top of it: {floating}"
    );
}

/// The float test is a comparison, so it must move when either side does.
/// A liquid denser than the solid floats it.
#[test]
fn the_same_solid_floats_in_a_liquid_dense_enough() {
    use kerotakis_core::buoyancy::floats_in;
    use kerotakis_core::SpeciesId;
    let pet = SpeciesId::new("PET");
    assert_eq!(floats_in(&pet, 1.00), Some(false));
    assert_eq!(floats_in(&pet, 1.50), Some(true));
    // A species with no reviewed density cannot be asked. `None` is not
    // `false`: a solid whose density nobody recorded has not been shown to
    // sink.
    assert_eq!(floats_in(&SpeciesId::new("not-a-species"), 1.0), None);
}

/// Whole-object bulk density, rather than the density of whichever chemical
/// ingredients happen to be resolved, decides ordinary material buoyancy.
#[test]
fn named_objects_use_their_reviewed_bulk_density() {
    for floater in ["apple", "pumice"] {
        let observed = words(&["add v1 water 500mL", &format!("add v1 {floater} 5g")]);
        assert!(
            observed.contains(floater) && observed.contains("floats on top"),
            "{floater} is less dense than water: {observed}"
        );
    }

    let potato = words(&["add v1 water 500mL", "add v1 potato 5g"]);
    assert!(
        potato.contains("potato") && potato.contains("is at the bottom"),
        "a potato is denser than water: {potato}"
    );
}

/// KID-19b's own finding: a salt solution reads exactly the density of the
/// water it was made from, because every ion carries water's density as a
/// structural default. The number is not corrected — it cannot be, without
/// partial molar volumes — so the bench must say what it leaves out.
#[test]
fn a_salt_solution_says_what_its_density_reading_omits() {
    use kerotakis_core::buoyancy::{ionic_volume_unaccounted, liquid_density_g_per_ml};
    use kerotakis_core::species::Phase;
    use kerotakis_core::units::Moles;
    use kerotakis_core::SpeciesId;
    // The ions have to be put in as ions: `Bench::step`'s default stack has
    // no aqueous engine, so `add NaCl` leaves a salt rather than the sodium
    // and chloride the shipped bench speciates it into. This is the state
    // the CLI reaches — 1.0266 mol of each in 200 mL — written directly.
    let mut bench = Bench::new();
    let op = parse_op("add v1 water 200mL")
        .expect("valid")
        .expect("operator");
    bench.step(op).expect("step");
    let vessel = &mut bench.vessels[0];
    vessel.deposit(SpeciesId::new("Na+"), Moles(1.0266), Phase::Aqueous);
    vessel.deposit(SpeciesId::new("Cl-"), Moles(1.0266), Phase::Aqueous);
    let vessel = &bench.vessels[0];
    let density = liquid_density_g_per_ml(vessel).expect("a liquid");
    assert!(
        (density - 1.0).abs() < 0.02,
        "the placeholder densities make brine read as water: {density}"
    );
    assert!(
        ionic_volume_unaccounted(vessel),
        "and the bench must know that it cannot account for the ions"
    );
    // Sucrose has a real measured density, so a sugar solution is not
    // subject to the caveat — which is exactly why the density meter's own
    // tests missed this and K32 found it.
    let mut sweet = Bench::new();
    for command in ["add v1 water 100mL", "add v1 sucrose 150g"] {
        let op = parse_op(command).expect("valid").expect("operator");
        sweet.step(op).expect("step");
    }
    assert!(!ionic_volume_unaccounted(&sweet.vessels[0]));
}
