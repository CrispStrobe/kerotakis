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
