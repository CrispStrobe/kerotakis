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

/// The reason flour's starch fraction is resolved rather than folded into its
/// unresolved remainder: a school iodine test finds it, and now so does the
/// engine. Nothing about this positive is scripted for flour — it is the same
/// computed amylose-polyiodide band the cornstarch test uses.
#[test]
fn named_lugol_solution_finds_the_starch_in_named_flour() {
    ensure_lugol_recipe();
    let mut bench = Bench::new();
    run(&mut bench, "add v1 water 100mL");
    run(&mut bench, "add v1 Mehl 2g");
    run(&mut bench, "add v1 Lugol-Lösung_1% 2mL");

    let vessel = bench.vessel(VesselId(0)).expect("vessel");
    assert!(vessel.moles_of(&SpeciesId::new("starch")).0 > 0.0);
    // The 30% the recipe cannot name is still in the beaker.
    assert!((vessel.unresolved_materials[0].amount - 0.6).abs() < 1e-12);

    let observed = appearance::observe(vessel);
    let colour = observed.liquid.expect("visible complex colour");
    assert!(colour.b > colour.r && colour.b > colour.g, "{colour:?}");
    assert!(observed.words.contains("blue-black"), "{}", observed.words);
}

/// The same test on a plain flour-and-water dough. Its starch is resolved for
/// the same reason, so the indicator reaches it too.
#[test]
fn named_lugol_solution_finds_the_starch_in_a_plain_dough() {
    ensure_lugol_recipe();
    let mut bench = Bench::new();
    run(&mut bench, "add v1 water 100mL");
    run(&mut bench, "add v1 Teig 3g");
    run(&mut bench, "add v1 Lugol-Lösung_1% 2mL");

    let observed = appearance::observe(bench.vessel(VesselId(0)).expect("vessel"));
    assert!(observed.words.contains("blue-black"), "{}", observed.words);
}

/// And the control that keeps the positive meaningful: wax has no starch
/// in it, so the indicator stays brown next to it.
///
/// This test used to be named for the conserved-unresolved-solid role,
/// because candle wax was one — KID-12 resolved it into paraffin so that
/// a candle could burn, and the role has no user in the shipped registry
/// until a material with nothing installed arrives. The control is the
/// same control: a solid that is not starch must not read as starch, and
/// it must still be in the vessel afterwards.
///
/// It also pins a boundary KID-12 exposed rather than created. An
/// insoluble solid in water reads as a suspension however large the
/// lump — the appearance model has no particle size, which is the same
/// limitation `grind` exists because of — so five grams of wax now
/// clouds the water it is sitting in. A floating candle is not modelled.
#[test]
fn lugol_beside_a_solid_with_no_starch_in_it_stays_brown() {
    ensure_lugol_recipe();
    let mut bench = Bench::new();
    run(&mut bench, "add v1 water 100mL");
    run(&mut bench, "add v1 Kerzenwachs 5g");
    run(&mut bench, "add v1 Lugol-Lösung_1% 2mL");

    let vessel = bench.vessel(VesselId(0)).expect("vessel");
    let observed = appearance::observe(vessel);
    assert!(observed.words.contains("brown"), "{}", observed.words);
    assert!(!observed.words.contains("blue"), "{}", observed.words);
    assert!(
        vessel
            .moles_of(&kerotakis_core::species::SpeciesId::new("paraffin"))
            .0
            > 0.0,
        "the wax is still there: {}",
        observed.words
    );
}
