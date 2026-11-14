use kerotakis_core::ops::{Event, Operator};
use kerotakis_core::script::parse_op;
use kerotakis_core::Bench;
use kerotakis_core::{Moles, Phase, SpeciesId};

fn run(bench: &mut Bench, command: &str) -> Vec<Event> {
    bench.step(parse_op(command).unwrap().unwrap()).unwrap()
}

#[test]
fn naked_egg_owns_its_inventory_and_osmosis_conserves_mass() {
    let mut bench = Bench::new();
    run(&mut bench, "add v1 water 100g");
    run(&mut bench, "add v1 naked_egg 50g");
    assert_eq!(bench.vessels[0].material_objects.len(), 1);
    let before = bench.vessels[0].mass().0;
    let events = bench.step(Operator::Wait { seconds: 3600.0 }).unwrap();
    assert_eq!(bench.vessels[0].elapsed_seconds, 3600.0);
    assert_eq!(
        bench.vessels[0].material_objects[0].state.elapsed_seconds,
        3600.0
    );
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e, Event::OsmosisChanged { .. }))
            .count(),
        1
    );
    assert!(events
        .iter()
        .any(|e| matches!(e, Event::OsmosisChanged { mass_change_g, .. } if *mass_change_g > 0.0)));
    assert!((bench.vessels[0].mass().0 - before).abs() < 1e-8);

    let scene = kerotakis_core::scene::scene(&bench);
    let object = &scene.vessels[0].material_objects[0];
    let osmosis = object.osmosis.as_ref().expect("standing osmosis readout");
    assert_eq!(osmosis.direction, "into_object");
    assert!(osmosis.water_moles > 0.0);
    assert!(osmosis.mass_change_g > 0.0);
    assert!((object.exchanged_water_moles - osmosis.water_moles).abs() < 1e-12);
    assert!(osmosis.basis.contains("osmolarity-gradient teaching model"));
    assert!(osmosis.basis.contains("24 h exponential timescale"));
    assert!(osmosis.basis.contains("not object size"));
    assert!(scene.vessels[0]
        .words
        .contains("final equilibrium are not modeled"));

    let restored_bench: Bench =
        serde_json::from_str(&serde_json::to_string(&bench).unwrap()).unwrap();
    let restored_scene = kerotakis_core::scene::scene(&restored_bench);
    assert_eq!(restored_scene.vessels[0].material_objects[0], *object);

    let mut old_wire = serde_json::to_value(&scene).unwrap();
    old_wire["vessels"][0]["material_objects"][0]
        .as_object_mut()
        .unwrap()
        .remove("osmosis");
    let restored: kerotakis_core::scene::Scene = serde_json::from_value(old_wire).unwrap();
    assert!(restored.vessels[0].material_objects[0].osmosis.is_none());
}

#[test]
fn standing_osmosis_preserves_outward_transfer_direction() {
    let mut bench = Bench::new();
    run(&mut bench, "add v1 water 100g");
    run(&mut bench, "add v1 sucrose 1mol");
    run(&mut bench, "add v1 naked_egg 50g");
    let events = bench.step(Operator::Wait { seconds: 3600.0 }).unwrap();
    assert!(events.iter().any(
        |event| matches!(event, Event::OsmosisChanged { mass_change_g, .. } if *mass_change_g < 0.0)
    ));

    let scene = kerotakis_core::scene::scene(&bench);
    let osmosis = scene.vessels[0].material_objects[0]
        .osmosis
        .as_ref()
        .expect("standing outward osmosis readout");
    assert_eq!(osmosis.direction, "out_of_object");
    assert!(osmosis.water_moles < 0.0);
    assert!(osmosis.mass_change_g < 0.0);
}

#[test]
fn cut_apple_browns_only_as_a_prepared_surface() {
    let mut bench = Bench::new();
    run(&mut bench, "add v1 cut_apple 20g");
    let events = bench.step(Operator::Wait { seconds: 900.0 }).unwrap();
    assert!(events.iter().any(
        |e| matches!(e, Event::BrowningChanged { browned_fraction, .. } if *browned_fraction > 0.0)
    ));
    assert!(bench.vessels[0].material_objects[0].state.browned_fraction > 0.0);
}

#[test]
fn liquid_pour_does_not_move_a_coherent_object() {
    let mut bench = Bench::new();
    run(&mut bench, "add v1 water 100g");
    run(&mut bench, "add v1 cut_apple 20g");
    run(&mut bench, "decant v1 v2 0.5");
    assert_eq!(bench.vessels[0].material_objects.len(), 1);
    assert!(bench
        .vessels
        .get(1)
        .is_none_or(|v| v.material_objects.is_empty()));
}

#[test]
fn old_snapshot_without_new_aggregate_state_still_decodes() {
    let bench = Bench::new();
    let mut value = serde_json::to_value(&bench).unwrap();
    for vessel in value["vessels"].as_array_mut().unwrap() {
        vessel.as_object_mut().unwrap().remove("soap_scum");
        vessel.as_object_mut().unwrap().remove("material_objects");
    }
    let restored: Bench = serde_json::from_value(value).unwrap();
    assert!(restored.vessels[0].soap_scum.is_none());
}

#[test]
fn declared_fatty_soap_forms_a_stoichiometric_conserved_aggregate() {
    let mut bench = Bench::new();
    bench.vessels[0].deposit(SpeciesId::new("Ca+2"), Moles(0.001), Phase::Aqueous);
    let before = bench.vessels[0].mass().0;
    let events = run(&mut bench, "add v1 fatty_soap 1g");
    assert!(events.iter().any(|e| matches!(e, Event::SoapScumFormed { divalent_ion_moles, .. } if (*divalent_ion_moles - 0.001).abs() < 1e-12)));
    let scum = bench.vessels[0].soap_scum.as_ref().unwrap();
    assert!((scum.soap_equivalent_moles - 0.002).abs() < 1e-12);
    // Calcium-stearate-equivalent and released sodium use rounded registry
    // molar masses, so conservation is pinned at the declared surrogate scale.
    let after = bench.vessels[0].mass().0;
    assert!(
        (after - (before + 1.0)).abs() < 2e-3,
        "before={before} after={after}"
    );
}

#[test]
fn magnesium_soap_scum_also_conserves_mass() {
    let mut bench = Bench::new();
    bench.vessels[0].deposit(SpeciesId::new("Mg+2"), Moles(0.001), Phase::Aqueous);
    let before = bench.vessels[0].mass().0;
    run(&mut bench, "add v1 fatty_soap 1g");
    let after = bench.vessels[0].mass().0;
    assert!(
        (after - (before + 1.0)).abs() < 1e-9,
        "before={before} after={after}"
    );
}

#[test]
fn prepared_objects_are_visible_without_releasing_their_ingredients() {
    for material in ["cut_apple", "naked_egg"] {
        let mut bench = Bench::new();
        run(&mut bench, &format!("add v1 {material} 5g"));
        let before = bench.vessels[0].clone();
        let seen = kerotakis_core::appearance::observe(&before);
        assert!(seen.clauses.iter().any(|p| p.key == "look.prepared-object"));
        assert!(!seen.clauses.iter().any(|p| p.key == "look.nothing-visible"));
        assert!(seen.deposit.is_none());
        assert!(seen.liquid.is_none());
        let scene = kerotakis_core::scene::scene(&bench);
        assert_eq!(scene.vessels[0].words, seen.words);
        assert_eq!(bench.vessels[0].material_objects, before.material_objects);
        assert!(bench.vessels[0].contents.is_empty());
        assert_eq!(bench.vessels[0].mass().0, 5.0);
        for locale in [
            kerotakis_core::Locale::parse("de"),
            kerotakis_core::Locale::parse("fr"),
        ] {
            assert_ne!(seen.say(locale), seen.words);
            assert!(!seen.say(locale).contains(material));
        }
        assert_eq!(
            seen.clauses
                .iter()
                .any(|p| p.key == "look.prepared-surface-browned"),
            material == "cut_apple"
        );
    }
}

#[test]
fn prepared_clock_advances_once_and_browning_composes_across_waits() {
    let mut whole = Bench::new();
    run(&mut whole, "add v1 cut_apple 5g");
    let before = whole.vessels[0].material_objects[0].clone();
    let mut split = whole.clone();
    let events = whole.step(Operator::Wait { seconds: 1800.0 }).unwrap();
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e, Event::BrowningChanged { .. }))
            .count(),
        1
    );
    for seconds in [600.0, 1200.0] {
        split.step(Operator::Wait { seconds }).unwrap();
    }
    for bench in [&whole, &split] {
        let v = &bench.vessels[0];
        let object = &v.material_objects[0];
        assert_eq!(v.elapsed_seconds, 1800.0);
        assert_eq!(object.state.elapsed_seconds, 1800.0);
        // Independent closed-form value for the existing 900 s teaching law.
        assert!((object.state.browned_fraction - (1.0 - (-2.0_f64).exp())).abs() < 1e-12);
        assert_eq!(object.components, before.components);
        assert_eq!(object.mass_g, before.mass_g);
        let seen = kerotakis_core::appearance::observe(v);
        assert!(seen.words.contains("86% browned"), "{}", seen.words);
        assert_eq!(
            kerotakis_core::scene::scene(bench).vessels[0].words,
            seen.words
        );
        assert!(seen
            .say(kerotakis_core::Locale::parse("de"))
            .contains("86 % gebräunt"));
        assert!(seen
            .say(kerotakis_core::Locale::parse("fr"))
            .contains("86 %"));
    }
    assert_eq!(
        whole.vessels[0].material_objects,
        split.vessels[0].material_objects
    );
}

#[test]
fn apple_observation_retains_ascorbate_and_oxygen_controls() {
    let mut bench = Bench::new();
    run(&mut bench, "new");
    run(&mut bench, "add v1 cut_apple 5g");
    run(&mut bench, "add v2 cut_apple 5g");
    run(&mut bench, "add v2 ascorbic_acid 0.0001mol");
    let mut split = bench.clone();
    bench.step(Operator::Wait { seconds: 1800.0 }).unwrap();
    for seconds in [600.0, 1200.0] {
        split.step(Operator::Wait { seconds }).unwrap();
    }
    for (whole, split) in bench.vessels.iter().zip(&split.vessels) {
        assert_eq!(whole.elapsed_seconds, split.elapsed_seconds);
        assert_eq!(whole.material_objects[0].state.elapsed_seconds, 1800.0);
        assert_eq!(split.material_objects[0].state.elapsed_seconds, 1800.0);
        assert!(
            (whole.material_objects[0].state.browned_fraction
                - split.material_objects[0].state.browned_fraction)
                .abs()
                < 1e-12
        );
    }
    let fractions: Vec<_> = bench
        .vessels
        .iter()
        .map(|v| v.material_objects[0].state.browned_fraction)
        .collect();
    assert!((fractions[0] / 2.0 - fractions[1]).abs() < 1e-12);
    assert!(kerotakis_core::appearance::observe(&bench.vessels[1])
        .words
        .contains("43% browned"));
    assert!(bench.vessels[1]
        .contents
        .iter()
        .any(|p| p.species.0 == "ascorbic_acid" && p.moles.0 == 0.0001));
    // Raw apple remains a different recipe without a prepared browning surface.
    let mut raw = Bench::new();
    run(&mut raw, "add v1 apple 5g");
    raw.step(Operator::Wait { seconds: 1800.0 }).unwrap();
    assert!(raw.vessels[0].material_objects.is_empty());
    assert!(!kerotakis_core::appearance::observe(&raw.vessels[0])
        .clauses
        .iter()
        .any(|p| p.key == "look.prepared-surface-browned"));
    // Oxygen-free boundary is independently controlled by the existing law.
    assert_eq!(
        kerotakis_core::kitchen_biology::apple_browning(1800.0, 0.0, 0.0),
        Some(0.0)
    );
}
