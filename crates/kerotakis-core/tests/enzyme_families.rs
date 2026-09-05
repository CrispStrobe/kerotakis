use kerotakis_core::enzyme::{self, EnzymeFamily, SubstrateClass};
use kerotakis_core::material::{self, MaterialRole};
use kerotakis_core::script::parse_op;
use kerotakis_core::{species, Bench, SpeciesId};

#[test]
fn six_enzyme_catalysts_are_declared_with_distinct_substrates() {
    assert_eq!(enzyme::FAMILIES.len(), 6);
    assert_eq!(
        enzyme::profile("lactase").map(|profile| profile.family),
        Some(EnzymeFamily::Lactase)
    );
    assert_eq!(
        enzyme::profile("protease").map(|profile| profile.acts_on),
        Some("protein peptide bonds")
    );
    assert_eq!(
        enzyme::profile("lipase").map(|profile| profile.acts_on),
        Some("triglycerides")
    );
    assert_eq!(
        enzyme::profile("catalase").map(|profile| profile.acts_on),
        Some("hydrogen peroxide")
    );
    // Three proteases cut the same bond and differ in where they do it.
    for key in ["protease", "pepsin", "bromelain"] {
        assert_eq!(
            enzyme::profile(key).and_then(|profile| profile.substrate),
            Some(SubstrateClass::Protein),
            "{key}"
        );
    }
    // Catalase's chemistry is a curated stoichiometric reaction, so it
    // deliberately owns no bounded-activity substrate class.
    assert_eq!(
        enzyme::profile("catalase").and_then(|profile| profile.substrate),
        None
    );
}

#[test]
fn every_catalyst_carries_a_reviewed_temperature_and_acidity_window() {
    for profile in enzyme::FAMILIES {
        assert!(
            profile.optimum_temperature_k > 250.0 && profile.optimum_temperature_k < 400.0,
            "{}",
            profile.species
        );
        assert!(profile.temperature_width_k > 0.0, "{}", profile.species);
        assert!(
            profile.denatures_above_k > profile.optimum_temperature_k,
            "{}",
            profile.species
        );
        assert!(
            (0.0..=14.0).contains(&profile.optimum_ph),
            "{}",
            profile.species
        );
        assert!(profile.ph_width > 0.0, "{}", profile.species);
    }
    // The one ordering the corpus asks about by name.
    let pepsin = enzyme::profile("pepsin").expect("pepsin");
    let protease = enzyme::profile("protease").expect("protease");
    assert!(pepsin.optimum_ph < 3.0);
    assert!(pepsin.optimum_ph < protease.optimum_ph);
}

#[test]
fn newly_exposed_enzyme_catalysts_can_be_added_to_the_bench() {
    for key in [
        "lactase",
        "protease",
        "lipase",
        "catalase",
        "pepsin",
        "bromelain",
    ] {
        assert!(species::lookup(&SpeciesId::new(key)).is_some(), "{key}");
        let mut bench = Bench::new();
        let op = parse_op(&format!("add v1 {key} 0.000001mol"))
            .expect("parse")
            .expect("operator");
        bench.step(op).expect("known enzyme");
        assert_eq!(bench.vessels[0].moles_of(&SpeciesId::new(key)).0, 1e-6);
    }
}

/// The registry validator deliberately cannot check an enzyme-source key:
/// an enzyme is not a registry identity, which is the whole reason the role
/// exists. This is the check that would otherwise be missing, and a typo in
/// a recipe would leave a food silently carrying nothing.
#[test]
fn every_enzyme_source_role_names_a_catalyst_this_bench_has() {
    let mut carriers = 0;
    for recipe in material::all() {
        for role in &recipe.roles {
            if let MaterialRole::EnzymeSource {
                enzyme,
                catalyst_equivalent_per_gram,
                denatures_above_k,
            } = role
            {
                carriers += 1;
                let profile = enzyme::profile(enzyme)
                    .unwrap_or_else(|| panic!("{}: unknown catalyst {enzyme}", recipe.id));
                assert!(
                    profile.substrate.is_some(),
                    "{}: {enzyme} has no bounded-activity substrate to cut",
                    recipe.id
                );
                assert!(*catalyst_equivalent_per_gram > 0.0, "{}", recipe.id);
                assert!(*denatures_above_k > 273.15, "{}", recipe.id);
            }
        }
    }
    assert!(carriers > 0, "no food carries its own enzyme");
}
