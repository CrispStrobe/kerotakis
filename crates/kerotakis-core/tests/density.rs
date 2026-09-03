//! KID-19a: density is a measurement, and it answers what a balance cannot.

use kerotakis_core::ops::{Event, Instrument};
use kerotakis_core::script::parse_op;
use kerotakis_core::Bench;

fn readings(commands: &[&str]) -> (Vec<f64>, Vec<String>) {
    let mut bench = Bench::new();
    let mut values = Vec::new();
    let mut refusals = Vec::new();
    for command in commands {
        let op = parse_op(command)
            .unwrap_or_else(|error| panic!("parse {command}: {error}"))
            .expect("operator");
        for event in bench
            .step(op)
            .unwrap_or_else(|error| panic!("{command}: {error}"))
        {
            match event {
                Event::Measured {
                    instrument: Instrument::Densitometer,
                    value,
                    unit,
                    ..
                } => {
                    assert_eq!(unit, "g/mL");
                    values.push(value);
                }
                Event::NotYetModeled { what, .. } => refusals.push(what),
                _ => {}
            }
        }
    }
    (values, refusals)
}

/// The point of the instrument: three pieces of the same mass, three
/// different substances, and the balance cannot tell them apart.
#[test]
fn three_metals_of_equal_mass_are_told_apart_only_by_density() {
    let (copper, _) = readings(&["add v1 Cu 5g", "measure v1 density"]);
    let (zinc, _) = readings(&["add v1 Zn 5g", "measure v1 density"]);
    let (aluminium, _) = readings(&["add v1 Al 5g", "measure v1 density"]);
    assert!((copper[0] - 8.96).abs() < 0.05, "{copper:?}");
    assert!((zinc[0] - 7.14).abs() < 0.05, "{zinc:?}");
    assert!((aluminium[0] - 2.70).abs() < 0.05, "{aluminium:?}");
    assert!(
        copper[0] > zinc[0] && zinc[0] > aluminium[0],
        "the order is the whole lesson"
    );
}

/// A liquid answers through the solution's own density, solute volume
/// included — the arithmetic KID-13 had to fix. Dissolving sugar makes
/// water heavier, which is what a density tower is built on.
#[test]
fn dissolved_sugar_makes_the_water_denser_and_the_reading_says_so() {
    let (plain, _) = readings(&["add v1 water 100mL", "measure v1 density"]);
    let (sweet, _) = readings(&[
        "add v1 water 100mL",
        "add v1 sucrose 150g",
        "measure v1 density",
    ]);
    assert!((plain[0] - 1.0).abs() < 0.01, "{plain:?}");
    assert!(
        (1.2..1.4).contains(&sweet[0]),
        "a heavy syrup, not a hydrometer that ignores the sugar: {sweet:?}"
    );
}

/// A hydrometer floats in the liquid, so a liquid answers even with
/// solids sitting in it.
#[test]
fn a_liquid_answers_even_when_a_solid_is_sitting_in_it() {
    let (values, _) = readings(&["add v1 water 100mL", "add v1 Cu 5g", "measure v1 density"]);
    assert_eq!(values.len(), 1);
    assert!((values[0] - 1.0).abs() < 0.02, "{values:?}");
}

/// Density belongs to one substance. A heap of two powders has a mass and
/// a volume and no density anyone should be told — and the refusal names
/// what it found rather than saying nothing.
#[test]
fn a_mixture_of_two_solids_refuses_and_names_them() {
    let (values, refusals) = readings(&["add v1 Cu 5g", "add v1 Zn 5g", "measure v1 density"]);
    assert!(values.is_empty(), "no number for a heap: {values:?}");
    assert_eq!(refusals.len(), 1);
    assert!(
        refusals[0].contains("copper") && refusals[0].contains("zinc"),
        "{refusals:?}"
    );
}

/// An empty vessel is a refusal too, and for its own reason.
#[test]
fn an_empty_vessel_has_no_density() {
    let (values, refusals) = readings(&["measure v1 density"]);
    assert!(values.is_empty());
    assert_eq!(refusals.len(), 1);
    assert!(refusals[0].contains("empty"), "{refusals:?}");
}

/// A conserved object answers from its recipe's reviewed bulk density —
/// the raisin, which is why it sinks.
#[test]
fn a_conserved_object_answers_from_its_recipe() {
    let (values, _) = readings(&["add v1 Rosine 5g", "measure v1 density"]);
    assert!((values[0] - 1.35).abs() < 1e-9, "{values:?}");
}
