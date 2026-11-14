use kerotakis_core::authority::SpillDestination;
use kerotakis_core::ops::Instrument;
use kerotakis_core::script::parse_op;
use kerotakis_core::{Bench, Event, Operator, VesselId};

fn run(bench: &mut Bench, command: &str) -> Vec<Event> {
    bench
        .step(parse_op(command).expect(command).expect("operation"))
        .expect(command)
}

fn close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-8, "{actual} != {expected}");
}

#[test]
fn mass_fraction_recipes_weigh_the_whole_dose_once_in_every_physical_form() {
    // Yeast contributes a resolved catalyst, fresh yeast also contributes
    // water, and milk is the existing homogeneous-liquid control. Neither
    // resolved ingredients nor unresolved solids may be omitted or doubled.
    for material in ["dry_yeast", "fresh_yeast", "milk"] {
        let mut bench = Bench::new();
        run(&mut bench, &format!("add v1 {material} 1g"));
        close(bench.vessels[0].mass().0, 1.0);
        assert!(!bench.vessels[0].unresolved_materials.is_empty());
    }
}

#[test]
fn original_fe003_ingredient_bases_have_honest_balance_readings() {
    for (water, yeast, expected) in [
        ("50mL", "dry_yeast 0.30g", 51.15),
        ("49.3mL", "fresh_yeast 1g", 51.1521),
        ("50mL", "fresh_yeast 0.30g", 51.15),
    ] {
        let mut bench = Bench::new();
        run(&mut bench, &format!("add v1 water {water} @ 30C"));
        run(&mut bench, "add v1 sucrose 1g");
        run(&mut bench, &format!("add v1 {yeast}"));
        close(bench.vessels[0].mass().0, expected);
        let events = run(&mut bench, "measure v1 balance");
        let measured = events.iter().find_map(|event| match event {
            Event::Measured {
                instrument: Instrument::Balance,
                value,
                ..
            } => Some(*value),
            _ => None,
        });
        close(measured.expect("balance"), expected);
    }
}

#[test]
fn pouring_then_discarding_keeps_resolved_and_unresolved_mass_in_their_owners() {
    for material in ["dry_yeast", "fresh_yeast", "milk"] {
        let mut bench = Bench::new();
        run(&mut bench, "add v1 water 50g");
        run(&mut bench, &format!("add v1 {material} 1g"));
        run(&mut bench, "new");
        bench
            .step(Operator::Decant {
                from: VesselId(0),
                to: VesselId(1),
                fraction: 0.5,
            })
            .expect("pour");
        close(bench.vessels.iter().map(|v| v.mass().0).sum(), 51.0);
        run(&mut bench, "discard v1");
        run(&mut bench, "discard v2");
        close(bench.vessels.iter().map(|v| v.mass().0).sum(), 0.0);
        let waste = bench.spill(&SpillDestination::Waste).expect("waste");
        close(waste.as_vessel_probe().mass().0, 51.0);
    }
}
