use kerotakis_core::appearance;
use kerotakis_core::ops::Event;
use kerotakis_core::script::parse_op;
use kerotakis_core::{Bench, Phase, SpeciesId, VesselId};

fn ensure_lugol_recipe() {
    let document: serde_json::Value = serde_json::from_str(include_str!(
        "../../../data/registry/registry-source-v1.json"
    ))
    .expect("registry source");
    let recipe = document["material_recipes"]
        .as_array()
        .expect("material recipes")
        .iter()
        .find(|recipe| recipe["canonical_key"] == "lugol_solution_1_percent")
        .cloned()
        .expect("Lugol recipe in registry source");
    let recipe = serde_json::from_value(recipe).expect("valid Lugol recipe");
    kerotakis_core::material::register_loaded(vec![recipe]);
}

fn run(bench: &mut Bench, command: &str) -> Vec<Event> {
    let op = parse_op(command)
        .unwrap_or_else(|error| panic!("parse {command}: {error}"))
        .expect("operator");
    bench
        .step(op)
        .unwrap_or_else(|error| panic!("run {command}: {error}"))
}

#[test]
fn named_lugol_solution_stains_named_cornstarch_blue_black() {
    ensure_lugol_recipe();
    let mut bench = Bench::new();
    run(&mut bench, "add v1 water 100mL");
    run(&mut bench, "add v1 Speisestärke 1g");
    let events = run(&mut bench, "add v1 Lugol-Lösung_1% 2mL");

    let vessel = bench.vessel(VesselId(0)).expect("vessel");
    assert!(vessel.contents.iter().any(|portion| {
        portion.species == SpeciesId::new("I2") && portion.phase == Phase::Aqueous
    }));
    assert!(events.iter().any(|event| matches!(
        event,
        Event::Dissolved { species, moles, .. }
            if species.0 == "I2" && moles.0 > 0.0
    )));
    assert!(!events.iter().any(|event| matches!(
        event,
        Event::NotYetModeled { what, .. }
            if what.contains("iodine") || what.contains("starch")
    )));

    let observed = appearance::observe(vessel);
    let colour = observed.liquid.expect("visible complex colour");
    assert!(colour.b > colour.r && colour.b > colour.g, "{colour:?}");
    assert!(observed.words.contains("blue-black"), "{}", observed.words);
}

#[test]
fn lugol_without_starch_stays_brown_not_blue() {
    ensure_lugol_recipe();
    let mut bench = Bench::new();
    run(&mut bench, "add v1 water 100mL");
    run(&mut bench, "add v1 Lugol-Lösung_1% 2mL");
    let observed = appearance::observe(bench.vessel(VesselId(0)).expect("vessel"));
    let colour = observed.liquid.expect("visible Lugol colour");
    assert!(
        colour.r > colour.b,
        "free iodine/iodide should be brown: {colour:?}"
    );
    assert!(observed.words.contains("brown"), "{}", observed.words);
    assert!(!observed.words.contains("blue"), "{}", observed.words);
}
