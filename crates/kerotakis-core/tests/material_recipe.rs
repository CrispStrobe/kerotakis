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
fn steel_wool_keeps_its_alloy_remainder_explicit() {
    let recipe = kerotakis_core::material::lookup("Stahlwolle", Some("de"))
        .expect("localized steel-wool surrogate");
    let expansion = recipe.expand(10.0, 0).expect("fixed expansion");
    assert_eq!(expansion.components.len(), 1);
    assert_eq!(expansion.components[0].species_id, "Fe");
    assert!((expansion.components[0].amount - 9.8).abs() < 1e-12);
    assert!((expansion.unresolved_amount - 0.2).abs() < 1e-12);
}

#[test]
fn epsom_salt_is_one_dry_hydrate_species_not_premixed_liquid_water() {
    let recipe = kerotakis_core::material::lookup("Bittersalz", Some("de"))
        .expect("localized Epsom-salt recipe");
    let expansion = recipe.expand(24.6471, 0).expect("fixed expansion");
    assert_eq!(expansion.components.len(), 1);
    assert_eq!(expansion.components[0].species_id, "epsomite");
    assert!((expansion.components[0].amount - 24.6471).abs() < 1e-12);
    assert!(expansion.unresolved_amount.abs() < 1e-12);
    assert!(
        expansion
            .components
            .iter()
            .all(|component| component.species_id != "water"),
        "crystal water must remain in the hydrate formula until dissolution"
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

// --- BRD-014 checkpoints 34-37: wax, paper, flour, dough, juice, glass.

/// Candle wax is a blend of long-chain alkanes, none of which the registry
/// installs. The recipe therefore conserves the whole dose rather than
/// inventing a molecule to stand in for it.
#[test]
fn candle_wax_is_conserved_whole_with_no_invented_component() {
    let recipe =
        kerotakis_core::material::lookup("Kerzenwachs", Some("de")).expect("localized candle wax");
    assert_eq!(recipe.canonical_key, "candle_wax");
    let expansion = recipe.expand(25.0, 0).expect("fixed expansion");
    assert!(
        expansion.components.is_empty(),
        "no alkane of candle wax is an installed species"
    );
    assert!((expansion.unresolved_amount - 25.0).abs() < 1e-12);
}

/// The bare words a candle is usually called by are deliberately not claimed:
/// beeswax and paraffin wax are different materials, British English calls a
/// lamp fuel "paraffin", and a candle is a wick-and-flame object the bench
/// does not have.
#[test]
fn bare_wax_paraffin_and_candle_remain_unclaimed() {
    let wax = kerotakis_core::material::lookup("Kerzenwachs", Some("de")).expect("candle wax");
    for (name, language) in [
        ("Wachs", Some("de")),
        ("wax", Some("en")),
        ("Paraffin", Some("de")),
        ("paraffin", Some("en")),
        ("Kerze", Some("de")),
        ("candle", Some("en")),
    ] {
        assert!(!wax.matches(name, language), "{name} must stay unclaimed");
    }
}

/// A conserved unresolved solid is still in the beaker, and the bench says
/// so. Silence would repeat the defect the whole unresolved-fraction contract
/// exists to prevent: matter the engine holds, reported as absent.
#[test]
fn conserved_unresolved_solids_are_visible_without_entering_the_chemistry() {
    let mut bench = Bench::new();
    for command in ["add v1 water 100mL", "add v1 Kerzenwachs 20g"] {
        let op = parse_op(command)
            .unwrap_or_else(|error| panic!("parse {command}: {error}"))
            .expect("operator");
        bench
            .step(op)
            .unwrap_or_else(|error| panic!("{command}: {error}"));
    }
    let vessel = &bench.vessels[0];
    assert_eq!(vessel.unresolved_materials.len(), 1);
    assert!((vessel.unresolved_materials[0].amount - 20.0).abs() < 1e-12);

    let solids = kerotakis_core::material::conserved_unresolved_solids(vessel);
    assert_eq!(solids.len(), 1);
    assert_eq!(solids[0].material, "candle wax");
    assert_eq!(solids[0].colour_word, "off-white");

    let observed = kerotakis_core::appearance::observe(vessel);
    assert!(
        observed.words.contains("off-white candle wax"),
        "{}",
        observed.words
    );
    // The water it sits in is untouched: wax contributes no solute, no
    // colour and no cloudiness.
    assert!(observed.words.contains("colourless"), "{}", observed.words);
    assert!(observed.cloudiness < 0.01);
    assert!(vessel
        .contents
        .iter()
        .all(|portion| portion.species == SpeciesId::new("water")));
}

/// Paper is cellulose, and cellulose is not in the registry. The honest
/// answer is to name the sheet and conserve it, not to resolve it into a
/// fibre the engine does not have.
#[test]
fn paper_is_conserved_unresolved_with_its_identity_stated() {
    let recipe = kerotakis_core::material::lookup("Papier", Some("de")).expect("localized paper");
    assert_eq!(recipe.canonical_key, "paper_sheet");
    assert_eq!(recipe.name, "paper");
    assert!(recipe.matches("sheet of paper", Some("en")));
    let expansion = recipe.expand(4.0, 0).expect("fixed expansion");
    assert!(expansion.components.is_empty());
    assert!((expansion.unresolved_amount - 4.0).abs() < 1e-12);
    assert!(
        recipe
            .lot_assumptions
            .iter()
            .any(|assumption| assumption.contains("cellulose")),
        "the unresolved substance must be named, not merely omitted"
    );

    let op = parse_op("add v1 Papier 4g")
        .expect("valid material command")
        .expect("operator");
    let mut bench = Bench::new();
    let events = bench.step(op).expect("add paper");
    assert!(events.iter().any(|event| matches!(
        event,
        Event::MaterialAdded { recipe_id, components, unresolved_amount, .. }
            if recipe_id == "household/paper-sheet"
                && components.is_empty()
                && (*unresolved_amount - 4.0).abs() < 1e-12
    )));
    let observed = kerotakis_core::appearance::observe(&bench.vessels[0]);
    assert!(observed.words.contains("white paper"), "{}", observed.words);
}

/// White wheat flour resolves the starch a school iodine test actually finds,
/// and conserves the protein, moisture, fibre, lipid and ash that no
/// installed species describes.
#[test]
fn wheat_flour_resolves_its_starch_and_conserves_the_remainder() {
    let recipe = kerotakis_core::material::lookup("Mehl", Some("de")).expect("localized flour");
    assert_eq!(recipe.canonical_key, "wheat_flour");
    assert!(recipe.matches("flour", Some("en")));
    let expansion = recipe.expand(50.0, 0).expect("fixed expansion");
    assert_eq!(expansion.components.len(), 1);
    assert_eq!(expansion.components[0].species_id, "starch");
    assert!((expansion.components[0].amount - 35.0).abs() < 1e-12);
    assert!((expansion.unresolved_amount - 15.0).abs() < 1e-12);
    assert!(
        expansion
            .components
            .iter()
            .all(|component| component.species_id != "water"),
        "sorbed flour moisture must not become free liquid water"
    );
}

/// A dough is wet, but its water is held in the flour. Resolving it would
/// put a pool of free liquid in the beaker that a real dough never releases,
/// so the water stays inside the conserved remainder and the boundary is
/// stated rather than approximated.
#[test]
fn dough_keeps_its_water_out_of_the_vessel_s_free_liquid() {
    let recipe = kerotakis_core::material::lookup("Teig", Some("de")).expect("localized dough");
    assert_eq!(recipe.canonical_key, "flour_water_dough");
    let expansion = recipe.expand(100.0, 0).expect("fixed expansion");
    assert_eq!(expansion.components.len(), 1);
    assert_eq!(expansion.components[0].species_id, "starch");
    assert!((expansion.components[0].amount - 42.0).abs() < 1e-12);
    assert!((expansion.unresolved_amount - 58.0).abs() < 1e-12);

    let op = parse_op("add v1 Teig 100g")
        .expect("valid material command")
        .expect("operator");
    let mut bench = Bench::new();
    bench.step(op).expect("add dough");
    let vessel = &bench.vessels[0];
    assert!(
        vessel.liquid_volume().0 < 1e-12,
        "a dough in a dry beaker is not a beaker of water"
    );
    assert!(vessel.moles_of(&SpeciesId::new("starch")).0 > 0.0);
}

/// Apple juice's sugar is mostly fructose and glucose, and neither is an
/// installed species. The surrogate resolves only the sucrose that really is
/// sucrose and leaves the rest — including the malic acid that makes juice
/// tart — conserved and explicitly unmodelled.
#[test]
fn apple_juice_resolves_only_the_sugar_it_can_honestly_name() {
    // KID-20: this test used to hold that apple juice resolved *one* sugar
    // and no acid, because fructose, glucose and malic acid were not in the
    // registry when it was written. All three are now, so what it holds has
    // moved: the juice resolves each sugar as itself in the cited
    // proportions and carries the acid its tartness is actually made of.
    // What has not moved is the rule underneath — the juice must never
    // borrow an acid the engine merely happens to have.
    let recipe =
        kerotakis_core::material::lookup("Apfelsaft", Some("de")).expect("localized apple juice");
    let expansion = recipe.expand(100.0, 0).expect("fixed expansion");
    let resolved: f64 = expansion.components.iter().map(|part| part.amount).sum();
    assert!((resolved + expansion.unresolved_amount - 100.0).abs() < 1e-12);
    let amount = |species: &str| -> f64 {
        expansion
            .components
            .iter()
            .find(|part| part.species_id == species)
            .map_or(0.0, |part| part.amount)
    };
    // Fructose first, then glucose, then sucrose: the order the cited
    // composition puts them in, which is the point of resolving them apart.
    assert!(amount("fructose") > amount("glucose"));
    assert!(amount("glucose") > amount("sucrose"));
    assert!(
        amount("malic_acid") > 0.0,
        "the tartness is a real component"
    );

    let op = parse_op("add v1 Apfelsaft 100mL")
        .expect("valid material command")
        .expect("operator");
    let mut bench = Bench::new();
    bench.step(op).expect("add apple juice");
    let vessel = &bench.vessels[0];
    assert!(vessel.moles_of(&SpeciesId::new("water")).0 > 5.0);
    assert!(vessel.moles_of(&SpeciesId::new("malic_acid")).0 > 0.0);
    // Still no borrowed acid: computing a pH from the wrong molecule would
    // be worse than the honest refusal the aqueous engine gives for malate.
    for acid in ["CH3COOH", "H3PO4", "HCl", "H2SO4"] {
        assert!(
            vessel.moles_of(&SpeciesId::new(acid)).0 < 1e-12,
            "apple juice must not borrow {acid} for its tartness"
        );
    }
}

/// Glass is silica-dominant and unreactive in the terms the aqueous engine
/// models. Its network modifiers stay conserved rather than becoming free
/// oxides that would invent an alkaline dissolution.
#[test]
fn soda_lime_glass_resolves_silica_and_conserves_its_modifiers() {
    let recipe = kerotakis_core::material::lookup("Glas", Some("de")).expect("localized glass");
    assert_eq!(recipe.canonical_key, "glass");
    assert!(recipe.matches("window glass", Some("en")));
    let expansion = recipe.expand(100.0, 0).expect("fixed expansion");
    assert_eq!(expansion.components.len(), 1);
    assert_eq!(expansion.components[0].species_id, "SiO2");
    assert!((expansion.components[0].amount - 73.0).abs() < 1e-12);
    assert!((expansion.unresolved_amount - 27.0).abs() < 1e-12);
    for modifier in ["CaO", "MgO"] {
        assert!(
            expansion
                .components
                .iter()
                .all(|component| component.species_id != modifier),
            "{modifier} in a glass network is not {modifier} in a beaker"
        );
    }
    assert!(matches!(
        recipe.physical_form,
        kerotakis_core::material::MaterialPhysicalForm::CompositeObject { geometry: Some(_) }
    ));
}

/// Named chalk in named vinegar fizzes (checkpoint 27). Named glass in the
/// same vinegar does nothing, and the acid is still all there afterwards —
/// the contrast is computed, not narrated.
#[test]
fn glass_in_vinegar_is_unreactive_where_chalk_fizzes() {
    let mut bench = Bench::new();
    for command in ["add v1 Essig 50mL", "add v1 Glas 10g"] {
        let op = parse_op(command)
            .unwrap_or_else(|error| panic!("parse {command}: {error}"))
            .expect("operator");
        let events = bench
            .step(op)
            .unwrap_or_else(|error| panic!("{command}: {error}"));
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, Event::ReactionOccurred { .. })),
            "{command} must not react"
        );
    }
    let vessel = &bench.vessels[0];
    let acid = vessel.moles_of(&SpeciesId::new("CH3COOH")).0
        + vessel.moles_of(&SpeciesId::new("CH3COO-")).0;
    assert!(
        acid > 0.03,
        "the vinegar's acetic acid is untouched: {acid}"
    );
    assert!(vessel.moles_of(&SpeciesId::new("CO2")).0 < 1e-12);
    assert!((vessel.moles_of(&SpeciesId::new("SiO2")).0 - 7.3 / 60.084).abs() < 1e-6);
}

/// Unknown material names still report themselves as unknown materials
/// rather than as unknown species.
#[test]
fn a_material_this_tranche_did_not_claim_still_refuses_clearly() {
    let error = parse_op_typed("add v1 Wachs 10g").expect_err("bare wax is unclaimed");
    assert_eq!(error.kind, ParseErrorKind::UnknownSpecies);
}

/// KID-1: a bottle nobody can type is a bottle that is not on the shelf.
///
/// The `.lab` grammar splits on whitespace, so `household vinegar` — which
/// was the *only* English alias the vinegar recipe carried — could never be
/// written down. Thirty of the fifty shipped recipes had that shape, and
/// twelve of the thirty children's activities in `KIDS.md` died on it. Every
/// recipe must now be reachable by at least one whitespace-free spelling of
/// every name it advertises.
#[test]
fn every_recipe_name_has_a_writable_spelling_that_resolves() {
    for recipe in kerotakis_core::material::all() {
        let mut names = vec![recipe.canonical_key.clone(), recipe.name.clone()];
        names.extend(recipe.aliases.values().flatten().cloned());
        for name in names {
            let writable = name.replace(char::is_whitespace, "_");
            let resolved = kerotakis_core::material::lookup(&writable, None)
                .unwrap_or_else(|| panic!("'{writable}' (from '{name}') resolves to no recipe"));
            assert_eq!(
                resolved.canonical_key, recipe.canonical_key,
                "'{writable}' resolved to the wrong recipe"
            );
            // The amount must be written in the recipe's own basis, or the
            // grammar refuses on the unit rather than on the name — which
            // is a different (and correct) refusal.
            let unit = match recipe.basis {
                MaterialBasis::MassFraction => "g",
                MaterialBasis::MoleFraction => "mol",
                MaterialBasis::VolumeFraction => "mL",
            };
            assert!(
                parse_op(&format!("add v1 {writable} 1{unit}")).is_ok(),
                "'{writable}' is not accepted by the add grammar"
            );
        }
    }
}

/// The same normalization must not make two different bottles answer to one
/// name, and must not shadow a species key.
#[test]
fn no_recipe_name_collides_after_normalization() {
    let mut seen: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for recipe in kerotakis_core::material::all() {
        let mut names = vec![recipe.canonical_key.clone(), recipe.name.clone()];
        names.extend(recipe.aliases.values().flatten().cloned());
        for name in names {
            let writable = name.replace(char::is_whitespace, "_");
            assert!(
                kerotakis_core::species::lookup_key(&writable).is_none(),
                "material name '{writable}' shadows a species key"
            );
            if let Some(other) = seen.insert(writable.to_lowercase(), recipe.canonical_key.clone())
            {
                assert_eq!(
                    other, recipe.canonical_key,
                    "two recipes answer to the same writable name"
                );
            }
        }
    }
}

/// KID-20: a drink that reports no acidity is a drink the bench lies about.
///
/// `apple_juice` resolved to water and sucrose and nothing else, so a pH map
/// of the kitchen reported *nothing at all* for it (KIDS.md, K50). The
/// recipe explained the omission — "malic acid ... is not in the registry"
/// — and that sentence had since stopped being true, which is the worse
/// failure of the two: a reason that has expired reads as current.
///
/// The list is explicit because a recipe is a curation act: adding a drink
/// means deciding what its acidity is made of, and this is where that
/// decision is recorded.
#[test]
fn every_drink_recipe_carries_the_acid_its_taste_is_made_of() {
    // Species the registry ships that are acids in the sense a tongue means.
    const ACIDS: &[&str] = &[
        "CO2",
        "H3PO4",
        "CH3COOH",
        "citric_acid",
        "malic_acid",
        "ascorbic_acid",
        "H2C2O4",
        "HCl",
        "H2SO4",
    ];
    for key in [
        "apple_juice",
        "cola_drink",
        "sparkling_water",
        "white_vinegar_5_percent",
    ] {
        let recipe = kerotakis_core::material::lookup(key, None)
            .unwrap_or_else(|| panic!("{key} is on the shelf"));
        assert!(
            recipe
                .components
                .iter()
                .any(|component| ACIDS.contains(&component.species_id.as_str())),
            "{key} has no acid component, so it can report no acidity: {:?}",
            recipe
                .components
                .iter()
                .map(|c| c.species_id.as_str())
                .collect::<Vec<_>>()
        );
    }
}

/// KID-20: blue vitriol is blue.
///
/// Copper(II) sulfate pentahydrate carried the appearance string "deep blue
/// crystals" and no sRGB, so the appearance layer fell back to its default
/// pale grey and described the product of "grow blue crystals" as white.
#[test]
fn the_blue_salt_is_drawn_blue() {
    let data = kerotakis_core::species::lookup_key("chalcanthite").expect("chalcanthite ships");
    let colour = data.colour.expect("chalcanthite has a reviewed swatch");
    assert!(
        colour.b > colour.r && colour.b > colour.g,
        "blue vitriol must be bluest in blue: {colour:?}"
    );
}

/// KID-10: vinegar smells of vinegar.
///
/// The odour table has held `CH3COOH` since it was written, and `smell` on a
/// beaker of vinegar answered "no odour a careful waft detects" — because
/// the aqueous solver had already turned the acetic acid into `CH3COO-`, and
/// the table is keyed on the acid. This is the third model caught looking a
/// species up by a key the solver replaces (curdling was the first), so the
/// family test now lives in `species::same_bronsted_family`.
#[test]
fn a_beaker_of_vinegar_smells_of_vinegar() {
    let op = parse_op("add v1 white_vinegar_5_percent 30mL")
        .expect("valid material command")
        .expect("operator");
    let mut bench = Bench::new();
    bench.step(op).expect("add vinegar");
    let vessel = &bench.vessels[0];
    let wafted = kerotakis_core::senses::waft(vessel);
    assert!(
        wafted.iter().any(|odour| odour.species == "CH3COOH"),
        "a waft over vinegar must find acetic acid: {:?}",
        wafted.iter().map(|o| o.species).collect::<Vec<_>>()
    );
}

/// The family rule must not over-reach: two species that merely share some
/// elements are not one acid and its conjugate base.
#[test]
fn the_bronsted_family_is_a_family_and_not_a_neighbourhood() {
    use kerotakis_core::species::same_bronsted_family;
    assert!(same_bronsted_family("CH3COOH", "CH3COO-"));
    assert!(same_bronsted_family("H3PO4", "H2PO4-"));
    assert!(same_bronsted_family("NH3", "NH4+"));
    // Same elements, different skeletons.
    assert!(!same_bronsted_family("CH3COOH", "H2C2O4"));
    assert!(!same_bronsted_family("water", "H2O2"));
    assert!(!same_bronsted_family("CO2", "HCO3-"));
}

/// KID-10: the calcium a child actually owns colours a flame.
///
/// `Ca+2` and `Ca(OH)2` carried a flame colour and `CaCl2` did not, so the
/// de-icing salt in the cupboard — the one a flame-test activity reaches for
/// — answered "nothing happens". Calcium is the one metal in that experiment
/// a household has.
#[test]
fn every_calcium_salt_on_the_shelf_colours_the_flame() {
    for key in ["CaCl2", "CaCO3", "CaO", "gypsum"] {
        let data = kerotakis_core::species::lookup_key(key).unwrap_or_else(|| panic!("{key}"));
        assert!(
            data.flame_colour.is_some(),
            "{key} carries calcium and must colour a flame"
        );
    }
}
