use kerotakis_core::material::MaterialBasis;
use kerotakis_core::ops::{Event, Operator};
use kerotakis_core::script::{parse_op, parse_op_typed, ParseErrorKind};
use kerotakis_core::species::SpeciesId;
use kerotakis_core::Bench;

#[test]
fn household_peroxide_volume_expands_and_pins_recipe() {
    let op = parse_op("add v1 Wasserstoffperoxid_3% 100mL")
        .expect("valid material command")
        .expect("operator");
    let encoded = serde_json::to_string(&op).expect("serialize operator");
    let replayed: Operator = serde_json::from_str(&encoded).expect("replay operator");
    assert_eq!(replayed, op);

    let mut bench = Bench::new();
    let events = bench.step(op).expect("add material");
    let material = events
        .iter()
        .find_map(|event| match event {
            Event::MaterialAdded {
                recipe_id,
                recipe_version,
                total_amount,
                basis,
                sample_seed,
                components,
                unresolved_amount,
                ..
            } => Some((
                recipe_id,
                recipe_version,
                total_amount,
                basis,
                sample_seed,
                components,
                unresolved_amount,
            )),
            _ => None,
        })
        .expect("material event");
    assert_eq!(material.0, "household/hydrogen-peroxide-3-percent");
    assert_eq!(*material.1, 1);
    assert!((*material.2 - 101.0).abs() < 1e-12); // 100 mL × 1.01 g/mL
    assert_eq!(*material.3, MaterialBasis::MassFraction);
    assert_eq!(*material.4, 0);
    assert_eq!(material.5.len(), 2);
    assert!(material.6.abs() < 1e-12);

    let peroxide = material
        .5
        .iter()
        .find(|component| component.species == SpeciesId::new("H2O2"))
        .expect("peroxide component");
    let water = material
        .5
        .iter()
        .find(|component| component.species == SpeciesId::new("water"))
        .expect("water component");
    assert!((peroxide.basis_amount - 3.03).abs() < 1e-12);
    assert!((water.basis_amount - 97.97).abs() < 1e-12);
    assert!((peroxide.basis_amount + water.basis_amount - 101.0).abs() < 1e-12);
}

#[test]
fn material_identity_is_not_reported_as_unknown_species() {
    assert!(matches!(
        parse_op_typed("add v1 Essig 10mL"),
        Ok(Some(Operator::AddMaterial { .. }))
    ));
    assert_eq!(
        parse_op_typed("add v1 imaginary_cleaner 10mL")
            .expect_err("unknown identity")
            .kind,
        ParseErrorKind::UnknownSpecies
    );
}

#[test]
fn dry_yeast_lot_records_when_water_first_hydrates_it() {
    let mut bench = Bench::new();
    let yeast = parse_op("add v1 Hefe 1g")
        .expect("valid yeast material")
        .expect("yeast operator");
    bench.step(yeast).expect("add dry yeast");
    let catalase = bench.vessels[0]
        .lots
        .iter()
        .find(|lot| lot.species == SpeciesId::new("catalase"))
        .expect("recipe component has lot provenance");
    assert_eq!(
        catalase.source.as_deref(),
        Some("material recipe household/dry-yeast-catalase-surrogate")
    );
    assert_eq!(
        catalase.hydrated_at, None,
        "an empty vessel is not hydration"
    );

    let water = parse_op("add v1 water 10mL")
        .expect("valid water addition")
        .expect("water operator");
    bench.step(water).expect("hydrate yeast");
    let catalase = bench.vessels[0]
        .lots
        .iter()
        .find(|lot| lot.species == SpeciesId::new("catalase"))
        .expect("catalase lot survives hydration");
    assert_eq!(catalase.hydrated_at, Some(0.0));
}

#[test]
fn fresh_yeast_resolves_water_and_scales_activity_by_dry_solids() {
    let recipe = kerotakis_core::material::lookup("Frischhefe", Some("de"))
        .expect("localized fresh yeast recipe");
    let expansion = recipe.expand(10.0, 0).expect("positive dose");
    let water = expansion
        .components
        .iter()
        .find(|component| component.species_id == "water")
        .expect("resolved moisture");
    let catalase = expansion
        .components
        .iter()
        .find(|component| component.species_id == "catalase")
        .expect("activity proxy");
    assert!((water.amount - 7.0).abs() < 1e-12);
    assert!((catalase.amount - 0.000_003).abs() < 1e-12);
    assert!((expansion.unresolved_amount - 2.999_997).abs() < 1e-12);
    assert!(recipe.matches("compressed yeast", Some("en")));
}

#[test]
fn familiar_powders_expand_to_the_existing_solver_species() {
    for (name, language, expected) in [
        ("Natron", Some("de"), "NaHCO3"),
        ("Waschsoda", Some("de"), "Na2CO3"),
        ("Speisestärke", Some("de"), "starch"),
        ("bicarbonate of soda", Some("en"), "NaHCO3"),
    ] {
        let recipe = kerotakis_core::material::lookup(name, language).expect(name);
        let expansion = recipe.expand(10.0, 0).expect("fixed expansion");
        assert_eq!(expansion.components.len(), 1);
        assert_eq!(expansion.components[0].species_id, expected);
        assert!((expansion.components[0].amount - 10.0).abs() < 1e-12);
        assert!(expansion.unresolved_amount.abs() < 1e-12);
    }
}

#[test]
fn familiar_solid_objects_expand_exactly_to_solver_species() {
    for (name, language, expected) in [
        ("Tafelsalz", Some("de"), "NaCl"),
        ("Kreidestück", Some("de"), "CaCO3"),
        ("Magnesiumband", Some("de"), "Mg"),
        ("Zinkstreifen", Some("de"), "Zn"),
        ("Eisennagel", Some("de"), "Fe"),
        ("Kupferdraht", Some("de"), "Cu"),
        ("aluminum foil", Some("en"), "Al"),
    ] {
        let recipe = kerotakis_core::material::lookup(name, language).expect(name);
        let expansion = recipe.expand(5.0, 0).expect("fixed expansion");
        assert_eq!(expansion.components.len(), 1, "{name}");
        assert_eq!(expansion.components[0].species_id, expected, "{name}");
        assert!((expansion.components[0].amount - 5.0).abs() < 1e-12);
        assert!(expansion.unresolved_amount.abs() < 1e-12);
    }
}

#[test]
fn familiar_object_addition_preserves_recipe_and_solid_inventory() {
    let op = parse_op("add v1 Zinkstreifen 6.538g")
        .expect("valid localized object command")
        .expect("operator");
    let mut bench = Bench::new();
    let events = bench.step(op).expect("add zinc strip");

    assert!(events.iter().any(|event| matches!(
        event,
        Event::MaterialAdded { recipe_id, components, .. }
            if recipe_id == "school/zinc-strip"
                && components.len() == 1
                && components[0].species == SpeciesId::new("Zn")
    )));
    let zinc = bench.vessels[0].moles_of(&SpeciesId::new("Zn")).0;
    assert!(
        (zinc - 0.1).abs() < 2e-6,
        "6.538 g Zn should be about 0.1 mol, got {zinc}"
    );
}

#[test]
fn ambiguous_bare_salt_is_not_claimed_by_the_table_salt_recipe() {
    let table_salt =
        kerotakis_core::material::lookup("Tafelsalz", Some("de")).expect("localized table salt");
    assert!(!table_salt.matches("Salz", Some("de")));
    assert!(!table_salt.matches("salt", Some("en")));
}

#[test]
fn hand_soap_and_dish_soap_are_distinct_localized_materials() {
    let hand = kerotakis_core::material::lookup("Flüssigseife", Some("de"))
        .expect("liquid hand-soap alias");
    let dish = kerotakis_core::material::lookup("Spülmittel", Some("de")).expect("dish-soap alias");
    assert_eq!(hand.canonical_key, "liquid_hand_soap");
    assert_eq!(dish.canonical_key, "dish_soap");
    assert_ne!(hand.id, dish.id);

    let expansion = hand.expand(20.0, 0).expect("fixed expansion");
    assert!((expansion.components[0].amount - 15.0).abs() < 1e-12);
    assert!((expansion.unresolved_amount - 5.0).abs() < 1e-12);
    assert_eq!(hand.roles.len(), 1, "hand soap retains gas as foam");
}

#[test]
fn rubbing_alcohol_keeps_its_labelled_volume_basis() {
    let recipe = kerotakis_core::material::lookup("Isopropanol 70%", Some("de"))
        .expect("localized rubbing-alcohol recipe");
    assert_eq!(recipe.basis, MaterialBasis::VolumeFraction);
    let expansion = recipe.expand(100.0, 0).expect("fixed expansion");
    assert_eq!(expansion.components.len(), 2);
    assert!((expansion.components[0].amount - 70.0).abs() < 1e-12);
    assert!((expansion.components[1].amount - 30.0).abs() < 1e-12);

    let op = parse_op("add v1 Isopropanol_70% 100mL")
        .expect("valid material command")
        .expect("operator");
    let mut bench = Bench::new();
    let events = bench.step(op).expect("add rubbing alcohol");
    assert!(events.iter().any(|event| matches!(event,
        Event::MaterialAdded { basis: MaterialBasis::VolumeFraction, total_amount, components, .. }
            if (*total_amount - 100.0).abs() < 1e-12 && components.len() == 2
    )));
    let vessel = &bench.vessels[0];
    assert!(vessel.moles_of(&SpeciesId::new("isopropanol")).0 > 0.9);
    assert!(vessel.moles_of(&SpeciesId::new("water")).0 > 1.6);
}

#[test]
fn cola_keeps_unknown_ingredients_explicit() {
    let cola =
        kerotakis_core::material::lookup("Cola", Some("de")).expect("localized cola surrogate");
    let expansion = cola.expand(104.0, 0).expect("fixed expansion");
    let resolved: f64 = expansion.components.iter().map(|part| part.amount).sum();
    assert!((resolved + expansion.unresolved_amount - 104.0).abs() < 1e-12);
    assert!((expansion.unresolved_amount - 11.284).abs() < 1e-12);
    assert!(expansion
        .components
        .iter()
        .any(|part| part.species_id == "CO2"));
    assert!(expansion
        .components
        .iter()
        .any(|part| part.species_id == "H3PO4"));
}
