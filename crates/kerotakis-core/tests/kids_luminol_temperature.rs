//! K59: exercise the child-facing script through the real parser, material
//! expansion, heat/clock state, and scene contract rather than only testing
//! the observation equation in isolation.

use kerotakis_core::scene::{scene, SceneChemiluminescence};
use kerotakis_core::script::parse_op;
use kerotakis_core::{Bench, SpeciesId, VesselId};

fn run(bench: &mut Bench, command: &str) {
    let op = parse_op(command)
        .unwrap_or_else(|error| panic!("parse {command}: {error}"))
        .expect("operator");
    bench
        .step(op)
        .unwrap_or_else(|error| panic!("run {command}: {error}"));
}

fn glow(bench: &Bench, vessel: usize) -> SceneChemiluminescence {
    scene(bench).vessels[vessel]
        .chemiluminescence
        .clone()
        .expect("prepared solution plus peroxide glows")
}

#[test]
fn matched_warm_and_cold_luminol_systems_trade_brightness_for_lifetime() {
    let mut bench = Bench::new();
    for command in [
        "add v1 luminol_glow_solution 20g",
        "add v1 hydrogen_peroxide_3_percent 5g",
        "cool v1 2kJ",
        "new beaker",
        "add v2 luminol_glow_solution 20g",
        "add v2 hydrogen_peroxide_3_percent 5g",
        "heat v2 2kJ",
    ] {
        run(&mut bench, command);
    }

    let cold_now = glow(&bench, 0);
    let warm_now = glow(&bench, 1);
    assert!(warm_now.temperature_k > cold_now.temperature_k);
    assert!(warm_now.relative_intensity > cold_now.relative_intensity);
    assert!(warm_now.half_life_s < cold_now.half_life_s);

    let peroxide_before = [VesselId(0), VesselId(1)].map(|id| {
        bench.vessel(id)
            .unwrap()
            .moles_of(&SpeciesId::new("H2O2"))
            .0
    });
    run(&mut bench, "wait 30s");
    let cold_later = glow(&bench, 0);
    let warm_later = glow(&bench, 1);
    assert!(cold_later.relative_intensity < cold_now.relative_intensity);
    assert!(warm_later.relative_intensity < warm_now.relative_intensity);
    assert!(
        warm_later.relative_intensity / warm_now.relative_intensity
            < cold_later.relative_intensity / cold_now.relative_intensity,
        "the warmer system must lose a larger fraction of its light"
    );

    // Peroxide belongs to the engine's ordinary decomposition network and is
    // consumed while the clock advances. That still does not claim luminol's
    // emitting products or an absolute photon balance.
    for (index, id) in [VesselId(0), VesselId(1)].into_iter().enumerate() {
        let after = bench
            .vessel(id)
            .unwrap()
            .moles_of(&SpeciesId::new("H2O2"))
            .0;
        assert!(after < peroxide_before[index], "the activator must advance with time");
    }
}

#[test]
fn peroxide_is_required_and_its_bounded_dose_controls_the_light() {
    let mut bench = Bench::new();
    run(&mut bench, "add v1 luminol_glow_solution 20g");
    assert!(scene(&bench).vessels[0].chemiluminescence.is_none());
    run(&mut bench, "add v1 hydrogen_peroxide_3_percent 1g");
    let low = glow(&bench, 0);

    let mut high = Bench::new();
    run(&mut high, "add v1 luminol_glow_solution 20g");
    run(&mut high, "add v1 hydrogen_peroxide_3_percent 5g");
    assert!(glow(&high, 0).relative_intensity > low.relative_intensity);
}
