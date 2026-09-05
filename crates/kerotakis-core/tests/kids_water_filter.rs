//! K33: a reviewed quartz-sand teaching suspension has a visible before and
//! after, and filtration changes phase location rather than deleting matter.

use kerotakis_core::ops::Event;
use kerotakis_core::scene::scene;
use kerotakis_core::script::parse_op;
use kerotakis_core::{Bench, Phase, SpeciesId, VesselId};

fn run(bench: &mut Bench, command: &str) -> Vec<Event> {
    let op = parse_op(command)
        .unwrap_or_else(|error| panic!("parse {command}: {error}"))
        .expect("operator");
    bench
        .step(op)
        .unwrap_or_else(|error| panic!("run {command}: {error}"))
}

fn solid_silica(bench: &Bench, vessel: VesselId) -> f64 {
    bench
        .vessel(vessel)
        .unwrap()
        .contents
        .iter()
        .filter(|portion| portion.species.0 == "SiO2" && portion.phase == Phase::Solid)
        .map(|portion| portion.moles.0)
        .sum()
}

#[test]
fn reviewed_sand_suspension_filters_to_clear_water_with_retained_solid() {
    let mut bench = Bench::new();
    run(&mut bench, "add v1 water 100mL");
    run(&mut bench, "add v1 quartz_sand 5g");

    let before = scene(&bench);
    let cloudy = before.vessels[0]
        .liquid
        .as_ref()
        .expect("water remains visible")
        .cloudiness;
    assert!(cloudy > 0.9, "the reviewed suspension must look cloudy: {cloudy}");

    let silica_before: f64 = bench
        .vessels
        .iter()
        .map(|vessel| vessel.moles_of(&SpeciesId::new("SiO2")).0)
        .sum();
    let solid_before = solid_silica(&bench, VesselId(0));
    let events = run(&mut bench, "filter v1 v2");
    assert!(events.iter().any(|event| matches!(event, Event::Filtered { .. })));

    bench.vessel(VesselId(0)).expect("filter residue");
    bench.vessel(VesselId(1)).expect("receiver");
    assert!((solid_silica(&bench, VesselId(0)) - solid_before).abs() < 1e-12);
    assert!(solid_silica(&bench, VesselId(1)) < 1e-12);

    let after = scene(&bench);
    let clear = after.vessels[1]
        .liquid
        .as_ref()
        .expect("filtrate water")
        .cloudiness;
    assert!(clear < 1e-12, "the solid-free filtrate must look clear: {clear}");
    assert!(
        after.vessels[0].solids.iter().any(|solid| solid.species == "SiO2"),
        "the sand's resolved quartz must remain as filter residue"
    );

    let silica_after: f64 = bench
        .vessels
        .iter()
        .map(|vessel| vessel.moles_of(&SpeciesId::new("SiO2")).0)
        .sum();
    assert!((silica_after - silica_before).abs() < 1e-12, "filtration conserves silica");
}
