use kerotakis_core::ops::Event;
use kerotakis_core::script::parse_op;
use kerotakis_core::{Bench, SpeciesId, VesselId};

fn run(bench: &mut Bench, command: &str) -> Vec<Event> {
    let op = parse_op(command)
        .unwrap_or_else(|error| panic!("parse {command}: {error}"))
        .expect("operator");
    bench
        .step(op)
        .unwrap_or_else(|error| panic!("run {command}: {error}"))
}

#[test]
fn magnet_separates_named_iron_filings_from_named_sand() {
    let mut bench = Bench::new();
    run(&mut bench, "add v1 Eisenfeilspäne 5g");
    run(&mut bench, "add v1 Spielsand 10g");
    run(&mut bench, "new beaker");
    let events = run(&mut bench, "magnet v1 v2");

    let source = bench.vessel(VesselId(0)).expect("source");
    let target = bench.vessel(VesselId(1)).expect("target");
    assert!(source.moles_of(&SpeciesId::new("Fe")).0 < 1e-12);
    assert!(source.moles_of(&SpeciesId::new("SiO2")).0 > 0.15);
    assert!(target.moles_of(&SpeciesId::new("Fe")).0 > 0.089);
    assert!(target.moles_of(&SpeciesId::new("SiO2")).0 < 1e-12);
    assert!(
        (source.unresolved_materials[0].amount - 0.5).abs() < 1e-12,
        "the sand's 5% variable mineral fraction must stay with the sand"
    );
    assert!(target.unresolved_materials.is_empty());

    assert!(events.iter().any(|event| matches!(
        event,
        Event::MagnetSeparated { attracted, remained, .. }
            if attracted == &[SpeciesId::new("Fe")]
                && remained == &[SpeciesId::new("SiO2")]
    )));
}
